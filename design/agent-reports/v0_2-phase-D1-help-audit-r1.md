# Phase D.1 — Upstream --help Audit (R1)

**Date:** 2026-05-12
**Auditor:** feature-dev:code-explorer
**Subcommands audited:** md ×7, ms ×4, mk ×4 = 15
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase D.1

---

## Summary

| CLI | Subcommand | Flags | Positionals | Simple/Conditional | Notes |
|-----|------------|-------|-------------|--------------------|-------|
| md  | encode     | 11    | 1           | Conditional        | TEMPLATE XOR --from-policy (runtime); --context cond-req; --unspendable-key value-disabled |
| md  | decode     | 1     | 1           | Simple             | Trivial |
| md  | verify     | 4     | 1           | Simple             | --template clap-required; --key/--fingerprint blank upstream help |
| md  | bytecode   | 1     | 1           | Simple             | Low-level inspector |
| md  | vectors    | 1     | 0           | Simple             | Maintainer tool; --out = dir path |
| md  | compile    | 3     | 1           | Conditional        | --context clap-required; --unspendable-key disabled by --context value |
| md  | address    | 9     | 1           | Conditional        | PHRASES XOR --template; --key/--fingerprint require --template |
| ms  | encode     | 5     | 0           | Conditional        | --phrase XOR --hex; both secret; LANG tokens differ from mnemonic.rs |
| ms  | decode     | 2     | 1           | Simple             | LANG tokens hyphenated; positional supports stdin `-` |
| ms  | verify     | 3     | 1           | Simple             | --phrase secret (round-trip) |
| ms  | vectors    | 1     | 0           | Simple             | --pretty only |
| mk  | encode     | 9     | 0           | Conditional        | --origin-fingerprint XOR --privacy-preserving explicit |
| mk  | decode     | 1     | 1           | Simple             | Trivial |
| mk  | verify     | 6     | 1           | Simple             | All content-match flags optional; --policy-id-stub order-sensitive |
| mk  | vectors    | 2     | 0           | Simple             | --pretty silently ignored when --out set |

**Totals:** 59 flags, 10 positionals. 5 Conditional, 10 Simple.

---

## Three findings that drive D.2/D.3 decisions

1. **LANG_MS token drift (critical accuracy).** The ms CLI accepts `"chinese-simplified"` / `"chinese-traditional"` (hyphenated). mnemonic.rs uses fused tokens (`"simplifiedchinese"`). A separate `LANG_MS` const must be defined in `ms.rs`. Using the wrong const silently emits argv rejected by the binary.

2. **Two new conditional-fn patterns** (first occurrences in the codebase):
   - **Positionals-check** — `md_encode` and `md_address` must read `state.positionals[0]`. Add `FormState::has_positional(idx: usize) -> bool` helper.
   - **Dropdown value-inspect** — `md_encode` and `md_compile` must compare the string value of `--context` (not just presence) to gate `--unspendable-key`. Add `state.dropdown_value(name: &str) -> Option<&str>` helper.

3. **Secret additions.** `--phrase` (ms encode + verify) and `--hex` (ms encode) are new secret-bearing flags. 2 additions to `SECRET_FLAG_NAMES` set (via `secret: true` in schema source). No secrets in md or mk.

---

## Per-subcommand tables

### md encode

Encode a wallet policy into MD backup string(s). Two mutually exclusive input modes: `[TEMPLATE]` positional or `--from-policy`. Neither is clap-required; runtime errors if both absent.

| Flag | Kind | Required | Repeating | Secret |
|------|------|----------|-----------|--------|
| `--from-policy` | `Text` | false | false | false |
| `--context` | `Dropdown(["tap","segwitv0"])` | false | false | false |
| `--unspendable-key` | `Text` | false | false | false |
| `--path` | `Text` | false | false | false |
| `--key` | `Text` | false | true | false |
| `--fingerprint` | `Text` | false | true | false |
| `--network` | `Dropdown(NETWORKS)` | false | false | false |
| `--force-chunked` | `Boolean` | false | false | false |
| `--force-long-code` | `Boolean` | false | false | false |
| `--policy-id-fingerprint` | `Boolean` | false | false | false |
| `--json` | `Boolean` | false | false | false |

Positional: `template` (false, false). Conditional: TEMPLATE XOR --from-policy; --context cond-req when --from-policy; --unspendable-key value-disabled when --context=segwitv0.

### md decode
1 flag (`--json` Boolean); 1 positional (`strings`, true, true). Simple.

### md verify
4 flags (`--template` Text required; `--key` Text repeating; `--fingerprint` Text repeating; `--network` Dropdown). 1 positional (`strings`, true, true). Simple.

### md bytecode
1 flag (`--json` Boolean). 1 positional (`strings`, true, true). Simple.

### md vectors
1 flag (`--out` Path, no stdio_sentinel). 0 positionals. Simple. Maintainer tool.

### md compile
3 flags: `--context` Dropdown(["tap","segwitv0"]) clap-required; `--unspendable-key` Text; `--json` Boolean. 1 positional (`expr`, true, false). Conditional: --unspendable-key disabled when --context=segwitv0.

### md address
9 flags: `--template` Text; `--key` Text repeating; `--fingerprint` Text repeating; `--network` Dropdown(NETWORKS); `--chain` Number{0,65535}; `--change` Boolean; `--index` Number{0,2147483647}; `--count` Number{1,10000}; `--json` Boolean. 1 positional (`phrases`, false, true). Conditional: PHRASES XOR --template; --key/--fingerprint require --template; --change/--chain TBD (source-audit).

### ms encode
5 flags: `--phrase` Text **secret**; `--hex` Text **secret**; `--language` Dropdown(LANG_MS); `--no-engraving-card` Boolean; `--json` Boolean. 0 positionals. Conditional: --phrase XOR --hex; --language hidden under --hex.

### ms decode
2 flags: `--language` Dropdown(LANG_MS); `--json` Boolean. 1 positional (`ms1`, false, false). Simple. Note: ms1 positional secret if read from stdin context; not flagged for now.

### ms verify
3 flags: `--phrase` Text **secret**; `--language` Dropdown(LANG_MS); `--json` Boolean. 1 positional (`ms1`, false, false). Simple.

### ms vectors
1 flag (`--pretty` Boolean). 0 positionals. Simple.

### mk encode
9 flags: `--xpub` Text required; `--origin-fingerprint` Text; `--origin-path` Text required; `--policy-id-stub` Text repeating; `--from-md1` Text repeating; `--privacy-preserving` Boolean; `--force-chunked` Boolean; `--force-long-code` Boolean; `--json` Boolean. 0 positionals. Conditional: --origin-fingerprint conflicts_with --privacy-preserving.

### mk decode
1 flag (`--json` Boolean). 1 positional (`mk1-strings`, false, true). Simple.

### mk verify
6 flags: `--xpub` Text; `--origin-fingerprint` Text; `--origin-path` Text; `--policy-id-stub` Text repeating; `--from-md1` Text repeating; `--json` Boolean. 1 positional (`mk1-strings`, false, true). Simple.

### mk vectors
2 flags: `--pretty` Boolean; `--out` Path (no stdio_sentinel). 0 positionals. Simple. `--pretty` silently ignored when `--out` set — not a clap conflicts_with.

---

## Schema-entry draft estimate

| File | Net new lines |
|------|---------------|
| `src/schema/md.rs` | ~403 |
| `src/schema/ms.rs` | ~180 |
| `src/schema/mk.rs` | ~224 |
| `src/form/conditional.rs` | ~115 |
| **D.2 + D.3 grand total** | **~922 LOC** |

## Shared constants needed

- `md.rs`: `NETWORKS` (4 values, same as mnemonic.rs); `SCRIPT_CONTEXTS = ["tap", "segwitv0"]` (new).
- `ms.rs`: `LANG_MS` — 10 BIP-39 wordlist values with HYPHENATED Chinese (NOT mnemonic.rs's fused tokens).
- `mk.rs`: no new shared consts.

## Conditional fn entries needed (5 new in conditional.rs)

| Fn | Subcommand | Constraints | Value-inspect | Positionals-check |
|----|-----------|-------------|---------------|-------------------|
| `md_encode` | md encode | 4 (XOR + cond-req + value-disable + positional-disables) | ✓ (--context) | ✓ |
| `md_compile` | md compile | 1 (unspendable-key disabled when context=segwitv0) | ✓ (--context) | – |
| `md_address` | md address | 3 (XOR + key/fp require template + change/chain TBD) | – | ✓ |
| `ms_encode` | ms encode | 2 (XOR + language hidden under hex) | – | – |
| `mk_encode` | mk encode | 1 (origin-fp XOR privacy-preserving) | – | – |

## Secret flag additions

- `--phrase` (ms encode, ms verify): BIP-39 mnemonic.
- `--hex` (ms encode): raw entropy bytes.

No new md/mk secret flags (xpubs, fingerprints, derivation paths are public material; md1/mk1 strings are opaque encoded tokens).

---

Audit complete. Phase D.2 + D.3 can proceed against this report's tables. D.4 (egui_kittest cells 4+5) covers representative new subcommands.
