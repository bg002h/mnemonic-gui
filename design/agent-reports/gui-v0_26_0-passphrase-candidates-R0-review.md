# R0 Architect Review — SPEC mnemonic-gui v0.26.0 (passphrase-candidates-file flag + toolkit-v0.46.0 pin)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-05.
**Branch:** `gui-v0.26.0-passphrase-candidates-flag`. **Verdict:** **0 Critical / 0 Important — GREEN.** (3 non-blocking Minors, folded.)

> Persisted verbatim per CLAUDE.md. Reviewer verified the toolkit gui-schema *emitter mechanism* from source (no Bash). GREEN ⇒ implementation may proceed; the 3 Minors are SPEC-checklist polish folded without re-dispatch.

## Critical / Important
None / None.

## Minor (folded)
- **M1 — §5 omits the dedicated xpub-search flag-name gate.** `tests/xpub_search_schema_mirror.rs::umbrella_flag_names_match_toolkit_gui_schema_json` (`:235-272`) is a second set-equality gate over the four `xpub-search-*` subcommands (incl. passphrase-of-xpub). Satisfied by the same §3 flag-add, but §5 should list it (run with pinned `MNEMONIC_BIN`).
- **M2 — §4 omits the two README install-command version sites.** `README.md:42` (`mnemonic-gui-v0.25.0`) + `:50` (`mnemonic-toolkit-v0.44.0`) are currently CORRECT (the v0.25.0 cycle bumped them — established practice). Following §4 literally would ship v0.26.0 with a README telling users to install v0.25.0/v0.44.0. Ungated (no README guard in `tests/` or `.github/`), doc-only. Fix: README:42 → v0.26.0, README:50 → v0.46.0.
- **M3 — anti-blind-`sed` scope.** Bump ONLY module-doc `:1` + `pinned_version` `:3672`; the historical-provenance comments at `mnemonic.rs:344`/`:359`/`:514` legitimately carry `v0.44.0` and must stay.

## What verified clean (current source)
- **Single-flag delta ACCURATE:** toolkit CHANGELOG `[0.46.0]` + `[0.45.0]` confirm `--passphrase-candidates-file` is the only flag-name addition v0.44.0→v0.46.0 (v0.45.0 behavior-only on `--format`); v0.44.0 restore deltas already in GUI `RESTORE_FLAGS`. `--no-auto-repair` false positive real (array ends `NO_AUTO_REPAIR_FLAG` `:2839`).
- **FlagSchema literal compiles:** `schema/mod.rs:64-110` field set; `FlagKind::Path{stdio_sentinel:bool}` (`mod.rs:136-138`); template `--decrypt-password-file` (`mnemonic.rs:2147-2156`) is exactly `Path{stdio_sentinel:false}, secret:false`.
- **Secret classification ACCURATE + gated:** toolkit clap `Option<PathBuf>` → emitter `classify_kind`→`path` (`gui_schema.rs:1282`), `flag_is_secret` closed-set excludes it (`secrets.rs:49-64`) → non-secret. `schema_mirror_secret_drift` keys on `secret==true` → unaffected (would catch an accidental GUI `secret:true`). No `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` change → lib const-assert compiles.
- **No conditional ACCURATE:** `build_subcommand_conditional_rules` (`gui_schema.rs:336-346`) has no passphrase-of-xpub arm → `conditional_rules: []`; GUI SubcommandSchema `conditional: None` (`mnemonic.rs:3641-3648`); `gui_schema_conditional_drift` skips empty (`:228-230`), no orphan check. The 3-way `ArgGroup` is not projected.
- **Pin/version sites ACCURATE:** `Cargo.toml:42`(tag)+`:3`(version); `pinned-upstream.toml:22`; `mnemonic.rs:1`+`:3672`; `pin_coherence:24-38`. Siblings md 0.6.2/ms 0.7.0/mk 0.7.0 current.
- **Lockstep/scope ACCURATE:** no dropdown/subcommand add (`cli_gui_schema` freeze unaffected); `xpub_search_widgets.rs` uses `contains`/no-panic, not an exact snapshot → Path flag auto-renders; `manual_anchor_coverage.rs` `#[ignore]` (`:49-54`) + RED-by-design (manual-gui doesn't exist) → out of scope; toolkit FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump` open (`FOLLOWUPS.md:326-334`), prescription matches the SPEC literal.

## VERDICT: 0 Critical / 0 Important — GREEN. Implementation may proceed.

---

## Fold note (applied after persisting)
- **M1 — FOLDED:** §5 adds `tests/xpub_search_schema_mirror.rs` to the GREEN-gate list.
- **M2 — FOLDED:** §4 adds README:42 (`mnemonic-gui-v0.26.0`) + README:50 (`mnemonic-toolkit-v0.46.0`).
- **M3 — FOLDED:** §4 states bump ONLY module-doc `:1` + `pinned_version` `:3672` (NOT a blind sed; the `:344`/`:359`/`:514` provenance comments stay).
- GREEN ⇒ no re-dispatch.
