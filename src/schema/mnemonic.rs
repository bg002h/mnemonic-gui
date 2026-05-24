//! Pinned schema for the `mnemonic` CLI from mnemonic-toolkit-v0.13.0.
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
// toolkit v0.36.0: `verify-message --format` value-enum.
const VERIFY_FORMATS: &[&str] = &["auto", "legacy", "bip322"];

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
];

const BSMS_FORMS: &[&str] = &["2-line", "4-line"];

// R1 C-2 fold: NODE_TYPES exactly mirrors upstream
// `NodeType::as_str()` ordering in
// `crates/mnemonic-toolkit/src/cmd/convert.rs:48-64`. Drift here is invisible
// to the schema-mirror flag-name test, so we hand-pin against the upstream
// source. Master xpub / fingerprint plumbing for cosigner identification
// lives in SlotSubkey (slot_input.rs), NOT NodeType — `--from`/`--to`
// never see those tokens.
const NODE_TYPES: &[&str] = &[
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
        name: "--from",
        kind: FlagKind::NodeValueComposite(NODE_TYPES),
        required: true,
        repeating: false,
        help: "Source node: <node>=<value>. `=-` reads value from stdin.",
        secret: false, // secrecy is value-dependent; per-row paste-warn fires
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(NODE_TYPES),
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
        help: "Bitcoin Core `timestamp` field. `now` or unix seconds.",
        secret: false,
        default_value: Some("now"),
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
        secret: false, // value-dependent; per-row paste-warn fires
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
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read master BIP-39 phrase from stdin.",
        secret: false,
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
        secret: false,
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
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read master BIP-39 phrase from stdin.",
        secret: false,
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
        secret: false,
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
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read master BIP-39 phrase from stdin.",
        secret: false,
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
        secret: false,
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

// Phase 5: wire the conditional-visibility fn pointers per subcommand.
// v0.3: `derive-child` gained `--passphrase-stdin conflicts_with passphrase`
// at toolkit v0.13.0; conditional flipped from `None` to
// `Some(crate::form::conditional::derive_child)`.
const SUBCOMMANDS: &[SubcommandSchema] = &[
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
];

// `pinned_version` is rendered as a monospace label in the action-bar
// `Pinned:` row at `src/main.rs:347`. There is NO runtime comparison
// logic that consumes this field — `schema_check.rs` reads
// `pinned-upstream.toml` + the `<cli> gui-schema` JSON, not this string.
// Bump it in lockstep with the toolkit tag for human-display parity only;
// drift here is a cosmetic banner mismatch, not a functional error.
pub const SCHEMA: Schema = Schema {
    cli_name: "mnemonic",
    pinned_version: "mnemonic 0.36.1",
    subcommands: SUBCOMMANDS,
};
