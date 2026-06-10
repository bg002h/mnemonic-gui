# R0 round-1 architect review — SPEC_gui_v0_35_0_persistence_wiring (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Spec @ design/SPEC_gui_v0_35_0_persistence_wiring.md, GUI 1a1615a. Verdict: YELLOW (0 Critical / 4 Important / 7 Minor). Review verbatim below.

---

## Critical

None.

## Important

**I1 — Decision 4's "correctness-irrelevant" save/zeroize-order rationale is factually false; the order is LOAD-BEARING.**
The spec (SPEC §Decisions 4) claims order vs the zeroize sweep is "correctness-irrelevant — `save()` redacts internally and `secret_widgets` cannot serialize". But `secrets::zeroize_form_state` (src/secrets.rs:278-308) wipes far more than secrets: it zeroizes **every** `Text`/`Dropdown`/`Path`/`NodeValueComposite` value (:279-291), **all slot rows** (:292-294), and all positionals — i.e. the entire watch-only form content that persistence exists to save. Zeroize-before-save (or any interleaving) persists an all-blank state. The pinned order (save first) is correct, but the false rationale invites a future "order doesn't matter" refactor that silently destroys saved state, and T5 (testing pure helpers, not `on_exit`) would not catch it. **Fix:** rewrite the decision-4 parenthetical to state the order is load-bearing because `zeroize_form_state` blanks non-secret values too; ideally add a P2 test or comment pinning it at the `on_exit` call site.

**I2 — `on_exit` PersistedState construction is unspecified, and the obvious implementation (move/`mem::take`) silently kills the zeroize sweep.**
`PersistedState.form_state_per_subcommand` owns `BTreeMap<String, FormState>` and `FormState` is not `Clone` (src/schema/mod.rs:317-321 doc; persistence.rs:38-43). The spec covers the load-direction move (Decision 1) but says nothing about how `on_exit` assembles the save-side struct. The natural implementation — `std::mem::take(&mut self.form_state)` into the `PersistedState` — leaves `self.form_state` empty, so the existing sweep `for state in self.form_state.values_mut() { zeroize_form_state(state) }` (main.rs:903-905) becomes a no-op: **secrets are never zeroized at exit** (SPEC §9 regression), with zero test coverage. **Fix (pin one):** (a) build the map borrow-side via `self.form_state.iter().map(|(k,v)| (k.clone(), redact_for_persistence(v))).collect()` — `redact_for_persistence(&FormState) -> FormState` (persistence.rs:74) returns owned without `Clone`, `save()`'s internal re-redaction is idempotent (proven by cell_9, tests/persistence.rs:331-340), and `self.form_state` stays intact for the sweep; or (b) if `mem::take` is used, the sweep MUST run over the moved map. Option (a) is the clean fix; spec should mandate it.

**I3 — Per-frame geometry snapshot must guard on `Some`, else minimize-then-quit loses geometry.**
egui-winit's `update_viewport_info` sets `inner_rect`/`outer_rect` to `None` while minimized (`has_a_position = match window.is_minimized() { Some(true) => false, … }`, egui-winit-0.31.1/src/lib.rs:~975; fields `Option<Rect>` at egui-0.31.1/src/data/input.rs:217-222). The app's 1 Hz keepalive thread (main.rs:139-146) guarantees `update()` frames keep firing while minimized, so a naive unconditional snapshot overwrites the last good geometry with `None`; minimize → close/SIGTERM then saves `None` and the next launch falls back to 920×720. **Fix:** spec must state the snapshot only overwrites the app fields when the rect is `Some` (`if let Some(r) = …`). (Same for `outer_rect`/position.)

**I4 — Env-seam test discipline unpinned: `MNEMONIC_GUI_STATE_PATH` reads/writes are process-global; the spec must state the isolation rule.**
Verified no test today calls `default_state_path` (sole occurrence src/persistence.rs:210), so nothing existing becomes env-sensitive — good. But T4 (`set_var` + assert override) and any future integration test driving the app's own path resolution share one process-global var per **test binary**; cargo runs `#[test]`s in parallel threads, and edition is 2021 (Cargo.toml:4 — `set_var` is safe-but-racy; on POSIX, `setenv` concurrent with `getenv` from another thread is genuinely hazardous). The repo already has a flaky-parallel precedent (`runner-tracing-test-flaky-under-parallel-load`, FOLLOWUPS.md). **Fix:** pin in the spec: each `tests/*.rs` file is its own process, so the env-seam test(s) live in a dedicated test file (or are the only env-touching cells in `tests/persistence.rs`), at most ONE test per binary mutates `MNEMONIC_GUI_STATE_PATH`, and T5 uses explicit `&Path` args to `save`/`load` (they already take paths — persistence.rs:176/193) rather than the env seam. No `serial_test` dep needed if that rule holds; say so explicitly.

## Minor

**M1 — Resolve the state path ONCE in `main()` and store it on the app.** If `on_exit` re-calls `default_state_path()`, load and save can diverge (env mutated mid-run, ProjectDirs edge). Spec should state: `main()` resolves `Option<PathBuf>`, uses it for `load()`, passes it into `new()`; `on_exit` saves to the stored path; `None` → skip save silently.

**M2 — Demo-seed vs restored `form_state` merge rule unspecified.** `new()` hardcodes the `"mnemonic:bundle"` seed (main.rs:221-236). Pin the rule: when a loaded state exists, restored map wins for keys it contains; seed applies only when the key is absent (or skip the seed entirely when load succeeded — either is fine; first-frame `or_default()` at main.rs:416-419 covers missing keys). Otherwise an implementer may re-seed over a user-emptied form.

**M3 — T2's "lib fn OR `#[cfg(test)]` main-adjacent unit" should be pinned to the lib.** `MnemonicGuiApp` is bin-private (verified: no test constructs it; kittest cells drive closures — tests/widget_interaction.rs:241); `tests/` integration cells can only see `mnemonic_gui::*`, and bin-crate unit tests need `cargo test --bin` (the cycle-B lesson). A lib helper like `fn restore_selections(&PersistedState, avail: impl Fn(CliTab)->bool) -> (CliTab, BTreeMap<CliTab, String>)` keeps T2/T5 in `tests/`. Also note `schema_for` is a private bin method (main.rs:252-259); the lib helper replicates it from `schema::{mnemonic,md,ms,mk}::SCHEMA`.

**M4 — Double-instance + torn-write note.** Two concurrent GUIs → last-writer-wins at exit (benign, but worth one README/FOLLOWUP line). `save()` uses non-atomic `fs::write` (persistence.rs:186); a write torn by the signal handler's 3 s `process::exit(130)` grace (main.rs:181-183) yields malformed JSON — which the new `.bak`-on-malformed leg now handles gracefully (nice synergy; worth stating). Atomic temp+rename = optional follow-up, not this cycle.

**M5 — HiDPI position fidelity caveat.** Captured `outer_rect.min` is in points; `with_position` is multiplied by creation-time `pixels_per_point` (egui-winit-0.31.1/src/lib.rs:1691), which on multi-monitor/mixed-DPI setups may differ from capture-time ppp → small offset. Same class as the already-accepted clamping risk; extend the §Risks bullet.

**M6 — Maximized state is not persisted** (no field in `PersistedState`). Close-while-maximized restores an unmaximized window at maximized size. Acceptable; add one line to Risks/README so it's a decision, not an omission.

**M7 — Docs/FOLLOWUPS nits.** (a) `MNEMONIC_GUI_STATE_PATH` becomes user-visible production behavior, not just a test seam — document in README alongside "delete state.json to reset", not only the fn doc. (b) `persistence-unwired-redaction-never-runs` at FOLLOWUPS.md:26 is a **bullet inside the audit-backlog index entry**, not a standalone entry with its own Status line — P3's "resolve the [obs]" should specify dispositioning the index bullet (and/or filing the resolution where the backlog report indexes it). (c) serde-other cross-cite target verified live at FOLLOWUPS.md:534; README self-pin at :42 verified.

## Verified claims (no finding)

- main.rs anchors all check out: `run_native` + fixed `with_inner_size([920,720])` :38-48; `new(cc)` :102-250; defaults :202-206; toggles :244-246 (+fields :80-82, checkboxes :303-305); `on_exit(&mut self)` no-ctx :900-906; SIGINT/SIGTERM → `ViewportCommand::Close` :161-200.
- Key-scheme identity confirmed: `form_key` = `"{bin_name}:{sub}"` (main.rs:261-263) ≡ `form_state_per_subcommand` key form (persistence.rs:57). Direct move is sound.
- `tab_available` exists (src/app.rs:97-99); `CliTab::bin_name` :27-34 (no inverse — `from_bin_name` genuinely new); `schema_for` exists (main.rs:252-259).
- egui 0.31.1 API confirmed: `ViewportInfo.inner_rect`/`outer_rect: Option<Rect>` in points, `InputState::viewport()`, `ViewportBuilder::with_position` (maps to winit outer position). `window_size` from `inner_rect.size()` is symmetric with `with_inner_size`.
- `.bak`-on-malformed breaks nothing: cell_6 asserts only `is_none()`; cell_5 unaffected; cell_4 untouched. T3's "RED today" claim correct.
- `secret_widgets` never restores, by type; already pinned both directions (tests/persistence.rs:376-428).
- No kittest/integration test constructs `MnemonicGuiApp` → `new()` signature change test-safe. `App::save` without the persistence feature is inert.
- SemVer MINOR correct; phase split + TDD ordering sound.

## Verdict

**YELLOW — 0 Critical / 4 Important / 7 Minor.** Fold I1–I4 and re-dispatch. The wiring design itself is architecturally right — load-in-`main()` before `run_native`, per-frame snapshot, exit-only save, hand-rolled persistence — and all recon-inherited anchors verified clean.
