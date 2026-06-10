# SPEC — GUI v0.36.0: debounced autosave + atomic state.json writes

**Cycle:** mnemonic-gui v0.36.0 (MINOR) · **Source SHA:** `1244b7c` (= v0.35.0) · **Resolves:** `gui-persistence-autosave-debounce` (FOLLOWUPS.md, filed v0.35.0).
**Context:** v0.35.0 wired exit-only persistence; this cycle closes its two recorded gaps — crash/SIGKILL session loss, and the non-atomic `fs::write` a signal-handler's 3s grace can tear.

## Design

### D1 — atomic `save()` (persistence.rs)

`save()` writes to a sibling temp file — **`state.json.<pid>.tmp` via `std::process::id()`, built by file_name string APPEND (NOT `with_extension`, which would yield `state.tmp` — R0-r1 M1), same directory** (rename is only atomic within a filesystem) — then `fs::rename(tmp, path)`. **PID suffix is LOAD-BEARING (R0-r1 I1):** a shared fixed-name tmp lets two instances interleave writes and atomically install TORN content (the rename is atomic, the bytes aren't) — which the malformed→`.bak` leg would then "recover" by silently resetting the session every 30s. Per-process names reduce two-instance contention to plain last-writer-wins (the accepted v0.35.0 class). Orphan policy: a crash between tmp-write and rename strands one inert `*.tmp` per crashed PID — ACCEPTED and documented (tiny files; no glob-clean). `fs::rename` replace-on-existing is the DOCUMENTED std contract on both Unix (`rename(2)`) and Windows (POSIX-semantics rename on Win10 1607+) — NO remove-then-rename fallback, ever (it would recreate a destination-absent window; R0-r1 I2). Windows degradation: a sharing-violation rename failure surfaces as the existing warn-on-Err with the old file intact — correct posture; note the GUI test suite runs on ubuntu only, so the Windows leg ships on the documented contract, not CI evidence. Power-loss contract (R0-r1 M5): before the rename → old file intact; after, a non-ordered filesystem may expose partial bytes → the malformed→`.bak` leg recovers to fresh-default; fsync stays a non-goal. On serialize/write error the temp file is best-effort removed. Behavior contract unchanged otherwise (redact + version-stamp + create parent dirs).

### D2 — change-gated autosave (lib helper + main.rs timer)

- **Lib helper** `persistence::save_if_changed(state: &PersistedState, path: &Path, last_serialized: &mut Option<String>) -> io::Result<bool>`: serialize the redacted+stamped state ONCE; if it equals `*last_serialized`, skip (return `Ok(false)`); else write atomically (sharing D1's path) and update the cache (return `Ok(true)`). **Cache updates ONLY after the write returns Ok** — a failed write leaves it untouched so the next interval retries (R0-r1 M2). This gives debounce-by-content with ZERO dirty-flag plumbing through mutation sites — the serialization of a redacted state this size is trivially cheap at the chosen cadence. (String-compare correctness depends on the PersistedState maps being `BTreeMap` — deterministic serialization order; do NOT refactor them to HashMap, which would silently kill the skip while T2 stays green — R0-r2 M8.) (Refactor note: `save()` and `save_if_changed` share one internal `serialize_redacted(state) -> io::Result<String>` + one `write_atomic(path, body)`.)
- **main.rs:** extract the v0.35.0 on_exit `PersistedState` construction into `fn build_persisted_state(&self) -> PersistedState` (verbatim move — the borrow-side/no-`mem::take` invariant and its comment move with it). New app fields: `last_autosave: std::time::Instant` (seeded `now` at construction — first autosave only after a full interval) + `last_saved_snapshot: Option<String>`. In `update()` (after the geometry snapshot): if `self.state_path.is_some() && self.last_autosave.elapsed() >= AUTOSAVE_INTERVAL` → `save_if_changed(&self.build_persisted_state(), path, &mut self.last_saved_snapshot)` (warn-on-Err like on_exit) → reset the timer REGARDLESS of write-vs-skip. `const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(30)` (matches eframe's own auto_save default; the 1 Hz keepalive guarantees the timer is evaluated at least once per second even when idle).
- **on_exit:** unchanged semantics — unconditional `save()` (final write must not be skipped by a stale snapshot match… actually a content-match skip would be CORRECT there too, but unconditional is simpler and the order-comment stays untouched). Keep `save()`.

### D3 — docs/ritual

- README session-restore section: add one sentence ("state also autosaves every ~30 s, so a crash loses at most the last interval").
- CHANGELOG `[0.36.0]`; version bump (Cargo.toml + Cargo.lock + README self-pin); FOLLOWUPS: resolve `gui-persistence-autosave-debounce`.
- No schema/redaction changes; no pin impact; no companions. SemVer MINOR (new periodic-write behavior).

## Tests (TDD red-first)

- **T1 (atomic):** after `save()`, no `*.tmp` sibling remains (same-process glob) and the content round-trips; `save()` over an EXISTING file replaces it (the rename-over-existing leg). (RED-able: pre-create **the EXACT D1 name, computed in-test as `format!("state.json.{}.tmp", std::process::id())`** (test and save() share a process — R0-r2 I3; pre-creating a generic `state.json.tmp` would be red against a CORRECT implementation and its verbatim-preserving "fix" would reverse I1) with garbage; post-save it must be GONE (consumed by the rename) while `path` holds valid state. Plus the existing 11 persistence cells (cell_1..cell_11) + the v0.35.0 T5 round-trip stay green — they exercise save() heavily (R0-r1 M4).)
- **T2 (save_if_changed):** first call writes (`Ok(true)`, file exists, cache set); second call with identical state skips (`Ok(false)`, file mtime/bytes unchanged — assert by pre-deleting the file: a skip must NOT recreate it); a mutated state writes again (`Ok(true)`).
- **T3 (cadence helper logic, if any extracted):** the timer check is two lines in update() — no dedicated cell; T2 carries the logic. The on_exit path is already pinned by v0.35.0 cells.
- Full suite + clippy under the pinned-binary env discipline.

## Risks / Non-goals

- Two instances autosaving every 30 s sharpens last-writer-wins from "at exit" to "continuously" — same accepted class as v0.35.0 (README already documents it); PER-PROCESS tmp names ensure the loser never tears the file (whole-file last-writer-wins remains).
- Non-goals: dirty-flag plumbing; fsync/durability guarantees; autosave-interval configurability; any redaction change; eframe-native `App::save`/`auto_save_interval` (the `persistence` feature is OFF and its ron/storage shape is not `state.json` — do not "simplify" onto it; R0-r1 M6).
