# R0 round-1 architect review — SPEC_gui_v0_36_0_autosave_atomic (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Spec @ design/SPEC_gui_v0_36_0_autosave_atomic.md, GUI 1244b7c. Verdict: YELLOW (0 Critical / 2 Important / 7 Minor). Review verbatim below.

---

## Critical

None.

## Important

**I1 — The two-instance risk claim is false as written: a shared fixed-name `.tmp` can install a torn file ATOMICALLY. Pin a per-process tmp name (PID suffix).**
SPEC §Risks: "atomic writes ensure the loser never tears the file." Not true with a single shared `<path>.tmp`: instance A opens/truncates `state.json.tmp`, instance B truncates and rewrites it mid-write, A's `fs::rename` then installs B's-partial/interleaved bytes — the rename is atomic, the *content* is torn. Next `load()` hits the malformed-JSON leg (persistence.rs:262-269), `.bak`s the file, and silently resets the session — exactly the failure class the cycle exists to close, now firing every 30 s instead of only at exit. The fix is two lines: name the temp `state.json.<pid>.tmp` via `std::process::id()`. This is deterministic per-process, so T1's pre-create-garbage pin still works (the test and `save()` share a process — the test computes the same name). **Fold:** (a) PID-suffix the tmp name in D1; (b) correct the Risks sentence ("per-process tmp names ensure the loser never tears the file; losing whole-file writes remains last-writer-wins, same accepted class as v0.35.0"); (c) add one sentence on orphan policy — a crash between tmp-write and rename strands an inert `*.tmp` per crashed PID; accept-and-document (tiny files) or best-effort glob-clean in `save()` — accept is fine, but say which.

**I2 — The D1 hedge resolves AGAINST the fallback: `fs::rename` replace-on-existing is the documented contract on both platforms — DELETE the "remove-then-rename is acceptable" language, it licenses an implementer to reintroduce the non-atomic window.**
Verified from the toolchain's std docs (`std::fs::rename`, local rustdoc, rustc 1.85.0): "Renames a file or directory to a new name, **replacing the original file if `to` already exists**." Platform note: `rename(2)` on Unix; `SetFileInformationByHandle` with `FileRenameInfoEx` POSIX semantics on Windows 10 1607+ (older MoveFileExW-class fallback otherwise). So no fallback is needed — and the spec's conditional escape hatch ("remove-then-rename is acceptable") is worse than useless: on Windows it recreates a window where `state.json` is *absent*, defeating D1's whole point, and an implementer reading the spec is licensed to ship it. **Fold:** replace the hedge with the verified contract + doc citation; state the Windows degradation honestly instead: rename can fail with a sharing violation if another process holds the destination open without `FILE_SHARE_DELETE` — that surfaces as the existing warn-on-Err (old file intact), which is the correct posture. Note also that GUI CI runs tests on ubuntu only (build.yml:46-48 builds Windows but only schema-mirror.yml:133 runs `cargo test --workspace`, ubuntu) — the Windows leg ships on documented contract, not CI evidence; one sentence in the spec records that.

## Minor

**M1 — Pin the exact tmp-name construction.** "`<path>.tmp`" is ambiguous between append (`state.json.<pid>.tmp`) and `Path::with_extension` (`with_extension("tmp")` → `state.tmp` — the same footgun class as load()'s deliberate `with_extension("json.bak")` at persistence.rs:266). T1 hardcodes the name, so a spec/impl mismatch REDs immediately, but pin it anyway: build via `file_name` string append, sibling of `path`, after the existing `create_dir_all` (persistence.rs:240-243). No collision with `.json.bak`.

**M2 — Make the cache-update ordering in `save_if_changed` explicit.** Add: "the cache updates ONLY after the write returns Ok — a failed write leaves the cache untouched so the next interval retries." Optionally a T2 leg (read-only parent dir, cfg(unix)) asserting Err does not poison the cache.

**M3 — T1 mechanics confirmed sound; optional sharper cell.** The pre-create-garbage pin is deterministically RED today (current `save()` is a direct `fs::write`, persistence.rs:249) and GREEN after. Optional cfg(unix) mode-0o444 cell pins "never writes through the destination handle" directly.

**M4 — Citation: "the existing 12 persistence cells" is 11.** tests/persistence.rs has `cell_1`..`cell_11`; `save()` is additionally exercised by tests/persistence_wiring_v0_35_0.rs:215. All stay green. Fix the count.

**M5 — State the power-loss contract in D1 prose.** Power loss before the rename leaves the old `state.json` intact; power loss after rename on a non-ordered filesystem can expose an empty/partial file — the existing malformed→`.bak` leg recovers to fresh-default; fsync durability stays an explicit non-goal.

**M6 — One line on why not eframe's native auto_save.** eframe 0.31's `App::save` + `auto_save_interval` (default 30 s, eframe-0.31.1/src/epi.rs:201-203) would do the cadence, but the `persistence` feature is off and its ron/storage shape isn't `state.json`. Non-goal sentence so a future reader doesn't "simplify" onto it.

**M7 — Confirmed-fine items (audit trail):** 30 s + 1 Hz keepalive guarantee real (main.rs:176-184); timer-after-geometry correct (main.rs:342-350); reset-regardless right; `build_persisted_state(&self)` borrow-side by signature (v0.35.0 I2 invariant holds by construction; I1 order invariant lives only at on_exit, kept); on_exit snapshot non-update fine; modal interplay non-issue; on_exit inherits D1 atomicity (the FOLLOWUP's torn-by-3s-grace concern); first-autosave-always-writes acceptable; SemVer/docs/FOLLOWUPS plan correct.

## Verdict

**YELLOW — 0 Critical / 2 Important.** Fold I1 + I2 (+ minors as convenient), re-dispatch. The design itself — sibling-tmp+rename, content-compare debounce with zero dirty-flag plumbing, 30 s cadence, unconditional on_exit save — is sound and correctly preserves both v0.35.0 load-bearing invariants.
