# R0 Architect Review — Round 2 — `SPEC_canonicity_drift_per_fixture_table.md`

> Persisted verbatim from the opus-architect R0 round-2 (convergence) dispatch. Cycle C — canonicity-drift gate per-fixture table, mnemonic-gui repo.

---

Lockstep confirmed: `Cargo.toml:42` dep tag = `mnemonic-toolkit-v0.47.3`, matching `pinned-upstream.toml:22`. Both the compile-time dep and the CI-runtime pin point at the same tag, so the dual-sense of "load-bearing" (advisor point 2) is harmless — both referents resolve to v0.47.3 here. And since this test shells out to `MNEMONIC_BIN` rather than using the compiled-in toolkit, a hypothetical mismatch couldn't affect this test anyway. This is wording-confidence only, not a finding.

I have everything for the final verdict.

---

## Verdict: **GREEN — 0 Critical / 0 Important / 3 Minor**

**Convergence confirmed. Implementation can begin.** All four Round-1 findings (I1 gating + M1–M3 guardrails) folded correctly and completely. The fold introduced no new contradictions or drift, and did not perturb the four-accumulator test logic that R1 already verified sound. The three Minor items below are wording-polish for the implementer; none block, and none require another review round.

---

## Critical
None.

## Important
None. **I1 is fully folded** — see the verification below.

---

## Minor

### M4 — §1 line 54 parenthetical re-invites the exact `gui == want` collapse that line 60 (M2) forbids
**Where:** SPEC §1 line 54, trailing parenthetical: *"(Equivalently assert `gui == want` since `tk == want`.)"* — four lines above the M2 guardrail at line 60 that explicitly forbids collapsing `disagreements` to `gui != want`.
Not a contradiction (line 60 is explicit and authoritative, and line 54's primary clause correctly keys on `gui != tk`), but the parenthetical is a redundant invitation toward the wrong path the guardrail exists to close. Suggest dropping the "Equivalently…" clause or appending "— but the accumulator must still key on `tk`, per M2."

### M5 — "load-bearing toolkit binary" (SPEC line 7) collides surface-wise with `pinned-upstream.toml`'s own "documentary only / Cargo.toml is the load-bearing pin" comment
**Where:** SPEC line 7 names `pinned-upstream.toml:22 [mnemonic].tag` "the load-bearing toolkit binary"; `pinned-upstream.toml:7-16` calls that same tag field "documentary only" and names `Cargo.toml [dependencies]` "the load-bearing pin."
Different referents, both correct: the SPEC means the CI **runtime** `MNEMONIC_BIN` binary the gate executes (scoped correctly as "the binary the gate runs against"); the file means the **compile-time** dependency pin. I confirmed they resolve to the same tag — `Cargo.toml:42` and `pinned-upstream.toml:22` are both `mnemonic-toolkit-v0.47.3` (lockstep). Suggest one disambiguating clause in the SPEC header ("load-bearing *for this gate's runtime*, i.e. the `cargo install`-ed `MNEMONIC_BIN`") so the wording doesn't read as contradicting the cited file. Wording only; no behavioral impact, and this test shells out to `MNEMONIC_BIN` rather than the compiled-in toolkit, so a dep/runtime mismatch could not affect it.

### M6 — name the empirical contingency GREEN rests on (not a SPEC defect)
The 11/4/3 verdict table was originally captured at `dcbd14c`; I1 re-pinned the **prose** to v0.47.3/`8502723` but the table was not re-captured during the fold. GREEN therefore rests on (a) R1's mechanistic argument — the 3 `/**` parse-fails come from the toolkit's own single-star lexer regex (`parse_descriptor.rs:70`), stable across v0.43→v0.47, and the 15 verdicts are anchored by the unchanged `canonical_origin` table — and (b) SPEC §3-step-1 **mandating** an actual build/run against v0.47.3 before ship. This deferral is appropriate for a SPEC (it is the empirical gate, not optional), so it is not a finding against the SPEC — but the implementer must treat §3-step-1 + the §3-step-2 negative checks as load-bearing, not skippable.

---

## I1 fold — verified complete (the gating finding)

The R1 fix required three things; all three landed, with `dcbd14c` removed from every operative instruction:
- **Capture re-pinned (§1 line 23):** now "at the CI-pinned toolkit binary `mnemonic-toolkit-v0.47.3` / `8502723`."
- **Verify re-pinned (§3 step 1, line 69):** `MNEMONIC_BIN=<mnemonic built from mnemonic-toolkit-v0.47.3>`, cross-cited to `pinned-upstream.toml:22`.
- **Load-bearing pin stated (header line 7):** the CI `cargo install`-ed pinned tag is named as the binary the table is captured/verified against.

The only remaining occurrences of `dcbd14c` (line 7 and line 69) are explicit "earlier draft was wrong, corrected to the CI pin; folded R0-r1 I1" fold-notes — the correct way to record a fold, not residual drift. Ground-truth re-confirmed this session: `pinned-upstream.toml:22` and `Cargo.toml:42` both pin `mnemonic-toolkit-v0.47.3`; GUI `HEAD` = `c440e91` matches SPEC line 6; CI gate (`schema-mirror.yml:50-56` install, `:121-127` `cargo test --workspace` with `MNEMONIC_BIN=mnemonic`) runs that installed tag.

## M1–M3 folds — verified complete (guardrails)
- **M1:** §1 lines 58-59 instruct dropping the stale "Canonical" group headers and forbid re-introducing a "Canonical" label adjacent to a ParseFails row; the rewritten table (lines 29-48) already has no group headers, only per-row `Expect::`/`// → toolkit exit 2` annotations. Premise re-confirmed live: the GUI regex genuinely classes all three `@N/**` rows Canonical (`canonicity_classifier.rs:25,42,56`), so a "Canonical" header over a ParseFails row would conflate GUI-regex vs toolkit source-of-truth. Landed.
- **M2:** §1 line 54 keeps `disagreements` keyed on `gui != tk`; line 60 forbids the `gui != want` collapse with rationale. Landed (see M4 for the residual invitation).
- **M3:** §1 line 61 preserves the empty-descriptor exclusion comment + `mnemonic_bin()` early-skip verbatim and forbids adding an empty-string fixture; the covering test `descriptor_non_canonical_default_path_notice.rs:76 empty_descriptor_returns_none()` confirmed to exist. Landed.

## Test-logic soundness — carries forward
The fold touched only the verification prose (I1) and the three guardrail notes (M1–M3); it did not perturb the per-fixture `Expect{Canonical,NonCanonical,ParseFails}` table replacing the `classified >= FIXTURES.len()/2` floor, nor the four accumulators (`newly_parsed`/`regressed`/`wrong_verdict`/`disagreements`). R1's soundness verdict — the design strictly dominates the old floor + agreement-only check (every fixture gets an exact expectation; broad regression now fails via `regressed`; absolute verdict pinned via `wrong_verdict`; parser-improvement surfaces as actionable `newly_parsed` rather than a silent floor-trip) — therefore carries forward unchanged. The 18-fixture / 11 Canonical + 4 NonCanonical + 3 ParseFails / 15-classify arithmetic (line 49) is internally consistent.

## Disposition — confirmed correct
**Test-only, no version bump, no tag.** No CLI/schema/pin surface changes; `schema_mirror` / `pin_coherence` / `readme_pin_coherence` gates untouched; the `Cargo.toml` and `pinned-upstream.toml` pins stay at v0.47.3. `FIXTURES`/`canonicity_drift` references are confined to `tests/canonicity_drift.rs`. SPEC §4-step-3 cross-repo bookkeeping (file GUI FOLLOWUP resolved + flip toolkit companion in lockstep) is the right closure.

---

**Bottom line: GREEN (0 Critical / 0 Important). The reviewer-loop has converged — implementation may begin.** M4–M6 are optional same-pass wording polish for the implementer and do not require a further review round.
