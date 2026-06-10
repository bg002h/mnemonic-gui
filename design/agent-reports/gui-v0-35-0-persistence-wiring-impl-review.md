# Implementation review — GUI v0.35.0 persistence wiring (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Spec @ design/SPEC_gui_v0_35_0_persistence_wiring.md (R0 GREEN r2). Verdict: GREEN (0 Critical / 0 Important / 4 Minor — ALL folded post-review: T4 empty-string leg added, CHANGELOG RED-first wording corrected, cell_6 doc note, README "at the next exit"). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

1. **T4 doesn't exercise the empty-string guard.** `default_state_path()` deliberately ignores `MNEMONIC_GUI_STATE_PATH=""` (`src/persistence.rs:290-294`), but `tests/persistence_env_seam.rs:13-31` only tests set→override and unset→fallback. The empty-string leg is untested. Fix: add a third sequential assertion inside the SAME test fn (`set_var(..., "")` → assert fallback) — still one mutator per binary, so the R0-r1 I4 isolation rule holds.
2. **CHANGELOG wording slightly overstates RED-first scope.** `CHANGELOG.md` [0.35.0] tests bullet says "`.bak`-on-malformed RED-first ×3". Empirically only `t3_malformed_json_renames_to_bak_and_returns_none` is RED-first (verified: scratch-reverting the `load()` hunk reds exactly that cell at `tests/persistence_wiring_v0_35_0.rs:132`; the version-mismatch and missing-file legs were green pre-change). Fix: "RED-first ×1 + 2 symmetry/missing-file re-pins" or just drop "×3" after "RED-first".
3. **Pre-existing `cell_6_malformed_json_yields_none` (`tests/persistence.rs:278-283`) now under-describes behavior** — it still passes (asserts only `None`) but the load it exercises now also renames the file to `.bak`. Comment-only drift; a one-line doc note would prevent future confusion. No behavior issue.
4. **README "a fresh default is written" is temporally loose**: after a `.bak` rename, the fresh default is written *at next exit*, not at load time. "and a fresh default is written at next exit" would be exact.

## Verdict

**GREEN** — 0 Critical / 0 Important. Evidence per the five review axes:

**1. Lib (P1)** — all four deliverables present and correct:
- `CliTab::from_bin_name` (`src/app.rs:37-46`): exact inverse of `bin_name()`, unknown → `None`, doc'd; T1 round-trips all 4 via `CliTab::ALL` + rejects unknown/empty/case-mismatch.
- `restore_selections` (`src/persistence.rs:192-218`): tab parse → `.filter(avail)` → `unwrap_or(Mnemonic)`; per-tab subcommand validated against the replicated `schema_for` lookup (`:173-180`); every tab gets an entry via `CliTab::ALL` iteration with `default_subcommand` fallback (`bundle`/`inspect`, `:185-190`) — `t2_default_persisted_state_yields_hardcoded_defaults` pins the None-loaded/default-loaded convergence.
- `.bak`-on-malformed (`src/persistence.rs:262-269`): symmetric with the version-mismatch leg; missing file stays plain `None` (the `.ok()?` at `:259` short-circuits before any rename — pinned by `t3_missing_file_stays_plain_none_no_bak`).
- Env seam (`src/persistence.rs:289-294`): **non-empty guard present**; documented in the fn doc AND README.

**2. Wiring (P2) — all four spec-critical invariants verified in main.rs:**
- (a) Path resolved ONCE: `main.rs:45`, moved into `new()` and stored as `self.state_path`; `on_exit` uses `if let Some(path) = &self.state_path` — `None` → skip, no re-resolution anywhere.
- (b) Borrow-side construction: `on_exit` builds `PersistedState` via `self.form_state.iter().map(|(k,v)| (k.clone(), persistence::redact_for_persistence(v)))` — no `mem::take`; save runs BEFORE the zeroize sweep with the LOAD-BEARING comment at the call site (and a back-reference comment on the sweep itself). `save()` re-stamps `SCHEMA_VERSION` and `create_dir_all`s the parent so the fresh-install path works.
- (c) Geometry snapshot in `update()` is `Some`-guarded on both rects, with fields seeded from the loaded state so Wayland/minimized sessions re-persist prior values instead of dropping them.
- (d) Demo seed gated on `!form_state.contains_key("mnemonic:bundle")`; the three toggles extracted from `&loaded` BEFORE `unwrap_or_default()` with the Default-vs-serde-default trap explicitly commented (None → `(true,true,true)`).

**3. Tests** — 10 cells in `tests/persistence_wiring_v0_35_0.rs` (T1×1, T2×5, T3×3, T5×1) + 1 in `tests/persistence_env_seam.rs` (T4) = 10+1, matching spec. T5 uses explicit `&Path` (tempdir), never the env seam; grep confirms `persistence_env_seam.rs` is the SOLE env mutator across tests/+src/ and no test executes the GUI binary. **RED-first verified empirically**: reverted the `load()` malformed hunk in scratch → exactly `t3_malformed_json_renames_to_bak_and_returns_none` FAILED; restored byte-identical (sha256 `9df87fbc…` before == after).

**4. Docs** — README session-restore section: correct path, reset-via-delete + `.bak` note, env-var override, all 4 caveats. CHANGELOG [0.35.0] claims all verified true of the diff (modulo Minor 2 wording). FOLLOWUPS: the `[obs]` bullet dispositioned in place; `gui-persistence-autosave-debounce` filed with the atomic temp+rename note and the never-`mem::take` warning; serde-other entry carries the now-live-relevant note. Version 0.35.0 at Cargo.toml:3, Cargo.lock, README self-pin.

**5. Suite + clippy + release smoke** — full suite with the pinned-binary env discipline: **52/52 test binaries ok, 0 failures** (1 pre-existing `#[ignore]`), incl. `tests/persistence.rs` (12) and `tests/persist_redaction_v0_34_0.rs` (8). `cargo clippy --all-targets -- -D warnings`: clean. `cargo build --release`: green (mnemonic-gui v0.35.0).

Tree left exactly as found.
