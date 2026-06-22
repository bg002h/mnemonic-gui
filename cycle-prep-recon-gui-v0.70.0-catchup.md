# cycle-prep recon — 2026-06-22 — GUI schema-mirror catch-up toolkit v0.60.0 → v0.70.0

**Toolkit master SHA / tag:** `423a4ad6` = `mnemonic-toolkit-v0.70.0`
**GUI branch:** `lockstep/toolkit-v0.70.0-pin-bump` (off `mnemonic-gui` master `5ce9d53`, v0.46.0)
**Truth binary:** `mnemonic 0.70.0` (`/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic gui-schema`)
**Method:** 4-agent parallel recon, each verified against authoritative source (live v0.70.0 binary output + toolkit source + GUI consuming code), NOT the changelog prose alone.

The GUI pin was 10 toolkit versions behind (the v0.60→v0.69.1 constellation burndown never re-pinned the GUI). This recon establishes the COMPLETE GUI-affecting accumulated delta — gated and ungated — before backfill.

---

## Per-surface verification

### 1. Flag-NAME parity (gated by `schema_mirror`) — DELTA = 3 flags, nothing else
Authoritative: `gui-schema` JSON of the v0.70.0 binary vs `src/schema/mnemonic.rs` flag-name sets, per subcommand (incl. flattened nested).
- `restore`: + `--search-cosigner-subset` (was 25 → 26 upstream).
- `verify-bundle`: + `--own-account-max`, + `--search-cosigner-subset` (27 → 29 upstream).
- bundle / convert / export-wallet / derive-child / all others: **exact match**.
- No removed/renamed flags. No subcommand-presence delta (30 = 30).
- All 3 documented in the v0.70.0 CHANGELOG `[Added]`. Hypothesis "only these 3" — **CONFIRMED**.

### 2. Dropdown VALUE enums (gated) — FULLY IN SYNC, no backfill
- All 18 `FlagKind::Dropdown` flags (27 subcommand×flag instances) match the binary value sets byte-for-byte (`NETWORKS`, `TEMPLATES`, `EXPORT_FORMATS`, `IMPORT_WALLET_FORMATS`, `ARCHETYPES`, `MD1_FORMS`, `BSMS_FORMS`, `SEARCH_CHAINS`, …).
- Changelog v0.61.0→v0.70.0: **zero** enum value add/remove/rename (every release states "no dropdown change").
- Benign: `--address-type` (xpub-search-address-of-xpub) is a GUI-narrowed dropdown the binary reports as `kind: text` — same safe pattern as `--separator`; NOT a drift.

### 3. `--json` wire-shape/value (UNGATED — schema_mirror does NOT cover this) — NO consumer-code change
- Only two `--json` changes in the whole range, both v0.66.0, both wire-VALUE corrections on EXISTING fields (no shape change):
  - **M7** — `bundle --json` `threshold` now real K (was cosigner count N), descriptor / `--import-json` mode.
  - **M1** — `import-wallet --json` `bundle.account` now real BIP-32 account (was hardcoded 0).
- **The GUI deserializes exactly ONE subcommand's `--json`: `build-descriptor`** (`src/form/tree_form.rs` `apply_validate_result` / `apply_emit_spec_result` — keys `descriptor`/`cost`/`diagnostics`). Every other subcommand (incl. `bundle`, `import-wallet`, `export-wallet`, `restore`, `verify-bundle`, `xpub-search`) is rendered as RAW TEXT (`main.rs` monospace panel), never deserialized.
- `build-descriptor`'s envelope had **zero** change in the range (the v0.63 S-NET change is stderr-only).
- VERDICT: M7/M1 land on subcommands the GUI does not parse → **no impact**; even if parsed, value-only on an existing same-typed field would not break serde.

### 4. Stale prose / behavior mirror (ungated, GUI-discretionary) — ONE real fix
- GUI flag `help` strings are **free-form / GUI-authored**, NOT verbatim-mirrored and NOT gated by any test (`schema_mirror` reads only `--<name>` tokens). So stale help is discretionary — but a flatly-WRONG string is a real user-visible defect.
- **STALE (fix):** `restore --own-account-max` help + preceding comment (`src/schema/mnemonic.rs:709-711, 720-722`) still say *"reserved/refused this cycle … NOT SUPPORTED YET — refused"*. v0.70.0 flipped this flag refuse→active subset-search. Funds-misleading; rewrite + write the new `verify-bundle --own-account-max` help fresh (don't copy the "refused" pattern).
- **NOT MIRRORED — leave unmodeled:** the `--own-account-max ⊕ --account` clap `conflicts_with` is NOT projected into the toolkit's `gui-schema conditional_rules` (restore stays 1 rule, verify-bundle 10 — pinned in `tests/gui_schema_conditional_drift.rs`). Adding a GUI conditional rule would BREAK the drift-gate count. Runtime-only concern.
- L8 / L9 / L21 / decay-ordering / S-NET behavior changes: **NO IMPACT** — all runtime fail-closed refusals with no contradicting GUI prose/notice.

---

## Backfill worklist (complete + bounded)
1. `restore`: rewrite stale `--own-account-max` comment+help; add `--search-cosigner-subset` (Boolean).
2. `verify-bundle`: add `--own-account-max` (Number, mirror restore) + `--search-cosigner-subset` (Boolean), fresh help.
3. Do NOT model the mutex; do NOT touch any `--json` consumer; do NOT change dropdown values.
4. Version sites: README `cargo install --tag` (failing `readme_pin_coherence`); `Cargo.toml` v0.46.0 → **v0.47.0** (MINOR — flag additions); CHANGELOG; `pinned-upstream.toml` (done); `Cargo.lock` (done).
5. GREEN gate: full `cargo test` with `MNEMONIC_BIN=<v0.70.0 build>`; then post-impl adversarial review of the diff.

**SemVer: MINOR** (new clap-flag surface mirrored). No `cargo fmt` on the GUI. No sibling-codec lockstep (toolkit-only flags).
