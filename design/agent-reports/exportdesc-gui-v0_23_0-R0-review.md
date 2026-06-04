# mnemonic-gui v0.23.0 — R0 Review (export-wallet --format descriptor lockstep)

**Verdict: GREEN (0 Critical / 0 Important).** Cleared to tag `mnemonic-gui-v0.23.0`.

Single-phase mechanical schema-mirror + pin-coherence lockstep for `mnemonic-toolkit-v0.42.0`'s new `export-wallet --format descriptor` value. This one opus R0 (full shell, gate re-run independently) serves as both per-phase and end-of-cycle. Branch `export-wallet-format-descriptor-gui`, 1 commit `ea1ada2` over master (v0.22.0).

## Critical
None.

## Important
None.

## Minor

**M1 — GUI `schema_mirror` gates flag-NAMES only, NOT dropdown value-enums.** `src/schema_check.rs:97-104` (`struct GuiSchemaFlag`) deserializes only `name`, dropping `choices`; `tests/schema_mirror.rs:91-120` set-compares flag-NAME sets only. So the `EXPORT_FORMATS += "descriptor"` value-parity is **not** enforced by GUI `schema_mirror` — it is enforced by (a) manual review + set-equality check (done in this R0, see ledger) and (b) the **toolkit's** own `tests/cli_gui_schema.rs` `choices.len() == 11` assertion. This is the same conceptual gap already tracked in toolkit `CLAUDE.md` / FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` (and the v0.27.2 `BSMS_FORMS` backfill case study, where value/enum gaps surfaced via downstream import-path failures, not `schema_mirror`). **Zero impact on this cycle** — `descriptor` was added correctly and confirmed set-equal. Recommend a FOLLOWUP to extend the GUI gate to deserialize + set-compare `choices`. Non-blocking.

## Verification ledger (reviewer ran every command)

**Branch / diff:** HEAD `export-wallet-format-descriptor-gui`; `git log --oneline master..HEAD` = 1 commit `ea1ada2`; `git diff --stat master..HEAD` = exactly 5 files (`CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `pinned-upstream.toml`, `src/schema/mnemonic.rs`), 13+/8-; `git status --porcelain` empty (no stray edits).

**Version / pin greps:** `src/schema/mnemonic.rs:1` module-doc `mnemonic-toolkit-v0.42.0`; `:69` `EXPORT_FORMATS += "descriptor"`; `:3453` `pinned_version: "mnemonic 0.42.0"`; `:804` `--format` → `FlagKind::Dropdown(EXPORT_FORMATS)` default `bitcoin-core`. `Cargo.toml:3` `version = "0.23.0"`; `:42` toolkit `tag = "mnemonic-toolkit-v0.42.0"`. `pinned-upstream.toml:22` `[mnemonic].tag = "mnemonic-toolkit-v0.42.0"`. `Cargo.lock` mnemonic-gui `0.23.0`, mnemonic-toolkit `0.42.0` source `tag=mnemonic-toolkit-v0.42.0#6566941…` (SHA matches v0.42.0 tag HEAD). `CHANGELOG.md [0.23.0] — 2026-06-03`, MINOR, describes lockstep + pin bump + no-secret-delta.

**Binaries:** `mnemonic --version` = `0.42.0`; `ms 0.7.0`, `md 0.6.2`, `mk 0.7.0` present.

**Dropdown set-equality:** `mnemonic gui-schema` `export-wallet → --format` `choices` (11) == GUI `EXPORT_FORMATS` (11) as a SET — only-in-toolkit = ∅, only-in-gui = ∅. Both include `descriptor`.

**Gate (`cargo +1.94.0`, all four `*_BIN` set):** `test --workspace --no-fail-fast` → all `ok`, 0 failed across 38 suites (1 network-gated ignored). Named gates: `pin_coherence` ok 1/1; `schema_mirror` ok 21/21 (incl flag-NAME set-equality across all four CLIs' subcommands, binary exec'd not skipped); `schema_mirror_secret_drift` ok 1/1. `clippy --all-targets -- -D warnings` → exit 0.

**Over-reach / missed-delta:** toolkit `git diff v0.41.0..v0.42.0` CLI-surface change = exactly one new dropdown VALUE (`CliExportFormat::Descriptor`, `#[value(name="descriptor")]`); no new flag NAME / subcommand / other dropdown value. Secret-projection consts (`SECRET_NODE_TYPES`, `SECRET_SLOT_SUBKEYS`) untouched — `export-wallet` is structurally watch-only (refuses secret slots), so `--format descriptor` emits public material only; `schema_mirror_secret_drift` green confirms no projection delta needed.
