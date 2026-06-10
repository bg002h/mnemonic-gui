# R0 round-3 architect review — SPEC_gui_v0_36_0_autosave_atomic (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 3, post-fold verification). GUI 1244b7c. Verdict: GREEN (0 Critical / 0 Important / 2 audit-trail Minors). Review verbatim below.

---

## Critical

None.

## Important

None.

## Fold-verification

**I3 (R0-r2) — VERIFIED FOLDED, CORRECT AND UNAMBIGUOUS.** T1 (spec line 26) now pre-creates "the EXACT D1 name, computed in-test as `format!("state.json.{}.tmp", std::process::id())`" with the preserved rationale. Name agreement with D1 confirmed (every existing persistence cell builds its destination as `dir.path().join("state.json")`). RED-ability survives: current `save()` is a direct `fs::write` → PID-named garbage remains pre-impl → RED; consumed by rename post-impl → GREEN. The reverse-I1 trap is structurally closed (the test's name includes the PID, so only a PID-suffixed implementation can consume it). The same-process glob clause additionally catches a wrong-named tmp.

**M8 (R0-r2) — VERIFIED LANDED AND ACCURATE.** D2 carries the BTreeMap-determinism parenthetical; verified against source: `last_subcommand_per_tab` + `form_state_per_subcommand` are BTreeMap (persistence.rs:49, :59); nested FormState fully deterministic (values Vec, positionals Vec, slots Vec; secret_widgets serde-skip). No HashMap anywhere in the serialized closure.

**Round-1 folds (I1, I2, M1/M2/M4/M5/M6) — re-spot-checked, all intact post-round-3.** D1's PID-suffix prose, the "NO remove-then-rename fallback, ever" contract, orphan policy, power-loss contract, 11-cell count (confirmed exactly 11 `fn cell_` tests + the wiring T5 exercising save()), cache-after-Ok, eframe-native non-goal (persistence feature absent, Cargo.toml:13) — present and consistent.

## Minor

**M10 (audit trail only)** — T1's `format!` hardcodes the `state.json` destination basename; consistent because the test controls the name and all cells use `state.json`. A future basename refactor must track it.

**M11 (audit trail only)** — D2 call-shape borrow-checks (disjoint field borrows; `build_persisted_state(&self)` returns owned); the on_exit construction is all `&self` reads (main.rs:999-1018) with the two invariant comments at :986-998 moving verbatim. FOLLOWUPS slug confirmed (FOLLOWUPS.md:534). Timer/keepalive anchors re-verified. No `AUTOSAVE_INTERVAL`/`last_autosave` symbols pre-exist — clean additions.

## Verdict

**GREEN — 0 Critical / 0 Important.** Both round-2 findings folded correctly; whole-spec scan clean; the design is internally consistent and matches the source at 1244b7c. Implementation may begin.
