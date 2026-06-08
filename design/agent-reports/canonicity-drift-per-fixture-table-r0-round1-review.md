# R0 Architect Review — Round 1 — `SPEC_canonicity_drift_per_fixture_table.md`

> Persisted verbatim from the opus-architect R0 round-1 dispatch (Cycle C — canonicity-drift gate per-fixture table, mnemonic-gui repo). Recovered from the toolkit session transcript and persisted per the persist-before-fold convention.

---

Confirmed: the CI pin `mnemonic-toolkit-v0.47.3` resolves to commit `8502723`, NOT `dcbd14c`. The SPEC's stated capture/verify commit `dcbd14c` is a different binary than the one CI actually runs against.

This is the verdict-discriminator. Two considerations:

1. **Does it change the test outcome?** No — for the reason I traced: the 3 `/**` parse-fails come from the toolkit's own single-star lexer regex (`parse_descriptor.rs:70`), not rust-miniscript version drift. That regex is stable across the relevant range, and the existing mirror-pinned test plus the FOLLOWUP both empirically show 3 parse-fails. So against v0.47.3 the table is still green. The 15 stable verdicts are anchored by `canonical_origin` (also unchanged). So this is NOT a Critical (the test won't false-red in CI).

2. **Is it a SPEC defect worth flagging?** Yes — Important. The SPEC's verification protocol (§1 "at toolkit `dcbd14c`", §3 step 1 "toolkit dcbd14c debug mnemonic") points the implementer at the *wrong binary*. CI green is what gates the merge, and CI runs v0.47.3 (`8502723`). An implementer who builds `dcbd14c` and gets green has verified against a binary CI never runs. The capture/verify commit must be re-pinned to the CI binary (v0.47.3 / `8502723`), and the SPEC should document that this is the load-bearing pin. The SPEC even has a precedent convention (CLAUDE.md: "Document the source SHA in the spec for future readers") — it documented the GUI source SHA (`c440e91`) but mis-stated the toolkit binary SHA.

I have everything needed for a complete verdict. Let me also quickly confirm there isn't a `dcbd14c`-vs-`8502723` divergence in the `parse_descriptor` lexer between those commits that could matter — but since v0.47.3 is the *newer* released tag and `dcbd14c` is an unreleased local capture point, and the lexer regex I read locally (HEAD) is single-star, the behavior is consistent. The risk is only theoretical. I'll note it as the substance of the Important finding.

Let me write up the final R0 review.

---

# R0 Architect Review — `SPEC_canonicity_drift_per_fixture_table.md`

## Verdict: **YELLOW — 0 Critical / 1 Important / 3 Minor**

The SPEC's core design (per-fixture `Expect` table replacing the lenient floor) is **correct, sound, and strictly dominates** the old gate. The one Important finding is a verification-protocol defect: the SPEC pins capture/verify to the wrong toolkit binary (`dcbd14c`) relative to what CI actually runs (`v0.47.3` = `8502723`). The test logic is unaffected; the reproducibility/audit claim is wrong. Fix is a one-line re-pin. **Hold the gate until I1 is folded**, then GREEN.

---

## Critical
None.

---

## Important

### I1 — SPEC pins capture/verify to toolkit `dcbd14c`, but the CI gate runs `v0.47.3` (`8502723`); the verification protocol targets a binary CI never executes
**Where:** SPEC §1 line 22 ("at toolkit `dcbd14c`"), §3 step 1 ("`MNEMONIC_BIN=<toolkit dcbd14c debug mnemonic>`").
**Evidence:**
- The CI drift gate is `cargo test --workspace` in `.github/workflows/schema-mirror.yml:121-127` with `MNEMONIC_BIN: mnemonic`, where `mnemonic` is `cargo install`-ed from the **pinned tag** `steps.pins.outputs.mnemonic_tag` (lines 53-56).
- That tag is `mnemonic-toolkit-v0.47.3` (`/scratch/code/shibboleth/mnemonic-gui/pinned-upstream.toml:22`), which resolves to commit **`8502723`** — verified against the GitHub release. **Not `dcbd14c`.**
- So the merge-gating binary is `v0.47.3`/`8502723`; the SPEC's "empirically captured at `dcbd14c`" and "verify against a `dcbd14c` debug build" instruct the implementer to validate against a *different* binary than CI runs. CLAUDE.md's own convention ("Document the source SHA in the spec for future readers"; "re-grep against current `origin/master`") is the discipline this misses — the SPEC correctly pinned the GUI SHA (`c440e91`, §line 6) but mis-stated the load-bearing toolkit binary SHA.

**Why this is Important, not Critical:** The test will still be **green in CI against v0.47.3**. I traced the 3 `/**` parse-fails to the toolkit's *own* lexer regex (`mnemonic-toolkit/.../parse_descriptor.rs:70` — `(/\*(?:'|h)?)?` matches a *single* star; the second `*` in `/**` survives lexing and corrupts the descriptor downstream → `DescriptorParse` error → exit 2 → `None`). That lexer is toolkit-internal and stable across the v0.43→v0.47 range, so the parse-fail set is not rust-miniscript-version-fragile in this window. Combined with the existing mirror-pinned test's own body comment (`canonicity_drift.rs:102-108`) and the toolkit FOLLOWUP both empirically recording "15 classify, 3 parse-fail," the 11/4/3 table holds against v0.47.3. The defect is in the **reproducibility claim**, not the outcome.

**Fix:** In §1 and §3, change the capture/verify binary from `dcbd14c` to the CI pin `mnemonic-toolkit-v0.47.3` (commit `8502723`). State explicitly that the load-bearing binary is the `pinned-upstream.toml` `[mnemonic].tag` (the binary CI `cargo install`s), and that §3-step-1 must build/run *that* tag so local green matches CI green. (If the SPEC author has a specific reason `dcbd14c` was the capture point — e.g. an unreleased fix — surface it; but the gate is bound to the pin, so the pin is what must be verified.)

---

## Minor

### M1 — Stale group-header comment must not carry "Canonical" onto the `/**` ParseFails rows
**Where:** old `canonicity_drift.rs:61` group header "// Canonical pkh single-key shapes." spans `pkh(@0/**)`, which the new table marks `ParseFails`; similarly the "Canonical wpkh"/"Canonical tr" headers span `wpkh(@0/**)` and `tr([deadbeef/86'/0'/0']@0/**)`. The SPEC's inline `// → toolkit exit 2` comments are correct, but instruct the implementer to **drop the misleading "Canonical" group headers** rather than leave them adjacent to ParseFails rows. (The GUI's *own* regex does classify these Canonical — see `canonicity_classifier.rs:25,42,56` — but the new table records the *toolkit* expectation, which is ParseFails. The header would conflate the two source-of-truths.)

### M2 — Keep the drift accumulator keyed on `gui != tk`, not collapsed to `gui != want`
The SPEC already says this (§1 line 53 parenthetical), but flag it for the implementer: although `gui == want` is *equivalent* to `gui == tk` while the table is green (since `tk == want` is asserted just above), the `disagreements` list must compare `gui` against the **live toolkit verdict `tk`**, not the frozen `want`. This preserves the original drift-gate failure message ("gui=X toolkit=Y") and gives a truer diagnostic when both the toolkit-verdict pin AND the agreement check would fire. Don't let the implementer "simplify" it to `want`.

### M3 — Preserve the empty-descriptor exclusion comment + `mnemonic_bin()` skip verbatim
SPEC §1 line 57 says keep these unchanged — confirmed correct and load-bearing. The empty-descriptor divergence (GUI→NonCanonical, toolkit→exit-2) is covered separately at `descriptor_non_canonical_default_path_notice.rs::empty_descriptor_returns_none` (per the existing `canonicity_drift.rs:52-59` comment), so it must stay *out* of the fixture table. Ensure the implementer doesn't add an empty-string fixture.

---

## Verified-correct

- **The enum:** `pub enum Canonicity { Canonical, NonCanonical }` exists at `mnemonic-gui/src/form/conditional.rs:67-77`. The SPEC's new local `enum Expect { Canonical, NonCanonical, ParseFails }` is a distinct test-local type (correct — `Canonicity` has no parse-fail variant; `None` from `toolkit_classify` carries that signal). ✓
- **The floor location:** `assert!(classified >= FIXTURES.len() / 2, …)` is at `canonicity_drift.rs:131-136` (the toolkit FOLLOWUP's cited `:138` is a stale snapshot — the SPEC correctly uses `:131-136`). `toolkit_classify` returns `None` on non-success exit (`:39-41`) = parse-fail; `mnemonic_bin()` early-skip at `:90-93`. ✓
- **The 11/4/3 verdict table vs `canonical_origin` + existing comments:** I cross-checked every fixture against the authoritative `md-codec/src/canonical_origin.rs:45-79` AND the existing test's group comments. All 11 Canonical (pkh/wpkh/tr-keypath/wsh-multi-sortedmulti/sh-wsh-multi-sortedmulti) map to `Some(_)`; all 4 NonCanonical map to `None` — and the existing comments agree exactly: `tr(NUMS,...)` = "Non-canonical: tr with taptree" (`canonical_origin.rs:56` `tree:Some → None`); `wsh(andor(...))` = inner not multi/sortedmulti → `None`; `sh(sortedmulti)` + `sh(multi)` = "Non-canonical: sh(...) legacy P2SH multi (no wsh wrap)" (`canonical_origin.rs:65-74` requires inner `Wsh`). **No fixture's SPEC-assigned Expect contradicts the existing comment or the source.** ✓
- **The 3 ParseFails (`pkh(@0/**)`, `wpkh(@0/**)`, `tr([deadbeef/86'/0'/0']@0/**)`):** confirmed against the existing test's own body comment (`:102-108`) AND the toolkit FOLLOWUP (`mnemonic-toolkit/design/FOLLOWUPS.md:2237`), AND mechanistically via the toolkit lexer single-star regex (`parse_descriptor.rs:70`). ✓
- **Loop-logic soundness / strict domination:** (a) `ParseFails → None`, else `newly_parsed` FAIL — correct; turns a parser-improvement into an actionable failure (the FOLLOWUP's brittleness concern handled the right way, vs the rejected brittle `len()-4` floor). (b) `Canonical/NonCanonical → Some(want)`, else `regressed` FAIL (this is the exact broad-regression case the old floor tolerated up to 9 of); `Some(tk≠want)` → `wrong_verdict` FAIL (pins the absolute verdict, stronger than agreement-only — catches a both-classifiers-drift-together flip); plus `gui != tk` → `disagreements` (original drift check preserved). **Every one of the 18 fixtures now has an exact expectation; strictly dominates floor + agreement-only.** ✓
- **The floor loses nothing:** its stated purpose ("broad toolkit regression silently passes") is covered strictly better — each expected-classify fixture now individually fails on regression via `regressed`. ✓
- **Disposition (test-only, no bump, no tag):** confirmed. `canonicity_drift` / `FIXTURES` references are confined to `canonicity_drift.rs` (+ the SPEC). The GUI gates `schema_mirror.rs`, `pin_coherence.rs`, `readme_pin_coherence.rs` are independent files unaffected by a fixture-table reshape; `canonicity_classifier.rs` (GUI-regex-only) and `descriptor_non_canonical_default_path_notice.rs` are not touched. No CLI/schema/pin surface change → no `schema_mirror` impact. ✓
- **CI binding (NOT a fig-leaf skip):** the strengthened gate runs against a **real** pinned-tag binary in CI (`schema-mirror.yml:53-56` install + `:121-127` full-suite with `MNEMONIC_BIN=mnemonic`), so it does not vacuously skip — closing the recurring stale-PATH/placeholder-skip risk. ✓
- **Cross-repo bookkeeping:** the toolkit companion `canonicity-drift-gate-floor-too-lenient` is `Status: open` with `Companion: bg002h/mnemonic-gui FOLLOWUPS.md entry to file in lockstep` (`mnemonic-toolkit/design/FOLLOWUPS.md:2239-2241`). The GUI `design/FOLLOWUPS.md` does **not yet exist** — SPEC §4 step 3 "create if absent" is correct. Filing the GUI entry resolved + flipping the toolkit companion to resolved in lockstep is the right bookkeeping. ✓
- **Intended "good failure":** pinning the absolute verdict (`tk == want`) means a *legitimate* future evolution of md-codec's canonical table forces a table update — this is intended (the SPEC's §2 third bullet + §1 `newly_parsed` "promote it" message). Confirmed it's a feature, not a new failure mode. ✓

---

## Required before GREEN
Fold **I1** (re-pin §1/§3 capture+verify binary from `dcbd14c` → the CI pin `mnemonic-toolkit-v0.47.3` / `8502723`; state the `pinned-upstream.toml` tag is the load-bearing binary). Address M1–M3 in the same pass (they're implementer-guardrails the SPEC mostly already states). Persist this review to `design/agent-reports/`, fold, re-dispatch the architect per the reviewer-loop convention. Once I1 is folded, this converges to GREEN (0C/0I) — the test logic itself is already sound.

**Relevant files:**
- `/scratch/code/shibboleth/mnemonic-gui/design/SPEC_canonicity_drift_per_fixture_table.md`
- `/scratch/code/shibboleth/mnemonic-gui/tests/canonicity_drift.rs`
- `/scratch/code/shibboleth/mnemonic-gui/src/form/conditional.rs` (`Canonicity` :67-77, `classify_descriptor_canonicity` :99)
- `/scratch/code/shibboleth/mnemonic-gui/pinned-upstream.toml` (`:22` toolkit pin = v0.47.3)
- `/scratch/code/shibboleth/mnemonic-gui/.github/workflows/schema-mirror.yml` (`:53-56` install, `:121-127` full-suite gate)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (`:1329-1338` `--classify-descriptor`)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs` (`:70` single-star lexer → `/**` parse-fail)
- `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs` (`:45-79` authoritative table)
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md` (`:2233-2241` companion entry to flip)
