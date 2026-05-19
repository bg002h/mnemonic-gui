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
    "jade",
    "sparrow",
    "specter",
    "electrum",
    "green",
];

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
    NO_AUTO_REPAIR_FLAG,
];

// Two-element option list for `--format` on import-wallet. Distinct from
// `EXPORT_FORMATS` (the inbound parser supports only BSMS + Bitcoin Core
// in v0.26.0; outbound emitter supports 8 formats).
const IMPORT_WALLET_FORMATS: &[&str] = &["bsms", "bitcoin-core"];

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
];

// `pinned_version` is rendered as a monospace label in the action-bar
// `Pinned:` row at `src/main.rs:347`. There is NO runtime comparison
// logic that consumes this field — `schema_check.rs` reads
// `pinned-upstream.toml` + the `<cli> gui-schema` JSON, not this string.
// Bump it in lockstep with the toolkit tag for human-display parity only;
// drift here is a cosmetic banner mismatch, not a functional error.
pub const SCHEMA: Schema = Schema {
    cli_name: "mnemonic",
    pinned_version: "mnemonic 0.24.0",
    subcommands: SUBCOMMANDS,
};
