# Implementation review — GUI v0.36.0 autosave + atomic writes (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Spec @ design/SPEC_gui_v0_36_0_autosave_atomic.md (R0 GREEN r3). Verdict: GREEN (0 Critical / 0 Important / 3 Minor — ALL folded post-review: doc-comment split between serialize_redacted/save, write-Err tmp cleanup added, this review persisted). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

- **M1 — doc-comment drift from the save() split.** The pre-existing save() doc block now sits atop `serialize_redacted`, whose opening sentence is false for that function, and `pub fn save()` is left with no doc comment. Suggest splitting: write-contract sentence → `save()`, stamp note → stays.
- **M2 — tmp cleanup covers rename-Err only, not write-Err.** A failed `fs::write(&tmp, …)` (e.g. ENOSPC partial write) propagates via `?` leaving the partial tmp behind. SPEC D1 says "On serialize/write error the temp file is best-effort removed." Consequence identical to the accepted crash-orphan class; one-line `inspect_err` closes it.
- **M3 — CHANGELOG cites an impl-review artifact that does not yet exist.** Persist this review before commit.

## Verdict

**GREEN** (0 Critical / 0 Important; 3 Minors, none blocking).

### Evidence

**1. persistence.rs** — split clean (`serialize_redacted` redact+stamp+pretty-JSON; `write_atomic` :266-282; `save()` composes; `save_if_changed` serializes ONCE, skips on byte-equality, cache updated ONLY after Ok). **Temp name appends, verified:** `path.file_name()` → `"state.json"`, `with_file_name(format!("{file_name}.{pid}.tmp"))` yields `state.json.<pid>.tmp` — append, not the `with_extension` trap. PID via `std::process::id()`. Rename with best-effort tmp cleanup on Err. No remove-then-rename anywhere. `create_dir_all` preserved inside `write_atomic` — BOTH save paths get parent-dir creation.

**2. main.rs** — `build_persisted_state` is a **verbatim move** (diffed field-by-field against `git show HEAD:src/main.rs`: all 9 fields, both closures byte-identical); `&self` borrow-side signature with the no-`mem::take` invariant in the doc. Both v0.35.0 invariant comments still at on_exit; on_exit still unconditional `save()` with warn-on-Err; zeroize sweep after. New fields seeded (`last_autosave: Instant::now()`, snapshot None). `AUTOSAVE_INTERVAL = 30s`. Timer block after the geometry snapshot, gated on elapsed, `save_if_changed` with warn-on-Err, reset REGARDLESS. (Impl nests `if let Some(path)` inside the elapsed check — behaviorally equivalent to the spec's conjunction and correctly bounds serialization to once per interval.)

**3. Tests** — T1 pre-creates the EXACT PID name (R0-r2 I3); asserts consumed + no `*.tmp` sibling + round-trip. T1b rename-over-existing. T2 write→skip(deleted file NOT recreated)→write-on-change. T2b read-only-dir Err + cache stays None. **TDD integrity verified live:** scratch-reverted `write_atomic` to plain `fs::write` → exactly T1 FAILED ("pre-created garbage temp survived"); restored sha256-identical (`e640e10c…`).

**4. Docs/ritual** — CHANGELOG [0.36.0] claims true of the diff; README autosave sentence accurate; FOLLOWUPS resolution matches the shipped mechanism; version 0.36.0 at Cargo.toml/Cargo.lock/README self-pin.

**5. Suite + clippy** — full suite under the pinned-binary env: all targets 0 failed (lib 91, persistence 12, wiring 10, redaction 8, new autosave 4); clippy clean.

Tree left exactly as found.
