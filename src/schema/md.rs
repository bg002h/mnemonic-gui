//! Pinned schema for the `md` CLI (descriptor-mnemonic-md-cli-v0.5.0).
//!
//! v0.2 scope: `inspect` (from v0.1) plus `encode`, `decode`, `verify`,
//! `bytecode`, `vectors`, `compile`, `address`. See Phase D.1 audit report
//! at `design/agent-reports/v0_2-phase-D1-help-audit-r1.md` for per-flag
//! provenance.
//!
//! NOTE: `pinned_version` is the literal `<bin> --version` output string
//! that the runtime soft-check compares against (R1 I-1 fold). The
//! `pinned-upstream.toml::[md].tag` field is the git tag for CI install
//! commands (separate concern); both are bumped in lockstep when the GUI
//! advances its md pin.

use super::{FlagKind, FlagSchema, PositionalArgSchema, Schema, SubcommandSchema};

/// Networks accepted by md CLI. Same 4 values as mnemonic.rs::NETWORKS,
/// but defined locally to avoid cross-module coupling (each CLI's schema
/// is independent per SPEC §B.3).
pub const NETWORKS: &[&str] = &["mainnet", "testnet", "signet", "regtest"];

/// Script contexts accepted by `md encode --context` and `md compile --context`.
pub const SCRIPT_CONTEXTS: &[&str] = &["tap", "segwitv0"];

// ─── inspect ─────────────────────────────────────────────────────────────

// `md inspect <STRINGS>... [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit structured JSON instead of pretty-printed text.",
    secret: false,
}];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "md1-strings",
    required: true,
    repeating: true,
    help: "One or more md1 strings to decode and pretty-print.",
}];

// ─── encode ──────────────────────────────────────────────────────────────

// `md encode [TEMPLATE] [--from-policy <EXPR>] [--context <CTX>]
//            [--unspendable-key <KEY>] [--path <PATH>] [--key <@i=XPUB>]...
//            [--fingerprint <@i=HEX>]... [--network <NETWORK>]
//            [--force-chunked] [--force-long-code] [--policy-id-fingerprint]
//            [--json]`
//
// Upstream: `[TEMPLATE]` positional XOR `--from-policy` (runtime
// pre-check; neither clap-required individually). `--context` is
// conditionally required when `--from-policy` is set. `--unspendable-key`
// is rejected when `--context` value == "segwitv0" (value-inspect, not
// presence-check). Conditional fn at `form::conditional::md_encode`.
const ENCODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--from-policy",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Compile a sub-Miniscript-Policy expression into a template (cli-compiler). \
               Mutually exclusive with the [TEMPLATE] positional.",
        secret: false,
    },
    FlagSchema {
        name: "--context",
        kind: FlagKind::Dropdown(SCRIPT_CONTEXTS),
        required: false,
        repeating: false,
        help: "Script context for --from-policy. Conditionally required when --from-policy is set.",
        secret: false,
    },
    FlagSchema {
        name: "--unspendable-key",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Tap-context only: fallback unspendable internal key for `compile_tr`. \
               Defaults to BIP-341 NUMS H-point when omitted. Rejected when --context segwitv0.",
        secret: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Override the inferred origin path with a single shared path. Accepts named \
               (bip44|48|49|84|86), hex (0xNN), or literal (m/...) forms.",
        secret: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder `@i` (e.g. @0=xpub...). Repeatable.",
        secret: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder `@i` (e.g. @0=DEADBEEF). Repeatable.",
        secret: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation (and JSON output labeling). Default mainnet.",
        secret: false,
    },
    FlagSchema {
        name: "--force-chunked",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force chunked encoding even for short policies.",
        secret: false,
    },
    FlagSchema {
        name: "--force-long-code",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force the long BCH code even when the regular code suffices.",
        secret: false,
    },
    FlagSchema {
        name: "--policy-id-fingerprint",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Print the freshly-computed PolicyId fingerprint after the phrase.",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
    },
];

const ENCODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "template",
    required: false,
    repeating: false,
    help: "BIP 388 template, e.g. `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`. \
           Mutually exclusive with --from-policy.",
}];

// ─── decode ──────────────────────────────────────────────────────────────

// `md decode <STRINGS>... [--json]`
const DECODE_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON output.",
    secret: false,
}];

const DECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "strings",
    required: true,
    repeating: true,
    help: "One or more md1 backup strings to decode into a wallet policy template.",
}];

// ─── verify ──────────────────────────────────────────────────────────────

// `md verify --template <TEMPLATE> <STRINGS>... [--key <@i=XPUB>]...
//            [--fingerprint <@i=HEX>]... [--network <NETWORK>]`
//
// `--template` is clap-required at the subcommand level.
const VERIFY_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--template",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "BIP 388 template to verify the strings against.",
        secret: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder `@i` (e.g. @0=xpub...). Repeatable.",
        secret: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder `@i` (e.g. @0=DEADBEEF). Repeatable.",
        secret: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation. Default mainnet.",
        secret: false,
    },
];

const VERIFY_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "strings",
    required: true,
    repeating: true,
    help: "One or more md1 strings to verify re-encode to the template.",
}];

// ─── bytecode ────────────────────────────────────────────────────────────

// `md bytecode <STRINGS>... [--json]` — low-level inspector.
const BYTECODE_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON output.",
    secret: false,
}];

const BYTECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "strings",
    required: true,
    repeating: true,
    help: "One or more md1 strings whose raw payload bits to dump.",
}];

// ─── vectors ─────────────────────────────────────────────────────────────

// `md vectors [--out DIR]` — maintainer tool. `--out` is a directory path.
const VECTORS_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--out",
    kind: FlagKind::Path {
        stdio_sentinel: false,
    },
    required: false,
    repeating: false,
    help: "Output directory for regenerated test-vector corpus.",
    secret: false,
}];

const VECTORS_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── compile ─────────────────────────────────────────────────────────────

// `md compile --context <CTX> <EXPR> [--unspendable-key <KEY>] [--json]`
//
// Upstream: `--context` is clap-required. `--unspendable-key` is rejected
// when `--context` value == "segwitv0" (value-inspect). Conditional fn at
// `form::conditional::md_compile`.
const COMPILE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--context",
        kind: FlagKind::Dropdown(SCRIPT_CONTEXTS),
        required: true,
        repeating: false,
        help: "Script context for compilation.",
        secret: false,
    },
    FlagSchema {
        name: "--unspendable-key",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Tap-context only: fallback unspendable internal key for `compile_tr`. \
               Defaults to BIP-341 NUMS H-point. Rejected when --context segwitv0.",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
    },
];

const COMPILE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "expr",
    required: true,
    repeating: false,
    help: "Sub-Miniscript-Policy expression to compile into a BIP 388 template.",
}];

// ─── address ─────────────────────────────────────────────────────────────

// `md address [PHRASES]... | --template <TEMPLATE>  [--key <@i=XPUB>]...
//             [--fingerprint <@i=HEX>]... [--network <NETWORK>]
//             [--chain <CHAIN>] [--change] [--index <INDEX>]
//             [--count <COUNT>] [--json]`
//
// Upstream: `[PHRASES]` positional XOR `--template`. `--key` and
// `--fingerprint` require `--template`. `--change` documented as
// "Sugar for --chain 1"; clap `conflicts_with` between them not confirmed
// from --help, so the conditional fn leaves the pair as Visible pending
// md-cli source audit. Conditional fn at `form::conditional::md_address`.
const ADDRESS_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--template",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP 388 template. Requires at least one --key. Mutually exclusive with phrases.",
        secret: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder `@i`. Repeatable. Requires --template.",
        secret: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder `@i`. Repeatable. Requires --template.",
        secret: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation and address rendering. Default mainnet.",
        secret: false,
    },
    FlagSchema {
        name: "--chain",
        kind: FlagKind::Number { min: 0, max: 65_535 },
        required: false,
        repeating: false,
        help: "Multipath alternative selector (0 = receive, 1 = change for canonical <0;1>/*).",
        secret: false,
    },
    FlagSchema {
        name: "--change",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Sugar for --chain 1.",
        secret: false,
    },
    FlagSchema {
        name: "--index",
        kind: FlagKind::Number {
            min: 0,
            max: 2_147_483_647,
        },
        required: false,
        repeating: false,
        help: "Starting index along the wildcard. Default 0.",
        secret: false,
    },
    FlagSchema {
        name: "--count",
        kind: FlagKind::Number { min: 1, max: 10_000 },
        required: false,
        repeating: false,
        help: "Number of consecutive addresses to derive starting at --index. Default 1.",
        secret: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
    },
];

const ADDRESS_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "phrases",
    required: false,
    repeating: true,
    help: "One or more md1 phrases. Mutually exclusive with --template.",
}];

// ─── SCHEMA constant ─────────────────────────────────────────────────────

const SUBCOMMANDS: &[SubcommandSchema] = &[
    SubcommandSchema {
        name: "inspect",
        human_name: "Inspect (decode + pretty-print)",
        flags: INSPECT_FLAGS,
        positional_args: INSPECT_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "encode",
        human_name: "Encode (template → md1)",
        flags: ENCODE_FLAGS,
        positional_args: ENCODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_encode),
    },
    SubcommandSchema {
        name: "decode",
        human_name: "Decode (md1 → template)",
        flags: DECODE_FLAGS,
        positional_args: DECODE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "verify",
        human_name: "Verify (md1 ↔ template)",
        flags: VERIFY_FLAGS,
        positional_args: VERIFY_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "bytecode",
        human_name: "Bytecode (raw payload bits)",
        flags: BYTECODE_FLAGS,
        positional_args: BYTECODE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "vectors",
        human_name: "Vectors (test-vector corpus)",
        flags: VECTORS_FLAGS,
        positional_args: VECTORS_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "compile",
        human_name: "Compile (policy → template)",
        flags: COMPILE_FLAGS,
        positional_args: COMPILE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_compile),
    },
    SubcommandSchema {
        name: "address",
        human_name: "Address (derive from md1)",
        flags: ADDRESS_FLAGS,
        positional_args: ADDRESS_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_address),
    },
];

pub const SCHEMA: Schema = Schema {
    cli_name: "md",
    pinned_version: "md 0.5.0",
    subcommands: SUBCOMMANDS,
};
