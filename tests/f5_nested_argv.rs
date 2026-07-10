//! F5 — nested-argv split (constellation-eval F5; SPEC `SPEC_f5_f6_gui_recovery_wiring.md`).
//!
//! The toolkit's `gui-schema` FLATTENS each nested `#[command(subcommand)]`
//! subcommand as `format!("{parent}-{child}")`, but clap parses the NESTED
//! grammar — `mnemonic seed-xor combine …`, NOT `mnemonic seed-xor-combine …`
//! (the flattened form is clap exit 64 "unrecognized subcommand"). The GUI's
//! single run/copy argv assembler must therefore split the flattened name back
//! into the `[parent, child]` token pair for the 5 nested parents while passing
//! flat subcommands (`import-wallet`/`export-wallet`/…) through unsplit.
//!
//! These cells assert the assembler emits the nested PAIR for all 12 nested
//! subcommands (`argv[1]` a bare parent, `argv[2]` the child, never the
//! hyphen-flattened token) and that flat subcommands pass through unsplit. A
//! copy-command cell confirms the human-facing "Copy command" string starts
//! with the nested invocation. `assemble_argv` reads ONLY `FormState`, so an
//! empty form suffices to exercise the subcommand-token emission.

use mnemonic_gui::form::invocation::{assemble_argv, render_copy_command, ShellFlavor};
use mnemonic_gui::schema::{self, FormState, SubcommandSchema};

fn subcommand(name: &str) -> &'static SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {name} not in schema"))
}

/// The 12 nested subcommands and their expected `[parent, child]` argv pair.
const NESTED: &[(&str, [&str; 2])] = &[
    ("seed-xor-split", ["seed-xor", "split"]),
    ("seed-xor-combine", ["seed-xor", "combine"]),
    ("seedqr-encode", ["seedqr", "encode"]),
    ("seedqr-decode", ["seedqr", "decode"]),
    ("slip39-split", ["slip39", "split"]),
    ("slip39-combine", ["slip39", "combine"]),
    ("ms-shares-split", ["ms-shares", "split"]),
    ("ms-shares-combine", ["ms-shares", "combine"]),
    ("xpub-search-path-of-xpub", ["xpub-search", "path-of-xpub"]),
    (
        "xpub-search-account-of-descriptor",
        ["xpub-search", "account-of-descriptor"],
    ),
    ("xpub-search-address-of-xpub", ["xpub-search", "address-of-xpub"]),
    (
        "xpub-search-passphrase-of-xpub",
        ["xpub-search", "passphrase-of-xpub"],
    ),
];

#[test]
fn all_nested_subcommands_emit_the_parent_child_pair() {
    for (flat, [parent, child]) in NESTED {
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand(flat),
            &FormState::default(),
        );
        assert_eq!(argv[0], "mnemonic", "argv[0] must be the cli name; got {argv:?}");
        assert_eq!(
            &argv[1], parent,
            "argv[1] must be the BARE parent token `{parent}`; got {argv:?}"
        );
        assert_eq!(
            &argv[2], child,
            "argv[2] must be the child token `{child}`; got {argv:?}"
        );
        assert!(
            argv.windows(2).any(|w| w == [*parent, *child]),
            "argv must carry the nested pair [{parent}, {child}]; got {argv:?}"
        );
        // The flattened token must NEVER appear (it is clap exit 64).
        assert!(
            !argv.iter().any(|t| t == flat),
            "the flattened token `{flat}` must NOT appear in argv; got {argv:?}"
        );
    }
}

#[test]
fn flat_subcommands_pass_through_unsplit() {
    // These begin with none of the complete-token nested-parent prefixes and
    // must emit as a SINGLE argv[1] token (the anti-corruption pin).
    for flat in ["import-wallet", "export-wallet", "verify-bundle", "decode-address"] {
        let argv = assemble_argv(
            &schema::mnemonic::SCHEMA,
            subcommand(flat),
            &FormState::default(),
        );
        assert_eq!(
            argv[1], flat,
            "flat subcommand `{flat}` must be argv[1] unsplit; got {argv:?}"
        );
    }
}

#[test]
fn copy_command_renders_the_nested_invocation() {
    // The human-facing "Copy command" string uses the same assembler; it must
    // read `mnemonic seed-xor combine …`, not the corrupt flattened form.
    let argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("seed-xor-combine"),
        &FormState::default(),
    );
    let posix = render_copy_command(&argv, ShellFlavor::Posix);
    assert!(
        posix.starts_with("mnemonic seed-xor combine"),
        "posix copy-command must start with the nested invocation; got {posix:?}"
    );
    assert!(
        !posix.contains("seed-xor-combine"),
        "posix copy-command must not contain the flattened token; got {posix:?}"
    );
}
