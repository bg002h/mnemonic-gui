# SPEC — mnemonic-gui v0.26.0 — `xpub-search passphrase-of-xpub --passphrase-candidates-file` schema mirror + toolkit-v0.46.0 pin catch-up

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** `mnemonic-toolkit` FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`.
**Source SHA:** mnemonic-toolkit `3115f51` (tag `mnemonic-toolkit-v0.46.0`); GUI base `a9abac2` (tag `mnemonic-gui-v0.25.0`).
**SemVer:** MINOR (`schema_mirror` flag-name delta → `0.25.0 → 0.26.0`; mirrors the v0.25.0 catch-up precedent).

---

## 1. Summary

`mnemonic-toolkit-v0.46.0` added `--passphrase-candidates-file` to `xpub-search passphrase-of-xpub` (candidate-list passphrase scan). The GUI (pinned toolkit v0.44.0, tag mnemonic-gui-v0.25.0) drifts: `schema_mirror` fails with `xpub-search-passphrase-of-xpub: only in upstream: ["--passphrase-candidates-file"]` (RED baseline captured vs the v0.46.0 binary). This cycle bumps the toolkit pin v0.44.0 → v0.46.0 and mirrors the one new flag, shipping v0.26.0. **GUI-repo-only.**

**Single-flag delta.** The only flag-name change between v0.44.0 and v0.46.0 is `--passphrase-candidates-file` (v0.45.0's multisig `restore --format` was behavior-only on an existing flag → no `schema_mirror` delta). The `--no-auto-repair` seen in a naive diff is the usual global-`NO_AUTO_REPAIR_FLAG` const-ref false positive.

## 2. Empirical baseline (captured pre-implementation)

- `schema_mirror` (`+1.94.0`, `MNEMONIC_BIN=`v0.46.0): `mnemonic_schema_flag_names_match_help_text` FAILS — `only in upstream: ["--passphrase-candidates-file"]` for `xpub-search-passphrase-of-xpub`; **no other subcommand drifted**.
- Toolkit `gui-schema` for `xpub-search-passphrase-of-xpub`: `--passphrase-candidates-file` is `kind=path`, `secret=None` (non-secret); `conditional_rules: 0` (the 3-way passphrase `ArgGroup` is NOT projected → no GUI conditional fn change). The existing `--passphrase`/`--passphrase-stdin` stay `required:false` (the mutex is clap-enforced, not GUI-modeled — unchanged posture).
- Toolchain: CI `dtolnay/rust-toolchain@stable`; run local builds/tests with `+1.94.0` (the default nightly ICEs).

## 3. Schema change — `src/schema/mnemonic.rs`

Add ONE `FlagSchema` to `XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS`, mirroring the established path-flag shape `--decrypt-password-file` (`:2148-2156`):

```rust
FlagSchema {
    name: "--passphrase-candidates-file",
    kind: FlagKind::Path { stdio_sentinel: false },
    required: false,
    repeating: false,
    help: "Scan a file of candidate BIP-39 passphrases (one per line); first \
           match against --target-xpub wins. A PATH (non-secret); the file \
           itself holds secret candidates.",
    secret: false,
    default_value: None,
    global: false,
},
```

Placement: alongside the other passphrase-source flags (`--passphrase`/`--passphrase-stdin`) in the array. `secret: false` is correct (a PATH, not the secret value — mirrors `--decrypt-password-file`/`--secret-file`; the toolkit emits `secret=None`). `FlagKind::Path { stdio_sentinel: false }` (no `-` stdin sentinel — it is always a real path). **No conditional fn** (toolkit emits `conditional_rules: 0`; `xpub-search-passphrase-of-xpub`'s `SubcommandSchema.conditional` stays `None`).

## 4. Pin + version bump

- `Cargo.toml` `[dependencies].mnemonic-toolkit.tag`: `v0.44.0` → `v0.46.0` (`:42`).
- `pinned-upstream.toml` `[mnemonic].tag`: `v0.44.0` → `v0.46.0` (`:22`). `pin_coherence` asserts the two agree.
- `Cargo.lock`: regenerated to `mnemonic-toolkit v0.46.0` (stage it). The lib re-export const-assert (`SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS`) is unchanged v0.44.0→v0.46.0 (the new flag is non-secret; no new NodeType/SlotSubkey) → still compiles.
- `Cargo.toml` `version`: `0.25.0` → `0.26.0` (`:3`).
- `src/schema/mnemonic.rs` module-doc (`:1`) + `pinned_version` (`:3672`): `0.44.0`/`v0.44.0` → `0.46.0`/`v0.46.0`.
- `CHANGELOG.md`: new `## mnemonic-gui [0.26.0]` entry.

## 5. Tests
- `schema_mirror` GREEN vs the v0.46.0 binary once §3 lands (run with all four pinned `*_BIN`: mnemonic 0.46.0 / md 0.6.2 / ms 0.7.0 / mk 0.7.0).
- `schema_mirror_secret_drift`: unaffected (new flag is `secret:false` → not in the secret set; `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` unchanged).
- `gui_schema_conditional_drift`: unaffected (`xpub-search-passphrase-of-xpub` emits 0 rules; still skipped).
- `pin_coherence`: GREEN after §4 (both pins v0.46.0).
- `cli_gui_schema` subcommand-name freeze: unaffected (a flag-add to an existing subcommand adds no subcommand).
- Full `cargo +1.94.0 test -p mnemonic-gui --no-fail-fast` (4 pinned bins) + `cargo +1.94.0 clippy -p mnemonic-gui --all-targets` GREEN.
- **Widget:** `--passphrase-candidates-file` is `FlagKind::Path` → auto-rendered by the schema-driven path widget (same as `--decrypt-password-file`); no bespoke widget. Confirm no kittest snapshot pins the passphrase-of-xpub flag list.

## 6. Lockstep / ship
- **Toolkit repo:** flip FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump` `open → resolved (mnemonic-gui-v0.26.0)` (separate toolkit-repo commit after the GUI tag exists).
- **manual-gui** (`mnemonic-toolkit/docs/manual-gui/`, pinned `mnemonic-gui-v0.3.0`, `#[ignore]`-gated): OUT of scope (separate deferred track; prior catch-ups never touched it).
- **`mnemonic-gui/CLAUDE.md`:** no per-cycle note needed (the FOLLOWUPS cross-cite is the record).

## 7. Phased plan
- **Phase 1 (RED):** baseline already captured (§2). No new GUI test cell needed (`schema_mirror` is the gate; it's currently RED).
- **Phase 2 (GREEN):** §3 flag add. `schema_mirror` + full suite (4 pinned bins, `+1.94.0`) + clippy GREEN. Per-phase opus review → persist to `design/agent-reports/`.
- **Phase 3 (pin/version):** §4 pin + version + module-doc/pinned_version + CHANGELOG; `pin_coherence` GREEN; full suite + clippy GREEN. Per-phase review.
- **Phase 4 (ship):** clean tree → `git checkout master && ff-merge` → tag `mnemonic-gui-v0.26.0` → push master + tag → watch CI (`build`, `schema-mirror`). Then flip the toolkit FOLLOWUP.

## 8. Risk
Very low — single non-secret path flag mirroring an established shape + a one-version pin bump. No new conditional, no secret-projection change, no new dropdown enum, no bespoke widget.
