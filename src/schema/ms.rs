//! Pinned schema for the `ms` CLI (ms-cli-v0.7.0).
//!
//! Scope: `inspect`/`encode`/`decode`/`verify`/`vectors` plus `repair`
//! (backfilled into the mirror at v0.5 — it shipped in v0.4 but was never
//! mirrored), the v0.5 read-only `derive` (master fingerprint + account
//! xpub), and the v0.7.0 BIP-93 codex32 K-of-N `split`/`combine` pair. See
//! `design/agent-reports/v0_2-phase-D1-help-audit-r1.md` for the original
//! per-flag provenance.

use super::{FlagKind, FlagSchema, PositionalArgSchema, Schema, SubcommandSchema};

/// BIP-39 wordlist tokens accepted by the `ms` CLI. Hyphenated
/// Chinese variants (NOT the fused tokens used by mnemonic.rs:
/// `simplifiedchinese` / `traditionalchinese`). Using the wrong
/// tokens silently emits argv rejected by the binary — see D.1
/// audit R1 finding #1.
pub const LANG_MS: &[&str] = &[
    "english",
    "japanese",
    "korean",
    "spanish",
    "chinese-simplified",
    "chinese-traditional",
    "french",
    "italian",
    "czech",
    "portuguese",
];

// ─── inspect ─────────────────────────────────────────────────────────────

// `ms inspect [MS1] [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON instead of text verdict + fields.",
    secret: false,
    default_value: None,
    global: false,
}];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to inspect. Use `-` or omit to read from stdin.",
}];

// ─── encode ──────────────────────────────────────────────────────────────

// `ms encode --phrase|--hex [--language] [--no-engraving-card] [--json]`
//
// Upstream: `--phrase` XOR `--hex` (required_one_of with mutual exclusion).
// Both are secret-bearing (BIP-39 mnemonic / raw entropy bytes). When
// `--hex` is supplied, `--language` is ignored (upstream help). Conditional
// fn wires this in `form::conditional::ms_encode`.
const ENCODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic phrase. XOR with --hex.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Raw entropy as hex. XOR with --phrase. --language is ignored.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english). Ignored when --hex is set.",
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
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const ENCODE_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── decode ──────────────────────────────────────────────────────────────

// `ms decode [MS1] [--language] [--json]`
const DECODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english) used to render the phrase.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const DECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to decode. Use `-` or omit to read from stdin.",
}];

// ─── verify ──────────────────────────────────────────────────────────────

// `ms verify [MS1] --phrase [--language] [--json]`
//
// `--phrase` is secret-bearing (round-trip check against the supplied ms1).
const VERIFY_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic phrase to round-trip against the ms1.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const VERIFY_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to verify. Use `-` or omit to read from stdin.",
}];

// ─── vectors ─────────────────────────────────────────────────────────────

// `ms vectors [--pretty]` — maintainer tool, emits test-vector JSON.
const VECTORS_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--pretty",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Pretty-print JSON output.",
    secret: false,
    default_value: None,
    global: false,
}];

const VECTORS_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── derive (v0.5) ───────────────────────────────────────────────────────
// `ms derive [ms1] [--hex|--phrase] [--template] [--account] [--network]
//            [--passphrase|--passphrase-stdin] [--language] [--json]`.
// `--template`/`--network`/`--language` reflect as ValueEnum dropdowns;
// `--account` reflects as text (ms-cli gui-schema classifies u32 as text).
const DERIVE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Raw entropy hex (alternative to the ms1).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 phrase (alternative to the ms1).",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--template",
        kind: FlagKind::Dropdown(&["bip44", "bip49", "bip84", "bip86"]),
        required: false,
        repeating: false,
        help: "Account-path template; emits an account xpub.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--account",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Account index for --template (default 0).",
        secret: false,
        default_value: Some("0"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(&["mainnet", "testnet"]),
        required: false,
        repeating: false,
        help: "Network for the account xpub serialization + coin-type.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--passphrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 passphrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--passphrase-stdin",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Read the BIP-39 passphrase from stdin.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist (load-bearing; default english).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const DERIVE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string. Use `-` or omit to read from stdin.",
}];

// ─── repair (v0.4; backfilled into the mirror at v0.5) ───────────────────
const REPAIR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--ms1",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "ms1 string to repair via BCH error correction. `-` reads stdin.",
        // v0.33.0 deliberate GUI-side override (audit I4 ms.rs half): the
        // to-be-repaired ms1 IS master-secret material (BCH-corrupted BIP-39
        // entropy; the lone false twin of the 8-site --ms1 census). No
        // automated gate covers ms.rs secret bits (schema_mirror_secret_drift
        // walks schema::mnemonic only), and ms-cli has no gui-schema surface
        // to mirror — see FOLLOWUPS.md::ms-repair-ms1-not-secret-classified.
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const REPAIR_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── split (v0.7.0) ──────────────────────────────────────────────────────
//
// `ms split (--phrase|--hex) --threshold K --shares N [--language] [--json]`
// BIP-93 codex32 K-of-N share splitter. `--phrase` / `--hex` are both
// secret-bearing (BIP-39 mnemonic / raw entropy). schema_mirror gates the
// flag-NAME set; the `--json` wire-shape is NOT gated (FOLLOWUP
// `ms-kofn-json-wire-shape-ungated`).
const SPLIT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--phrase",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP-39 mnemonic to split. `-` reads stdin. XOR with --hex.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--hex",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Hex-encoded entropy to split (16/20/24/28/32 B). XOR with --phrase.",
        secret: true,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--threshold",
        kind: FlagKind::Number { min: 2, max: super::NumberMax::Static(9) },
        required: true,
        repeating: false,
        help: "Threshold K — minimum shares needed to recombine (2..=9).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--shares",
        kind: FlagKind::Number { min: 2, max: super::NumberMax::Static(31) },
        required: true,
        repeating: false,
        help: "Total shares N to produce (K <= N <= 31).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--language",
        kind: FlagKind::Dropdown(LANG_MS),
        required: false,
        repeating: false,
        help: "BIP-39 wordlist for the input phrase; ignored under --hex. \
               A non-English wordlist produces a `mnem` share-set (default english).",
        secret: false,
        default_value: Some("english"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON object on stdout instead of multi-line text.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const SPLIT_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── combine (v0.7.0) ────────────────────────────────────────────────────
//
// `ms combine <SHARES>... [--to phrase|entropy|ms1] [--json]`
// The shares are POSITIONAL (secret-equivalent — flagged on the positional).
const COMBINE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--to",
        kind: FlagKind::Dropdown(COMBINE_TO_SHAPES),
        required: false,
        repeating: false,
        help: "Output form (default phrase): phrase (mnem -> wire language, entr -> english), \
               entropy (hex), or ms1 (single unshared string).",
        secret: false,
        default_value: Some("phrase"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON object on stdout instead of text.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const COMBINE_TO_SHAPES: &[&str] = &["phrase", "entropy", "ms1"];

const COMBINE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "shares",
    required: true,
    repeating: true,
    help: "Distributed share strings to recombine (K or more, distinct indices). Secret-equivalent.",
}];

// ─── SCHEMA constant ─────────────────────────────────────────────────────

const SUBCOMMANDS: &[SubcommandSchema] = &[
    SubcommandSchema {
        name: "inspect",
        human_name: "Inspect (verdict + fields)",
        flags: INSPECT_FLAGS,
        positional_args: INSPECT_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "encode",
        human_name: "Encode (phrase/hex -> ms1)",
        flags: ENCODE_FLAGS,
        positional_args: ENCODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::ms_encode),
    },
    SubcommandSchema {
        name: "decode",
        human_name: "Decode (ms1 -> phrase)",
        flags: DECODE_FLAGS,
        positional_args: DECODE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "verify",
        human_name: "Verify (phrase <-> ms1 round-trip)",
        flags: VERIFY_FLAGS,
        positional_args: VERIFY_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "vectors",
        human_name: "Vectors (test-vector dump)",
        flags: VECTORS_FLAGS,
        positional_args: VECTORS_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "derive",
        human_name: "Derive (master fingerprint + account xpub)",
        flags: DERIVE_FLAGS,
        positional_args: DERIVE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "repair",
        human_name: "Repair (BCH error correction)",
        flags: REPAIR_FLAGS,
        positional_args: REPAIR_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // v0.7.0: BIP-93 codex32 K-of-N share split / combine.
    SubcommandSchema {
        name: "split",
        human_name: "Split (BIP-93 codex32 K-of-N share splitter)",
        flags: SPLIT_FLAGS,
        positional_args: SPLIT_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "combine",
        human_name: "Combine (recombine >=K codex32 shares)",
        flags: COMBINE_FLAGS,
        positional_args: COMBINE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
];

pub const SCHEMA: Schema = Schema {
    cli_name: "ms",
    pinned_version: "ms 0.7.0",
    subcommands: SUBCOMMANDS,
};
