//! Pinned schema for the `md` CLI (descriptor-mnemonic-md-cli-v0.7.0).
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

use super::{FlagKind, FlagSchema, NumberMax, PositionalArgSchema, Schema, SubcommandSchema};

/// Networks accepted by md CLI. Same 4 values as mnemonic.rs::NETWORKS,
/// but defined locally to avoid cross-module coupling (each CLI's schema
/// is independent per SPEC §B.3).
pub const NETWORKS: &[&str] = &["mainnet", "testnet", "signet", "regtest"];

// mstring display-grouping (md-cli v0.7.0): `--separator` keyword values.
// SPEC §I7 — keyword dropdown (space|hyphen|comma); the toolkit reports
// `--separator` as kind `text`, the GUI narrows it. Names-only gate.
// F-679 (md-cli v0.20.3): `hyphen` and `comma` are RETIRED — md refuses them
// ("--separator is whitespace-only across the constellation (SPEC §6c)").
// The drift gate cannot see it (md reports `--separator` as `text`), so this
// was measured against the release binary.
const SEPARATORS: &[&str] = &["space"];

/// Script contexts accepted by `md encode --context` and `md compile --context`.
pub const SCRIPT_CONTEXTS: &[&str] = &["tap", "segwitv0"];

// ─── inspect ─────────────────────────────────────────────────────────────

// `md inspect <STRINGS>... [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit structured JSON instead of pretty-printed text.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read md1 strings from FILE, one per line, instead of the \
               positional.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "md1-strings",
    required: false,
    repeating: true,
    help: "One or more md1 strings to decode and pretty-print.",
    secret: false,
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
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(65535),
        },
        required: false,
        repeating: false,
        help: "Group the ENGRAVING CARD on stderr into N-character groups \
               (default 5; 0 = unbroken). stdout is always the unbroken md1 \
               string. Cosmetic — intake strips separators.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Engraving-card separator: `space` only (hyphen and comma were \
               retired constellation-wide). Cosmetic — non-load-bearing.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--from-policy",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Compile a sub-Miniscript-Policy expression into a template (cli-compiler). \
               Mutually exclusive with the [TEMPLATE] positional.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--context",
        kind: FlagKind::Dropdown(SCRIPT_CONTEXTS),
        required: false,
        repeating: false,
        help: "Script context for --from-policy. Conditionally required when --from-policy is set.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--unspendable-key",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Tap-context only: fallback unspendable internal key for `compile_tr`. \
               Defaults to BIP-341 NUMS H-point when omitted. Rejected when --context segwitv0.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Override the inferred origin path with a single shared path. Accepts named \
               (bip44|48|49|84|86), hex (0xNN), or literal (m/...) forms.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder `@i` (e.g. @0=xpub...). Repeatable.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder `@i` (e.g. @0=DEADBEEF). Repeatable.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation (and JSON output labeling). Default mainnet.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--force-chunked",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force chunked encoding even for short policies.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--force-long-code",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force the long BCH code even when the regular code suffices.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--policy-id-fingerprint",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Print the freshly-computed PolicyId fingerprint after the phrase.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the BIP 388 template from FILE instead of the positional. \
               Surrounding whitespace is trimmed.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write the md1 artifact to FILE (created 0600; OVERWRITES) instead \
               of stdout. The stderr engraving card is unaffected.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--experimental",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Admit a spend path that requires NO signature (e.g. a hashlock + \
               timelock recovery tier). Whoever learns a keyless path's preimage \
               can spend it alone. Prints a warning on every use.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const ENCODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "template",
    required: false,
    repeating: false,
    help: "BIP 388 template, e.g. `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`. \
           Mutually exclusive with --from-policy.",
    secret: false,
}];

// ─── decode ──────────────────────────────────────────────────────────────

// `md decode <STRINGS>... [--json]`
const DECODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read md1 strings from FILE, one per line, instead of the \
               positional.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const DECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "strings",
    required: false,
    repeating: true,
    help: "One or more md1 backup strings to decode into a wallet policy template.",
    secret: false,
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
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder `@i` (e.g. @0=xpub...). Repeatable.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder `@i` (e.g. @0=DEADBEEF). Repeatable.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation. Default mainnet.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read md1 strings from FILE, one per line, instead of the \
               positional.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Override the inferred origin path with a single shared path \
               (named bip44|48|49|84|86, hex 0xNN, or literal m/...). Mirrors \
               `md encode --path`.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--experimental",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Accept a template with a spend path that requires no signature, \
               mirroring `md encode --experimental` — without it a card authored \
               that way cannot be read here.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const VERIFY_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "strings",
    required: false,
    repeating: true,
    help: "One or more md1 strings to verify re-encode to the template.",
    secret: false,
}];

// ─── bytecode ────────────────────────────────────────────────────────────

// `md bytecode <STRINGS>... [--json]` — low-level inspector.
const BYTECODE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read md1 strings from FILE, one per line, instead of the \
               positional.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const BYTECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "strings",
    required: false,
    repeating: true,
    help: "One or more md1 strings whose raw payload bits to dump.",
    secret: false,
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
    default_value: None,
    global: false,
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
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--unspendable-key",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Tap-context only: fallback unspendable internal key for `compile_tr`. \
               Defaults to BIP-341 NUMS H-point. Rejected when --context segwitv0.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const COMPILE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "expr",
    required: true,
    repeating: false,
    help: "Sub-Miniscript-Policy expression to compile into a BIP 388 template.",
    secret: false,
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
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder `@i`. Repeatable. Requires --template.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder `@i`. Repeatable. Requires --template.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation and address rendering. Default mainnet.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--chain",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(65_535) },
        required: false,
        repeating: false,
        help: "Multipath alternative selector (0 = receive, 1 = change for canonical <0;1>/*).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--change",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Sugar for --chain 1.",
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
        required: false,
        repeating: false,
        help: "Starting index along the wildcard. Default 0.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--count",
        kind: FlagKind::Number { min: 1, max: NumberMax::Static(10_000) },
        required: false,
        repeating: false,
        help: "Number of consecutive addresses to derive starting at --index. Default 1.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Shared origin path applied PER SLOT to any @i the template gave \
               no inline origin (an inline origin always wins). Named, hex or \
               literal (m/...) forms.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-mk1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "mk1 key-card string, repeatable. Supplied TOGETHER WITH the keyless \
               md1 phrases of a policy card: each card is seated in the slot whose \
               declared origin it satisfies. Watch-only (xpub) material.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-mk1-file",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read mk1 key-card strings from FILE, one per line (blank lines and \
               `#` comments skipped). Combines with --from-mk1.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--seat",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Assert the seating of one slot: `@i=<chunk-set-id>` (append \
               `#<k>` to pick one of several collided cards). Repeatable.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--experimental",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Accept a template with a spend path that requires no signature, \
               mirroring `md encode --experimental` — without it a card authored \
               that way cannot be read here.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const ADDRESS_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "phrases",
    required: false,
    repeating: true,
    help: "One or more md1 phrases. Mutually exclusive with --template.",
    secret: false,
}];

// ─── repair ────────────────────────────────────────────────────────────────

// `md repair [--json] <MD1_STRINGS>...` (md-cli v0.6.2). BCH error-correction
// for chunked-form md1 strings. v0.22.0: closes the schema/binary subcommand-
// coverage gap (9 binary subcommands vs 8 schema; schema_mirror iterates only
// schema-declared subcommands, so a binary-only subcommand was invisible).
const REPAIR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit a single JSON envelope on stdout instead of the text-form report.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read md1 strings from FILE, one per line, instead of the \
               positional.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const REPAIR_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "md1-strings",
    required: false,
    repeating: true,
    help: "One or more md1 strings to repair (BCH error-correction). `-` reads one per line \
           from stdin. Chunked-form md1 only.",
    secret: false,
}];

// ─── gen-man ─────────────────────────────────────────────────────────────
//
// man-pages cycle (md-cli v0.11.0): emit roff man pages for the whole CLI
// tree into `--out <DIR>`. Single required directory-path flag; no
// positionals, no global flags.
const GEN_MAN_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--out",
    kind: FlagKind::Path { stdio_sentinel: false },
    required: true,
    repeating: false,
    help: "Directory to write the `*.1` man pages into (created if absent). \
           One page per (nested) subcommand, hyphen-joined parent→child.",
    secret: false,
    default_value: None,
    global: false,
}];

const GEN_MAN_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── compose (DESIGN secret channels Part B, B1) ────────────────────────────

// `md compose` (md-cli v0.20.3): the Wallet Policy composer's CLI. No secret
// input. `--wrapper` is clap-required; `--path` (repeating, ORDER MEANINGFUL)
// XOR `--preset`; `--unspendable` only with `--wrapper tr`. Conditional fn at
// `form::conditional::md_compose`.
pub const COMPOSE_WRAPPERS: &[&str] = &["tr", "wsh", "sh-wsh", "sh"];
/// `--unspendable`: the leading `""` is the GUI-side "(none)" sentinel (never
/// emitted) — omitting the flag is md's own default.
pub const COMPOSE_UNSPENDABLE: &[&str] = &["", "nums", "liana"];

const COMPOSE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--wrapper",
        kind: FlagKind::Dropdown(COMPOSE_WRAPPERS),
        required: true,
        repeating: false,
        help: "Script wrapper: tr | wsh | sh-wsh | sh. Required.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "One spend path, in listed order (order is meaningful): \
               `<k>of<n>[,older=N|older=Nu|after=H|after=Tt][,<hash>=HEX][,unsorted]` \
               or `keyless,<hash>=HEX[,older=..|after=..]`. Repeatable. Mutually \
               exclusive with --preset.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--preset",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "One of the six named archetypes: `<name>[,<k>of<n>]*[,<param>=<value>]*`, \
               e.g. `kofn-recovery,2of3,older=26280`. Mutually exclusive with --path.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--experimental",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Admit key-less paths and unsorted-where-sorted-was-legal, with a warning.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON: the templates, the slot map, the taproot internal-key path \
               and the EXPERIMENTAL marks.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--unspendable",
        kind: FlagKind::Dropdown(COMPOSE_UNSPENDABLE),
        required: false,
        repeating: false,
        help: "Which unspendable taproot internal key to use when no spend path \
               supplies a real one: `nums` (the BIP-341 H-point, md's default) or \
               `liana`. Only with --wrapper tr.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--md-only",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Compose even when every wallet coordinator md knows refuses the \
               policy. md can still rebuild such a wallet from the card; md cannot sign.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const COMPOSE_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── shape-key (B2) ─────────────────────────────────────────────────────────

// `md shape-key` (md-cli v0.20.3): `[PHRASES]...` XOR `--descriptor` (clap
// "cannot be used with"). Conditional fn at `form::conditional::md_shape_key`.
const SHAPE_KEY_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--descriptor",
    kind: FlagKind::Text,
    required: false,
    repeating: false,
    help: "A multipath (`<0;1>`) BIP-380 descriptor instead of a card: a wallet's \
           own export or `md descriptor`'s output. Public keys only (a pasted xprv \
           is masked and never persisted). Mutually exclusive with phrases.",
    secret: false,
    default_value: None,
    global: false,
}];

const SHAPE_KEY_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "phrases",
    required: false,
    repeating: true,
    help: "One or more md1 strings of one card. Mutually exclusive with --descriptor.",
    secret: false,
}];

// ─── descriptor (B3) ────────────────────────────────────────────────────────

// `md descriptor` (md-cli v0.20.3): three input modes — A: md1 phrases; B:
// --template + ≥1 --key (+ --fingerprint); C: --from-mk1/--from-mk1-file with
// keyless md1 phrases (+ --seat). `--emit md1` only with mode C; --out,
// --group-size, --separator are meaningful only with --emit md1; --chain XOR
// --change. Conditional fn at `form::conditional::md_descriptor`.
/// `--emit`: `""` is the "(none)" sentinel (the concrete descriptor, md's
/// default output).
pub const DESCRIPTOR_EMIT: &[&str] = &["", "md1"];

const DESCRIPTOR_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--template",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "BIP 388 template. Requires at least one --key. Mutually exclusive with phrases.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--key",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Concrete xpub for placeholder @i (`@i=XPUB`), or the origin-notated \
               `@i=[fingerprint/path]XPUB` form. Repeatable. Requires --template. Public \
               keys only (a pasted xprv is masked and never persisted).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Master-key fingerprint for placeholder @i (`@i=HEX`). Repeatable. Requires --template.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Shared origin path, applied PER SLOT to whichever @i the template gave \
               no inline origin (an inline origin always wins). Named, hex or literal forms.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-mk1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "mk1 key-card string, repeatable. Supplied TOGETHER WITH the keyless md1 \
               phrases of a policy card: each card is seated in the slot whose declared \
               origin it satisfies. Watch-only (xpub) material.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-mk1-file",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read mk1 key-card strings from FILE, one per line (blank lines and `#` \
               comments skipped). Combines with --from-mk1.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--seat",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Assert the seating of one slot: `@i=<chunk-set-id>` (append `#<k>` to \
               pick one of several collided cards). Repeatable. With --from-mk1.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network for xpub validation. Default mainnet.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
    FlagSchema {
        name: "--chain",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(1) },
        required: false,
        repeating: false,
        help: "Collapse the multipath group to ONE chain (0 = receive, 1 = change). \
               Omit for the multipath <0;1> form. Mutually exclusive with --change.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--change",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Sugar for --chain 1. Mutually exclusive with --chain.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--emit",
        kind: FlagKind::Dropdown(DESCRIPTOR_EMIT),
        required: false,
        repeating: false,
        help: "`md1`: put the KEYED md1 card on stdout instead of the concrete \
               descriptor, minted from the seating result. Needs --from-mk1 / \
               --from-mk1-file input.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--out",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Write the KEYED md1 artifact to FILE (created 0600) instead of stdout. \
               Only with --emit md1.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--group-size",
        kind: FlagKind::Number {
            min: 0,
            max: NumberMax::Static(255),
        },
        required: false,
        repeating: false,
        help: "Insert a separator every N characters in the engraving card on stderr \
               (0 = unbroken). Default 5. Only with --emit md1.",
        secret: false,
        default_value: Some("5"),
        global: false,
    },
    FlagSchema {
        name: "--separator",
        kind: FlagKind::Dropdown(SEPARATORS),
        required: false,
        repeating: false,
        help: "Separator for the engraving card. Default space. Only with --emit md1.",
        secret: false,
        default_value: Some("space"),
        global: false,
    },
    FlagSchema {
        name: "--json",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Emit JSON output.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--verify-against",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "SPEND-EQUAL comparison target: an md1 string or a FILE holding one or \
               more. Exit 0 = spend-equal, 5 = NOT spend-equal. Any input mode.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--experimental",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Accept a template with a spend path that requires no signature, \
               mirroring `md encode --experimental`.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const DESCRIPTOR_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "phrases",
    required: false,
    repeating: true,
    help: "One or more md1 phrases (input mode A; with --from-mk1, the KEYLESS \
           policy card's phrases). Mutually exclusive with --template.",
    secret: false,
}];

// ─── decompose (B4) ─────────────────────────────────────────────────────────

// `md decompose` (md-cli v0.20.3): `<DESCRIPTORS>` (exactly one; the CLI
// refuses two with the receive/change-pair guidance) XOR `--in`. Conditional
// fn at `form::conditional::md_decompose`.
pub const DECOMPOSE_EMIT: &[&str] = &["all", "template", "keys", "fingerprints", "descriptor", "commands"];

const DECOMPOSE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--in",
        kind: FlagKind::Path {
            stdio_sentinel: false,
        },
        required: false,
        repeating: false,
        help: "Read the descriptor from FILE instead of argv. Blank lines and `#` \
               comments are skipped. Mutually exclusive with the positional.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--emit",
        kind: FlagKind::Dropdown(DECOMPOSE_EMIT),
        required: false,
        repeating: false,
        help: "Which artifact to print: all (template, key lines and fingerprint \
               flags), template, keys, fingerprints, descriptor, or commands. Default all.",
        secret: false,
        default_value: Some("all"),
        global: false,
    },
    FlagSchema {
        name: "--network",
        kind: FlagKind::Dropdown(NETWORKS),
        required: false,
        repeating: false,
        help: "Network the descriptor's extended keys must belong to. Default mainnet.",
        secret: false,
        default_value: Some("mainnet"),
        global: false,
    },
];

const DECOMPOSE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "descriptors",
    required: false,
    repeating: false,
    help: "The concrete output descriptor — exactly one. Public keys only (a pasted \
           xprv is masked and never persisted). Mutually exclusive with --in.",
    secret: false,
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
        human_name: "Encode (template -> md1)",
        flags: ENCODE_FLAGS,
        positional_args: ENCODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_encode),
    },
    SubcommandSchema {
        name: "decode",
        human_name: "Decode (md1 -> template)",
        flags: DECODE_FLAGS,
        positional_args: DECODE_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "verify",
        human_name: "Verify (md1 <-> template)",
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
        human_name: "Compile (policy -> template)",
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
    SubcommandSchema {
        name: "repair",
        human_name: "Repair (BCH error-correction)",
        flags: REPAIR_FLAGS,
        positional_args: REPAIR_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "gen-man",
        human_name: "Gen Man (emit roff man pages for the whole CLI tree)",
        flags: GEN_MAN_FLAGS,
        positional_args: GEN_MAN_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    // DESIGN secret channels Part B (B1–B4): the four md verbs surfaced.
    SubcommandSchema {
        name: "compose",
        human_name: "Compose (spend paths -> wallet policy)",
        flags: COMPOSE_FLAGS,
        positional_args: COMPOSE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_compose),
    },
    SubcommandSchema {
        name: "shape-key",
        human_name: "Shape Key (card or descriptor -> shape key)",
        flags: SHAPE_KEY_FLAGS,
        positional_args: SHAPE_KEY_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_shape_key),
    },
    SubcommandSchema {
        name: "descriptor",
        human_name: "Descriptor (md1 / template / key cards -> descriptor)",
        flags: DESCRIPTOR_FLAGS,
        positional_args: DESCRIPTOR_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_descriptor),
    },
    SubcommandSchema {
        name: "decompose",
        human_name: "Decompose (descriptor -> template, keys, fingerprints)",
        flags: DECOMPOSE_FLAGS,
        positional_args: DECOMPOSE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::md_decompose),
    },
];

pub const SCHEMA: Schema = Schema {
    cli_name: "md",
    pinned_version: "md 0.20.3",
    subcommands: SUBCOMMANDS,
};
