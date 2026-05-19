# CLAUDE.md — mnemonic-gui repo notes

This file is auto-loaded by Claude Code when starting a session in this repository.

## What this is

`mnemonic-gui` is the cross-platform GUI overlay for the **m-format constellation** CLIs (`mnemonic`, `md`, `ms`, `mk`). Built with [`egui`](https://github.com/emilk/egui); single statically-linked binary per platform (Linux x86_64/aarch64, macOS x86_64/ARM, Windows x86_64).

The GUI does not implement CLI logic itself — it assembles flag-argv arrays and spawns the toolkit/codec binaries pinned in `pinned-upstream.toml`. The constellation source repos are:

- [`mnemonic-toolkit`](https://github.com/bg002h/mnemonic-toolkit) — top-level integration crate + `mnemonic` CLI; the primary upstream.
- [`md-codec`](https://github.com/bg002h/descriptor-mnemonic) — wallet descriptor codec; CLI `md`.
- [`mk-codec`](https://github.com/bg002h/mnemonic-key) — xpub codec; CLI `mk`.
- [`ms-codec`](https://github.com/bg002h/mnemonic-secret) — BIP-39 entropy codec; CLI `ms`.

## Cross-repo follow-ups

When GUI work surfaces an action item that affects a sibling codec or the toolkit, mirror an entry in BOTH repos' `FOLLOWUPS.md` / `design/FOLLOWUPS.md` with cross-citing `Companion:` lines. When the action ships, both entries update in lockstep.

## GUI schema-mirror coverage (toolkit lockstep)

`src/schema/mnemonic.rs` is the hand-maintained clap-flag schema mirror of the `mnemonic-toolkit` CLI surface (subcommand-by-subcommand flag listings + dropdown value enums), enforced by the `schema_mirror` integration test which runs `mnemonic gui-schema` against the pinned toolkit binary and compares against the hand-maintained schema.

**Mirror invariant (companion to toolkit `CLAUDE.md`):** any toolkit CLI surface change (clap flag / option / subcommand / dropdown value add / remove / rename) MUST update `src/schema/mnemonic.rs` in lockstep — same PR if cross-repo authoring is feasible, otherwise a paired sibling PR. The `schema_mirror` test fires on pin bumps (`pinned-upstream.toml` bump) as a **lagging indicator**; the leading discipline is the paired-PR rule.

When ingesting a toolkit pin bump, run `cargo test --test schema_mirror` FIRST. If it fails with missing flags, those represent accumulated lockstep gaps — address them inline in the pin-bump PR.

Historical case study (v0.27.0 + v0.27.1): neither toolkit cycle paired its CLI additions with a GUI schema-mirror update. The v0.11.1 pin bump v0.26.0 → v0.27.2 fired the drift gate against 8 accumulated missing flags (`bundle --import-json`, `--import-json-index`; `export-wallet --bsms-form`, `--from-import-json`, `--from-import-json-index`, `bsms` format, `BSMS_FORMS` enum; `import-wallet --bsms-round1`, `--bsms-verify-strict`). Phase 3 of v0.11.1 had to backfill all 8 in one go.

See toolkit `design/FOLLOWUPS.md` entry `gui-schema-mirror-lockstep-discipline` for the canonical record. Companion convention lives in `mnemonic-toolkit/CLAUDE.md`.

## Conventions

- Source in `src/`; schema mirror at `src/schema/mnemonic.rs`.
- Design artifacts in `design/`: plan docs, session handoffs, `FOLLOWUPS.md` at repo root.
- Per-phase TDD: tests written before impl. Per-phase reviewer-loop until 0 critical / 0 important.
- Stage paths explicitly (no `git add -A`).
- Pin bumps: update `pinned-upstream.toml` AND run `cargo test --test schema_mirror` before committing.
