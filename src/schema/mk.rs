//! Pinned schema for the `mk` CLI (mk-cli-v0.2.0).
//!
//! v0.2 scope: `inspect` (from v0.1) plus `encode`, `decode`, `verify`,
//! `vectors`. See Phase D.1 audit report at
//! `design/agent-reports/v0_2-phase-D1-help-audit-r1.md` for per-flag
//! provenance.

use super::{FlagKind, FlagSchema, PositionalArgSchema, Schema, SubcommandSchema};

// ─── inspect ─────────────────────────────────────────────────────────────

// `mk inspect [MK1_STRINGS]... [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit structured JSON instead of multi-line text.",
    secret: false,
}];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings. Use `-` to read one string per line from stdin.",
}];

// ─── encode ──────────────────────────────────────────────────────────────

// `mk encode --xpub --origin-path [--origin-fingerprint|--privacy-preserving]
//            [--policy-id-stub]... [--from-md1]... [--force-chunked]
//            [--force-long-code] [--json]`
//
// Upstream: `--origin-fingerprint` conflicts_with `--privacy-preserving`
// (bidirectional, explicit in help). Conditional fn at
// `form::conditional::mk_encode`.
const ENCODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--xpub",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "Extended public key to encode.",
        secret: false,
    },
    FlagSchema {
        name: "--origin-fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master fingerprint (8 hex chars). Conflicts with --privacy-preserving.",
        secret: false,
    },
    FlagSchema {
        name: "--origin-path",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "BIP-32 derivation path from master to the supplied xpub.",
        secret: false,
    },
    FlagSchema {
        name: "--policy-id-stub",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Policy-id stub binding the mk1 to a policy. Repeating (order-sensitive).",
        secret: false,
    },
    FlagSchema {
        name: "--from-md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Derive --policy-id-stub from the supplied md1. Repeating.",
        secret: false,
    },
    FlagSchema {
        name: "--privacy-preserving",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Omit master fingerprint from the mk1. Conflicts with --origin-fingerprint.",
        secret: false,
    },
    FlagSchema {
        name: "--force-chunked",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force chunked encoding for testing.",
        secret: false,
    },
    FlagSchema {
        name: "--force-long-code",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force long-code encoding for testing.",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
    },
];

const ENCODE_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── decode ──────────────────────────────────────────────────────────────

// `mk decode [MK1_STRINGS]... [--json]`
const DECODE_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON instead of text output.",
    secret: false,
}];

const DECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings to decode. Use `-` to read one string per line from stdin.",
}];

// ─── verify ──────────────────────────────────────────────────────────────

// `mk verify [MK1_STRINGS]... [--xpub] [--origin-fingerprint] [--origin-path]
//            [--policy-id-stub]... [--from-md1]... [--json]`
//
// All content-match flags are optional; --policy-id-stub is order-sensitive.
const VERIFY_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--xpub",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Expected extended public key.",
        secret: false,
    },
    FlagSchema {
        name: "--origin-fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Expected master fingerprint (8 hex chars).",
        secret: false,
    },
    FlagSchema {
        name: "--origin-path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Expected BIP-32 derivation path.",
        secret: false,
    },
    FlagSchema {
        name: "--policy-id-stub",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Expected policy-id stub(s). Order-sensitive.",
        secret: false,
    },
    FlagSchema {
        name: "--from-md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Derive expected --policy-id-stub from the supplied md1. Repeating.",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON instead of text output.",
        secret: false,
    },
];

const VERIFY_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings to verify. Use `-` to read one string per line from stdin.",
}];

// ─── vectors ─────────────────────────────────────────────────────────────

// `mk vectors [--pretty] [--out PATH]` — maintainer tool. `--pretty` is
// silently ignored when `--out` is set (not a clap conflicts_with).
const VECTORS_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--pretty",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Pretty-print JSON output. Silently ignored when --out is set.",
        secret: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write vectors to PATH instead of stdout.",
        secret: false,
    },
];

const VECTORS_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── SCHEMA constant ─────────────────────────────────────────────────────

const SUBCOMMANDS: &[SubcommandSchema] = &[
    SubcommandSchema {
        name: "inspect",
        human_name: "Inspect (structural commentary)",
        flags: INSPECT_FLAGS,
        positional_args: INSPECT_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "encode",
        human_name: "Encode (xpub → mk1)",
        flags: ENCODE_FLAGS,
        positional_args: ENCODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::mk_encode),
    },
    SubcommandSchema {
        name: "decode",
        human_name: "Decode (mk1 → xpub)",
        flags: DECODE_FLAGS,
        positional_args: DECODE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "verify",
        human_name: "Verify (mk1 content-match)",
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
    cli_name: "mk",
    pinned_version: "mk 0.3.0",
    subcommands: SUBCOMMANDS,
};
