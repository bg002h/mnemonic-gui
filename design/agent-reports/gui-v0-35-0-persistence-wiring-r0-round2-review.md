# R0 round-2 architect review — SPEC_gui_v0_35_0_persistence_wiring (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). GUI 1a1615a. Verdict: GREEN (0 Critical / 0 Important / 2 cosmetic Minor — folded before P1). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

**M-NEW1 — Test-count nit in the Tests section: "Existing 19 persistence cells + the v0.34.0 suite stay green" miscounts/double-counts.** Measured at 1a1615a: `tests/persistence.rs` has **12** `#[test]` cells (cell_1–cell_11 + `secret_widgets_round_trip_never_persists_both_directions`); the v0.34.0 suite `tests/persist_redaction_v0_34_0.rs` has **8**. The recon's "19 tests" (recon §1 headline) was a module-wide total — if 19 already includes the v0.34.0 cells, the spec's "+ the v0.34.0 suite" double-counts; if not, persistence.rs doesn't have 19. Cosmetic — the operative P2 gate is the full suite — but reword to "tests/persistence.rs (12 cells) + tests/persist_redaction_v0_34_0.rs (8 cells) stay green" so the number doesn't decay into a false anchor.

**M-NEW2 — Decision 4's "blanks EVERY value" is technically overbroad.** `zeroize_form_state` (secrets.rs:278-310) has a `_ => {}` arm: `FlagValue::Number`/`Bool`/`Unset` are untouched (e.g. the demo seed's `--account Number(0)` would survive a wrong-order sweep). The enumerated parenthetical (Text/Dropdown/Path/composite + slot rows + positionals) is the accurate scope, and the load-bearing argument is unweakened (zeroize-before-save still persists a gutted form). One-word fix: "EVERY Text/Dropdown/Path/composite value" or "every string-bearing value". Cosmetic precision only.

## Fold-verification

All 11 round-1 findings folded correctly and completely; no fold-drift found.

- **I1 (save-then-zeroize LOAD-BEARING)** — FOLDED, Decision 4. The false "correctness-irrelevant" rationale is gone; mandates a pin-comment at the `on_exit` call site. Re-verified: `secrets::zeroize_form_state` at src/secrets.rs:278-310 zeroizes every `Text`/`Dropdown`/`Path` value (:281-284), `NodeValueComposite.value` (:286-287), all slot rows (:292-294), all positionals (:295-297), plus secret_widgets (:307-309).
- **I2 (borrow-side construction, no `mem::take`)** — FOLDED, Decision 4 with the exact iterator expression + the failure mode. Re-verified: `redact_for_persistence(&FormState) -> FormState` at persistence.rs:74 constructs owned without `Clone`; `save()` re-redacts via `redact_persisted_state` (:182); idempotence pinned by cell_9 (tests/persistence.rs:331-341). The exit sweep is real: main.rs:900-906.
- **I3 (Some-guarded geometry snapshot)** — FOLDED, Decision 3. Re-verified both legs: egui-winit-0.31.1 `update_viewport_info` (src/lib.rs:974-991) sets both rects `None` when minimized; the 1 Hz keepalive thread exists and keeps frames firing. Wayland caveat retained.
- **I4 (env-seam isolation rule)** — FOLDED, Decision 6: dedicated tests file, one mutating test per binary, T5 uses explicit `&Path` args (save/load take `&Path` at :176/:193). Complete.
- **M1** — FOLDED, Decision 1 (path resolved once, stored, None → silent skip).
- **M2** — FOLDED, Decision 2 bullet 3 (restored map wins; seed only when key absent — matches main.rs:221-237 + :416-419).
- **M3** — FOLDED, T2 (lib `restore_selections` signature, bin-private rationale, SCHEMA replication).
- **M4** — FOLDED, Risks bullet 2 (incl. .bak synergy + atomic temp+rename routed to the autosave FOLLOWUP).
- **M5** — FOLDED, Risks bullet 1. Re-verified egui-winit ~:1691 with_position × creation-time ppp.
- **M6** — FOLDED, Risks bullet 1; `PersistedState` confirmed to have no maximized field.
- **M7** — FOLDED. (a) Decision 6 README+fn-doc; (b) FOLLOWUPS.md:26 re-verified as the [obs] BULLET inside the audit-backlog index entry — disposition-in-place is the right shape; (c) :534 serde-other heading + README:42 self-pin both live.

**Whole-spec re-scan:** restored `FormState` deserialization safe by type; `FormState` non-Clone confirmed (derive at schema/mod.rs:290); the borrow-side snippet type-checks; T3's RED-today claim holds (load :195-198); T5 secret_widgets seeding feasible (precedent tests/persistence.rs:376); no `MNEMONIC_GUI_STATE_PATH` prior art collides. No new Critical/Important.

## Verdict

**GREEN — 0 Critical / 0 Important / 2 Minor (cosmetic, fold-at-will).** All 11 round-1 findings folded faithfully with no fold-induced drift; every factual claim the folds introduced re-verified against source. Implementation may begin.
