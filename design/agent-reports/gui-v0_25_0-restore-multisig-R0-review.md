# R0 Architect Review — SPEC mnemonic-gui v0.25.0 (`restore` multisig flags + toolkit-v0.44.0 pin)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate).
**Date:** 2026-06-05.
**SPEC:** `design/SPEC_gui_v0_25_0_restore_multisig_flags.md`.
**Verdict:** **0 Critical / 0 Important — GREEN.** (2 non-blocking Minors.)

> Persisted verbatim per CLAUDE.md BEFORE folding. The 2 Minors are folded into the SPEC/plan afterward (see fold note at end). GREEN ⇒ no re-dispatch required; implementation may proceed.

---

Reviewed `/scratch/code/shibboleth/mnemonic-gui/design/SPEC_gui_v0_25_0_restore_multisig_flags.md` in full against current source in both repos. I read the GUI schema definitions, the conditional engine, the FormState API, every relevant test, the toolkit `restore.rs` + `gui_schema.rs` emitter, and the toolkit FOLLOWUPS registry. I could not directly execute the v0.44.0 binary (no shell), so the one runtime-emission claim was instead resolved by reading the toolkit's emitter source — which is dispositive (see "Resolved decisive claim" below).

## Critical

None.

## Important

None.

## Minor

1. **Add `restore` to the `coverage_all_constrained_subcommands_have_conditional_fn` positive list.** (`tests/conditional_visibility.rs:316-345`, esp. the `is_some()` list at :322-330.) This test enumerates constrained subcommands in an explicit allowlist and asserts each has a conditional fn, plus an explicit denylist (`final-word`, `seed-xor-split`, `seed-xor-combine` at :338) asserting `is_none()`. `restore` is currently in **neither** list, so flipping it to `Some(restore)` per §3.3/§4 does NOT fail this test — but for completeness and to keep the coverage guard meaningful, restore (now a constrained subcommand) should be added to the `is_some()` list. SPEC §6 does not mention this test; it should. Non-blocking because the suite stays GREEN either way.

2. **The repair/inspect analogy in §4 is loosely stated but the conclusion is correct.** §4 frames restore's gate-safety via the repair/inspect precedent ("same posture as the GUI-authored repair/inspect at-least-one rules"). That analogy is imperfect: repair/inspect emit `[]` because toolkit D35 *dropped* their clap required-group (no clap attribute at all), whereas restore.rs:60 carries a real `required_unless_present="md1"` attribute. The *reason* restore still emits `[]` is different (the toolkit's `conditional_rules` projection is a hand-encoded allowlist that has no restore arm — not "it has no clap constraint"). The SPEC's actual stated mechanism in the §4 NB ("its projection does not capture `required_unless_present`") is accurate; only the surrounding "same shape as repair/inspect" framing is slightly off. Optional: tighten the comment to cite the hand-encoded `build_subcommand_conditional_rules` allowlist (gui_schema.rs:336-345) as the real reason.

## Resolved decisive claim (the one I flagged for extra scrutiny)

The load-bearing, binary-unverifiable claim — **"restore emits `conditional_rules: []` at v0.44.0"** (§3.3, §4 NB, §4 gate-safety bullet) — is **ACCURATE**, confirmed from toolkit source rather than the SPEC's self-assertion:

- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:336-345` — `build_subcommand_conditional_rules` is a hand-encoded `match name { ... }` with arms ONLY for `bundle`, `verify-bundle`, `export-wallet`, `convert`, `derive-child`, `compare-cost`. `restore` falls through to `_ => Vec::new()` (:344). The projection is NOT derived from clap's `required_unless_present` attribute; it is a hand-maintained allowlist, and restore is not on it.
- Therefore the §4 NB ("the toolkit gui-schema emits `conditional_rules: []` for restore … its projection does not capture `required_unless_present`") is correct, the drift gate skips restore (empty rules → `continue` at `gui_schema_conditional_drift.rs:228-230`), and the new toolkit FOLLOWUP in §7 ("promote restore's `--from required_unless_present` to a toolkit-emitted + drift-gated rule") is correctly framed as genuine future work, not a no-op.

## Design decision (Option B vs Option A) — sound

Option B (GUI-authored `restore()` at-least-one rule marking `--from` Required unless `--md1`) is the better call and is gate-safe:
- It typechecks exactly against the real API (verified below).
- It passes the drift gate whether or not the toolkit emits a rule: empty toolkit rules → restore skipped; were a rule ever emitted, the synthesized `Not(--md1)`→empty state → `restore(empty)` → `[("--from", Required)]` would match. No orphan-direction check exists, and `SUBCOMMAND_FLOORS` (`gui_schema_conditional_drift.rs:300-306`) omits restore, so no floor trips.
- It mirrors the established `verify-bundle` `required_unless_present` modeling (Required-on-Not(present), conditional.rs:421-425) — the correct precedent, which the SPEC's `restore()` fn body matches in shape.
- No test asserts "`conditional: Some` iff toolkit emits non-empty rules" — the only such enumerations are the explicit allow/deny lists in `coverage_all_constrained_subcommands_have_conditional_fn`, which don't list restore (see Minor 1).

## What I verified clean (with evidence)

- **§3.1 insertion point + FlagSchema field set.** `RESTORE_FLAGS` (mnemonic.rs:355-511) ends with `NO_AUTO_REPAIR_FLAG,` at :510; inserting the two literals before it keeps the global flag last (matches every other array). The `FlagSchema` struct (mod.rs:64-110) has exactly the fields the SPEC literals use, in order: `name, kind, required, repeating, help, secret, default_value, global`. `FlagKind::Text` exists (mod.rs:116). The SPEC literals will compile.
- **§3.1 secret/repeating semantics.** Both new flags `secret:false` is correct (toolkit `flag_is_secret` → false for non-passphrase restore flags; only `--passphrase`/`--passphrase-stdin` are secret, mnemonic.rs:467-487). `repeating:true` mirrors the toolkit `Vec<String>` shape (same as `--md1` in verify-bundle, mnemonic.rs:623-632). No `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` delta; `schema_mirror_secret_drift` unaffected.
- **§3.2 `--from` flip.** GUI currently has `--from required:true` (mnemonic.rs:357-366). Toolkit `RestoreArgs.from: Option<String>` with `#[arg(long, required_unless_present = "md1")]` (restore.rs:60-61). Flip to `required:false` is correct.
- **§3.3 / §4 conditional API typechecks.** `SubcommandSchema.conditional: Option<fn(&FormState) -> FlagVisibility>` (mod.rs:47). `FlagVisibility = Vec<(&'static str, Visibility)>` (mod.rs:226). `Visibility::Required` exists (mod.rs:206). `FormState::has_value(&str) -> bool` exists (mod.rs:304-312). The proposed `restore()` body (`if !state.has_value("--md1") { vis.push(("--from", Visibility::Required)); }`) compiles. restore's SubcommandSchema currently has `conditional: None` (mnemonic.rs:3411).
- **§5 pins/version.** `Cargo.toml [dependencies].mnemonic-toolkit.tag = "mnemonic-toolkit-v0.43.0"` (Cargo.toml:42); `version = "0.24.0"` (:3). `pinned-upstream.toml [mnemonic].tag = "mnemonic-toolkit-v0.43.0"` (:22). `pin_coherence.rs:24-37` asserts the two agree. Sibling pins md 0.6.2 / ms 0.7.0 / mk 0.7.0 are current (pinned-upstream.toml:39,47,53) — no bump needed, matching the SPEC.
- **§5 exact strings to bump.** `mnemonic.rs:1` module-doc (`…from mnemonic-toolkit-v0.43.0.`), `:3634` `pinned_version: "mnemonic 0.43.0"`, and the two cosmetic comments at `:344` and `:3401-3404` carry `v0.43.0`. (Note the `:3404` comment "gui-schema emits `conditional_rules: []` for restore → conditional: None" must be updated per §3.3 to record the v0.25.0 GUI-authored rule — SPEC §3.3 already calls this out.)
- **§6 test cell shape.** `conditional_visibility.rs` provides `run_conditional(name, &state)` (:32-36), `FormState::default()`, `FormState::from_pairs(vec![(...)])`, and `vis_of` (:62-69). The `repair`/`inspect` cells (:933-1062) are exactly the shape the SPEC's two restore cells mirror (`FormState::default()` → Required; `from_pairs([("--md1", Text(...))])` → not-Required-falls-through-to-Visible). The SPEC's assertions are expressible.
- **§6 no edits to schema_mirror / pin_coherence / gui_schema_conditional_drift.** `schema_mirror.rs` has no hardcoded restore-flag list (no `restore` match); it parses `--help` vs the GUI array, so the two new flags go GREEN once §3 lands. `gui_schema_conditional_drift.rs` skips empty-rule subcommands (:228) — restore stays skipped. `pin_coherence.rs` goes GREEN after §5. No edits needed.
- **§7 FOLLOWUP exists + open.** `mnemonic-toolkit/design/FOLLOWUPS.md:80` — `gui-restore-multisig-flags-pending-pin-bump`, Status `open` (this entry's body at :80-84 describes exactly the SPEC's plan: add `--md1`/`--cosigner` repeating `secret:false`, flip `--from` to `required:false`).
- **§8 out-of-scope holds.** `manual_anchor_coverage.rs:136` carries `#[ignore = "Requires Track M manual HTML build at $MANUAL_GUI_HTML_PATH…"]` — so it does not fire on this change.
- **No missed lockstep.** No README version-marker guard test exists in this repo (confirmed by the prior-cycle R0 artifacts + grep — only `pinned_version` string literals, no `readme_version_current`-style test). No `scripts/install.sh` self-pin (no `scripts/` dir). No widget/kittest snapshot pins restore's flag list (the three "restore" test hits in `argv_assembler_visibility.rs`/`persistence.rs`/`widget_unset_sentinel.rs` are the English word "restore"/"restored", not the subcommand). The two repeating-Text flags auto-render via the existing repeating-field widget; `--cosigner @N=` is plain text, no slot-editor coupling (slot-editor is `--slot`-only).

## VERDICT

**0 Critical / 0 Important — GREEN.**

The SPEC is accurate against current source on every load-bearing claim, including the one runtime-emission claim resolved from the toolkit emitter (`build_subcommand_conditional_rules` is a hand-encoded allowlist with no restore arm → restore emits `[]`). The two Minors (add restore to the coverage-guard `is_some()` list; tighten the §4 repair/inspect analogy comment) are non-blocking polish that the implementer may fold opportunistically. Implementation may proceed.

---

## Fold note (applied after persisting)

- **Minor 1 — FOLDED into SPEC §6:** added an explicit item to update `tests/conditional_visibility.rs::coverage_all_constrained_subcommands_have_conditional_fn` — add `restore` to the `is_some()` allowlist (Phase 2). Implementation will do this alongside wiring `conditional: Some(restore)`.
- **Minor 2 — FOLDED into SPEC §4:** the `restore()` fn doc-comment will cite the hand-encoded `build_subcommand_conditional_rules` allowlist (`gui_schema.rs:336-345`, no restore arm) as the real reason restore emits `[]`, rather than the looser "same shape as repair/inspect" framing. The closer precedent is `verify-bundle`'s `required_unless_present` modeling (`conditional.rs:421-425`).
- GREEN ⇒ no architect re-dispatch (the two folds are cosmetic SPEC/comment edits introducing no design drift).
