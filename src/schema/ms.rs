//! Pinned schema for the `ms` CLI (ms-cli-v0.1.0).
//!
//! v0.1 scope per Section A coverage table: `ms inspect` only.

use super::{FlagKind, FlagSchema, PositionalArgSchema, Schema, SubcommandSchema};

// `ms inspect [MS1] [--json]`
const INSPECT_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit JSON instead of text verdict + fields.",
    secret: false,
}];

const INSPECT_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "ms1",
    required: false,
    repeating: false,
    help: "ms1 string to inspect. Use `-` or omit to read from stdin.",
}];

const SUBCOMMANDS: &[SubcommandSchema] = &[SubcommandSchema {
    name: "inspect",
    human_name: "Inspect (verdict + fields)",
    flags: INSPECT_FLAGS,
    positional_args: INSPECT_POSITIONALS,
    allows_slots: false,
    conditional: None,
}];

pub const SCHEMA: Schema = Schema {
    cli_name: "ms",
    pinned_version: "ms 0.1.0",
    subcommands: SUBCOMMANDS,
};
