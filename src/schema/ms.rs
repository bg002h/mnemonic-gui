//! Pinned schema for the `ms` CLI (ms-cli-v0.2.1).
//!
//! v0.2 scope: `inspect` (from v0.1) plus `encode`, `decode`, `verify`,
//! `vectors`. See Phase D.1 audit report at
//! `design/agent-reports/v0_2-phase-D1-help-audit-r1.md` for per-flag
//! provenance.

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
];

pub const SCHEMA: Schema = Schema {
    cli_name: "ms",
    pinned_version: "ms 0.2.1",
    subcommands: SUBCOMMANDS,
};
