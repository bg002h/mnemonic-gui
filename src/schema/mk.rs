//! Pinned schema for the `mk` CLI (mk-cli-v0.2.0).
//!
//! v0.1 scope per Section A coverage table: `mk inspect` only.

use super::{FlagKind, FlagSchema, PositionalArgSchema, Schema, SubcommandSchema};

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

const SUBCOMMANDS: &[SubcommandSchema] = &[SubcommandSchema {
    name: "inspect",
    human_name: "Inspect (structural commentary)",
    flags: INSPECT_FLAGS,
    positional_args: INSPECT_POSITIONALS,
    allows_slots: false,
    conditional: None,
}];

pub const SCHEMA: Schema = Schema {
    cli_name: "mk",
    pinned_version: "mk 0.2.0",
    subcommands: SUBCOMMANDS,
};
