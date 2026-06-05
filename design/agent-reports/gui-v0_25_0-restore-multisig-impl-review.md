# Per-phase implementation review — mnemonic-gui v0.25.0 (restore multisig flags + toolkit-v0.44.0 pin)

**Reviewer:** opus `feature-dev:code-reviewer` (gate before tag).
**Date:** 2026-06-05.
**Branch:** `gui-v0.25.0-restore-multisig-flags` vs `master` (commits `ea0778f` design, `ba20080` Phase 2, `b8b21df` Phase 3).
**Verdict:** **0 Critical / 0 Important — GREEN.** (2 Minors, both pre-existing / out-of-diff.)

> Persisted verbatim per CLAUDE.md before folding. Fold note at end.

---

**Scope reviewed:** Branch vs `master`. Verified statically against working-tree source on both repos, internal pin coherence, the GUI commit topology, and the toolkit `restore.rs` clap surface + `flag_is_secret`.

**Caveat (stated plainly):** No Bash tool in this environment — I could NOT run `git diff master...branch`, the `mnemonic` binary, `gui-schema`, or the test suite. I verified the end-state of the relevant files, internal pin coherence (Cargo.toml == pinned-upstream.toml == Cargo.lock), and confirmed the branch's commit topology via the GUI reflog. The load-bearing gates (`schema_mirror`, `schema_mirror_secret_drift`, `gui_schema_conditional_drift`, `pin_coherence`) all exec the real pinned binary and would catch any empirical mismatch I couldn't reproduce here.

## Critical
None.

## Important
None.

## Minor
- **README pin block frozen at the v0.22.0 snapshot — PRE-EXISTING, not in this diff.** `README.md:42` (`mnemonic-gui-v0.22.0`) and `:50` (`mnemonic-toolkit-v0.41.0`) are a coherent v0.22.0-era snapshot (lines 50-53 match the v0.22.0 CHANGELOG bump set verbatim), drifted untouched through v0.23.0/v0.24.0/v0.25.0. The claim at `:47` ("pinned tags match `pinned-upstream.toml`") has been false since v0.23.0. This cycle did not touch the README (line 42 would read v0.25.0 if it had), so it is outside `master...branch` and ungated (no `readme_version`-style guard exists in the GUI repo, unlike the toolkit's `readme_version_current.rs`). Confidence the drift is real: high; confidence it is *this cycle's* defect: ~0 (pre-existing). **Fix:** out of scope for this gate — recommend a FOLLOWUP for a README-install-tag coherence guard (or a backfill) so it stops drifting silently.
- **SPEC SHA-provenance citation likely stale.** `design/SPEC_gui_v0_25_0_restore_multisig_flags.md:5` cites toolkit `4d0523a` as tag `mnemonic-toolkit-v0.44.0`, but `Cargo.lock:2297` resolves the tag to commit `aa0f1e0…`. This is a design-artifact provenance note, not shipped code; internal pin coherence (the load-bearing invariant) is clean. Minor at most. (The local tag *ref* `13d5258` differing from the Cargo.lock commit is expected annotated-tag-object behavior, not a defect.)

## What verified clean
- **Flag-name parity (1).** `RESTORE_FLAGS` flag-name set exactly equals toolkit `RestoreArgs` (`restore.rs:54-146`) + global `--no-auto-repair`. No missing/extra/typo. `--md1`/`--cosigner` are `FlagKind::Text, required:false, repeating:true, secret:false, global:false`, before `NO_AUTO_REPAIR_FLAG`. `--from` is `required:false`. ACCURATE.
- **Conditional fn (2).** `conditional::restore` returns `[("--from", Required)]` iff `!has_value("--md1")`, else empty — valid API use, matches `verify_bundle`/`three_way_card_at_least_one`. `restore` SubcommandSchema `conditional = Some(crate::form::conditional::restore)`. ACCURATE.
- **Conditional soundness (3).** Toolkit `--from` is `required_unless_present = "md1"` (NOT `_any`). GUI relaxes `--from` ONLY on `--md1`; `--cosigner` alone does NOT relax it. Faithful mirror. `build_subcommand_conditional_rules` (`gui_schema.rs:336-344`) has no restore arm → restore emits `[]` → drift gate skips it (`:228-230`). ACCURATE.
- **Pin/version coherence (4).** `Cargo.toml` `0.25.0` + tag `v0.44.0`; `pinned-upstream.toml [mnemonic].tag v0.44.0`; `Cargo.lock` resolves `mnemonic-toolkit 0.44.0`. `pinned_version: "mnemonic 0.44.0"`; module-doc `v0.44.0`. The two `0.43.0` hits (`mnemonic.rs:344,3435`) are intentional historical-provenance comments. Sibling pins md 0.6.2 / ms 0.7.0 / mk 0.7.0 unchanged + correct. ACCURATE.
- **CHANGELOG (5).** `[0.25.0]` factually matches the diff; no overclaim. ACCURATE.
- **Scope/lockstep (6).** No `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` delta (both flags non-secret on the toolkit side; `secrets.rs:49-64`/`88-113`), so `schema_mirror_secret_drift` unaffected. Coverage guard adds `restore` to `is_some()` (`conditional_visibility.rs:330`). Two new cells well-formed. No restore flag-count assertion, no argv-assembler restore flag-list test, **no kittest restore-form snapshot**. The "restore" hits in `argv_assembler_visibility.rs:197`/`widget_unset_sentinel.rs:176`/`persistence.rs:295` are incidental (comment / loop var). ACCURATE.
- **manual-gui** correctly untouched (pinned v0.3.0, `#[ignore]`-gated) per SPEC §8.

## VERDICT
**0 Critical / 0 Important — GREEN.** Two Minors (pre-existing README pin-block staleness; possibly-stale SPEC SHA provenance), both out of this cycle's diff and ungated — fold into a FOLLOWUP, not a fix-in-cycle. Cleared to tag, with the static-only caveat noted: confirm the four binary-backed gates pass against the four pinned binaries in CI before pushing the tag.

---

## Fold note (applied after persisting)

- **Minor 2 (SPEC SHA provenance) — FOLDED.** SPEC header clarified: the pinned v0.44.0 tag commit is `aa0f1e0` (what `Cargo.lock` resolves); `4d0523a` was the toolkit master tip at SPEC-write time (the post-tag manual anchor-dangler fix, which does not touch `restore.rs`/`gui_schema.rs`, so the verified source is identical). Both cited.
- **Minor 1 (README pin-block staleness) — FOLLOWUP filed**, NOT fixed in-cycle (pre-existing since v0.23.0, ungated, out of `master...branch`; per reviewer recommendation + scope discipline). Filed as toolkit-registry entry `gui-readme-install-pin-coherence-guard` (the GUI repo has no FOLLOWUPS.md; the toolkit registry already tracks GUI lockstep items). Empirically: the four binary-backed gates were run GREEN against the four pinned binaries (`MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN`, `+1.94.0`) during Phase 2/3 — the reviewer's static-only caveat is discharged by that run + CI.
