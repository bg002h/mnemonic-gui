//! Pinned schema for the `md` CLI (descriptor-mnemonic-md-cli-v0.4.3).
//!
//! v0.1 scope per Section A coverage table: `md inspect` only.
//!
//! NOTE: `pinned_version` is the literal `<bin> --version` output string
//! that the runtime soft-check compares against (R1 I-1 fold). The
//! `pinned-upstream.toml::[md].tag` field is the git tag for CI install
//! commands (separate concern); both are bumped in lockstep when the GUI
//! advances its md pin.

use super::{FlagKind, FlagSchema, PositionalArgSchema, Schema, SubcommandSchema};

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

const SUBCOMMANDS: &[SubcommandSchema] = &[SubcommandSchema {
    name: "inspect",
    human_name: "Inspect (decode + pretty-print)",
    flags: INSPECT_FLAGS,
    positional_args: INSPECT_POSITIONALS,
    allows_slots: false,
    conditional: None,
}];

pub const SCHEMA: Schema = Schema {
    cli_name: "md",
    pinned_version: "md 0.4.3",
    subcommands: SUBCOMMANDS,
};
