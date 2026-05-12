# Phase 1 Schema Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `fbfa353 Phase 1: schema types + mnemonic schema (5 subcommands)`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.3 + §C Phase 1

## Verdict

**3C / 1I — fold needed**

Three critical findings hidden by the schema-mirror test (which checks only flag-name set equality, not dropdown contents); one important finding on runtime-version semantics.

---

## Critical findings

### C-1 — `ELECTRUM_VERSIONS` contains tokens upstream parser rejects

**Confidence:** 98
**File:** `src/schema/mnemonic.rs:80` (pre-fold)
**Source verified:** `crates/mnemonic-toolkit/src/cmd/convert.rs:272-286` (`parse_electrum_version_arg`)

The pre-fold dropdown was `&["standard", "segwit", "2fa", "2fa-segwit"]`. Upstream parser accepts ONLY `"standard"` and `"segwit"`; `"standard-2fa"` / `"segwit-2fa"` / `"101"` / `"102"` are explicitly refused with a 2FA-unsupported error; anything else returns a generic "must be one of" error. The bare `"2fa"` and `"2fa-segwit"` strings the GUI offered are NOT among the recognized refusal tokens — they fall through to the generic "must be one of" error. Any GUI selection of those two options would produce an unconditional upstream parse error.

**Fold:** `ELECTRUM_VERSIONS = &["standard", "segwit"]` with rationale comment citing the upstream line range.

### C-2 — `NODE_TYPES` missing `"minikey"`; spurious `"master_xpub"`

**Confidence:** 92
**File:** `src/schema/mnemonic.rs:59-73` (pre-fold)
**Source verified:** `crates/mnemonic-toolkit/src/cmd/convert.rs:48-83` (`NodeType::{as_str, from_token}`)

Pre-fold list had 13 entries including a spurious `"master_xpub"` (which is a `SlotSubkey` token from `slot_input.rs`, NOT a `NodeType` for `--from`/`--to`). The list was missing `"minikey"` (Casascius mini-key — present in upstream `NodeType::MiniKey`). Re-ordered to exactly mirror upstream `NodeType::as_str()` declaration order (13 tokens: phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address). Rationale comment cites the upstream line range.

### C-3 — `--bundle-json` `stdio_sentinel: true` with no upstream stdin support

**Confidence:** 95
**File:** `src/schema/mnemonic.rs:297-307` (pre-fold)
**Source verified:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:68, 526` (`bundle_json: Option<PathBuf>`, `std::fs::read_to_string(path)` unconditional)

Pre-fold schema declared `Path { stdio_sentinel: true }`. Upstream calls `std::fs::read_to_string` unconditionally on the path — there is no `-` → stdin code path. Emitter would generate `--bundle-json -` as valid argv, which upstream cannot handle (IO error attempting to open a file named `-`).

**Fold:** `stdio_sentinel: false` with rationale comment citing upstream line range. Future stdin support would require an upstream change first (cross-repo FOLLOWUPS at that time per CLAUDE.md mirror-invariant discipline).

### C-extra (caught during R1 verification, not in reviewer's report)

**`BIP85_APPLICATIONS` drift.** Pre-fold list `&["bip39", "wif", "xprv", "hd-seed", "rsa", "rsa-gpg", "dice"]` had:
- Spurious `"wif"` (no such application token upstream).
- Missing `"hex"`, `"password-base64"`, `"password-base85"` (all parse-valid per `cmd::derive_child.rs:121-176` match arms).

**Source verified:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs:117, 122, 135, 139, 143, 154, 165, 176` — match arms enumerate the 9 tokens upstream actually recognizes (7 parse-valid for execution, `rsa`/`rsa-gpg` parse-valid but explicitly refused).

**Fold:** `BIP85_APPLICATIONS = &["bip39", "hd-seed", "xprv", "hex", "password-base64", "password-base85", "dice", "rsa", "rsa-gpg"]` with rationale comment noting that `dice` IS parse-valid upstream even though the help-text labels it "out-of-scope" — the schema follows the parser, not the help-text prose.

---

## Important findings

### I-1 — `pinned_version: "mnemonic-toolkit-v0.8.1"` will never match `mnemonic --version`

**Confidence:** 93
**File:** `src/schema/mnemonic.rs:737` (pre-fold)

The `mnemonic-toolkit-v0.8.1` git tag did NOT bump `crates/mnemonic-toolkit/Cargo.toml::version` (still `"0.8.0"` at the tag). So `mnemonic --version` outputs `"mnemonic 0.8.0"`. SPEC §11 runtime soft-check compares `--version` output against `Schema.pinned_version` — the prior value would have triggered a mismatch banner on every GUI launch with the correct binary.

**Fold:** `pinned_version = "mnemonic 0.8.0"` with a rationale comment + dual-doc update in `src/schema/mod.rs::Schema.pinned_version` clarifying:
- `pinned_version` = `--version` output literal (runtime soft-check)
- `pinned-upstream.toml::[mnemonic].tag` = git-tag string (CI install)

Phase 9's `schema_check.rs` reads BOTH — git-tag for CI install commands, `pinned_version` for the runtime banner.

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| `--from` classified `repeating: false` despite clap `Append` action | 55 | Upstream enforces exactly-one-primary at runtime; single-value widget is functionally correct |
| `--slot` carrying a `FlagSchema` alongside `allows_slots: true` | 40 | SlotEditor replaces widget but flag name must surface for schema-mirror test |
| `--taproot-internal-key` `TaggedOrIndexed(&["nums"])` with single tag | — | Upstream accepts exactly `"nums"` + `@N`; complete |
| `FlagVisibility` as `Vec` vs `HashMap` | — | Order-preservation + `Copy`-friendly rationale valid for Phase 5 callbacks |
| `--electrum-language` as `Text` not `Dropdown` | — | Upstream accepts wordlist names + aliases; `Dropdown` defensible but not mandated |
| `FormState { values: Vec<(&'static str, String)> }` shape | — | Adequate for Phase 5 lookup-by-name |
| `extract_flag_names` scanner edge cases | — | No divergence from `grep -oE` for help-text corpus |

---

## Fold verification

`MNEMONIC_BIN=... cargo test --test schema_mirror` — 2 cells green post-fold.

`cargo build` — clean.
