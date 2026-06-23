## R0 Review — Wave-3 GUI wire-shape spec (`SPEC_W3_gui_wireshape_lane.md`)

**VERDICT: GREEN — 0 Critical / 0 Important / 4 Minor. Cleared to implement.**

Reviewed against current source in `/scratch/code/shibboleth/mnemonic-gui` (HEAD `6df305d`, crate 0.48.1), the live `mnemonic 0.70.0` binary (= the `Cargo.toml:42` pin), `mnemonic-toolkit` `origin/master`, and `mnemonic-key` mk-cli source. Every load-bearing claim was re-grepped/re-run, not taken from spec prose.

### Keystone (W3-1) wire-shapes — ALL confirmed LIVE against v0.70.0
| Cell | Live result | Matches spec? |
|---|---|---|
| `path-of-xpub` match | top=`{schema_version,mode,result,path,template,account,target_xpub_canonical,target_xpub_variant,searched_count}`, `path="m/84'/0'/0'"`, `template="bip84"`, `account=0`, exit 0 | YES |
| `path-of-xpub` no_match | `path/template/account` OMITTED (not null), exit 4 | YES — the drift the slug targets |
| `account-of-descriptor` match | `matched_cosigners[0]={cosigner_index,path,template,account}`, `descriptor_shape="literal_xpub"`, exit 0 | YES |
| `import-wallet` coldcard | top=`{bundle,coldcard_source_metadata,roundtrip,schema_version,source_format}`; `coldcard_source_metadata={bip_derivation,chain,dropped_fields,raw_account,xfp}`; `roundtrip={byte_exact,diff,semantic_match,status}` | YES (exact) |
| `import-wallet` BSMS multisig | top=`{bundle,roundtrip,schema_version,source_format}` (no source-meta); `bundle.multisig={cosigner_count,cosigners,path_family,template,threshold}`; `cosigners[i]={index,master_fingerprint,origin_path,xpub}` | YES (exact, incl. the §2.2 blockquote cosigner sub-keyset) |

The §2.3 CRITICAL "capture goldens FROM the binary" guidance is sound and is the crux of option (b).

### CI-gate discipline — verified (this is the hard bar from the preamble)
- **Gate 1 (`cargo-test-full-suite`, `MNEMONIC_BIN=mnemonic`)**: install-from-tag at L49-62, test job at L127-133 — confirmed; the new cells run here against the cargo-installed v0.70.0 binary. The CI-ONLY hazard (stale-fixture golden passes locally but REDs vs the installed v0.70.0 binary) is correctly identified and mitigated by capture-from-binary.
- **Gate 2 (`readme_pin_coherence`)**: VERIFIED CI-gated and currently GREEN at 0.48.1; `cargo_version()` reads `Cargo.toml [package].version` (L59-65), expectation tuple L75. README.md:42 is the GUI self-tag; sibling pins L50-53 are correctly excluded (no pin change). The lockstep README+Cargo bump is mandatory and reproducible locally (no binary). This is the Wave-2 G1-B class correctly caught.
- **Gate 3 (`schema_mirror`)**: VERIFIED — `assert_schema_matches_help` set-diffs flag NAMES only; `help:` prose is never compared. Ran the full suite: **21 passed**, incl. `mk_schema_flag_names_match_help_text` GREEN. W3-3 prose edit is provably non-drifting.
- **Gate 8 (no fmt)**: confirmed — grep for `cargo fmt`/`rustfmt` across both workflows returns nothing. mlock.rs is N/A (GUI carries no mlock byte-anchor) — correctly noted.
- **§6 ¶9 non-firing CI-only gates**: toolkit `manual-gui.yml gui-schema-coverage` (path-filtered to toolkit docs + `manual-gui-v*` tags — does NOT fire on a GUI-repo change) and `sibling-pin-check.yml` (no toolkit pin bumped) correctly excluded. The explicit "do NOT opportunistically bump the toolkit mk-cli pin" guard is the right call.

### Scope, SemVer, atomicity
- **Descope proven, not assumed**: `export-wallet --json` → `unexpected argument` (exit 64), correctly out of scope; md-leg, W3-4/W3-5, mk-cli source reword + manual edits all explicitly deferred (§3, §8). No scope creep.
- **SemVer MINOR (0.48.1→0.49.0)** justified (new test surface + new tests/ module + prose). Version sites complete (Cargo.toml:3 + README.md:42 CI-gated + CHANGELOG release-completeness).
- **FOLLOWUPS flip (§7)**: slug header L3452, Status L3458, Companion L3461 — all VERIFIED exact on origin/master; flip lands toolkit-side post-merge, not in this PR.
- **Fixture deletion safety (option A)**: grep confirms `cli_envelope_smoke.rs` is the ONLY consumer of `v0_27_0_envelopes/*` + `wallet_import/envelope_v0_27_0.json`; `descriptor_builder/*` fixtures have separate live consumers (`tree_round_trip.rs`, `tree_form.rs`) and are correctly NOT deleted. The spec's pre-delete grep step is the right guard.
- **Atomicity**: single-commit acceptable; no sibling-pin-check coupling exists in this repo, so no split-push hazard. Reasoning sound.

### Minors (non-blocking; fold opportunistically)
1. §2.5/§2.3 over-generalize "pass the seed via `--phrase-stdin` in the test" — `import-wallet` has NO `--phrase-stdin` (runs watch-only, no seed); the §2.2 import-wallet cell rows correctly omit it. Scope the prose to the xpub-search cells.
2. §3/§8.2 cite mk-cli `vectors.rs:70-73` for the honor-under-`--out` proof; the actual pass-through is L34-35 (`write_per_fixture_files(dir, args.pretty)`), L70-71 is the downstream pretty branch. Claim is correct; citation slightly off (deferred item anyway).
3. §4-item-11 SOURCE.md: add the BSMS row with the correct upstream subpath `tests/fixtures/wallet_import/` (not the coldcard `tests/export_wallet/` path) pinned at v0.70.0.
4. §2.3 "structurally identical to envelope_v0_27_0.json" is moot once option A deletes that fixture; the binary-captured golden stands on its own.

**No Critical or Important findings. Gate is GREEN — implementation may proceed.**