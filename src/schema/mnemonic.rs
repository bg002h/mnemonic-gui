//! Pinned schema for the `mnemonic` CLI from mnemonic-toolkit-v0.8.1.
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

use super::{FlagKind, FlagSchema, Schema, SubcommandSchema};

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

// ─── bundle ──────────────────────────────────────────────────────────────

const BUNDLE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: true,
        repeating: false,
        help: "Bitcoin network for derivations + address encoding.",
        secret: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Pre-built template name. Mutually-required-one-of with \
               --descriptor / --descriptor-file.",
        secret: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "User-supplied BIP-388 descriptor. XOR with --descriptor-file.",
        secret: false,
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
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic extension passphrase.",
        secret: true,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: 2_147_483_647,
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (default 0).",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit envelope JSON (ms1/mk1/md1 + metadata).",
        secret: false,
    },
    FlagSchema {
        name: "--no-engraving-card",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Suppress the human-readable engraving-card panel.",
        secret: false,
    },
    FlagSchema {
        name: "--multisig-path-family",
        kind: FlagKind::Dropdown(MULTISIG_PATH_FAMILIES),
        required: false,
        repeating: false,
        help: "Multisig derivation path family (default bip87).",
        secret: false,
    },
    FlagSchema {
        name: "--privacy-preserving",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Suppress master fingerprint from mk1 + engraving card.",
        secret: false,
    },
    FlagSchema {
        name: "--self-check",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Re-parse the emitted bundle and verify round-trip.",
        secret: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 1, max: 16 },
        required: false,
        repeating: false,
        help: "Multisig threshold K (1 ≤ K ≤ N ≤ 16).",
        secret: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "v0.4 unified slot input. Repeating flag — one occurrence per \
               (slot, subkey) tuple. Grammar: @N.<subkey>=<value>. Handled \
               by SlotEditor composite widget (SPEC §4).",
        secret: false,
    },
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
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Template. Mutually-required-one-of with --descriptor / \
               --descriptor-file.",
        secret: false,
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "User-supplied descriptor for the re-parse path.",
        secret: false,
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
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic extension passphrase.",
        secret: true,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: 2_147_483_647,
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (default 0).",
        secret: false,
    },
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot ms1 card (schema-2/3 single use; schema-4 repeating). \
               Empty string is watch-only sentinel.",
        secret: true,
    },
    FlagSchema {
        name: "--mk1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot mk1 card (repeating).",
        secret: false,
    },
    FlagSchema {
        name: "--md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Per-slot md1 card (repeating).",
        secret: false,
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
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON-shaped output.",
        secret: false,
    },
    FlagSchema {
        name: "--multisig-path-family",
        kind: FlagKind::Dropdown(MULTISIG_PATH_FAMILIES),
        required: false,
        repeating: false,
        help: "Multisig derivation path family.",
        secret: false,
    },
    FlagSchema {
        name: "--privacy-preserving",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Expect mk1 omits master fingerprint.",
        secret: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 1, max: 16 },
        required: false,
        repeating: false,
        help: "Multisig threshold K.",
        secret: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Slot input @N.<subkey>=<value>. Handled by SlotEditor.",
        secret: false,
    },
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
    },
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(NODE_TYPES),
        required: true,
        repeating: true,
        help: "Destination node (repeating: clap Append).",
        secret: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Bitcoin network.",
        secret: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(TEMPLATES),
        required: false,
        repeating: false,
        help: "Template (when --to involves derivation).",
        secret: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Explicit BIP-32 derivation path.",
        secret: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 PBKDF2 passphrase.",
        secret: true,
    },
    FlagSchema {
        name: "--bip38-passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-38 Scrypt passphrase (distinct from --passphrase).",
        secret: true,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read --passphrase value from stdin (preserves NULL bytes).",
        secret: true,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: 2_147_483_647,
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index.",
        secret: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master fingerprint (8 hex chars).",
        secret: false,
    },
    FlagSchema {
        name: "--xpub-prefix",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "SLIP-0132 prefix override for --to xpub.",
        secret: false,
    },
    FlagSchema {
        name: "--electrum-version",
        kind: FlagKind::Dropdown(ELECTRUM_VERSIONS),
        required: false,
        repeating: false,
        help: "Electrum seed-version selector for (Entropy, ElectrumPhrase).",
        secret: false,
    },
    FlagSchema {
        name: "--electrum-language",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Electrum wordlist (distinct from --language).",
        secret: false,
    },
    FlagSchema {
        name: "--script-type",
        kind: FlagKind::Dropdown(SCRIPT_TYPES),
        required: false,
        repeating: false,
        help: "Script-type selector for (Xpub, Address) derivation.",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON-shaped output.",
        secret: false,
    },
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
    },
    FlagSchema {
        name: "--descriptor",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "User-supplied BIP-388 descriptor.",
        secret: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 1, max: 16 },
        required: false,
        repeating: false,
        help: "Multisig threshold K (1 ≤ K ≤ N).",
        secret: false,
    },
    FlagSchema {
        name: "--multisig-path-family",
        kind: FlagKind::Dropdown(MULTISIG_PATH_FAMILIES),
        required: false,
        repeating: false,
        help: "Multisig path family (default bip87).",
        secret: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network (default mainnet).",
        secret: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "Ignored (watch-only); kept for slot-parser symmetry.",
        secret: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Number {
            min: 0,
            max: 2_147_483_647,
        },
        required: false,
        repeating: false,
        help: "BIP-32 account index (default 0).",
        secret: false,
    },
    FlagSchema {
        name: "--slot",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Slot input @N.<subkey>=<value>. Handled by SlotEditor.",
        secret: false,
    },
    FlagSchema {
        name: "--format",
        kind: FlagKind::Dropdown(EXPORT_FORMATS),
        required: false,
        repeating: false,
        help: "Output format (default bitcoin-core).",
        secret: false,
    },
    FlagSchema {
        name: "--output",
        kind: FlagKind::Path {
            stdio_sentinel: true,
        },
        required: false,
        repeating: false,
        help: "Output path. `-` (default) → stdout.",
        secret: false,
    },
    FlagSchema {
        name: "--range",
        kind: FlagKind::Range,
        required: false,
        repeating: false,
        help: "Bitcoin Core `range` field, comma-separated. Default 0,999.",
        secret: false,
    },
    FlagSchema {
        name: "--timestamp",
        kind: FlagKind::Timestamp,
        required: false,
        repeating: false,
        help: "Bitcoin Core `timestamp` field. `now` or unix seconds.",
        secret: false,
    },
    FlagSchema {
        name: "--bitcoin-core-version",
        kind: FlagKind::Number { min: 24, max: 25 },
        required: false,
        repeating: false,
        help: "Bitcoin Core target version (24 or 25, default 25).",
        secret: false,
    },
    FlagSchema {
        name: "--taproot-internal-key",
        kind: FlagKind::TaggedOrIndexed(&["nums"]),
        required: false,
        repeating: false,
        help: "Taproot internal-key designation for tr-multi-a / \
               tr-sortedmulti-a. `nums` or `@N` (cosigner N's xpub).",
        secret: false,
    },
    FlagSchema {
        name: "--wallet-name",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Wallet label. Required for Sparrow / Specter / Electrum / \
               Green formats.",
        secret: false,
    },
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
    },
    FlagSchema {
        name: "--application",
        kind: FlagKind::Dropdown(BIP85_APPLICATIONS),
        required: true,
        repeating: false,
        help: "BIP-85 application token.",
        secret: false,
    },
    FlagSchema {
        name: "--length",
        kind: FlagKind::Number { min: 0, max: 8192 },
        required: true,
        repeating: false,
        help: "Per-app length validator. Pass 0 for xprv / hd-seed.",
        secret: false,
    },
    FlagSchema {
        name: "--index",
        kind: FlagKind::Number {
            min: 0,
            max: 2_147_483_647,
        },
        required: true,
        repeating: false,
        help: "Hardened child index (0..2^31).",
        secret: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for emitted hd-seed / xprv (default mainnet).",
        secret: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANGUAGES),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for --application bip39 (default english).",
        secret: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase (used only with --from phrase=).",
        secret: true,
    },
    FlagSchema {
        name: "--dice-sides",
        kind: FlagKind::Number {
            min: 2,
            max: 4_294_967_295,
        },
        required: false,
        repeating: false,
        help: "Number of sides for --application dice.",
        secret: false,
    },
];

// ─── SCHEMA constant ─────────────────────────────────────────────────────

const SUBCOMMANDS: &[SubcommandSchema] = &[
    SubcommandSchema {
        name: "bundle",
        human_name: "Bundle (emit 3-card)",
        flags: BUNDLE_FLAGS,
        allows_slots: true,
        conditional: None,
    },
    SubcommandSchema {
        name: "verify-bundle",
        human_name: "Verify Bundle (round-trip)",
        flags: VERIFY_BUNDLE_FLAGS,
        allows_slots: true,
        conditional: None,
    },
    SubcommandSchema {
        name: "convert",
        human_name: "Convert (between formats)",
        flags: CONVERT_FLAGS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "export-wallet",
        human_name: "Export Wallet (watch-only)",
        flags: EXPORT_WALLET_FLAGS,
        allows_slots: true,
        conditional: None,
    },
    SubcommandSchema {
        name: "derive-child",
        human_name: "Derive Child (BIP-85)",
        flags: DERIVE_CHILD_FLAGS,
        allows_slots: false,
        conditional: None,
    },
];

// R1 I-1 fold: `pinned_version` MUST match the literal `--version` output
// string that the runtime soft-check (SPEC §11) reads at GUI launch. The
// upstream `mnemonic-toolkit-v0.8.1` git tag did NOT bump the crate
// package version from `0.8.0`, so `mnemonic --version` emits
// `"mnemonic 0.8.0"`. The git-tag string remains the source of truth for
// CI install commands and lives in `pinned-upstream.toml`'s `[mnemonic].tag`
// field; `pinned_version` here is the comparison string for the runtime
// banner. Phase 9's `schema_check.rs` reads BOTH: tag for CI install,
// `pinned_version` for runtime soft-check.
pub const SCHEMA: Schema = Schema {
    cli_name: "mnemonic",
    pinned_version: "mnemonic 0.8.0",
    subcommands: SUBCOMMANDS,
};
