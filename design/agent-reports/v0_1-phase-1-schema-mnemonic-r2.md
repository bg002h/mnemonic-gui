# Phase 1 Schema Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `089233e fold Phase 1 R1 (3C/1I + 1 R1-verification catch)`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.3 + §C Phase 1
**R1 report:** `design/agent-reports/v0_1-phase-1-schema-mnemonic-r1.md`

## Verdict

**0C / 0I — converge**

All 5 R1 folds applied correctly. No fold-introduced bugs. No new findings above threshold.

---

## Fold verification

### C-1 — `ELECTRUM_VERSIONS` — PASS

Post-fold: `&["standard", "segwit"]`. Upstream `parse_electrum_version_arg` (`convert.rs:272-286`) accepts exactly those two tokens.

### C-2 — `NODE_TYPES` order/completeness — PASS

13 tokens in exact `NodeType::as_str()` declaration order from `convert.rs:48-64`. Spurious `"master_xpub"` gone; `"minikey"` present. Grep across `src/**/*.rs` for `NODE_TYPES[` returns zero results — no downstream consumer uses NODE_TYPES by index, so reorder cannot break anything. Schema-mirror test reads `f.name` only — dropdown contents invisible.

### C-3 — `--bundle-json` `stdio_sentinel: false` — PASS

Upstream `load_bundle_json_into_args` at `verify_bundle.rs:525-527` calls `std::fs::read_to_string(path)` unconditionally; no `-` sentinel branch.

### C-extra — `BIP85_APPLICATIONS` — PASS

9 tokens verified exhaustively against `derive_child.rs:116-198` match arms (rsa, rsa-gpg, bip39, hd-seed, xprv, hex, password-base64, password-base85, dice). Comment's `line 121` reference is off-by-one vs the first match arm at line 122 — cosmetic, below threshold.

### I-1 — `pinned_version = "mnemonic 0.8.0"` — PASS

Confirmed by upstream `Cargo.toml` `version = "0.8.0"` and binary name `"mnemonic"`. Clap default `--version` emits `"<binary-name> <version>"`. String will match exactly. Dual-doc update in `src/schema/mod.rs:16-20` present.

---

## Hunt for fold-introduced bugs

- **Index-based NODE_TYPES consumer?** Zero matches for `NODE_TYPES[` across the codebase. All uses pass the slice by reference.
- **`pinned_version` string format?** Binary name `"mnemonic"` confirmed; clap default version output format will match `"mnemonic 0.8.0"`.
- **BIP85_APPLICATIONS in schema_mirror test?** Test collects `f.name` only; dropdown values are invisible. No test-path failure possible.
- **New spurious/missing flags from folds?** None — all five subcommand flag arrays are unchanged in arity; folds touched only constant values and comments.

---

## Confidence-filtered: omitted

No items above threshold.

---

## Build / test state

- `Cargo.toml` unchanged.
- `tests/schema_mirror.rs` unchanged.
- All fold edits scoped to `src/schema/mnemonic.rs` constant values + `src/schema/mod.rs` doc-comments.
