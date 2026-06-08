# SPEC — Cycle C: per-fixture expectation table for the canonicity drift gate

**Repo:** `mnemonic-gui` (NOT the toolkit — cross-repo Cycle C of the toolkit's 4-FOLLOWUP batch).
**Cycle:** GUI test-hygiene. Resolves `canonicity-drift-gate-floor-too-lenient` (toolkit FOLLOWUP; companion to be filed GUI-side here).
**Date:** 2026-06-08.
**Source SHA:** GUI `origin/master` == local `HEAD` == `c440e91`.
**Load-bearing toolkit binary (for this gate's runtime):** the CI drift gate (`schema-mirror.yml`) `cargo install`s the **pinned tag** `pinned-upstream.toml:22` `[mnemonic].tag = mnemonic-toolkit-v0.47.3` (= commit `8502723`) and runs it as `MNEMONIC_BIN`. That installed tag — NOT a local build — is the binary the gate executes, so it is the binary the table is captured and verified against. (Distinct sense from `pinned-upstream.toml`'s own comment, which calls `Cargo.toml`'s dep the "load-bearing pin" for *compile-time*; here we mean the gate's *runtime* `MNEMONIC_BIN`. They resolve to the same tag — `Cargo.toml:42` and `pinned-upstream.toml:22` are both `mnemonic-toolkit-v0.47.3` — so the distinction is harmless, and this test shells out to `MNEMONIC_BIN` regardless of the compiled-in toolkit. Folded R0-r1 I1 + R0-r2 M5: an earlier draft pinned capture/verify to an unreleased local commit `dcbd14c`; corrected to the CI pin.)
**Disposition:** GUI **test-only, no version bump, no tag** (no behavior/schema/pin change; the pin_coherence + readme_pin_coherence gates are untouched).
**Toolchain:** `cargo +1.94.0` (GUI uses pinned 1.94.0; local nightly ICEs).

---

## 0. Problem

`tests/canonicity_drift.rs:131-136` ends with a **lenient floor**:
```rust
assert!(classified >= FIXTURES.len() / 2, …)
```
The gate iterates 18 fixtures, shells each to `mnemonic gui-schema --classify-descriptor`, and (a) collects GUI-vs-toolkit disagreements, (b) counts how many the toolkit `classified` vs `parse_failed`. Today 15 classify + 3 parse-fail (the BIP-388 `@N/**` shorthand fixtures the toolkit refuses at exit 2). The 50% floor (= 9) means **a broad toolkit-parser regression — where 9+ fixtures silently start parse-failing — still passes the gate** (`feedback-ci-snapshot-test-substring-vacuity`: tight floors). The FOLLOWUP's right answer: a **per-fixture classified-expectation table**, not a count floor.

## 1. The fix — replace the floor with a per-fixture `Expect` table

Replace `const FIXTURES: &[&str]` + the floor with an explicit per-fixture expectation, empirically captured (`mnemonic gui-schema --classify-descriptor` on each, at the **CI-pinned toolkit binary `mnemonic-toolkit-v0.47.3` / `8502723`** — see "Load-bearing toolkit binary" above):

```rust
#[derive(Clone, Copy)]
enum Expect { Canonical, NonCanonical, ParseFails }

const FIXTURES: &[(&str, Expect)] = &[
    ("pkh(@0)",                                    Expect::Canonical),
    ("pkh(@0/<0;1>/*)",                            Expect::Canonical),
    ("pkh(@0/**)",                                 Expect::ParseFails),    // BIP-388 /** shorthand → toolkit exit 2
    ("pkh([deadbeef/44'/0'/0']@0/<0;1>/*)",        Expect::Canonical),
    ("wpkh(@0)",                                   Expect::Canonical),
    ("wpkh(@0/<0;1>/*)",                           Expect::Canonical),
    ("wpkh(@0/**)",                                Expect::ParseFails),
    ("tr(@0)",                                     Expect::Canonical),
    ("tr(@0/<0;1>/*)",                             Expect::Canonical),
    ("tr([deadbeef/86'/0'/0']@0/**)",              Expect::ParseFails),
    ("wsh(multi(2,@0,@1,@2))",                     Expect::Canonical),
    ("wsh(sortedmulti(2,@0,@1))",                  Expect::Canonical),
    ("sh(wsh(multi(2,@0,@1)))",                    Expect::Canonical),
    ("sh(wsh(sortedmulti(2,@0,@1)))",              Expect::Canonical),
    ("tr(NUMS,and_v(v:pk(@0),after(12000000)))",   Expect::NonCanonical),
    ("wsh(andor(pkh(@0),after(12000000),pk(@1)))", Expect::NonCanonical),
    ("sh(sortedmulti(2,@0,@1))",                   Expect::NonCanonical),
    ("sh(multi(2,@0,@1))",                         Expect::NonCanonical),
];
// 11 Canonical + 4 NonCanonical + 3 ParseFails = 18 (15 classify, 3 parse-fail).
```

Rewrite the loop to assert each fixture against its `Expect` (no floor):
- **`ParseFails`:** the toolkit MUST return `None` (exit 2). If it now returns `Some(_)`, push to a `newly_parsed` list → **FAIL** ("the toolkit parser now ACCEPTS this fixture; the BIP-388 `@N/**` shorthand started parsing — promote it to its real Canonical/NonCanonical expectation"). This is the FOLLOWUP's brittleness concern handled the RIGHT way: a parser improvement surfaces as an actionable failure, not a silent floor-trip.
- **`Canonical`/`NonCanonical` (= `want`):** the toolkit MUST return `Some(want)`. If `None` → push to `regressed` → **FAIL** (the load-bearing case: an expected-classify fixture parse-failing = the broad regression the floor missed). If `Some(tk)` with `tk != want` → push to `wrong_verdict` → **FAIL** (the toolkit canonical↔non-canonical verdict drifted — stronger than the old agreement-only check). AND the GUI verdict must equal the toolkit verdict (the original drift check) — push to `disagreements` if `gui != tk` (key on the **live** toolkit verdict `tk`, never the frozen `want` — see M2).

Assert all four accumulator lists empty, each with a descriptive message naming the source-of-truth (`md-codec canonical_origin.rs` / GUI `conditional.rs::classify_descriptor_canonicity`) — preserve the existing disagreement message; add the three new ones.

**Implementer guardrails (folded R0-r1 M1–M3):**
- **M1 — drop the stale "Canonical …" group-header comments.** The old `const FIXTURES` carried `// Canonical pkh single-key shapes.` / `// Canonical wpkh …` / `// Canonical tr …` group headers that each span a `@N/**` row now marked `ParseFails`. Leaving a "Canonical" header above a `ParseFails` row conflates the GUI-regex verdict (which *does* call `pkh(@0/**)` canonical, `canonicity_classifier.rs:25,42,56`) with the *toolkit* expectation the table now records (ParseFails). Drop those group headers; rely on the per-row inline `// → toolkit exit 2` / `Expect::` annotation. Do **not** re-introduce a "Canonical" label adjacent to a ParseFails fixture.
- **M2 — keep `disagreements` keyed on `gui != tk` (the live toolkit verdict), not collapsed to `gui != want`.** While the table is green `tk == want`, so the two are equivalent — but the accumulator MUST compare the GUI verdict against the *live* toolkit verdict `tk` so the original drift-gate message ("gui=X toolkit=Y") stays truthful when both the verdict pin and the agreement check fire. Do not "simplify" to `want`.
- **M3 — preserve the empty-descriptor exclusion comment + the `mnemonic_bin()` early-skip verbatim.** The empty-descriptor divergence (GUI→NonCanonical, toolkit→exit-2) is covered separately by `descriptor_non_canonical_default_path_notice.rs::empty_descriptor_returns_none`; it must stay OUT of the fixture table. Do not add an empty-string fixture.

## 2. Why this is strictly stronger
- **Kills the floor:** every one of the 18 fixtures now has an exact expectation; a regression on ANY expected-classify fixture fails (the old floor tolerated up to 9 regressing).
- **Pins the absolute verdict** (canonical/non-canonical), not just GUI/toolkit agreement — catches a toolkit-table verdict flip that the old test (agreement-only) would miss if BOTH classifiers drifted together.
- **Handles the FOLLOWUP's brittleness worry:** a `ParseFails` fixture that starts parsing fails LOUDLY with a "promote it" message (not a silent floor-trip, not a brittle `len()-4`).

## 3. Verification
1. Build/run against the **CI-pinned toolkit binary** (the load-bearing pin — `mnemonic-toolkit-v0.47.3` / `8502723`, `pinned-upstream.toml:22`), so local green == CI green: `MNEMONIC_BIN=<mnemonic built from mnemonic-toolkit-v0.47.3> cargo +1.94.0 test --test canonicity_drift` → green (all 18 expectations met). (The earlier `dcbd14c` capture was an unreleased local commit and is NOT what CI runs; folded R0-r1 I1.)
2. **Negative checks (prove the gate bites):** temporarily (a) flip one `Canonical`→`NonCanonical` in the table → the test must FAIL on `wrong_verdict`/`disagreements`; (b) flip a `ParseFails`→`Canonical` → must FAIL on the toolkit-returns-None `regressed` path. Revert both.
3. `cargo +1.94.0 build` + `cargo +1.94.0 clippy --tests` clean.
4. No GUI behavior/schema/pin change → `schema_mirror`, `pin_coherence`, `readme_pin_coherence` unaffected.

## 4. Ship plan
1. Apply §1 to `tests/canonicity_drift.rs`.
2. Verify §3 (incl. the negative checks).
3. File the GUI FOLLOWUP `canonicity-drift-gate-floor-too-lenient` → resolved in the existing **root-level** `mnemonic-gui/FOLLOWUPS.md` (NOT `design/FOLLOWUPS.md` — the registry already exists at the repo root; append a structured entry to its `## Resolved` section, matching the `gui-timestamp-default-value-drift-v0.47.3` precedent); cross-cite the toolkit FOLLOWUP. Flip the toolkit-side companion to resolved (`mnemonic-toolkit/design/FOLLOWUPS.md`) in lockstep.
4. Stage explicitly; commit (`git commit -F -`, Co-Authored-By) in the GUI repo; push to GUI `master`. No bump/tag.
5. Memory.

### Out of scope
- Adding NEW fixtures / new descriptor shapes (the corpus is unchanged; only its assertion shape changes).
- The GUI's `classify_descriptor_canonicity` regex itself (unchanged — the gate guards it, doesn't rewrite it).
