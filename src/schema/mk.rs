//! Pinned schema for the `mk` CLI (mk-cli-v0.9.0).
//!
//! Scope: `inspect` (from v0.1) plus `encode`, `decode`, `verify`, `vectors`;
//! `repair` (backfilled into the mirror at v0.6 — it shipped in v0.4 but was
//! never mirrored); and the v0.6 read-only derivation pair `address` + `derive`.
//! See Phase D.1 audit report at
//! `design/agent-reports/v0_2-phase-D1-help-audit-r1.md` for the original
//! per-flag provenance.

use super::{FlagKind, FlagSchema, NumberMax, PositionalArgSchema, Schema, SubcommandSchema};

// mstring display-grouping (mk-cli v0.9.0): `--separator` keyword values.
// SPEC §I7 — keyword dropdown (space|hyphen|comma); the toolkit reports
// `--separator` as kind `text`, the GUI narrows it. Names-only gate.
const SEPARATORS: &[&str] = &["space", "hyphen", "comma"];

// ─── inspect ─────────────────────────────────────────────────────────────

// `mk inspect [MK1_STRINGS]... [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit structured JSON instead of multi-line text.",
    secret: false,
    default_value: None,
    global: false,
}];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings. Use `-` to read one string per line from stdin.",
    secret: false,
}];

// ─── encode ──────────────────────────────────────────────────────────────

// `mk encode --xpub --origin-path [--origin-fingerprint|--privacy-preserving]
//            [--policy-id-stub]... [--from-md1]... [--chunk-set-id]
//            [--force-chunked]
//
// MIRRORED SINCE THE PIN MOVED TO mk-cli-v0.13.0. It exists in mk-cli 0.13.0 but this repo
// pins mk-cli-v0.11.0 (pinned-upstream.toml), and this schema mirrors the
// PINNED CLI, not whatever is on the developer's PATH. Adding it made
// `mk_schema_flag_names_match_help_text` pass locally against a 0.13.0 build
// and FAIL in CI with the drift inverted ("only in schema"). Re-add it in the
// same commit that moves the pin to >= 0.13.0.
//            [--force-long-code] [--json]`
//
// Upstream mutual-exclusion: `--origin-fingerprint` ↔ `--privacy-preserving`,
// enforced ONLY by a runtime guard at `mk-cli/src/cmd/encode.rs:58-62`
// (the clap `#[arg(long)]` attributes carry no `conflicts_with`). The
// doc-comments phrase it as "Mutually exclusive". Conditional fn at
// `form::conditional::mk_encode` (manual-gui batch-8 R0 catch).
const ENCODE_FLAGS: &[FlagSchema] = &[
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
        name: "--xpub",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "Extended public key to encode.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--origin-fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Master fingerprint (8 hex chars). Conflicts with --privacy-preserving.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--origin-path",
        kind: FlagKind::Text,
        required: true,
        repeating: false,
        help: "BIP-32 derivation path from master to the supplied xpub.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--policy-id-stub",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Policy-id stub binding the mk1 to a policy. Repeating (order-sensitive).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Derive --policy-id-stub from the supplied md1. Repeating.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--privacy-preserving",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Omit master fingerprint from the mk1. Conflicts with --origin-fingerprint.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--chunk-set-id",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        // Arrived in mk-cli 0.13.0. This schema mirrors the PINNED CLI, so it
        // could not be carried until pinned-upstream.toml moved off v0.11.0 --
        // adding it early made the mirror test pass against a local 0.13.0
        // build and FAIL in CI with the drift inverted ("only in schema").
        help: "Pin the 20-bit chunk_set_id (hex, 0x prefix optional) instead of \
               deriving it from the payload. Chunked output only. For vector \
               regeneration and conformance fixtures — the derived default is \
               already deterministic, so ordinary encoding never needs this.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--force-chunked",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force chunked encoding for testing.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--force-long-code",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Force long-code encoding for testing.",
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

// `mk decode [MK1_STRINGS]... [--json]`
const DECODE_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON instead of text output.",
    secret: false,
    default_value: None,
    global: false,
}];

const DECODE_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings to decode. Use `-` to read one string per line from stdin.",
    secret: false,
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
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--origin-fingerprint",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Expected master fingerprint (8 hex chars).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--origin-path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Expected BIP-32 derivation path.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--policy-id-stub",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Expected policy-id stub(s). Order-sensitive.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--from-md1",
        kind: FlagKind::Text,
        required: false,
        repeating: true,
        help: "Derive expected --policy-id-stub from the supplied md1. Repeating.",
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
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings to verify. Use `-` to read one string per line from stdin.",
    secret: false,
}];

// ─── vectors ─────────────────────────────────────────────────────────────

// `mk vectors [--pretty] [--out PATH]` — maintainer tool. `--pretty` is
// honored even when `--out` is set (each per-fixture file is pretty-printed;
// not a clap conflicts_with).
const VECTORS_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--pretty",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Pretty-print JSON output. Also honored when --out is set (each per-fixture file is pretty-printed).",
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
        help: "Write vectors to PATH instead of stdout.",
        secret: false,
        default_value: None,
        global: false,
    },
];

const VECTORS_POSITIONALS: &[PositionalArgSchema] = &[];

// ─── repair (v0.4; backfilled into the mirror at v0.6) ───────────────────

// `mk repair [MK1_STRINGS]... [--json]`
const REPAIR_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON instead of text output.",
    secret: false,
    default_value: None,
    global: false,
}];

const REPAIR_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings to repair via BCH error correction. Use `-` for stdin.",
    secret: false,
}];

// ─── address (v0.6) ──────────────────────────────────────────────────────

// `mk address [MK1_STRINGS]... [--address-type] [--count] [--range]
//             [--chain] [--network] [--json]`
const ADDRESS_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--address-type",
        kind: FlagKind::Dropdown(&["p2pkh", "p2sh-p2wpkh", "p2wpkh", "p2tr"]),
        required: false,
        repeating: false,
        help: "Address type; defaults to the account-depth origin-path heuristic.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--count",
        kind: FlagKind::Text,
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
        kind: FlagKind::Dropdown(&["mainnet", "testnet", "signet", "regtest"]),
        required: false,
        repeating: false,
        help: "Network override; must agree with the xpub's network kind.",
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

const ADDRESS_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings whose xpub controls the addresses. Use `-` for stdin.",
    secret: false,
}];

// ─── derive (v0.6) ───────────────────────────────────────────────────────

// `mk derive [MK1_STRINGS]... (--path | --index) [--json]`
const DERIVE_FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--path",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Relative derivation path, unhardened only (e.g. m/0/5).",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--index",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Single external-chain index: sugar for --path m/0/<N>.",
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
    name: "mk1-strings",
    required: false,
    repeating: true,
    help: "One or more mk1 strings to derive a child xpub from. Use `-` for stdin.",
    secret: false,
}];

// ─── gen-man ─────────────────────────────────────────────────────────────
//
// man-pages cycle (mk-cli v0.11.0): emit roff man pages for the whole CLI
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
        human_name: "Encode (xpub -> mk1)",
        flags: ENCODE_FLAGS,
        positional_args: ENCODE_POSITIONALS,
        allows_slots: false,
        conditional: Some(crate::form::conditional::mk_encode),
    },
    SubcommandSchema {
        name: "decode",
        human_name: "Decode (mk1 -> xpub)",
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
    SubcommandSchema {
        name: "repair",
        human_name: "Repair (BCH error correction)",
        flags: REPAIR_FLAGS,
        positional_args: REPAIR_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "address",
        human_name: "Address (xpub -> receive/change addresses)",
        flags: ADDRESS_FLAGS,
        positional_args: ADDRESS_POSITIONALS,
        allows_slots: false,
        conditional: None,
    },
    SubcommandSchema {
        name: "derive",
        human_name: "Derive (xpub -> child xpub)",
        flags: DERIVE_FLAGS,
        positional_args: DERIVE_POSITIONALS,
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
];

pub const SCHEMA: Schema = Schema {
    cli_name: "mk",
    pinned_version: "mk 0.11.0",
    subcommands: SUBCOMMANDS,
};
