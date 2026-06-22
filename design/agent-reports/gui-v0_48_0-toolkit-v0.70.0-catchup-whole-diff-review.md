> Reviewer: opus adversarial post-implementation reviewer — `mnemonic-gui` whole-diff ship gate · branch `lockstep/toolkit-v0.70.0-pin-bump` · toolkit pin v0.60.0 → v0.70.0 schema-mirror catch-up
>
> NOTE (version retarget): this review was conducted when the release was numbered **v0.47.0**. The cycle-15 Lane G secret-residue-zeroize cycle shipped its own v0.47.0 (PR #15) concurrently, so this catch-up was rebased atop it and **retargeted to v0.48.0**. The reviewed diff is substantively unchanged (only the version integer moved); the full suite was re-run GREEN against the rebased base (543 passed / 0 failed / 1 ignored). All "v0.47.0" references below should be read as v0.48.0.

# VERDICT: GREEN (0 Critical, 0 Important, 0 Minor)

Ship it.

## What I ran (commands + results)

**1. schema_mirror gate (21 subtests):**
```
cd /scratch/code/shibboleth/mnemonic-gui
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic cargo test --test schema_mirror
→ test result: ok. 21 passed; 0 failed; 0 ignored
```

**2. Independent flag-name diff** (binary `gui-schema` vs patched `src/schema/mnemonic.rs`, set-equality incl. the const-referenced global `NO_AUTO_REPAIR_FLAG`):
```
restore:        binary=26  gui=26   match=True
verify-bundle:  binary=29  gui=29   match=True
```
- `in BINARY not in GUI`: none.  `in GUI not in BINARY`: none.
- The 3 new flags present, correctly named, correct `FlagKind`: `restore --search-cosigner-subset` → Boolean; `verify-bundle --own-account-max` → Number; `verify-bundle --search-cosigner-subset` → Boolean. Binary `gui-schema` reports `boolean`/`number`/`boolean` respectively — match.
- (Note for the record: my first naive regex undercounted by 1 per subcommand because the global `--no-auto-repair` is referenced by const identifier inside each array, not as an inline `name:` field. After accounting for it, the effective sets are exact. No real drift.)

**3. Mutex / conditional_rules:**
```
binary gui-schema restore conditional_rules count: 1  (own-account-max present? False)
binary gui-schema verify-bundle conditional_rules count: 10  (own-account-max present? False)
git diff … src/schema/mnemonic.rs | grep ConditionalRule → 0 struct additions (only 2 explanatory comment lines)
cargo test --test gui_schema_conditional_drift → ok. 5 passed; 0 failed
```
Toolkit does not project the `--own-account-max ⊕ --account` mutex into `conditional_rules`; the GUI diff correctly added no conditional rule; the drift gate (restore=1, verify-bundle=10) is GREEN. Correct call — modeling it would have broken the count floors.

**4. Version sites + over-reach:**
```
cargo test --test pin_coherence       → ok. 1 passed
cargo test --test readme_pin_coherence → ok. 1 passed
git diff --stat → exactly 6 files (CHANGELOG, Cargo.lock, Cargo.toml, README, pinned-upstream.toml, src/schema/mnemonic.rs) + 1 untracked recon doc
```
All required sites moved: Cargo.toml `version 0.46.0→0.47.0`; pinned-upstream `[mnemonic].tag → v0.70.0`; README GUI self-tag `→v0.47.0` + toolkit install line `→v0.70.0`; Cargo.lock both the `mnemonic-gui 0.47.0` version and the `mnemonic-toolkit v0.70.0` source rev (+ transitive md-codec 0.37→0.39, ms-codec 0.4.4→0.6.0 re-resolve); schema doc-header `→v0.70.0`. No over-reach: the `src/schema/mnemonic.rs:3260` "toolkit v0.46.0: passphrase-candidates-file" provenance comment stays **v0.46.0**; `tests/secret_taxonomy_pin.rs` is **not in the diff** (its `mnemonic-toolkit-v0.60.0` provenance ref correctly unchanged); CHANGELOG diff has **zero deletions** (historical entries intact, only the new v0.47.0 block added).

**5. Consumer-code (`--json`) scope:** The only `serde_json::from_slice` on toolkit subprocess stdout is in `src/form/tree_form.rs` (`apply_emit_spec_result`, `apply_validate_result`), and both operate exclusively on `mnemonic build-descriptor` output. No other subcommand's JSON is deserialized (`bundle`/`import-wallet`/`export-wallet`/`restore`/`verify-bundle`/`xpub-search` are rendered as raw text). The two v0.66.0 wire-VALUE changes (M7 bundle threshold, M1 import-wallet account) land on non-parsed subcommands → no consumer change owed. Confirmed.

**6. Full suite:**
```
MNEMONIC_BIN=<v0.70.0> cargo test --no-fail-fast
→ 530 passed; 0 failed; 1 ignored  (summed across 60 test binaries)
```
Zero failures, zero panics, zero compile errors. Additionally ran the other named gates: `archetype_schema_mirror` (1), `xpub_search_schema_mirror` (9), `schema_mirror_secret_drift` (1), `canonicity_drift` (1) — all ok.

**7. Help-text accuracy:** The rewritten `restore --own-account-max` help no longer says "refused/NOT SUPPORTED" (grep for any "refused/reserved/NOT SUPPORTED" in own-account context → empty), and correctly states the active subset-search + `Mutually exclusive with --account` + `K ≤ 256` — consistent with the binary `--help`. The two new help strings are accurate paraphrases.

## SemVer + lockstep

- **MINOR is correct** — additive new clap-flag mirror surface, no removals/renames.
- No missed lockstep: toolkit-only flags → no sibling-codec (md/ms/mk) or manual obligation. No `cargo fmt` was applied (GUI has no fmt CI gate — correctly avoided).

Nothing would make this ship wrong, incomplete, or in violation of lockstep/SemVer discipline. The single count-difference I chased down (`--no-auto-repair`) resolved to an artifact of my extraction, not a defect — the effective flag sets are byte-exact and the gate proves it.
