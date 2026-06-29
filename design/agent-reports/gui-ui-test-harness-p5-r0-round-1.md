# P5 R0 review — one-time proptest coverage SWEEP (round 1)

**Phase:** P5 (the bug-finder sweep) of the `mnemonic-gui` UI-test harness.
**Branch:** `feat/ui-harness-p0-spike` — P5 commit `194a163` (master untouched @ `da47994`).
**Scope reviewed:** `tests/ui_harness_sweep.rs` (636 ln, new), `tests/ui_harness/mod.rs`
seed-table extension (+138 ln, purely additive — zero deletions), `Cargo.toml`
(+`proptest = "1"` dev-dep + comment), `Cargo.lock` (proptest + transitive).
**Reviewer:** opus architect, adversarial; verified against `src/` + live runs.

---

## Verdict: **GREEN — 0 Critical / 0 Important.**

P5's claimed outcome is **confirmed and reproduced**: 61/61 subcommands covered,
342/342 identity round-trips Checked (0 skipped / 0 narrowed / 0 self-gated /
0 findings), the 6 initial I1 failures are a **pure harness value-modeling
artifact** (NOT a real bug), and the harness fix **masks nothing**. The phase
ships. One **Minor** usability follow-up is recommended (below) to honor the
standing "usability is a first-class bar" preference, plus two Nits. None block
the gate.

---

## Ruling on the `type_text`-append triage — **HARNESS ARTIFACT (accept), with a separable Minor usability follow-up to FILE**

**This is the crux, so it is ruled explicitly and was reproduced empirically, not reasoned.**

I disabled the empty-seed re-seed in `prepared_eligible_base`
(`ui_harness_sweep.rs:191-199`) and re-ran `i1_wiring_sweep_all_61`. It produced
**exactly the 6 claimed failures, no more, no fewer**:

```
export-wallet/--output      (Path, default "-")
restore/--output            (Path, default "-")
nostr/--timestamp           (Text, default "0"  — the importdescriptors anchor, mnemonic.rs:3553)
import-wallet/--select-descriptor (Text, default "all", mnemonic.rs:2560)
compare-cost/--feerate      (Text, default "1.0", mnemonic.rs:2262)
ms derive/--account         (Text, default "0",  ms.rs:279)
```

Production path traced and confirmed:
- `render_with_dispatch` (`src/form/widget.rs:214-223`): an **absent** Text/Path
  flag is pre-filled with `default_flag_value_for_flag(flag)`, which for a flag
  carrying a schema `default_value` returns `FlagValue::Text/Path(default_str)`
  (`widget.rs:415-422`).
- `render_row` (`widget.rs:520`) binds it editably via `ui.text_edit_singleline(s)`.
- kittest `type_text` appends at cursor-end ⇒ `"1.0"` + `"SWEEP_FIXTURE_ALPHA"`
  = `"1.0SWEEP_FIXTURE_ALPHA"`.

**Why it is an artifact and not a wiring bug (decisive):** the failure mode is
*"flag emitted, a value bound immediately after it, but the value = default∥fixture"*
— NOT *"flag absent from argv"* nor *"value bound to the wrong flag"*. The
render→store→argv **seam is intact**; only the value content carried the
pre-fill. Re-enabling the empty-seed turns all 342 (including these 6) GREEN,
which is positive proof the store path persists a cleared-then-typed value
correctly (`assemble_argv` reads ONLY `FormState`; the typed string flows through
`text_edit_singleline`'s in-place mutation → `render_with_dispatch`'s write-back
at `widget.rs:220-223`). **No real wiring bug is masked for any of the 6.**

**The harness fix is correct.** Seeding `Text("")` / `Path("")` models "the user
clears the default before typing," a normal interaction, and enforces the §5 I1
discipline that the round-trip-asserted value be **solely widget-injected**. An
empty string is `has_value`-false exactly as an absent flag is, so the
conditional gate is unperturbed — the base stays faithful to the absent-flag
state. Number (Unset→Set→SetValue) and Dropdown (select) REPLACE rather than
append, so they correctly need no empty seed. This is the right modeling choice
for an I1 **wiring** gate.

**But the sweep did surface a genuine, separable usability observation** — and
the standing user preference makes it worth FILING (Minor, not gate-blocking):
production pre-fills these defaults as **editable** text, so a real user who
types `5` into a `1.0`-prefilled `--feerate` without first clearing gets `1.05`;
typing a path into the `-`-prefilled `--output` gets `-/path`. It is **visible**
(in the field AND in the masked copy-command preview) and **recoverable**, and is
standard form behavior for *genuine* defaults the user may want to keep — hence
**Minor**, not Critical/Important. For the cases where the default is more a
**sentinel/hint** than a value one edits in place — most clearly `--output -`
(`-` = stdout) — egui `hint_text` (a greyed placeholder that vanishes on type)
or select-all-on-focus would read better and eliminate the concatenation
papercut. This is outside P5's strict **functional** scope (triage bar m7: fix
only funds/secret-Critical, FILE the rest), so the disposition is **file a Minor
usability FOLLOWUP**, not block the gate.

Is the empty-seed "a dodge that hides the concatenation from the gate forever"?
For the I1 **wiring** invariant, deliberately starting empty is correct — the
gate's job is the flag→argv seam, not UX quality; the pre-fill/append behavior
belongs to a UX concern that the FOLLOWUP captures so it is not lost. Both are
true at once; there is no masking of a funds/secret/wiring defect.

---

## Critical
**None.**

## Important
**None.**

## Minor
1. **Editable pre-filled Text/Path defaults append-concatenate on type — file a
   usability FOLLOWUP** (`src/form/widget.rs:214-223` + `:520`; 6 flags above).
   Recommend an entry in repo-root `FOLLOWUPS.md` (e.g.
   `gui-editable-default-prefill-append-papercut`): consider `hint_text`/
   placeholder for hint-like defaults (notably `--output -`) and/or
   select-all-on-focus. **Out of P5 functional scope; honors the standing
   usability-is-first-class bar.** Note: P5 did not modify `FOLLOWUPS.md` on the
   branch — acceptable, since the sweep found **0 functional** bugs and the
   plan's "file sweep-found bug FOLLOWUPs" (plan ln 97) is a P6/ship-time action;
   but this usability item should be captured before ship.

## Nits
1. **Settled-state re-check covers the conditional but not mode-suppression.**
   `i1_cell` (`ui_harness_sweep.rs:226-231`) re-checks `effect_of` (→ `SelfGated`)
   after injection but does NOT re-call `is_render_suppressed` on the *settled*
   state. Currently **unreachable**: the only identity flag whose injection can
   activate a mode is `--archetype` (`active_archetype` reads
   `dropdown_value("--archetype")`, `archetype_form.rs:49`), and `--archetype` is
   explicitly NOT in `suppressed_in_archetype_mode` (`archetype_form.rs:42-44`);
   tree mode reads the separate `state.tree` field (`tree_form.rs:60`), which no
   flag injection touches. So no identity flag both *activates* and is
   *suppressed-by* the same mode → no false-green today. A one-line defensive
   re-guard (or a comment pinning the invariant) would harden it against a future
   schema that breaks that property.
2. **The non-Checked taxonomy is latent in this run.** Census reports
   `skipped(suppressed)=narrowed=self-gated=0`; the deterministic gate's
   non-vacuity rests entirely on the `Checked` path (342, asserted `>= 80`).
   The `SkippedSuppressed`/`Narrowed`/`SelfGated` branches are defensive and
   unexercised by the gate — fine, but worth knowing they are not load-bearing.
3. **M1 mechanism note.** For the **I1 sweep**, M1 is honored via the
   `is_render_suppressed` hard-guard in `prepared_eligible_base`
   (`ui_harness_sweep.rs:182`) + mode-free candidate bases — NOT via
   `render_whole_form`. The mode-aware `render_whole_form_harness` path is used by
   the **I2 proptest** (`:524`, gated by `eligible_for_label_check` → mod.rs:530).
   Both honor M1; the mechanism differs by cell. `is_render_suppressed`
   (mod.rs:376-398) faithfully mirrors the real form loop's three `continue`s.

---

## Adversarial checklist (all verified)

| # | Check | Result |
|---|-------|--------|
| 1 | type_text-append = artifact vs real bug | **ARTIFACT** — reproduced exactly 6/6; production pre-fill traced (`widget.rs:214-223,520`); failure = value∥, seam intact. Fix correct. Separable Minor usability item to file. |
| 2 | Harness fix masks a real wiring bug? | **No.** Empty-seed turns all 342 GREEN; failure was "value=default∥fixture", not "flag absent/wrong-flag". Spot-checked feerate(Text)/output(Path)/select-descriptor(Text). |
| 3 | 61/61 non-vacuity | **Genuine.** 342 Checked, every sub n/n, 0 skipped/narrowed/self-gated. `convert` 14/14 → `--xpub-prefix`/`--electrum-version`/`--electrum-language` (mnemonic.rs:1328/1338/1348, all `default_value: None`, real identity flags) are exposed + round-tripped, not papered. Per-flag union sound (each flag picks its first Visible/Required, non-suppressed base, asserts a distinct non-default value binds). |
| 4 | M1 discipline honored | **Yes.** I1 via `is_render_suppressed` guard + mode-free bases; I2 proptest via `render_whole_form_harness` + `eligible_for_label_check`. No identity flag self-suppresses (see Nit 1). |
| 5 | Proptest design | **Right.** 3 finders `#[ignore]` + `failure_persistence: None` (no regressions file written — verified across 3 runs); 2 deterministic cells normally-run. `proptest` is `[dev-dependencies]` only; **absent from the normal/shipped graph** (`cargo tree -e normal -i proptest` → "nothing to print"). |
| 6 | Gates | `cargo test --jobs 2`: **607 passed / 0 failed / 4 ignored**, 68 binaries (P0 spike 6, P1 i1 10, P2 i2 31, P3 i3 7, P4 i4 4, P5 sweep 2+3 ignored — all green). Finders `-- --ignored`: **3/3 pass × 3 runs, no flake**. `clippy --all-targets -D warnings`: **clean (exit 0)**. **No `src/` change** (P5 commit touches only tests + Cargo). No broad fmt (mod.rs additive, zero deletions). P0–P4 still green. |
| 7 | Critical/Important | **None.** |

**Branch left clean** (experimental edit reverted via `git checkout`;
`git diff --quiet HEAD -- tests/ui_harness_sweep.rs` → CLEAN; no tracked-file
modifications).

**Bottom line: GREEN, 0C/0I — not a rubber stamp.** The triage is sound, the
harness fix masks no real bug, and coverage is non-vacuous. Recommend filing the
one Minor usability FOLLOWUP before P6/ship.
