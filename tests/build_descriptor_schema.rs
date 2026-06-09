//! v0.29.0 — `build-descriptor` surfaced in the `mnemonic` SubcommandSchema
//! mirror (toolkit pin v0.47.3 → v0.50.0; descriptor-builder engine Release A).
//!
//! RED-first characterization (these `.expect()` on the missing subcommand
//! before Part-B lands):
//!   1. presence + the exact v0.50.0 `gui-schema` flag-NAME set;
//!   2. `--spec` is `Path { stdio_sentinel: true }` — LOAD-BEARING: the toolkit
//!      `--spec` is a file path (or `-`/stdin), never inline JSON, so a `Text`
//!      kind would emit raw JSON → the toolkit treats it as a path → ENOENT →
//!      broken form. Pinning Path guards against a regression to Text.
//!   3. argv assembly is panic-free for build-descriptor's flag kinds (the
//!      R0 M1 assembly-layer smoke; widget render is sound by construction —
//!      build-descriptor uses only Path/Boolean/Dropdown, all rendered on
//!      existing live forms via the FlagKind-dispatched `render_flag`).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::schema::{self, FlagKind, FormState};

fn build_descriptor() -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "build-descriptor")
        .expect("build-descriptor must appear in mnemonic SUBCOMMANDS")
}

#[test]
fn build_descriptor_flag_set_matches_v0_50_0_surface() {
    let sub = build_descriptor();
    let mut got: Vec<&str> = sub.flags.iter().map(|f| f.name).collect();
    got.sort_unstable();
    let mut want = [
        "--format",
        "--json",
        "--network",
        "--no-auto-repair",
        "--spec",
        "--spec-schema",
    ];
    want.sort_unstable();
    assert_eq!(
        got, want,
        "build-descriptor flag-NAME set must equal the v0.50.0 gui-schema surface"
    );
    assert!(sub.positional_args.is_empty(), "no positionals");
    assert!(!sub.allows_slots, "no --slot grammar");
    assert!(sub.conditional.is_none(), "no conditional (no clap conflicts)");
}

#[test]
fn build_descriptor_spec_is_path_with_stdio_sentinel() {
    let sub = build_descriptor();
    let spec = sub
        .flags
        .iter()
        .find(|f| f.name == "--spec")
        .expect("--spec present");
    assert!(
        matches!(spec.kind, FlagKind::Path { stdio_sentinel: true }),
        "--spec must be Path{{stdio_sentinel:true}} so the form emits a valid \
         path/`-`, not raw JSON"
    );
    assert!(!spec.required, "--spec is optional (stdin when omitted)");
    assert!(!spec.secret, "--spec is not a secret-bearing flag (watch-only)");
}

#[test]
fn build_descriptor_argv_assembles_without_panic() {
    // Default (empty) form state: assembly must not panic on build-descriptor's
    // flag kinds, the subcommand token is emitted, and an unset Path `--spec`
    // is omitted (empty Path = "not present").
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, build_descriptor(), &FormState::default());
    assert!(
        argv.contains(&"build-descriptor".to_string()),
        "argv must carry the subcommand token: {argv:?}"
    );
    assert!(
        !argv.contains(&"--spec".to_string()),
        "an unset Path --spec must be omitted: {argv:?}"
    );
}
