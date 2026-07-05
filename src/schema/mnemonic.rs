//! Pinned schema for the `mnemonic` CLI from mnemonic-toolkit-v0.70.0.
//!
//! Five subcommands covered in v0.1 (Section A coverage table):
//!   - bundle
//!   - verify-bundle
//!   - convert
//!   - export-wallet
//!   - derive-child
//!
//! `conditional` slots are all `None` at Phase 1; Phase 5 wires the 11
//! upstream `conflicts_with` / `required_unless_present_any` constraints
//! into hand-coded `fn(&FormState) -> FlagVisibility` callbacks here.
//!
//! v0.10.0 Tranche B.3 (D31/D33): mirrors toolkit v5 schema fields
//! (`default_value`, `global`, `secret`). Each FlagSchema entry is
//! reconciled with the toolkit's `gui-schema` JSON v5 output ground truth.
//! `--no-auto-repair` is wired per-subcommand as `global: true`, replacing
//! the v0.9.0 R7 action-bar fallback (D33 cycle close).

use super::{FlagKind, FlagSchema, NumberMax, PositionalArgSchema, Schema, SubcommandSchema};

/// Empty positional-args slice — all five mnemonic-toolkit subcommands
/// take their input via flags (`--slot`, `--from`, etc.) with zero
/// positionals.
const NO_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── Shared dropdown option lists ───────────────────────────────────────

const NETWORKS: &[&str] = &["mainnet", "testnet", "signet", "regtest"];
// mstring display-grouping (toolkit v0.56.0): `--separator` keyword values.
// SPEC §I7 — the GUI MUST emit a KEYWORD (space|hyphen|comma), never a
// literal space, to avoid argv/whitespace ambiguity through the GUI→argv
// path. The toolkit's gui-schema reports `--separator` as kind `text`
// (keyword-or-literal value_parser); the GUI narrows it to this dropdown.
// schema_mirror gates flag NAMES only, so the kind divergence is safe.
const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];
// toolkit v0.36.0: `verify-message --format` value-enum.
const VERIFY_FORMATS: &[&str] = &["auto", "legacy", "bip322"];
// toolkit v0.50.0: `build-descriptor --format` value-enum (CliBuildFormat).
const BUILD_FORMATS: &[&str] = &["descriptor", "bip388"];
// toolkit v0.51.0/v0.52.0: `build-descriptor --archetype` value-enum
// (CliArchetype order). The leading `""` is the GUI-side UNSET sentinel
// (v0.30.0 R0-r1 C1): `emit_one` skips an empty Dropdown, `has_value` is
// false, and the user can re-select the "(none)" display row — without it
// the default form would emit a guaranteed-refusal `--archetype
// decaying-multisig` AND the archetype↔spec mutex would deadlock `--spec`
// from frame 1. The sentinel is GUI-only; the toolkit never sees it
// (empty Dropdown is never emitted). Value-list parity with the binary is
// pinned by `tests/build_descriptor_schema.rs` (the value-enum is NOT
// gate-checked by `schema_mirror`, which compares flag NAMES only).
const ARCHETYPES: &[&str] = &[
    "",
    "decaying-multisig",
    "hashlock-gated",
    "kofn-recovery",
    "simple-timelocked-inheritance",
    "tiered-recovery",
];
// toolkit v0.52.0: `build-descriptor --allow` value-enum (sanity-rule
// opt-out names). Parity pinned by `tests/build_descriptor_schema.rs`.
const ALLOW_RULES: &[&str] = &[
    "malleable",
    "mixed-timelock",
    "repeated-keys",
    "resource-limit",
    "sigless-branch",
];

const TEMPLATES: &[&str] = &[
    "bip44",
    "bip49",
    "bip84",
    "bip86",
    "wsh-multi",
    "wsh-sortedmulti",
    "sh-wsh-multi",
    "sh-wsh-sortedmulti",
    "tr-multi-a",
    "tr-sortedmulti-a",
];

const LANGUAGES: &[&str] = &[
    "english",
    "simplifiedchinese",
    "traditionalchinese",
    "czech",
    "french",
    "italian",
    "japanese",
    "korean",
    "portuguese",
    "spanish",
];

const MULTISIG_PATH_FAMILIES: &[&str] = &["bip48", "bip87"];

const EXPORT_FORMATS: &[&str] = &[
    "bitcoin-core",
    "bip388",
    "coldcard",
    "coldcard-multisig",
    "jade",
    "sparrow",
    "specter",
    "electrum",
    "green",
    "bsms",
    "descriptor",
];

const BSMS_FORMS: &[&str] = &["2-line", "4-line"];

// toolkit v0.59.0 (#28): `bundle --md1-form` value-enum (`Md1Form`).
// `policy` (default) = the pre-#28 full wallet-policy md1; `template` =
// a KEYLESS, fingerprint-stripped, canonical-origin-elided single-sig
// template card (requires `descriptor.n == 1` + a single-leaf shape;
// every other shape is refused upstream with `TemplateFormUnsupportedShape`).
// clap derives the value names from the variant names lowercased
// (`Md1Form::Policy`/`Template` → `policy`/`template`). Value-list parity
// is NOT gate-checked by `schema_mirror` (flag NAMES only).
const MD1_FORMS: &[&str] = &["policy", "template"];

// toolkit v0.60.0 (#28 phase 2): `restore --search-chain` /
// `verify-bundle --search-chain` value-enum (`SearchChain`). Which BIP-32
// change-chain branch(es) `--search-address` scans during a multisig
// template completion: `receive` (chain 0, default), `change` (chain 1),
// or `both`. clap derives the value names from the variant names lowercased.
// Value-list parity is NOT gate-checked by `schema_mirror` (flag NAMES
// only) — declared here for paired-PR discipline (the toolkit's gui-schema
// reports `--search-chain` as a dropdown with these three choices).
const SEARCH_CHAINS: &[&str] = &["receive", "change", "both"];

// L13 (cycle-11a): `--from` and `--to` carry DIFFERENT node-token sets — they
// are NOT the same list. Drift here is invisible to the schema-mirror
// flag-name test, so we hand-pin against the upstream source.
//
// `CONVERT_FROM_NODES` mirrors the INPUT enum `NodeType::as_str` ordering in
// `crates/mnemonic-toolkit/src/cmd/convert.rs:54-72` — it INCLUDES `seedqr`
// (index 1, after `phrase`). The toolkit accepts `--from seedqr=<digits>`.
//
// `CONVERT_TO_NODES` mirrors the toolkit's `--to` `PossibleValuesParser` in
// `crates/mnemonic-toolkit/src/cmd/convert.rs:209-223` — it EXCLUDES `seedqr`
// (seedqr is decode/input-only; the toolkit refuses `--to seedqr`).
//
// Master xpub / fingerprint plumbing for cosigner identification lives in
// SlotSubkey (slot_input.rs), NOT NodeType — `--from`/`--to` never see those
// tokens.
const CONVERT_FROM_NODES: &[&str] = &[
    "phrase",
    "seedqr",
    "entropy",
    "xpub",
    "xprv",
    "wif",
    "fingerprint",
    "path",
    "ms1",
    "mk1",
    "bip38",
    "minikey",
    "electrum-phrase",
    "address",
];

const CONVERT_TO_NODES: &[&str] = &[
    "phrase",
    "entropy",
    "xpub",
    "xprv",
    "wif",
    "fingerprint",
    "path",
    "ms1",
    "mk1",
    "bip38",
    "minikey",
    "electrum-phrase",
    "address",
];

// R1 C-extra fold (caught during R1 verification): BIP-85 applications
// exactly mirror upstream `cmd::derive_child.rs:121-176` match-arm tokens
// plus the rsa/rsa-gpg refusal arm at line 117. `dice` IS parse-valid
// upstream despite the --help text labeling it "out-of-scope" — the GUI
// follows the parser, not the help-text prose.
const BIP85_APPLICATIONS: &[&str] = &[
    "bip39",
    "hd-seed",
    "xprv",
    "hex",
    "password-base64",
    "password-base85",
    "dice",
    "rsa",
    "rsa-gpg",
];

const SCRIPT_TYPES: &[&str] = &["p2wpkh", "p2sh-p2wpkh", "p2tr"];

// R1 C-1 fold: upstream `parse_electrum_version_arg` (convert.rs:272-286)
// accepts ONLY "standard" and "segwit". "standard-2fa" / "segwit-2fa" /
// "101" / "102" are explicitly REFUSED with a specific 2FA-unsupported
// error. Other strings produce a generic "must be one of" error. So the
// GUI dropdown must offer only the two accepted tokens.
const ELECTRUM_VERSIONS: &[&str] = &["standard", "segwit"];

// ─── Global flags (toolkit v5 schema: `global: true`) ────────────────────
//
// v0.10.0 B.3 (D33): `--no-auto-repair` propagates to every mnemonic
// subcommand via clap-derive's `global = true`. Per the v5 schema, it is
// emitted in every subcommand's flag list with `global: true`. The GUI
// mirrors this by including the FlagSchema entry in every subcommand's
// flag array — the argv assembler emits it like any other Boolean flag.
//
// Pre-v0.10.0 the GUI carried an action-bar `no_auto_repair` checkbox +
// `runner::prepend_no_auto_repair` runner helper as the load-bearing R7
// fallback for the v4 schema's missing per-subcommand emission. v0.10.0
// drops the action-bar checkbox + runner helper in lockstep with the
// toolkit v5 schema-emission fix.
const NO_AUTO_REPAIR_FLAG: FlagSchema = FlagSchema {
    name: "--no-auto-repair",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Disable the auto-fire BCH error-correction short-circuit on \
           convert/inspect/verify-bundle. Global flag — propagates to \
           every subcommand.",
    secret: false,
    default_value: None,
    global: true,
};

// ─── bundle ──────────────────────────────────────────────────────────────

const BUNDLE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Display grouping: break the emitted card into groups of N \
               characters (default 5; 0 = unbroken single line). Cosmetic — \
               intake strips separators, so any grouping re-ingests.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Display-grouping separator keyword (space|hyphen|comma; \
               default space). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: true,
        repeating: false,
        help: "Bitcoin network for derivations + address encoding.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Pre-built template name. Mutually-required-one-of with \
               --descriptor / --descriptor-file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "User-supplied BIP-388 descriptor. XOR with --descriptor-file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor-file",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Path to a single-line UTF-8 descriptor file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--import-json",
        kind: FlagKind::Path {
            stdio_sentinel: true,
        },
        required: false,
        repeating: false,
        help: "v0.27.0 — synthesize a bundle from an import-wallet --json \
               envelope. Accepts a file path or - for stdin. Mutually \
               exclusive with --template / --descriptor / --descriptor-file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--import-json-index",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "v0.27.0 — pick a specific entry from a multi-entry envelope \
               array. Required when the envelope has > 1 entry.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic extension passphrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit envelope JSON (ms1/mk1/md1 + metadata).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--no-engraving-card",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Suppress the human-readable engraving-card panel.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--multisig-path-family",
        kind: FlagKind::Dropdown(MULTISIG_PATH_FAMILIES),
        required: false,
        repeating: false,
        help: "Multisig derivation path family (default bip87).",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.59.0 (#28): keyless single-sig template md1 form selector.
    FlagSchema {
        name: "--md1-form",
        kind: FlagKind::Dropdown(MD1_FORMS),
        required: false,
        repeating: false,
        help: "md1 card shape: `policy` (default) = full wallet-policy md1; \
               `template` = a keyless, fingerprint-stripped, \
               canonical-origin-elided single-sig template (descriptor.n == 1; \
               account/origin supplied at restore via --account/--origin).",
        secret: false,
        default_value: Some("policy"),
        global: false,
    },
    FlagSchema {
        name: "--privacy-preserving",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Suppress master fingerprint from mk1 + engraving card.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--self-check",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Re-parse the emitted bundle and verify round-trip.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 1, max: NumberMax::FromSlotCount },
        required: false,
        repeating: false,
        help: "Multisig threshold K (1 <= K <= N <= 16).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "v0.4 unified slot input. Repeating flag -- one occurrence per \
               (slot, subkey) tuple. Grammar: @N.<subkey>=<value>. Handled \
               by SlotEditor composite widget (SPEC §4).",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── restore (toolkit v0.43.0; +v0.44.0 multisig --md1/--cosigner) ──────────
//
// `restore` re-derives a wallet export (its inverse-ish sibling of
// `export-wallet`) from a third-party source given via `--from` (required).
// It shares export-wallet's `--format` (EXPORT_FORMATS, 11 values),
// `--template` (TEMPLATES), `--language` (LANGUAGES), `--network`
// (NETWORKS) dropdowns plus `--account` / `--output` defaults. The two
// passphrase flags are the only secret-bearing flags (mirrored from the
// toolkit v5 schema's per-flag `secret` field; see
// schema_mirror_secret_drift gate). `--no-auto-repair` is listed as the
// shared global FlagSchema, matching every other subcommand.
const RESTORE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::Text,
        // toolkit v0.44.0: `required_unless_present = "md1"` — required for
        // single-sig restore, optional own-cosigner cross-check in multisig
        // (--md1) mode. The `conditional::restore` fn marks it Required
        // while --md1 is empty; the base schema requiredness is false.
        required: false,
        repeating: false,
        help: "Source wallet export to restore from. @env:VAR / - (stdin) \
               for secret values.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(EXPORT_FORMATS),
        required: false,
        repeating: false,
        help: "Source export format (bitcoin-core, bip388, coldcard, …).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Pre-built descriptor template for the restored wallet.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network (default mainnet).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist language for phrase / seedqr sources \
               (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        // toolkit v0.60.0 (#28 phase 2): widened from a scalar to a
        // comma-separated list (`Vec<u32>`) — single-sig restore uses the
        // first value; a MULTISIG template completion uses the whole list
        // (one own key per account, e.g. `--account 0,1,2,3`). The toolkit's
        // gui-schema still reports `kind: number` (clap value-shape is not on
        // the wire), so the GUI keeps `FlagKind::Number` (the Number widget
        // does not model multiplicity); the comma-list is documented here for
        // the form tooltip. schema_mirror gates flag NAMES only.
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index(es) (default 0). Single-sig: the first \
               value. Multisig template completion: a comma-list, one own \
               account per slot (e.g. 0,1,2,3).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--count",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Number of accounts / entries to restore (default 1).",
        secret: false,
        default_value: Some("1"),
        global: false,
    },
    // toolkit v0.59.0 (#28): explicit origin derivation path + wallet-id
    // assertion for keyless single-sig template restore (restore --md1
    // <keyless-template>). `--origin` supplies the derivation path the
    // template elided; `--expect-wallet-id` recomputes the WalletPolicyId
    // from the completed fully-keyed explicit-origin descriptor and refuses
    // on mismatch (only checked when no --origin override is supplied).
    FlagSchema {
        name: "--origin",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Explicit origin derivation path for keyless single-sig \
               template restore (supplies the path the template elided).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--expect-wallet-id",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Assert the restored wallet's WalletPolicyId starts with this \
               prefix (refuse on mismatch; skipped when --origin overrides \
               the canonical account path).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--expect-fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Assert the restored wallet's master fingerprint matches \
               this value (refuse on mismatch unless --allow-mismatch).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--expect-xpub",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Assert the restored wallet's account xpub matches this \
               value (refuse on mismatch unless --allow-mismatch).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--allow-mismatch",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Downgrade --expect-fingerprint / --expect-xpub mismatch \
               from a refusal to a stderr advisory.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase (seed sources). @env:VAR supported.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the BIP-39 passphrase from stdin (conflicts with \
               --passphrase).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--output",
        kind: FlagKind::Path {
            stdio_sentinel: true,
        },
        required: false,
        repeating: false,
        help: "Output path. `-` (default) -> stdout.",
        secret: false,
        default_value: Some("-"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope instead of the text export.",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.44.0 multisig-cosigner restore: the shared wallet-policy
    // md1 card chunk(s) + per-position cross-check assertions. Both
    // repeating + non-secret (watch-only). `--from` is required_unless
    // `--md1` is present (see `conditional::restore`).
    FlagSchema {
        name: "--md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Multisig-cosigner restore: the shared wallet-policy `md1` \
               card chunk(s). Reconstructs the watch-only multisig \
               descriptor from the md1 alone; wsh / sh(wsh) only. Repeat \
               for chunked cards.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--cosigner",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Cross-check assertion (multisig mode): `@N=<mk1-chunk|xpub>` \
               — cosigner at position N is this public key. A mismatch is a \
               hard error (exit 4) unless --allow-mismatch. Watch-only \
               (non-secret).",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.60.0 (#28 phase 2): multisig / general-policy keyless
    // TEMPLATE completion (`bundle --md1-form=template`, n≥2). The own seed
    // (`--from`) + cosigner keys (`--cosigner`) plus a disambiguation input
    // (`--account <N[,N,…]>` list, `--own-account-max` range fallback,
    // `--search-address`, or `--expect-wallet-id`) drive a key→slot search.
    // toolkit v0.70.0: `--own-account-max` flips refuse→subset-search — it
    // over-supplies the own seed's accounts (0..K) and resolves the unique
    // template assignment via an own-anchored k-permutation search (own-only
    // by default; `--search-cosigner-subset` opts into bounded cosigner
    // over-supply). Mutually exclusive with `--account` (clap conflicts_with);
    // the mutex is NOT projected into gui-schema conditional_rules, so it is
    // deliberately not modeled GUI-side (the drift gate pins restore at 1 rule).
    FlagSchema {
        name: "--own-account-max",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Range fallback for the own seed's account(s) when the exact \
               accounts are unknown: derive the own seed at every account in \
               0..K and let the multisig-template own-account subset-search \
               select the subset actually used (own-only — cosigners must be \
               EXACT unless --search-cosigner-subset). Mutually exclusive with \
               --account. K ≤ 256.",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.70.0 (#28 phase 2 / P3): opt-in bounded cosigner-subset
    // search. Default OFF — cosigner cards are matched EXACTLY (own-only).
    FlagSchema {
        name: "--search-cosigner-subset",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Opt-in: allow over-supplying --cosigner cards (more than the \
               template's cosigner slots) and let the subset-search resolve \
               the correct cosigner subset too. Default OFF (cosigners matched \
               exactly). Enlarges the search space, so a longer \
               --expect-wallet-id prefix (or --search-address) may be needed; \
               bounded by the hard ceiling + adaptive time-cap.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--search-address",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "A known receive (or change) address of the wallet; triggers \
               address-search for a multisig template completion (full \
               scriptPubKey match — collision-free).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--search-addr-min",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Inclusive lower address index for --search-address (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--search-addr-max",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Exclusive upper address index for --search-address \
               (default 20). Deepen (0..20, then 20..40, …) if not found.",
        secret: false,
        default_value: Some("20"),
        global: false,
    },
    FlagSchema {
        name: "--search-chain",
        kind: FlagKind::Dropdown(SEARCH_CHAINS),
        required: false,
        repeating: false,
        help: "Which BIP-32 change-chain branch(es) --search-address scans: \
               receive (0, default), change (1), or both.",
        secret: false,
        default_value: Some("receive"),
        global: false,
    },
    FlagSchema {
        name: "--accept-search-time",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Override the 1-hour search-time ceiling for a multisig \
               template completion. Must be ≥ the printed estimate. Accepts \
               a humantime duration (e.g. 2h, 90min).",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── verify-bundle ───────────────────────────────────────────────────────

const VERIFY_BUNDLE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: true,
        repeating: false,
        help: "Bitcoin network.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Template. Mutually-required-one-of with --descriptor / \
               --descriptor-file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "User-supplied descriptor for the re-parse path.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor-file",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Path to a single-line UTF-8 descriptor file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic extension passphrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    // toolkit v0.70.0 (#28 phase 2 / P4): RANGE fallback for the own seed's
    // account, NEW on verify-bundle (mirrors `restore --own-account-max`).
    // Over-supplies the own accounts (0..K) and resolves the unique template
    // assignment via the own-anchored subset-search, threaded into the SAME
    // shared `complete_multisig_template` engine restore uses. Mutually
    // exclusive with `--account` (clap conflicts_with; mutex not projected
    // into conditional_rules → not modeled GUI-side, drift gate pins 10 rules).
    FlagSchema {
        name: "--own-account-max",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Range fallback for the own seed's account when the exact \
               account is unknown: derive the own seed at every account in \
               0..K and let the multisig-template own-account subset-search \
               select the account actually used (mirrors restore \
               --own-account-max; cosigners EXACT unless \
               --search-cosigner-subset). Mutually exclusive with --account. \
               K ≤ 256.",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.59.0 (#28): explicit origin override + wallet-id assertion
    // for verifying + recomposing a keyless single-sig template bundle.
    // Mirrors `restore --origin` / `--expect-wallet-id`; only meaningful for
    // a keyless template bundle, ignored otherwise. `--expect-wallet-id` is
    // NOT checked when `--origin` overrides the canonical account path.
    FlagSchema {
        name: "--origin",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Explicit origin derivation path for verifying + recomposing a \
               keyless single-sig template bundle (mirrors restore --origin).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--expect-wallet-id",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Assert the recomposed wallet's WalletPolicyId starts with this \
               prefix (refuse on mismatch; skipped when --origin overrides \
               the canonical account path).",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.60.0 (#28 phase 2): multisig / general-policy keyless
    // TEMPLATE completion for verify-bundle (`bundle --md1-form=template`,
    // n≥2), mirroring the same-named restore flags. The own seed (`--from`)
    // + cosigner keys (`--cosigner`) plus a disambiguation input
    // (`--search-address` / `--expect-wallet-id`) drive the key→slot search.
    // `--from` mirrors restore's `--from` classification: non-secret here
    // (the seed source supports `@env:VAR` / `-` stdin for secret values,
    // but the schema marks restore's `--from` secret:false, so we mirror it).
    FlagSchema {
        name: "--from",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "The operator's OWN seed for completing a keyless multisig (or \
               general-policy) template bundle (same grammar as restore \
               --from: ms1=/phrase=/entropy=/seedqr=; @env:VAR / - stdin). \
               Required to complete a multisig template; ignored otherwise.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--cosigner",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "An unassigned cosigner key (mk1/xpub) for completing a keyless \
               multisig / general template bundle; repeat per cosigner card. \
               Bare = search-placed; `@N=<mk1|xpub>` assigns explicitly. \
               Watch-only (non-secret).",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.70.0 (#28 phase 2 / P4): opt-in bounded cosigner-subset
    // search, NEW on verify-bundle (mirrors `restore --search-cosigner-subset`).
    FlagSchema {
        name: "--search-cosigner-subset",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Opt-in: allow over-supplying --cosigner cards and let the \
               subset-search resolve the correct cosigner subset (mirrors \
               restore --search-cosigner-subset). Default OFF (cosigners \
               matched exactly). Bounded by the hard ceiling + adaptive \
               time-cap.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--search-address",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "A known receive (or change) address of the wallet; triggers \
               address-search for a multisig template completion (mirrors \
               restore --search-address; full scriptPubKey match).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--search-addr-min",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Inclusive lower address index for --search-address (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--search-addr-max",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Exclusive upper address index for --search-address \
               (default 20; mirrors restore).",
        secret: false,
        default_value: Some("20"),
        global: false,
    },
    FlagSchema {
        name: "--search-chain",
        kind: FlagKind::Dropdown(SEARCH_CHAINS),
        required: false,
        repeating: false,
        help: "Which BIP-32 change-chain branch(es) --search-address scans: \
               receive (0, default), change (1), or both (mirrors restore).",
        secret: false,
        default_value: Some("receive"),
        global: false,
    },
    FlagSchema {
        name: "--accept-search-time",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Override the 1-hour search-time ceiling for a multisig \
               template completion (mirrors restore --accept-search-time). \
               Must be ≥ the printed estimate. Humantime duration (2h, 90min).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot ms1 card (schema-2/3 single use; schema-4 repeating). \
               Empty string is watch-only sentinel.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--mk1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot mk1 card (repeating).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot md1 card (repeating).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bundle-json",
        // R1 C-3 fold: upstream `VerifyBundleArgs::bundle_json` is
        // `Option<PathBuf>` and `load_bundle_json_into_args`
        // (verify_bundle.rs:526) calls `std::fs::read_to_string(path)`
        // unconditionally — there is no `-` → stdin code path. Setting
        // `stdio_sentinel: false` so the emitter cannot generate the
        // upstream-rejected `--bundle-json -` argv. Future stdin support
        // is an upstream feature first; FOLLOWUPS cross-cite at that time.
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Path to the JSON envelope from `bundle --json`. Mutually \
               exclusive with --ms1/--mk1/--md1.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON-shaped output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--multisig-path-family",
        kind: FlagKind::Dropdown(MULTISIG_PATH_FAMILIES),
        required: false,
        repeating: false,
        help: "Multisig derivation path family.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--privacy-preserving",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Expect mk1 omits master fingerprint.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 1, max: NumberMax::FromSlotCount },
        required: false,
        repeating: false,
        help: "Multisig threshold K.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Slot input @N.<subkey>=<value>. Handled by SlotEditor.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── convert ─────────────────────────────────────────────────────────────

const CONVERT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Display grouping: break the emitted card into groups of N \
               characters (default 5; 0 = unbroken single line). Cosmetic — \
               intake strips separators, so any grouping re-ingests.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Display-grouping separator keyword (space|hyphen|comma; \
               default space). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(CONVERT_FROM_NODES),
        required: true,
        repeating: false,
        help: "Source node: <node>=<value>. `=-` reads value from stdin.",
        secret: false, // secrecy is node-dependent; composite paste-warn + argv-mask + run-confirm + persist-redact key on node_type_is_argv_secret (cycle-3)
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(CONVERT_TO_NODES),
        required: true,
        repeating: true,
        help: "Destination node (repeating: clap Append).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Bitcoin network.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Template (when --to involves derivation).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Explicit BIP-32 derivation path.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 PBKDF2 passphrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bip38-passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-38 Scrypt passphrase (distinct from --passphrase).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bip38-passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --bip38-passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master fingerprint (8 hex chars).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--xpub-prefix",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "SLIP-0132 prefix override for --to xpub.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--electrum-version",
        kind: FlagKind::Dropdown(ELECTRUM_VERSIONS),
        required: false,
        repeating: false,
        help: "Electrum seed-version selector for (Entropy, ElectrumPhrase).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--electrum-language",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Electrum wordlist (distinct from --language).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--script-type",
        kind: FlagKind::Dropdown(SCRIPT_TYPES),
        required: false,
        repeating: false,
        help: "Script-type selector for (Xpub, Address) derivation.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON-shaped output.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── export-wallet ───────────────────────────────────────────────────────

const EXPORT_WALLET_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Pre-built template. Mutually-required-one-of with --descriptor.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "User-supplied BIP-388 descriptor.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 1, max: NumberMax::FromSlotCount },
        required: false,
        repeating: false,
        help: "Multisig threshold K (1 <= K <= N).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--multisig-path-family",
        kind: FlagKind::Dropdown(MULTISIG_PATH_FAMILIES),
        required: false,
        repeating: false,
        help: "Multisig path family (default bip87).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network (default mainnet).",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "Ignored (watch-only); kept for slot-parser symmetry.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Slot input @N.<subkey>=<value>. Handled by SlotEditor.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(EXPORT_FORMATS),
        required: false,
        repeating: false,
        help: "Output format (default bitcoin-core).",
        secret: false,
        default_value: Some("bitcoin-core"),
        global: false,
    },
    FlagSchema {
        name: "--output",
        kind: FlagKind::Path {
            stdio_sentinel: true,
        },
        required: false,
        repeating: false,
        help: "Output path. `-` (default) -> stdout.",
        secret: false,
        default_value: Some("-"),
        global: false,
    },
    FlagSchema {
        name: "--range",
        kind: FlagKind::Range,
        required: false,
        repeating: false,
        help: "Bitcoin Core `range` field, comma-separated. Default 0,999.",
        secret: false,
        default_value: Some("0,999"),
        global: false,
    },
    FlagSchema {
        name: "--timestamp",
        kind: FlagKind::Timestamp,
        required: false,
        repeating: false,
        help: "Bitcoin Core `timestamp` field. `0` (default; rescan from genesis), `now`, or unix seconds.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--bitcoin-core-version",
        kind: FlagKind::Number { min: 24, max: NumberMax::Static(25) },
        required: false,
        repeating: false,
        help: "Bitcoin Core target version (24 or 25, default 25).",
        secret: false,
        default_value: Some("25"),
        global: false,
    },
    FlagSchema {
        name: "--taproot-internal-key",
        kind: FlagKind::TaggedOrIndexed(&["nums"]),
        required: false,
        repeating: false,
        help: "Taproot internal-key designation for tr-multi-a / \
               tr-sortedmulti-a. `nums` or `@N` (cosigner N's xpub).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--wallet-name",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Wallet label. Required for Sparrow / Specter / Electrum / \
               Green formats.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bsms-form",
        kind: FlagKind::Dropdown(BSMS_FORMS),
        required: false,
        repeating: false,
        help: "v0.27.0 — BSMS Round-2 emit shape. `4-line` (BIP-129-canonical) \
               is the default; `2-line` is the lenient excerpt. Ignored by \
               every other format.",
        secret: false,
        default_value: Some("4-line"),
        global: false,
    },
    FlagSchema {
        name: "--from-import-json",
        kind: FlagKind::Path {
            stdio_sentinel: true,
        },
        required: false,
        repeating: false,
        help: "v0.27.0 — emit wallet config from an import-wallet --json \
               envelope. Mutually exclusive with --template / --descriptor.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-import-json-index",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "v0.27.0 — pick a specific entry from a multi-entry envelope \
               array. Required when the envelope has > 1 entry.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── derive-child ────────────────────────────────────────────────────────

const DERIVE_CHILD_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(&["xprv", "phrase"]),
        required: true,
        repeating: false,
        help: "Master source. v0.7: xprv only; v0.8 also accepts \
               phrase=<bip39-mnemonic>. `=-` reads from stdin.",
        secret: false, // value-dependent
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--application",
        kind: FlagKind::Dropdown(BIP85_APPLICATIONS),
        required: true,
        repeating: false,
        help: "BIP-85 application token.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--length",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(8192) },
        required: true,
        repeating: false,
        help: "Per-app length validator. Pass 0 for xprv / hd-seed.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--index",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: true,
        repeating: false,
        help: "Hardened child index (0..2^31).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for emitted hd-seed / xprv (default mainnet).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for --application bip39 (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase (used only with --from phrase=).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--dice-sides",
        kind: FlagKind::Number {
            min: 2,
            max: NumberMax::Static(4_294_967_295),
        },
        required: false,
        repeating: false,
        help: "Number of sides for --application dice.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── slip39-split ────────────────────────────────────────────────────────

const SLIP39_FROM_NODES: &[&str] = &["phrase", "entropy"];

const SLIP39_SPLIT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(SLIP39_FROM_NODES),
        required: true,
        repeating: false,
        help: "Master secret. `phrase=<value-or->` (BIP-39) OR `entropy=<hex-or->`. `=-` reads from stdin.",
        secret: false, // value-dependent
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "SLIP-39 passphrase (NOT BIP-39 passphrase).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--group-threshold",
        kind: FlagKind::Number { min: 1, max: NumberMax::Static(16) },
        required: true,
        repeating: false,
        help: "K of the group layer (1..=group_count).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--group",
        kind: FlagKind::Text,
        required: true,
        repeating: true,
        help: "Per-group `<member_count>,<member_threshold>` (e.g., `2,2`). Repeating.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--iteration-exponent",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(15) },
        required: false,
        repeating: false,
        help: "Iteration exponent E (library-enforced 0..=15; default 0). G9 advisory at E >= 5.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for parsing --from phrase=...; ignored for entropy= input.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH. World-readable-path advisory on stderr.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── slip39-combine ──────────────────────────────────────────────────────

const SLIP39_TO_SHAPES: &[&str] = &["entropy", "phrase"];

const SLIP39_COMBINE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--share",
        kind: FlagKind::Text,
        required: true,
        repeating: true,
        help: "SLIP-39 share mnemonic. Repeating; at most ONE may be `-` (stdin).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "SLIP-39 passphrase used at split time.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(SLIP39_TO_SHAPES),
        required: false,
        repeating: false,
        help: "Output shape (default entropy: hex on stdout; phrase: BIP-39 mnemonic).",
        secret: false,
        default_value: Some("entropy"),
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for --to phrase; ignored for --to entropy.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── ms-shares-split ─────────────────────────────────────────────────────
//
// toolkit v0.40.0: `mnemonic ms-shares split` — BIP-93 codex32 K-of-N
// share splitter of an ms1 secret. Mirrors `mnemonic ms-shares split
// --help` (flattened reflective name `ms-shares-split` in gui-schema v5).
// Flag-name + dropdown-value parity per CLAUDE.md (the `--json` wire-shape
// is NOT gated — FOLLOWUP `ms-kofn-json-wire-shape-ungated`).

const MS_SHARES_FROM_NODES: &[&str] = &["phrase", "entropy"];

const MS_SHARES_SPLIT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Display grouping: break the emitted card into groups of N \
               characters (default 5; 0 = unbroken single line). Cosmetic — \
               intake strips separators, so any grouping re-ingests.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Display-grouping separator keyword (space|hyphen|comma; \
               default space). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(MS_SHARES_FROM_NODES),
        required: true,
        repeating: false,
        help: "Secret. `phrase=<value-or->` (BIP-39) OR `entropy=<hex-or->`. `=-` reads from stdin.",
        secret: false, // value-dependent
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 2, max: NumberMax::Static(9) },
        required: true,
        repeating: false,
        help: "Threshold K — minimum shares needed to recombine (2..=9).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--shares",
        kind: FlagKind::Number { min: 2, max: NumberMax::Static(31) },
        required: true,
        repeating: false,
        help: "Total shares N to emit (K <= N <= 31).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for the input phrase; ignored for entropy= input. \
               A non-English language produces a `mnem` share-set so the wordlist survives.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON object on stdout (`{\"shares\": [...]}`) instead of one-share-per-line text.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── ms-shares-combine ───────────────────────────────────────────────────
//
// toolkit v0.40.0: `mnemonic ms-shares combine` — recombine >=K codex32
// shares into the recovered secret. `--share` is secret (the share SET is
// secret-equivalent — mirrored in the v5 `secret` projection / secret-drift
// gate). The `--to` dropdown is `phrase|entropy|ms1`.

const MS_SHARES_TO_SHAPES: &[&str] = &["phrase", "entropy", "ms1"];

const MS_SHARES_COMBINE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Display grouping: break the emitted card into groups of N \
               characters (default 5; 0 = unbroken single line). Cosmetic — \
               intake strips separators, so any grouping re-ingests.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Display-grouping separator keyword (space|hyphen|comma; \
               default space). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--share",
        kind: FlagKind::Text,
        required: true,
        repeating: true,
        help: "A codex32 K-of-N share string. Repeating; supply at least K. At most ONE may be `-` (stdin).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(MS_SHARES_TO_SHAPES),
        required: false,
        repeating: false,
        help: "Output shape (default phrase): phrase (BIP-39 mnemonic), entropy (hex), or ms1 (single string).",
        secret: false,
        default_value: Some("phrase"),
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for --to phrase when the recovered secret is a plain entr payload; \
               ignored for mnem payloads and for --to entropy/ms1.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON object on stdout instead of the plain secret line.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── seed-xor-split ──────────────────────────────────────────────────────

const PHRASE_ONLY: &[&str] = &["phrase"];

const SEED_XOR_SPLIT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(PHRASE_ONLY),
        required: true,
        repeating: false,
        help: "Master BIP-39 phrase. `phrase=<value>` (inline) OR `phrase=-` (stdin).",
        secret: false, // value-dependent
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--shares",
        kind: FlagKind::Number { min: 2, max: NumberMax::Static(255) },
        required: true,
        repeating: false,
        help: "Number of XOR shares to emit. Must be >= 2.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for input + output (default english).",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--deterministic-from-master",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Coldcard SHA256d-deterministic share generation (interop with Coldcard's xor_seed.py).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── seed-xor-combine ────────────────────────────────────────────────────

const SEED_XOR_COMBINE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--share",
        kind: FlagKind::NodeValueComposite(PHRASE_ONLY),
        required: true,
        repeating: true,
        help: "Share phrase. `phrase=<value>` or `phrase=-`. At most ONE may be stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--shares",
        kind: FlagKind::Number { min: 2, max: NumberMax::Static(255) },
        required: true,
        repeating: false,
        help: "Asserted share count. Handler-side runtime check equals actual --share count.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for inputs + output (default english).",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── seedqr-encode ───────────────────────────────────────────────────────
//
// v0.15.0 (toolkit v0.30.0): SeedQR encode subsubcommand. Read a 12/24-word
// BIP-39 English phrase via `--from phrase=<VALUE|->`, emit the SeedQR
// numeric digit string on stdout (or `--json-out` envelope).

// v0.17.0 (toolkit v0.32.0): SeedQR variant selector. `standard` = decimal
// digit string; `compact` = CompactSeedQR entropy bytes as hex (12/24 only).
const SEEDQR_VARIANTS: &[&str] = &["standard", "compact"];

const SEEDQR_ENCODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(PHRASE_ONLY),
        required: true,
        repeating: false,
        help: "BIP-39 phrase. `phrase=<value>` (inline) OR `phrase=-` (stdin).",
        secret: false, // value-dependent
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--variant",
        kind: FlagKind::Dropdown(SEEDQR_VARIANTS),
        required: false,
        repeating: false,
        help: "SeedQR variant (v0.32.0+): standard (decimal digits, default) \
               or compact (CompactSeedQR entropy bytes as hex; 12/24 only).",
        secret: false,
        default_value: Some("standard"),
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── seedqr-decode ───────────────────────────────────────────────────────
//
// v0.15.0 (toolkit v0.30.0): SeedQR decode subsubcommand. Read an
// ASCII-digit SeedQR string, emit the BIP-39 English phrase on stdout
// (or `--json-out` envelope).
//
// v0.16.2 (toolkit v0.31.6): the input was unified into the shared
// `--from <node>=<value>` grammar. The canonical form is now
// `--from seedqr=<VALUE|->` (only the `seedqr` node type is accepted).
// The original `--digits` flag is preserved as a DEPRECATED alias
// (toolkit emits a stderr notice; mutually exclusive with `--from` via
// clap `conflicts_with`, exit 64). Both carry secret material (BIP-39
// entropy). See toolkit FOLLOWUP `seedqr-digits-from-input-unification`.

const SEEDQR_DECODE_FROM_NODES: &[&str] = &["seedqr"];

const SEEDQR_DECODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(SEEDQR_DECODE_FROM_NODES),
        required: false,
        repeating: false,
        help: "Canonical input (v0.31.6+): seedqr=<VALUE|->. SeedQR digit \
               string (48/60/72/84/96 ASCII digits). Only the `seedqr` node \
               type is accepted.",
        secret: false, // secrecy is node-dependent; composite paste-warn + argv-mask + run-confirm + persist-redact key on node_type_is_argv_secret (cycle-3)
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--variant",
        kind: FlagKind::Dropdown(SEEDQR_VARIANTS),
        required: false,
        repeating: false,
        help: "SeedQR variant (v0.32.0+): standard (decimal digits, default) \
               or compact (CompactSeedQR entropy bytes as hex; 12/24 only). \
               Determines how the --from seedqr= value is interpreted.",
        secret: false,
        default_value: Some("standard"),
        global: false,
    },
    FlagSchema {
        name: "--digits",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "DEPRECATED (v0.31.6): use --from seedqr=<VALUE|-> instead. \
               SeedQR numeric digit string (48/60/72/84/96 ASCII digits). \
               `-` reads from stdin. Mutually exclusive with --from.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── compare-cost ────────────────────────────────────────────────────────
//
// v0.11.0 (toolkit v0.26.0): wsh-vs-tr per-spending-condition cost
// comparison subcommand. Mutex `--miniscript` vs `--descriptor` is encoded
// via `crate::form::conditional::compare_cost`. `--feerate` is a Text
// widget rather than `FlagKind::Number` because the toolkit accepts decimal
// values (`f64` clap parser with bounds [0.0, 10000.0]); the GUI's Number
// kind is i64-only today and surfacing a Text widget with permissive parse
// avoids a v0.11.0-only schema-grammar extension. Validation is toolkit-
// side: bad feerate → exit 64.

const COMPARE_COST_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--miniscript",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Bare miniscript with abstract labels (`pk(A), or_b(...)`) or concrete hex keys. \
               Mutually exclusive with `--descriptor`.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Full descriptor — `wsh(M)` or `sh(wsh(M))`. The wrapper is stripped to recover M, \
               then wsh(M) is compared against tr(NUMS, {M}). tr() input deferred to v0.27 FOLLOWUP.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--feerate",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Sats per virtual byte (decimal, default 1.0; max 10000.0).",
        secret: false,
        default_value: Some("1.0"),
        global: false,
    },
    FlagSchema {
        name: "--max-conditions",
        kind: FlagKind::Number {
            min: 1,
            max: crate::schema::NumberMax::Static(1_000_000),
        },
        required: false,
        repeating: false,
        help: "Hard cap on raw enumeration size n_abs × n_rel × 2^(|signers|+|preimages|). \
               Default 4096.",
        secret: false,
        default_value: Some("4096"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON envelope on stdout instead of the plaintext table.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── final-word ──────────────────────────────────────────────────────────

const FINAL_WORD_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::NodeValueComposite(PHRASE_ONLY),
        required: true,
        repeating: false,
        help: "N-1 word partial phrase. `phrase=<words>` or `phrase=-`. Must be 11/14/17/20/23 words.",
        secret: false, // value-dependent
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Side-effect: write versioned JSON envelope to PATH. Candidate list still emitted to stdout.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── repair ──────────────────────────────────────────────────────────────
//
// v0.22.0 standalone BCH error-correction subcommand. Three card-class
// inputs (`--ms1` / `--mk1` / `--md1`) form an at-least-one required
// group: pre-v0.10.0 (toolkit pre-D35) the group was clap-mutex'd; the
// v0.10.0-lockstep toolkit (D35) now accepts any non-empty subset of
// the three in a single invocation. GUI C.2 wires the 3-way
// at-least-one rule via `crate::form::conditional::repair`. `--ms1` is
// single-occurrence per toolkit; `--mk1` / `--md1` are repeating per
// toolkit (multiple chunks in a single invocation).
//
// v0.10.0 B.3 (D33): the global `--no-auto-repair` flag is now mirrored
// per-subcommand from the toolkit v5 schema's `global: true` emission.
// Replaces the v0.9.0 R7 action-bar fallback (DELETED in this cycle).

const REPAIR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Single ms1 chunk to repair. `-` reads one chunk from stdin. \
               Combinable with --mk1 / --md1 (at least one card required).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--mk1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "One or more mk1 chunks to repair (repeating). `-` on a \
               single occurrence reads chunks from stdin (one per line). \
               Combinable with --ms1 / --md1 (at least one card required).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "One or more md1 chunks to repair (repeating). `-` on a \
               single occurrence reads chunks from stdin (one per line). \
               Combinable with --ms1 / --mk1 (at least one card required).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON envelope on stdout instead of the \
               text-form repair report.",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.37.1: indel (insert/delete) recovery budget for transcription
    // errors that changed the length of a chunk. Integer 0..=4; 0 = disabled
    // (default). Applies to ms1/mk1/md1 (md1 indel landed in toolkit v0.37.2).
    FlagSchema {
        name: "--max-indel",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(4),
        },
        required: false,
        repeating: false,
        help: "Maximum insert/delete (indel) distance for length-mismatch \
               recovery. 0 disables (default). ms1/mk1/md1.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    // toolkit v0.37.3: tolerate up to E substitution (wrong-but-in-place)
    // errors alongside the indels. Integer 0..=4; 0 = pure indel (default).
    // A recovery that consumed a substitution is a VERIFY-ME candidate
    // (exit 4), not a confident correction. Applies to ms1/mk1/md1.
    FlagSchema {
        name: "--max-subst",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(4),
        },
        required: false,
        repeating: false,
        help: "Also tolerate up to E substitution (wrong-but-in-place) errors \
               alongside the indels. 0 = pure indel (default). A substitution-\
               bearing recovery is printed as a VERIFY-ME candidate. ms1/mk1/md1.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── inspect ─────────────────────────────────────────────────────────────
//
// v0.22.0 inspection subcommand. Same 3-way at-least-one rule as
// `repair` (toolkit D35 dropped the prior clap mutex); adds
// `--reveal-secret` (ms1-only effect — opt-in to print the BIP-39
// entropy hex; no effect on mk1 / md1). GUI C.2 wires the same 3-way
// at-least-one rule via `crate::form::conditional::inspect`.
//
// v0.10.0 B.3 (D31): `--reveal-secret` is `secret: false` per toolkit v5
// ground truth — the flag itself is a presence-only Boolean (no
// secret-bearing value); the SECRET semantic applies to the *output* it
// unlocks (ms1 entropy hex), not the flag-input. The output-side
// protection lives elsewhere (paste-warn modal fires on the underlying
// `--ms1` input; the GUI's run-confirm modal already gates secret-bearing
// argv emissions). Reconciled with toolkit's `flag_is_secret` (which
// returns false for this flag).

const INSPECT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Single ms1 chunk to inspect. `-` reads one chunk from stdin. \
               Combinable with --mk1 / --md1 (at least one card required).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--mk1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "One or more mk1 chunks to inspect (repeating). `-` reads \
               chunks from stdin (one per line). Combinable with --ms1 / \
               --md1 (at least one card required).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "One or more md1 chunks to inspect (repeating). `-` reads \
               chunks from stdin (one per line). Combinable with --ms1 / \
               --mk1 (at least one card required).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON envelope on stdout instead of the \
               text-form inspect report.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--reveal-secret",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Reveal the ms1 entropy hex on stdout. Default suppresses it \
               (summary stays at length / bit-strength). No effect for mk1 \
               / md1 (those payloads carry no secret material).",
        // v0.10.0 B.3 (D31): reconciled with toolkit v5 ground truth — the
        // flag is `secret: false` (presence-only Boolean; the gated *output*
        // is secret, not the input flag itself).
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── import-wallet (v0.11.0 / toolkit v0.26.0) ───────────────────────────
//
// Lockstep with mnemonic-toolkit v0.26.0 `mnemonic import-wallet`. Source
// of truth: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:43-108` at
// toolkit HEAD `72575e2`.
//
// Subcommand intake: a third-party wallet blob (BSMS or Bitcoin Core
// listdescriptors JSON), with optional seed-overlay via repeating `--ms1`
// and/or `--slot @<N>.phrase=<phrase>`. The toolkit resolves `@env:<VAR>`
// sentinels on `--ms1` + secret-bearing slot subkeys per SPEC §3 (the GUI
// emits literal values today; env-var-channel injection is a FOLLOWUP —
// see `gui-import-wallet-env-var-secret-channel`).
const IMPORT_WALLET_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--blob",
        // Toolkit accepts `-` as a stdin sentinel. The GUI emits the path
        // (or `-`) verbatim; the kittest cells exercise both forms.
        kind: FlagKind::Path { stdio_sentinel: true },
        required: true,
        repeating: false,
        help: "Path to the third-party wallet blob (BSMS or Bitcoin Core \
               listdescriptors JSON). `-` reads from stdin.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(IMPORT_WALLET_FORMATS),
        required: false,
        repeating: false,
        help: "Format override. If absent, the blob is auto-detected via \
               sniff (SPEC §6).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        // Toolkit accepts an integer `N`, `active-receive`, `active-change`,
        // or `all`. Schema-side: free-form Text (the union of a discrete
        // dropdown and an integer escape doesn't fit FlagKind::Dropdown).
        // CLI rejection catches values outside the union; GUI surfaces the
        // raw error in the output pane.
        name: "--select-descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Multi-descriptor selector for Bitcoin Core blobs. Accepts \
               an integer (`0`, `1`, ...), `active-receive`, \
               `active-change`, or `all` (default).",
        secret: false,
        default_value: Some("all"),
        global: false,
    },
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Seed overlay (SPEC §8.3). Repeatable; positional cosigner- \
               index (i-th `--ms1` applies to cosigner i). Empty-string \
               preserves the v0.25.1 watch-only sentinel semantics.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot seed overlay via `--slot @<N>.phrase=<phrase>`. \
               v0.26.0 only accepts the `phrase` subkey on import-wallet. \
               Handled by SlotEditor.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope array on stdout (SPEC §7.4) instead of \
               the human-readable summary.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bsms-encryption-token",
        kind: FlagKind::Path { stdio_sentinel: true },
        required: false,
        repeating: false,
        help: "v0.31.0 — BIP-129 encryption-envelope Round-2 decrypt; \
               reads session TOKEN from PATH (or `-` for stdin). \
               16-hex STANDARD or 32-hex EXTENDED width. Combine with \
               --format bsms. MAC verify failure → exit 2 (typed BsmsMacMismatch).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bsms-round1",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: true,
        help: "v0.27.0 — BIP-129 Round-1 key record file for BIP-322 ECDSA \
               signature verification. Repeating — one per record. Verify \
               state propagates to --json envelope.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--bsms-verify-strict",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "v0.27.0 — make BIP-129 Round-1 SIG verification failures fatal \
               (exit 2). Without this flag, mismatches emit a stderr NOTICE.",
        secret: false,
        default_value: None,
        global: false,
    },
    // v0.33.2 (toolkit Cycle 19 Phase B) — Electrum BIE1 (user-password)
    // storage-encrypted wallet decryption. Optional, mutually-exclusive group;
    // only consumed when `--blob` is a BIE1 storage blob. `--decrypt-password`
    // + `--decrypt-password-stdin` are secret (mirrors toolkit
    // `flag_is_secret`, classified since v0.33.1); `--decrypt-password-file`
    // holds a PATH (non-secret).
    FlagSchema {
        name: "--decrypt-password",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Password for an Electrum BIE1 storage-encrypted wallet (inline). \
               Emits an argv-leakage advisory; prefer --decrypt-password-file \
               or --decrypt-password-stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--decrypt-password-file",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Read the BIE1 decryption password from a file (trailing newline \
               stripped).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--decrypt-password-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the BIE1 decryption password from stdin. Cannot co-exist \
               with --blob=- or --bsms-encryption-token=-.",
        secret: true,
        default_value: None,
        global: false,
    },
    // toolkit v0.34.6: signet/regtest disambiguation override. Reuses the
    // NETWORKS dropdown; honored only within the parsed coin-type class.
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Re-bind the imported network to disambiguate signet/regtest from \
               the coin-type-1→testnet collapse; honored only within the parsed \
               coin-type class (testnet ↔ testnet/signet/regtest; mainnet ↔ mainnet).",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// Eight-element option list for `--format` on import-wallet. Aligned with
// `EXPORT_FORMATS` after the toolkit v0.28.0 cycle wired 6 new inbound
// parsers (sparrow, specter, coldcard, coldcard-multisig, electrum, jade)
// in lockstep with the existing emitters. Alphabetical order matches
// the toolkit's `cmd/import_wallet.rs::PossibleValuesParser` literal at
// line ~107-121 (re-verify against pinned toolkit tag at schema_mirror
// gate fire-time).
const IMPORT_WALLET_FORMATS: &[&str] = &[
    "bitcoin-core",
    "bsms",
    "coldcard",
    "coldcard-multisig",
    "descriptor",
    "electrum",
    "jade",
    "sparrow",
    "specter",
];


// ─── xpub-search umbrella (v0.11.0; toolkit v0.26.0) ─────────────────────
//
// The toolkit v0.26.0 cycle introduces four `xpub-search` umbrella
// subcommands surfaced through clap-flatten so they appear at the top
// level as `xpub-search-<mode>`:
//
//   - xpub-search-path-of-xpub          (C1; toolkit `d28b170`)
//   - xpub-search-account-of-descriptor (C2; toolkit `196cc8a`)
//   - xpub-search-address-of-xpub       (C3; toolkit `a5bfbaf` + `365c0d1`)
//   - xpub-search-passphrase-of-xpub    (C4; toolkit `bc2a76a`)
//
// **v0.11.0 scope discipline (MVP, no bespoke widgets):** the toolkit
// gui-schema JSON is mirrored declaratively; the existing generic form
// renderer (`main.rs:346-602`) handles every flag via the FlagKind
// dispatcher. No pane abstraction / hub UI / bespoke composite widgets —
// the user reaches each mode through the standard subcommand-name
// ComboBox at `main.rs:359-371`. Polish items are queued as v0.12.0
// FOLLOWUPs (`xpub-search-gui-bespoke-hub-pane`,
// `xpub-search-gui-bespoke-widgets`, `xpub-search-gui-positional-intake`,
// `xpub-search-gui-flag-mutex-visibility`).
//
// **Positional-arg policy:** toolkit gui-schema emits a `positional`
// entry on P1/P2/P4 (HRP-autodetect ms1 intake). The GUI's argv
// assembler routes seed input via `--ms1` after autodetect, so all four
// SubcommandSchema entries declare `positional_args: NO_POSITIONALS`.
// CLI-direct positional usage still works for power-users; surfacing it
// in the GUI's form is a v0.12.0 polish item.
//
// **secret bool reconciliation:** toolkit v5 JSON marks `--ms1`,
// `--passphrase`, and `--passphrase-stdin` as `secret: true`. The GUI
// mirrors those bytes. `--phrase` and `--phrase-stdin` are NOT toolkit-
// declared as secret (consistent with other mnemonic subcommands), but
// they ARE phrase inputs — the GUI's hand-maintained `SECRET_FLAG_NAMES`
// list in `src/secrets.rs` is the GUI-side override. We do NOT mark
// these `secret: true` here because the schema_mirror_secret_drift gate
// would then flag a divergence. `--phrase` paste-warning routes through
// the GUI's secrets layer regardless.

/// SLIP-0132 + mk1 acceptable address-type tokens for
/// `xpub-search-address-of-xpub --address-type`. Mirrors the toolkit's
/// `address_of_xpub.rs:55-58` clap-derive doc-comment + the kebab-case
/// `ScriptType` JSON tag enumeration at `address_of_xpub.rs:104-114`.
const XPUB_SEARCH_ADDRESS_TYPES: &[&str] = &["p2pkh", "p2sh-p2wpkh", "p2wpkh", "p2tr"];

const XPUB_SEARCH_PATH_OF_XPUB_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master BIP-39 phrase (inline). Emits an argv-leakage advisory; \
               prefer --phrase-stdin for sensitive input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read master BIP-39 phrase from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "ms1 card carrying BIP-39 entropy (inline). Emits an \
               argv-leakage advisory; prefer --ms1-stdin for sensitive input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read ms1 card from stdin (single chunk).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase (inline). Emits an argv-leakage advisory.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read BIP-39 passphrase from stdin (NULL-byte-preserving; \
               single trailing newline stripped).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--target-xpub",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "Target xpub. Accepts any SLIP-0132 prefix (xpub/tpub/ypub/Ypub/\
               zpub/Zpub/upub/Upub/vpub/Vpub) or an mk1 bech32 card carrying \
               an xpub.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist language. Defaults to english.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network selector. Defaults to mainnet.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--min-account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Lower bound of account-index iteration (inclusive). Default 0.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--number-of-accounts",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Window size starting at --min-account (default 20).",
        secret: false,
        default_value: Some("20"),
        global: false,
    },
    FlagSchema {
        name: "--max-account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Optional upper bound. Effective end is \
               max(min_account + number_of_accounts, max_account + 1).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--add-path",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Additional derivation-path template. Repeatable. The literal \
               token `account'` (or `account`) is substituted with each \
               iterated account index. Templates without an `account` token \
               are searched once at the literal path.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope on stdout instead of the text-form report.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

const XPUB_SEARCH_ACCOUNT_OF_DESCRIPTOR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master BIP-39 phrase (inline). Emits an argv-leakage advisory; \
               prefer --phrase-stdin for sensitive input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read master BIP-39 phrase from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "ms1 card carrying BIP-39 entropy (inline). Emits an \
               argv-leakage advisory; prefer --ms1-stdin for sensitive input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read ms1 card from stdin (single chunk).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase (inline). Emits an argv-leakage advisory.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read BIP-39 passphrase from stdin (NULL-byte-preserving; \
               single trailing newline stripped).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-388 descriptor (inline). Must contain at least one xpub-\
               class key origin with parent fingerprint + derivation path; \
               XOR with --descriptor-from.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--descriptor-from",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Assemble descriptor from <node>=<value> input (e.g. \
               md1=<chunks>). XOR with --descriptor.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist language. Defaults to english.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network selector. Defaults to mainnet.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--min-account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Lower bound of account-index iteration (inclusive). Default 0.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--number-of-accounts",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Window size starting at --min-account (default 20).",
        secret: false,
        default_value: Some("20"),
        global: false,
    },
    FlagSchema {
        name: "--max-account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Optional upper bound. Effective end is \
               max(min_account + number_of_accounts, max_account + 1).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--add-path",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Additional derivation-path template. Repeatable. The literal \
               token `account'` (or `account`) is substituted with each \
               iterated account index.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope on stdout instead of the text-form report.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

const XPUB_SEARCH_ADDRESS_OF_XPUB_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--xpub",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Parent xpub. Accepts any SLIP-0132 single-sig prefix \
               (xpub/tpub/ypub/upub/zpub/vpub) or an mk1 bech32 card \
               carrying an xpub. Multisig SLIP-0132 prefixes (Ypub/Zpub/\
               Upub/Vpub) are refused — use account-of-descriptor instead.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--xpub-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read parent xpub from stdin (single line, trailing newline \
               stripped).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--target-address",
        kind: FlagKind::Text,
        required: true,
        repeating: true,
        help: "Target address. Repeatable; at least one required.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--gap-limit",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(1_000),
        },
        required: false,
        repeating: false,
        help: "Per-chain BIP-44 gap-limit window. Scan covers 0..gap_limit \
               indices on each chain. Default 20.",
        secret: false,
        default_value: Some("20"),
        global: false,
    },
    FlagSchema {
        name: "--external-only",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Restrict the scan to the external (receive) chain only; skip \
               internal (change) chain. Default: scan both chains.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--address-type",
        kind: FlagKind::Dropdown(XPUB_SEARCH_ADDRESS_TYPES),
        required: false,
        repeating: false,
        help: "Explicit script-type for address rendering. Required when the \
               parent xpub uses a neutral SLIP-0132 prefix (xpub/tpub) or \
               when overriding the prefix-inferred type. Accepts p2pkh / \
               p2sh-p2wpkh / p2wpkh / p2tr.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network selector. Defaults to network inferred from the xpub \
               version byte; --network signet / regtest overrides for \
               disambiguation.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope on stdout instead of text-form report.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

const XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master BIP-39 phrase (inline). Emits an argv-leakage advisory; \
               prefer --phrase-stdin for sensitive input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read master BIP-39 phrase from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "ms1 card carrying BIP-39 entropy (inline). Emits an \
               argv-leakage advisory; prefer --ms1-stdin for sensitive input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--ms1-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read ms1 card from stdin (single chunk).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase candidate (inline). Repeatable via wordlist; \
               this flag carries a single candidate. Emits argv-leakage \
               advisory.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read BIP-39 passphrase candidate from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    // toolkit v0.46.0: scan a file of candidate passphrases (one per line);
    // first match against --target-xpub wins. A PATH (non-secret) — the file
    // itself holds secret candidates. Mirrors the --decrypt-password-file shape.
    FlagSchema {
        name: "--passphrase-candidates-file",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Scan a file of candidate BIP-39 passphrases (one per line); \
               first match against --target-xpub wins. A PATH (non-secret); \
               the file itself holds secret candidates.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--target-xpub",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "Target xpub. Accepts any SLIP-0132 prefix or an mk1 bech32 \
               card carrying an xpub.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist language. Defaults to english.",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network selector. Defaults to mainnet.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--min-account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Lower bound of account-index iteration (inclusive). Default 0.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--number-of-accounts",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Window size starting at --min-account (default 20).",
        secret: false,
        default_value: Some("20"),
        global: false,
    },
    FlagSchema {
        name: "--max-account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Optional upper bound. Effective end is \
               max(min_account + number_of_accounts, max_account + 1).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--add-path",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Additional derivation-path template. Repeatable. The literal \
               token `account'` (or `account`) is substituted with each \
               iterated account index.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope on stdout instead of the text-form report.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── electrum-decrypt ──────────────────────────────────────────────────────
// toolkit v0.33.0: decrypt an Electrum field-encrypted secret
// (base64(iv ‖ aes-256-cbc(plaintext + PKCS7)), key = sha256d(password)) →
// recovered plaintext (Electrum-native seed phrase or BIP-32 xprv). The
// password is supplied via exactly one of an exclusive struct-level clap
// arg-group (--decrypt-password / -file / -stdin); the GUI presents all
// three but the CLI rejects 0 or >1. secret:true on the two inline-value
// password forms mirrors toolkit v0.33.1 `flag_is_secret`
// (--decrypt-password-file holds a PATH, not the secret; --ciphertext is
// encrypted material, not plaintext) — keeps `schema_mirror_secret_drift`
// green.
const ELECTRUM_DECRYPT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--ciphertext",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "The base64 Electrum field-encrypted value to decrypt. \
               `-` reads the ciphertext from stdin.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--decrypt-password",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Decryption password (inline). Leaks via argv/process table; \
               prefer --decrypt-password-file or --decrypt-password-stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--decrypt-password-file",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Read the decryption password from a file (one trailing \
               newline stripped).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--decrypt-password-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the decryption password from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json-out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Write a JSON envelope {schema_version, operation, plaintext} \
               to PATH instead of emitting plaintext on stdout.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── nostr ───────────────────────────────────────────────────────────────
// toolkit v0.34.0: derive Nostr keys (nsec/npub NIP-19) from a BIP-39
// seed. Accepts either a `--secret` (seed phrase / ms1 / BIP-38 / WIF /
// xprv, any secret NodeType) or a `--pubkey` (nsec/npub, for address/
// descriptor derivation only). `--network` drives address encoding.
// `--script-type` is a plain TEXT flag (custom value_parser in the toolkit,
// NOT a ValueEnum/dropdown). `--secret` + `--secret-stdin` are secret-class
// (zeroize, paste-warn, run-confirm). `--no-auto-repair` is the global flag.
// toolkit v0.34.2: `--import <readonly>` emits a read-only Bitcoin Core
// importdescriptors recipe; `--timestamp <now|unix>` (default `0`) is its
// rescan anchor. Both plain TEXT, non-secret (value_parsers in the toolkit).
const NOSTR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--all-script-types",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit addresses + descriptors for all script types.",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.34.2: read-only Bitcoin Core importdescriptors recipe.
    // Value-valued (`readonly`); `spending`/`both` are reserved (refused).
    FlagSchema {
        name: "--import",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Emit a ready-to-paste Bitcoin Core importdescriptors recipe \
               for the derived address(es). `readonly` = watch-only.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON envelope instead of human-readable output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Bitcoin network for address encoding (default mainnet).",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
    FlagSchema {
        name: "--pubkey",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Nostr public key (nsec or npub NIP-19). For address/descriptor \
               derivation; mutually exclusive with --secret.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--script-type",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Script type for address / descriptor derivation \
               (e.g. p2wpkh, p2sh-p2wpkh, p2tr).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--secret",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Secret input (BIP-39 phrase, ms1, xprv, WIF, BIP-38, etc.). \
               Mutually exclusive with --pubkey.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--secret-file",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Read secret input from a file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--secret-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read secret input from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    // toolkit v0.34.2: importdescriptors rescan anchor (only used with
    // --import). `now` or unix seconds; default `0` = rescan from genesis.
    FlagSchema {
        name: "--timestamp",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Bitcoin Core importdescriptors rescan anchor: `now` or unix \
               seconds. Default `0` (rescan from genesis). Used with --import.",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
];

// ─── silent-payment ──────────────────────────────────────────────────────
// toolkit v0.35.0: derive a BIP-352 Silent Payments **receiver** static
// address from a seed-bearing secret. Scan key `m/352'/coin'/account'/1'/0`
// + spend key `.../0'/0` → the base `sp1…`/`tsp1…` address (`B_scan‖B_spend`),
// plus labeled addresses via `--label <m>` (m≥1; `--label 0` is refused —
// the reserved change label must never be published). `--secret` +
// `--secret-stdin` are secret-class (zeroize, paste-warn, run-confirm);
// `--secret-file` is a plain path. Single-key WIF/minikey is refused (cannot
// derive `m/352'`). `--network` drives the HRP (`sp` mainnet / `tsp` else)
// + coin-type (0 mainnet / 1 testnet|signet|regtest). `--no-auto-repair` is
// the global flag. `--account` matches the BIP-32 hardened-index ceiling
// (2^31-1); `--label` is a repeating u32 (m≥1).
const SILENT_PAYMENT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (hardened; default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    // toolkit v0.36.1: also emit the BIP-352 m=0 CHANGE address (internal change
    // detection only — never hand out). Non-secret (public address).
    FlagSchema {
        name: "--change-address",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Also emit the BIP-352 m=0 change address (internal change \
               detection ONLY; never hand out as a receiving address).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON envelope instead of human-readable output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--label",
        kind: FlagKind::Number {
            min: 1,
            max: NumberMax::Static(4_294_967_295),
        },
        required: false,
        repeating: true,
        help: "Emit a labeled address for label index m (≥1; repeatable). \
               m=0 is reserved for change and is refused.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Bitcoin network: drives the HRP (sp mainnet / tsp else) and \
               coin-type (default mainnet).",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
    // toolkit v0.36.1: BIP-39 passphrase ("25th word"). SECRET (zeroize/mask/
    // paste-warn) — already covered by the toolkit's flag_is_secret.
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic-extension passphrase. Applies to phrase/ms1/\
               entropy inputs; ignored (with a warning) for an xprv input.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the BIP-39 passphrase from stdin (whitespace-preserving). \
               Mutually exclusive with --passphrase and --secret-stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--secret",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Seed-bearing secret (BIP-39 phrase, ms1, entropy-hex, or \
               master xprv). Single-key WIF/minikey is refused.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--secret-file",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Read the seed-bearing secret from a file.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--secret-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the seed-bearing secret from stdin.",
        secret: true,
        default_value: None,
        global: false,
    },
];

// ─── decode-address ──────────────────────────────────────────────────────
// toolkit v0.36.0: decode a Bitcoin address → network(s) / script type /
// witness version / scriptPubKey. Public-data; no secrets. Takes a required
// positional <address>; `--json` + the global `--no-auto-repair`.
const DECODE_ADDRESS_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "address",
    required: true,
    repeating: false,
    help: "The Bitcoin address to decode (P2PKH / P2SH / P2WPKH / P2WSH / P2TR, any network).",
    secret: false,
}];
const DECODE_ADDRESS_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope instead of the human-readable block.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── verify-message ──────────────────────────────────────────────────────
// toolkit v0.36.0: VERIFY-ONLY (no signing) Bitcoin message-signature
// verification — legacy "Bitcoin Signed Message" (P2PKH) + BIP-322 simple
// (P2WPKH/P2SH-P2WPKH/P2TR). Public operation; no secrets. Exactly one of
// --message / --message-file / --message-stdin is required (clap ArgGroup).
const VERIFY_MESSAGE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--address",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "The address the message was signed by.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(VERIFY_FORMATS),
        required: false,
        repeating: false,
        help: "Signature format: auto (default; legacy for P2PKH, BIP-322 otherwise), legacy, or bip322.",
        secret: false,
        default_value: Some("auto"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope instead of the human-readable line.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--message",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "The signed message, inline (exact bytes).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--message-file",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: false,
        repeating: false,
        help: "Read the message from a file (a single trailing newline is stripped).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--message-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the message from stdin (a single trailing newline is stripped).",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
    FlagSchema {
        name: "--signature",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "The signature (base64): a 65-byte recoverable sig (legacy) or a BIP-322 witness encoding.",
        secret: false,
        default_value: None,
        global: false,
    },
];

// ─── SCHEMA constant ─────────────────────────────────────────────────────

// toolkit v0.38.0: `mnemonic addresses` — batch watch-only address derivation.
// `--address-type` reflects as Text (custom value_parser, not a ValueEnum, so
// gui-schema cannot enumerate its 4 values). `--passphrase{,-stdin}` are secret.
// `--no-auto-repair` is the global flag (appended, not listed here).
const ADDRESSES_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "Source: xpub=<v> | phrase=<v> | entropy=<hex> | seedqr=<digits>. @env:VAR / - (stdin) for secret values.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--address-type",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "p2pkh | p2sh-p2wpkh | p2wpkh | p2tr. Selects the account path for seed sources and the render type.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(2147483647) },
        required: false,
        repeating: false,
        help: "Account index for seed sources (default 0; not applicable to xpub=).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--count",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(2147483648) },
        required: false,
        repeating: false,
        help: "Number of addresses per chain, from index 0 (default 10). Conflicts with --range.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--range",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Inclusive index range A,B. Conflicts with --count.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--chain",
        kind: FlagKind::Dropdown(&["receive", "change", "both"]),
        required: false,
        repeating: false,
        help: "Which chain(s) to render: receive (default), change, or both.",
        secret: false,
        default_value: Some("receive"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network override; must agree with an xpub's network kind.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase (seed sources). @env:VAR supported.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the BIP-39 passphrase from stdin (conflicts with --passphrase).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist language for phrase=/seedqr= (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope instead of text rows.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// toolkit v0.50.0: `build-descriptor` (descriptor-builder engine Release A).
// `--spec` is a FILE PATH (or `-` = stdin; omitted ⇒ stdin when not a TTY) —
// NEVER inline JSON, so it is `Path { stdio_sentinel: true }` (the `--blob`
// precedent), NOT `Text` (a Text widget would emit raw JSON → the toolkit
// would treat it as a path → ENOENT). `schema_mirror` is flag-NAME-only, so
// the Path-vs-Text kind (gui-schema reports "text") does not drift the gate.
const BUILD_DESCRIPTOR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--spec",
        kind: FlagKind::Path { stdio_sentinel: true },
        required: false,
        repeating: false,
        help: "Path to the JSON policy-tree spec (`-` reads from stdin; omitted \
               reads stdin when not a TTY).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--spec-schema",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Dump the versioned node-tree grammar JSON and exit (ignores other inputs).",
        secret: false,
        default_value: None,
        global: false,
    },
    // toolkit v0.51.0/v0.52.0 — archetype presets (12 flags). `--archetype`
    // carries the GUI-side `""` UNSET sentinel (default_value Some("") —
    // see the ARCHETYPES const comment). `--key`/`--recovery-key`/`--allow`
    // are repeating (row order = argv order = quorum order). The 10
    // `requires = "archetype"` clap edges + the `--emit-spec`
    // conflicts_with_all stay deliberately UN-projected (SPEC §4 recorded
    // decision: the CLI is the gate; the A2 archetype wizard supersedes the
    // generic form as the preset surface). The archetype↔spec mutex IS
    // projected (`conditional::build_descriptor`).
    FlagSchema {
        name: "--archetype",
        kind: FlagKind::Dropdown(ARCHETYPES),
        required: false,
        repeating: false,
        help: "Build from a curated archetype preset instead of a --spec node-tree; \
               parameters come from the preset flags. \"(none)\" = no preset.",
        secret: false,
        default_value: Some(""),
        global: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Primary-path key (`[fp/path]xpub…`); repeat per cosigner — row \
               order is preserved into the quorum.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number {
            min: 1,
            max: NumberMax::Static(20),
        },
        required: false,
        repeating: false,
        help: "Primary quorum threshold k.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--recovery-key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Recovery-path key; repeat per cosigner — row order is preserved.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--recovery-threshold",
        kind: FlagKind::Number {
            min: 1,
            max: NumberMax::Static(20),
        },
        required: false,
        repeating: false,
        help: "Recovery quorum threshold k.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--final-key",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Last-resort key (decaying-multisig tier 3).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--older",
        kind: FlagKind::Number {
            min: 1,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Relative timelock (blocks) on the gated path (decaying-multisig: \
               tier-1 timelock).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--recovery-older",
        kind: FlagKind::Number {
            min: 1,
            max: NumberMax::Static(2_147_483_647),
        },
        required: false,
        repeating: false,
        help: "Decaying-multisig tier-2 relative timelock (blocks); must exceed \
               --older.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--after",
        // clap accepts 0..=u32::MAX; GUI min 1 is deliberately tighter —
        // `after(0)` is gate-invalid miniscript (SPEC §2 / R0-r1 M8).
        kind: FlagKind::Number {
            min: 1,
            max: NumberMax::Static(4_294_967_295),
        },
        required: false,
        repeating: false,
        help: "Decaying-multisig tier-3 absolute locktime (block height or unix \
               time, per BIP-65 threshold).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hash",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "SHA-256 digest (64 hex chars) for hashlock-gated.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--emit-spec",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Print the lowered + gate-validated node-tree spec JSON instead of \
               building (review, edit, feed back via --spec).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--allow",
        kind: FlagKind::Dropdown(ALLOW_RULES),
        required: false,
        repeating: true,
        help: "Reviewed opt-out of ONE funds-safety sanity rule per occurrence \
               (repeatable). Every rule that fires is named in an unmissable \
               warning; never silent.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(BUILD_FORMATS),
        required: false,
        repeating: false,
        help: "Output payload format: `descriptor` (raw) or `bip388` (wallet policy JSON).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network (default mainnet).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit the full envelope (descriptor + bip388 + cost + diagnostics) as JSON.",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── gen-man ─────────────────────────────────────────────────────────────
//
// man-pages cycle (toolkit v0.73.0): emit roff man pages for the whole CLI
// tree into a directory. Exactly one subcommand flag — the required `--out
// <DIR>` directory path — plus the global `--no-auto-repair` flag that
// clap-derive propagates to every mnemonic subcommand (gui-schema emits both
// for `gen-man`). No conditional rules.
const GEN_MAN_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path { stdio_sentinel: false },
        required: true,
        repeating: false,
        help: "Directory to write the `*.1` man pages into (created if \
               absent). One page per (nested) subcommand, hyphen-joined \
               parent→child (e.g. `mnemonic-seed-xor-split.1`).",
        secret: false,
        default_value: None,
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// ─── word-card ───────────────────────────────────────────────────────────
//
// word-card cycle (toolkit v0.74.0): encode a BIP-39 mnemonic into a
// steel-engravable word-card (the wc-codec engine), or decode one back.
// schema_mirror gates flag NAMES only, so the kind / secret bits below are
// GUI-side ergonomics, not gate-checked by that test. No dropdown value-enums
// on any flag (gui-schema reports every flag with `choices: null`).
//
// The decode path takes a repeating positional `<WORD>...` (`words`,
// non-required, repeating). The toolkit's gui-schema emits it as a positional,
// so it rides the `positional_args` slice (mirroring decode-address's
// `<address>`). The toolkit emits `words` WITHOUT a `secret` bit (== false) —
// the `t4_secret_positional_census_matches_frozen_literal` gate freezes the
// secret-positional census at exactly the 5 ms.rs entries, so a `secret: true`
// here would both diverge from the toolkit projection AND break that census
// (it would also demand `secret_widgets["positional:words"]` assembler
// plumbing). We mirror the toolkit's `false`; the seed-value secrecy is a
// forms-intake concern (node-type classification + argv masking), not this
// schema bit. Per the **Positional-arg policy** comment above, this entry is
// the structured mirror; whether the GUI surfaces a positional intake widget
// is an orthogonal forms concern.
const WORD_CARD_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "words",
    required: false,
    repeating: true,
    help: "Decode path: the BIP-39 word-card words to decode (repeating).",
    secret: false,
}];
const WORD_CARD_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--decode",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Decode a Word Card back to its source mk1/md1 card (inverse \
               of the default encode direction).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--decode-plate",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "One RAID plate's word list for --decode reconstruction \
               (supply the surviving plates of an n+r array to reconstruct \
               a lost data plate).",
        secret: false,
        default_value: None,
        global: false,
    },
    // NOTE: the toolkit's gui-schema (v0.74.0) emits `--from` for word-card
    // with `secret: false` — it is NOT in `mnemonic_toolkit::secrets`'
    // hand-coded secret-flag list for this subcommand (the seed VALUE is
    // entered via the same node-grammar that the convert `--from` composite
    // already classifies per-node). The `schema_mirror_secret_drift` gate
    // enforces strict set-equality of `secret == true` flags against the
    // toolkit projection, so this MUST mirror the toolkit's `false`. (The
    // secret HANDLING of the seed value rides node-type classification +
    // argv masking, not this flag-level bool.)
    FlagSchema {
        name: "--from",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Source m*1 card to encode into a Word Card: an mk1 (xpub) \
               or md1 (descriptor) card. PUBLIC material — NOT a secret, \
               NOT a seed phrase. Use - for stdin.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--integrity-bits",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Integrity-check bit budget for the card (default 44).",
        secret: false,
        default_value: Some("44"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a JSON envelope instead of the human-readable card.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--parity-pct",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Parity redundancy expressed as a percentage of the payload.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--parity-words",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Parity redundancy expressed as an absolute number of parity \
               words.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--raid",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "RAID redundancy level — split the card across N plates for \
               loss tolerance (default 0 = single card).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    NO_AUTO_REPAIR_FLAG,
];

// Phase 5: wire the conditional-visibility fn pointers per subcommand.
// v0.3: `derive-child` gained `--passphrase-stdin conflicts_with passphrase`
// at toolkit v0.13.0; conditional flipped from `None` to
// `Some(crate::form::conditional::derive_child)`.
const SUBCOMMANDS: &[SubcommandSchema] = &[
    SubcommandSchema {
        name: "addresses",
        human_name: "Addresses (batch watch-only address listing)",
        flags: ADDRESSES_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "bundle",
        human_name: "Bundle (emit 3-card)",
        flags: BUNDLE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: true,
        conditional: Some(crate::form::conditional::bundle),
    },
    SubcommandSchema {
        name: "verify-bundle",
        human_name: "Verify Bundle (round-trip)",
        flags: VERIFY_BUNDLE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: true,
        conditional: Some(crate::form::conditional::verify_bundle),
    },
    SubcommandSchema {
        name: "convert",
        human_name: "Convert (between formats)",
        flags: CONVERT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::convert),
    },
    SubcommandSchema {
        name: "export-wallet",
        human_name: "Export Wallet (watch-only)",
        flags: EXPORT_WALLET_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: true,
        conditional: Some(crate::form::conditional::export_wallet),
    },
    // toolkit v0.50.0: descriptor-builder engine Release A. A JSON policy-tree
    // spec (`--spec`, file/stdin) → wsh descriptor + BIP-388 + cost preview +
    // node-addressed diagnostics. Watch-only (no secret flags); the recursive
    // node-tree wizard FORM is a later cycle — this entry yields a basic
    // file-picker `--spec` form.
    // v0.30.0 (toolkit v0.51.0/v0.52.0): archetype presets + `--allow` — the
    // archetype↔spec clap mutex is projected via
    // `conditional::build_descriptor`; the other preset clap edges stay
    // deliberately UN-projected (CLI is the gate — SPEC §4).
    SubcommandSchema {
        name: "build-descriptor",
        human_name: "Build Descriptor (policy-tree spec → wsh descriptor + BIP-388)",
        flags: BUILD_DESCRIPTOR_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::build_descriptor),
    },
    // toolkit v0.43.0: re-derive a wallet export from a third-party source
    // (`--from`) — the inverse-ish sibling of export-wallet. Input is via
    // `--from`, not `--slot` (allows_slots: false). v0.44.0 added the
    // multisig-cosigner half (`--md1`/`--cosigner`); `--from` is now
    // `required_unless_present="md1"`. Since toolkit v0.46.2 the gui-schema
    // projects this rule (`build_subcommand_conditional_rules` restore arm →
    // `not(flag_present "--md1") → {--from, required}`), so the at-least-one
    // rule in `conditional::restore` IS drift-gated by
    // `gui_schema_conditional_drift` (`("restore", 1)` in `SUBCOMMAND_FLOORS`).
    SubcommandSchema {
        name: "restore",
        human_name: "Restore (re-derive a wallet export from a source)",
        flags: RESTORE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::restore),
    },
    SubcommandSchema {
        name: "derive-child",
        human_name: "Derive Child (BIP-85)",
        flags: DERIVE_CHILD_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::derive_child),
    },
    // toolkit v0.33.0: Electrum field-encrypted-secret decryption.
    SubcommandSchema {
        name: "electrum-decrypt",
        human_name: "Electrum Decrypt (field-encrypted secret → plaintext)",
        flags: ELECTRUM_DECRYPT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // toolkit v0.34.0: Nostr key derivation from a BIP-39 seed / secret.
    SubcommandSchema {
        name: "nostr",
        human_name: "Nostr (derive Nostr nsec/npub from seed)",
        flags: NOSTR_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // toolkit v0.35.0: BIP-352 Silent Payments receiver static address.
    SubcommandSchema {
        name: "silent-payment",
        human_name: "Silent Payment (BIP-352 receiver sp1… address from seed)",
        flags: SILENT_PAYMENT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // toolkit v0.36.0: decode a Bitcoin address (public-data; positional <address>).
    SubcommandSchema {
        name: "decode-address",
        human_name: "Decode Address (address → network / type / scriptPubKey)",
        flags: DECODE_ADDRESS_FLAGS,
        positional_args: DECODE_ADDRESS_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // toolkit v0.36.0: verify a Bitcoin message signature (legacy + BIP-322).
    SubcommandSchema {
        name: "verify-message",
        human_name: "Verify Message (legacy signmessage + BIP-322)",
        flags: VERIFY_MESSAGE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "final-word",
        human_name: "Final Word (BIP-39 N-1 -> candidate Nth words)",
        flags: FINAL_WORD_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "seed-xor-split",
        human_name: "Seed XOR Split (Coldcard all-or-nothing splitter)",
        flags: SEED_XOR_SPLIT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "seed-xor-combine",
        human_name: "Seed XOR Combine (reconstruct from XOR shares)",
        flags: SEED_XOR_COMBINE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "seedqr-encode",
        human_name: "SeedQR Encode (BIP-39 phrase → SeedQR numeric string)",
        flags: SEEDQR_ENCODE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "seedqr-decode",
        human_name: "SeedQR Decode (SeedQR numeric string → BIP-39 phrase)",
        flags: SEEDQR_DECODE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "slip39-split",
        human_name: "SLIP-39 Split (K-of-N share splitter)",
        flags: SLIP39_SPLIT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::slip39_split),
    },
    SubcommandSchema {
        name: "slip39-combine",
        human_name: "SLIP-39 Combine (reconstruct from shares)",
        flags: SLIP39_COMBINE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::slip39_combine),
    },
    // toolkit v0.40.0: BIP-93 codex32 K-of-N share split / combine of an
    // ms1 secret. Flattened reflective names `ms-shares-split` /
    // `ms-shares-combine` in gui-schema v5. No conditional rules
    // (`conditional_rules: []` in gui-schema) → conditional: None.
    SubcommandSchema {
        name: "ms-shares-split",
        human_name: "ms-shares Split (BIP-93 codex32 K-of-N share splitter)",
        flags: MS_SHARES_SPLIT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "ms-shares-combine",
        human_name: "ms-shares Combine (recombine >=K codex32 shares)",
        flags: MS_SHARES_COMBINE_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // v0.9.0 (toolkit v0.22.0): standalone BCH error-correction subcommand.
    SubcommandSchema {
        name: "repair",
        human_name: "Repair (BCH error-correct ms1 / mk1 / md1)",
        flags: REPAIR_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::repair),
    },
    // v0.9.0 (toolkit v0.22.0): describe contents of an m-format card.
    SubcommandSchema {
        name: "inspect",
        human_name: "Inspect (describe ms1 / mk1 / md1 contents)",
        flags: INSPECT_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::inspect),
    },
    // v0.11.0 (toolkit v0.26.0): import a third-party wallet blob and
    // optionally overlay seed material to re-attach cosigner secrets.
    SubcommandSchema {
        name: "import-wallet",
        human_name: "Import Wallet (third-party BSMS / Bitcoin Core)",
        flags: IMPORT_WALLET_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: true,
        // v0.26.0 has no cross-flag conditional rules on import-wallet;
        // the v3 PinValue / v4 DisableOptions effects could be wired in a
        // future cycle (e.g. force --select-descriptor=all when --format=bsms).
        conditional: None,
    },
    // v0.11.0 (toolkit v0.26.0): xpub-search umbrella, 4 modes.
    // `conditional: None` for v0.11.0; cross-flag mutex visibility (e.g.
    // --phrase vs --phrase-stdin vs --ms1) is a v0.12.0 polish item
    // tracked by the `xpub-search-gui-flag-mutex-visibility` FOLLOWUP.
    // The toolkit clap-derive enforces mutual exclusion + at-least-one
    // rules at run-time; the GUI's run-confirm modal surfaces stderr.
    SubcommandSchema {
        name: "xpub-search-path-of-xpub",
        human_name: "xpub-search: Path of xpub (locate target xpub's path)",
        flags: XPUB_SEARCH_PATH_OF_XPUB_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "xpub-search-account-of-descriptor",
        human_name: "xpub-search: Account of descriptor (locate descriptor's account)",
        flags: XPUB_SEARCH_ACCOUNT_OF_DESCRIPTOR_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "xpub-search-address-of-xpub",
        human_name: "xpub-search: Address of xpub (locate addresses under xpub)",
        flags: XPUB_SEARCH_ADDRESS_OF_XPUB_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "xpub-search-passphrase-of-xpub",
        human_name: "xpub-search: Passphrase of xpub (locate passphrase from candidates)",
        flags: XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // v0.11.0 (toolkit v0.26.0): wsh-vs-tr per-spending-condition cost
    // comparison. `--miniscript` and `--descriptor` are mutually exclusive
    // exactly-one-of inputs (toolkit Phase 1+2). When neither is given the
    // toolkit Phase 3 dispatcher falls back to stdin — not surfaced as a
    // GUI input slot since the GUI always passes one of the two flags.
    SubcommandSchema {
        name: "compare-cost",
        human_name: "Compare Cost (wsh vs tr per spending condition)",
        flags: COMPARE_COST_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::compare_cost),
    },
    // man-pages cycle (toolkit v0.73.0): emit roff man pages for the whole
    // CLI tree into `--out <DIR>`. No conditional rules.
    SubcommandSchema {
        name: "gen-man",
        human_name: "Gen Man (emit roff man pages for the whole CLI tree)",
        flags: GEN_MAN_FLAGS,
        positional_args: NO_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // word-card cycle (toolkit v0.74.0): encode a BIP-39 mnemonic into a
    // steel-engravable word-card (wc-codec engine) or decode one back. The
    // decode path takes a repeating positional `<WORD>...` (`words`); the
    // gui-schema reports `conditional_rules: []` → conditional: None.
    SubcommandSchema {
        name: "word-card",
        human_name: "Word Card (encode/decode a steel-engravable BIP-39 word-card)",
        flags: WORD_CARD_FLAGS,
        positional_args: WORD_CARD_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
];

// `pinned_version` is rendered as a monospace label in the action-bar
// `Pinned:` row at `src/main.rs:347`. There is NO runtime comparison
// logic that consumes this field — `schema_check.rs` reads
// `pinned-upstream.toml` + the `<cli> gui-schema` JSON, not this string.
// Bump it in lockstep with the toolkit tag for human-display parity only;
// drift here is a cosmetic banner mismatch, not a functional error.
pub const SCHEMA: Schema = Schema {
    cli_name: "mnemonic",
    pinned_version: "mnemonic 0.75.0",
    subcommands: SUBCOMMANDS,
};
