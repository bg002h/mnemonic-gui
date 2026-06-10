//! v0.29.0 — `build-descriptor` surfaced in the `mnemonic` SubcommandSchema
//! mirror (toolkit pin v0.47.3 → v0.50.0; descriptor-builder engine Release A).
//! v0.30.0 — archetype presets + `--allow` (toolkit pin v0.50.0 → v0.52.0;
//! descriptor-builder Release B): the flag set grows 6 → 18, the
//! archetype↔spec mutex lands (`conditional::build_descriptor`), and the
//! two new Dropdown value lists are pinned byte-equal to the v0.52.0
//! binary's possible-values (SPEC §2 / R0-r1 I1: `schema_mirror` compares
//! flag NAMES only — kinds, `repeating`, and Dropdown values are NOT
//! gate-checked, so a const typo would be silent until runtime clap
//! rejection without these pins).
//!
//! Coverage:
//!   1. presence + the exact v0.52.0 `gui-schema` flag-NAME set (18) +
//!      `conditional.is_some()`;
//!   2. `--spec` is `Path { stdio_sentinel: true }` — LOAD-BEARING: the toolkit
//!      `--spec` is a file path (or `-`/stdin), never inline JSON, so a `Text`
//!      kind would emit raw JSON → the toolkit treats it as a path → ENOENT →
//!      broken form. Pinning Path guards against a regression to Text.
//!   3. argv assembly is panic-free for build-descriptor's flag kinds and the
//!      default (empty) form emits no `--spec`;
//!   4. `--archetype` value-list pin: `ARCHETYPES` = the `""` UNSET sentinel
//!      (R0-r1 C1) + the 5 binary archetypes in `CliArchetype` order, with
//!      `default_value: Some("")` so the seeded form emits nothing;
//!   5. `--allow` value-list pin: byte-equal to the binary's 5 sanity-rule
//!      names; `repeating: true` (the `--to` repeating-Dropdown precedent);
//!   6. `--key`/`--recovery-key` are repeating Text (row order = quorum
//!      order).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::schema::{self, FlagKind, FormState};

fn build_descriptor() -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "build-descriptor")
        .expect("build-descriptor must appear in mnemonic SUBCOMMANDS")
}

fn flag(name: &str) -> &'static schema::FlagSchema {
    build_descriptor()
        .flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("build-descriptor flag {} missing", name))
}

#[test]
fn build_descriptor_flag_set_matches_v0_52_0_surface() {
    let sub = build_descriptor();
    let mut got: Vec<&str> = sub.flags.iter().map(|f| f.name).collect();
    got.sort_unstable();
    let mut want = [
        "--after",
        "--allow",
        "--archetype",
        "--emit-spec",
        "--final-key",
        "--format",
        "--hash",
        "--json",
        "--key",
        "--network",
        "--no-auto-repair",
        "--older",
        "--recovery-key",
        "--recovery-older",
        "--recovery-threshold",
        "--spec",
        "--spec-schema",
        "--threshold",
    ];
    want.sort_unstable();
    assert_eq!(
        got, want,
        "build-descriptor flag-NAME set must equal the v0.52.0 gui-schema surface"
    );
    assert!(sub.positional_args.is_empty(), "no positionals");
    assert!(!sub.allows_slots, "no --slot grammar");
    assert!(
        sub.conditional.is_some(),
        "v0.30.0: the archetype↔spec clap mutex is projected \
         (conditional::build_descriptor)"
    );
}

#[test]
fn build_descriptor_spec_is_path_with_stdio_sentinel() {
    let spec = flag("--spec");
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

// ─── v0.30.0 value-list pins (R0-r1 I1) ─────────────────────────────────
//
// `schema_mirror` is flag-NAME set-equality only; Dropdown values are NOT
// gate-checked. These cells pin ARCHETYPES / ALLOW_RULES byte-equal to the
// v0.52.0 binary's possible-values lists (probed 2026-06-09; SPEC §2).

#[test]
fn build_descriptor_archetype_values_pin_v0_52_0() {
    let archetype = flag("--archetype");
    let FlagKind::Dropdown(opts) = &archetype.kind else {
        panic!("--archetype must be a Dropdown");
    };
    // The leading "" is the GUI-side UNSET sentinel (R0-r1 C1) — NOT part
    // of the binary's value list (an empty Dropdown is never emitted).
    assert_eq!(
        opts.first(),
        Some(&""),
        "ARCHETYPES[0] must be the \"\" UNSET sentinel"
    );
    assert_eq!(
        opts[1..],
        [
            "decaying-multisig",
            "hashlock-gated",
            "kofn-recovery",
            "simple-timelocked-inheritance",
            "tiered-recovery",
        ],
        "ARCHETYPES minus the sentinel must byte-equal the v0.52.0 binary's \
         --archetype possible-values (CliArchetype order)"
    );
    assert_eq!(
        archetype.default_value,
        Some(""),
        "--archetype default_value must be the \"\" sentinel so the seeded \
         default form emits no --archetype and the spec mutex stays open"
    );
    assert!(!archetype.repeating, "--archetype is single-valued");
}

#[test]
fn build_descriptor_allow_values_pin_v0_52_0() {
    let allow = flag("--allow");
    let FlagKind::Dropdown(opts) = &allow.kind else {
        panic!("--allow must be a Dropdown");
    };
    assert_eq!(
        *opts,
        [
            "malleable",
            "mixed-timelock",
            "repeated-keys",
            "resource-limit",
            "sigless-branch",
        ],
        "ALLOW_RULES must byte-equal the v0.52.0 binary's --allow \
         possible-values"
    );
    assert!(
        allow.repeating,
        "--allow is repeating (one sanity-rule opt-out per occurrence; the \
         repeating-Dropdown precedent is convert --to)"
    );
    assert!(allow.default_value.is_none(), "--allow has no default");
}

#[test]
fn build_descriptor_key_flags_are_repeating_text() {
    for name in ["--key", "--recovery-key"] {
        let f = flag(name);
        assert!(
            matches!(f.kind, FlagKind::Text),
            "{name} must be Text (xpub strings — NOT Path; the --spec lesson \
             cuts the other way)"
        );
        assert!(f.repeating, "{name} must be repeating (row order = quorum order)");
        assert!(!f.secret, "{name} carries xpubs, not secrets");
    }
}
