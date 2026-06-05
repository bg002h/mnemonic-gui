# mnemonic-gui v0.24.0 — R0 Review (mnemonic restore schema mirror)

**Verdict: GREEN (0 Critical / 0 Important).** Cleared to tag `mnemonic-gui-v0.24.0`.

Single-phase mechanical schema-mirror lockstep for `mnemonic-toolkit-v0.43.0`'s new `mnemonic restore` subcommand. One opus R0 (full shell, gate re-run independently) = both per-phase and end-of-cycle. Branch `mnemonic-restore-gui`, 1 commit `d7cdb62` (+ a CHANGELOG fold) over master (v0.23.0).

## Critical / Important
None.

## Minor
1. **CHANGELOG "16 flags" → actual 15 (FOLDED).** The `[0.24.0]` entry said "16 flags" but `restore` has 15 (14 explicit + the shared global `--no-auto-repair`), matching both gui-schema and the mirror. Cosmetic prose count; the mirror itself was always correct. Folded → "15 flags (14 explicit + the shared global --no-auto-repair)".
2. **Pre-existing README `--tag` install-block staleness (out of scope, NOT folded).** `README.md:42` pins `mnemonic-gui-v0.22.0` and `:50` `mnemonic-toolkit-v0.41.0` — stale since v0.22.0 across two cycles; no `readme_version_current`-style guard in this repo; the prior in-scope cycle (v0.23.0) set the same precedent. Candidate for a future README re-sync; left as-is.

## Verification ledger (every item RUN)
- **Binary:** `mnemonic --version` = 0.43.0; `git -C …/mnemonic-toolkit rev-parse mnemonic-toolkit-v0.43.0^{}` = `0f404aeca6…` = toolkit master tip.
- **Flag-NAME parity (schema_mirror core):** gui-schema `restore` NAME set == GUI `RESTORE_FLAGS` NAME set — SET EQUAL (15: `--account,--allow-mismatch,--count,--expect-fingerprint,--expect-xpub,--format,--from,--json,--language,--network,--no-auto-repair,--output,--passphrase,--passphrase-stdin,--template`); GUI−schema=∅, schema−GUI=∅.
- **Secret parity (secret_drift core):** SET EQUAL → `{--passphrase, --passphrase-stdin}` both sides; `--no-auto-repair` global=true/secret=false (shared `NO_AUTO_REPAIR_FLAG`, like all subcommands).
- **Dropdowns:** all 4 reused (`--format`→EXPORT_FORMATS[11 incl descriptor], `--template`→TEMPLATES, `--network`→NETWORKS, `--language`→LANGUAGES); none added. `allows_slots:false`, `conditional:None` (gui-schema `conditional_rules:[]`).
- **Named gates (cargo +1.94.0, 4 `*_BIN` set):** full workspace `test --workspace` → 38× ok, **0 failed**; `pin_coherence::cargo_toolkit_pin_matches_pinned_upstream_mnemonic_tag` ok; `schema_mirror` 21/21 incl `mnemonic_schema_flag_names_match_help_text` ok (runs v0.43.0 bin vs mirror); `schema_mirror_secret_drift::secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode` ok; `clippy --all-targets -D warnings` exit 0.
- **Pins:** Cargo.toml `version="0.24.0"` + git-dep `tag=mnemonic-toolkit-v0.43.0`; `pinned-upstream.toml [mnemonic].tag=mnemonic-toolkit-v0.43.0` (== Cargo.toml, pin_coherence); Cargo.lock mnemonic-toolkit `version=0.43.0` source `tag=mnemonic-toolkit-v0.43.0#0f404aeca6…` (SHA = tag deref = master tip).
- **Banners:** `pinned_version:"mnemonic 0.43.0"`; module-doc `…from mnemonic-toolkit-v0.43.0.`; CHANGELOG `[0.24.0] — 2026-06-04` MINOR.
- **Scope:** `git diff --stat master..HEAD` = 5 files (`src/schema/mnemonic.rs`, `Cargo.toml`, `Cargo.lock`, `pinned-upstream.toml`, `CHANGELOG.md`); NO toolkit-repo file touched.
- **Over/under-reach:** no other GUI change needed — `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` untouched (restore is `--from`-driven, `allows_slots:false`); `secret_drift` passing confirms the two passphrase flags need no new projection const/redaction rule/form-conditional.

**Bottom line:** GREEN 0C/0I. Cleared to tag `mnemonic-gui-v0.24.0`.
