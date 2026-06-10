# SPEC — GUI v0.35.0: Phase-8 persistence WIRING (state.json load/save lifecycle)

**Cycle:** mnemonic-gui v0.35.0 (MINOR) · **Source SHA:** `1a1615a` (= v0.34.0) · **Recon:** `cycle-prep-recon-phase8-persistence-wiring.md` (repo root — the detailed gap map; this spec adopts its §6 recommendations as decisions)
**Design source:** §10 of the converged v0.1 plan (`/home/bcg/.claude/plans/declarative-tumbling-shell.md`, R2 0C/0I) — the persistence MODULE is feature-complete and v0.31.1→v0.34.0-hardened; this cycle is **wiring only**.
**Resolves:** `persistence-unwired-redaction-never-runs` [obs] (FOLLOWUPS.md:26). **Preconditions:** I4 (v0.33.0) + I5/I6 (v0.34.0) all explicitly cleared — "Phase-8 persistence wiring is now UNBLOCKED".

## Decisions (recon §6, fixed here)

1. **Load placement:** `main()` resolves the state path ONCE (`Option<PathBuf>`), calls `load()` BEFORE `run_native`; loaded `window_size`/`window_position` seed `ViewportBuilder` (geometry cannot be applied later without flicker); the `PersistedState` AND the resolved path move into the `MnemonicGuiApp::new` closure — `on_exit` saves to the STORED path (re-resolving could diverge if env mutated mid-run; `None` → skip save silently; R0-r1 M1). Restore is by-value move (FormState is not Clone — fine, load yields owned data).
2. **Restore mapping in `new()`** (6 field groups):
   - `last_cli_tab` → `CliTab::from_bin_name` (NEW lib fn — inverse of `bin_name()`); unknown → `Mnemonic`; restore only if `tab_available`, else default.
   - `last_subcommand_per_tab` → validate each against `schema_for(tab).subcommands`; invalid/missing → the current hardcoded defaults (`bundle`/`inspect`).
   - `form_state_per_subcommand` → direct field move (identical `"cli:sub"` key scheme). Stale flag names in restored `values` are LEFT INERT (render + argv are schema-driven; no prune-on-load — zero-risk; recorded). **Demo-seed merge rule (R0-r1 M2):** the restored map wins for keys it contains; the hardcoded `mnemonic:bundle` demo seed applies ONLY when that key is absent from the restored map (never re-seed over a user-emptied form).
   - 3 output-pane toggles → direct.
3. **Geometry capture:** per-frame snapshot in `update()` (`ctx.input(|i| i.viewport().inner_rect/outer_rect)` → app fields); `on_exit` has no ctx. **`Some`-guarded (R0-r1 I3):** egui-winit sets both rects to `None` while MINIMIZED, and the 1 Hz keepalive thread keeps frames firing — an unconditional snapshot would overwrite good geometry with None (minimize→quit loses geometry). Only overwrite when the rect is `Some`. **Wayland:** outer position is compositor-private → `window_position` stays `None` there and `with_position` is a no-op — documented, not fought.
4. **Save cadence:** `on_exit` ONLY (the SIGINT/SIGTERM handlers already route through `ViewportCommand::Close` → `on_exit` runs). Order: **save FIRST, then the zeroize sweep — LOAD-BEARING (R0-r1 I1):** `zeroize_form_state` (secrets.rs:278-310) blanks every STRING-BEARING value (Text/Dropdown/Path/composite + all slot rows + positionals; Number/Bool/Unset untouched — R0-r2 M-NEW2), not just secrets — zeroize-before-save would persist an all-blank state. Pin the order with a comment at the on_exit call site. **Save-side construction (R0-r1 I2 — do NOT `mem::take`):** build the map borrow-side — `self.form_state.iter().map(|(k,v)| (k.clone(), redact_for_persistence(v))).collect()` (`redact_for_persistence` returns owned without Clone; `save()`'s internal re-redaction is idempotent, cell_9) — so `self.form_state` stays intact for the zeroize sweep; `mem::take` would silently turn the sweep into a no-op (secrets never zeroized at exit). Debounced autosave = follow-up FOLLOWUP (`gui-persistence-autosave-debounce`), not this cycle.
5. **`.bak`-on-malformed symmetry:** `load()` gains the same `rename → .json.bak` treatment for JSON-parse failure that version-mismatch already has (corrupt file preserved for diagnosis instead of silently overwritten at next save). Missing-file stays plain `None`.
6. **Test seam:** `default_state_path()` honors a `MNEMONIC_GUI_STATE_PATH` env override (smallest seam; integration tests never touch the real config dir). Document in the fn doc AND the README (it is user-visible production behavior — R0-r1 M7a). **Isolation rule (R0-r1 I4):** env mutation is process-global and cargo test threads share it — the env-seam cell(s) live in a DEDICATED tests/*.rs file (own process), at most ONE test per binary mutates the var, and T5 uses explicit `&Path` args to save/load (they already take paths) — never the env seam. No serial_test dep needed under that rule.
7. **NO eframe `persistence` feature, NO new deps** (`directories` already direct), NO redaction or schema changes, no `schema_mirror`/pin impact, no toolkit companion.

## Tests (TDD red-first where the behavior is new)

- **T1 (lib):** `CliTab::from_bin_name` round-trips all 4 + unknown→None (or fallback semantics — pick Option<CliTab>, caller defaults).
- **T2 (restore validation):** stale subcommand falls back to default; stale tab falls back to Mnemonic; valid entries restore. **Pinned to the LIB (R0-r1 M3):** a lib helper like `fn restore_selections(&PersistedState, avail: impl Fn(CliTab) -> bool) -> (CliTab, BTreeMap<CliTab, String>)` — `MnemonicGuiApp` is bin-private and `schema_for` is a private bin method; the helper replicates schema lookup from `schema::{mnemonic,md,ms,mk}::SCHEMA` so T2/T5 stay in `tests/`.
- **T3 (`.bak`-on-malformed):** malformed JSON → file renamed `.json.bak` + `None` (RED today: current code returns None leaving the file in place); version-mismatch leg re-asserted (existing cell_4 stays).
- **T4 (env seam):** `MNEMONIC_GUI_STATE_PATH` overrides `default_state_path`.
- **T5 (end-to-end round-trip):** build a PersistedState with toggles off + a non-secret form value + tab/subcommand selections → `save` → `load` → restore mapping → assert all six groups land (and that a seeded secret_widgets value did NOT survive — re-pins the type-level invariant at the wiring layer).
- Existing `tests/persistence.rs` (12 cells) + `tests/persist_redaction_v0_34_0.rs` (8 cells) stay green (R0-r2 M-NEW1 corrected the count).

## Phases

1. **P1:** lib helpers (`from_bin_name`, restore-validation, `.bak` symmetry, env seam) + T1-T4.
2. **P2:** `main()` load + ViewportBuilder seeding + `new()` mapping + per-frame geometry snapshot + `on_exit` save-then-zeroize + T5; full suite (`MNEMONIC_BIN`/`MS_BIN`/`MK_BIN`/`MD_BIN` env discipline) + clippy.
3. **P3:** docs (README "session restore + delete state.json to reset" + Wayland caveat; CHANGELOG `[0.35.0]`; version bump Cargo.toml/Cargo.lock/README self-pin :42) + FOLLOWUPS (the [obs] is a BULLET inside the audit-backlog index entry at FOLLOWUPS.md:26, not a standalone entry — disposition the bullet in place (R0-r1 M7b); file `gui-persistence-autosave-debounce` (incl. the atomic temp+rename note); cross-cite `gui-flag-value-unset-serde-other-externally-tagged-dependency` (:534) as now-live-relevant) → push → CI → tag `mnemonic-gui-v0.35.0`.

## Risks

- Restored geometry off-screen after monitor changes → eframe clamps to visible area on most platforms; accept (egui-native behavior); note in README. HiDPI/mixed-DPI position offset (with_position scales by creation-time pixels_per_point — R0-r1 M5) and unmaximized-restore-at-maximized-size (no maximized field — M6): same accepted class, one README line each.
- Double-instance last-writer-wins + non-atomic `fs::write` torn by the signal handler's 3s grace → malformed JSON, now handled gracefully by the new `.bak`-on-malformed leg (synergy; R0-r1 M4). Atomic temp+rename = optional follow-up note in the autosave FOLLOWUP.
- First-frame template-aware seed vs restored values: seed-on-empty only — restored values never overwritten (recon §3, verified).
- Crash (SIGKILL) loses session state — accepted for exit-only cadence; the autosave FOLLOWUP records the upgrade path.

## Non-goals

Autosave/debounce; pruning stale values; eframe-native persistence; egui memory persistence; capture-protection changes; any redaction change.
