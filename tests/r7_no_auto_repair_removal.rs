//! v0.10.0 Tranche B.3 (D33) — regression cells locking in the removal of
//! the v0.9.0 R7 action-bar `--no-auto-repair` fallback.
//!
//! Pre-v0.10.0 the GUI carried:
//!   - `MnemonicGuiApp.no_auto_repair: bool` field (`src/main.rs:99`).
//!   - Action-bar checkbox UI (`src/main.rs:322-326`).
//!   - `runner::prepend_no_auto_repair` helper (`src/runner.rs:48`).
//!   - Call site that wired the helper before spawn (`src/main.rs:788`).
//!
//! All five were deleted in lockstep with the toolkit v5 schema's
//! per-subcommand emission of `--no-auto-repair` as a `global: true` flag.
//! These cells anchor the contract:
//!   1. `--no-auto-repair` is declared as a FlagSchema entry in EVERY
//!      mnemonic subcommand (matching toolkit v5's global-flag projection).
//!   2. Each entry's `global` field is `true` (the v5-schema mirror).
//!   3. The argv assembler emits `--no-auto-repair` correctly when the
//!      Boolean is true and suppresses it when false (standard Boolean
//!      emission, no special-case routing).
//!
//! See `src/runner.rs` doc-comment for the post-removal contract.

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::schema::{self, FlagKind, FlagValue, FormState};

#[test]
fn r7_removal_no_auto_repair_declared_in_every_mnemonic_subcommand() {
    // Per toolkit v5 schema: `--no-auto-repair` (clap-global) appears
    // in every subcommand's flag list. Mirror requirement: the GUI's
    // hand-coded mnemonic schema must declare the flag for every
    // SubcommandSchema.
    let missing: Vec<&str> = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .filter(|sub| !sub.flags.iter().any(|f| f.name == "--no-auto-repair"))
        .map(|sub| sub.name)
        .collect();
    assert!(
        missing.is_empty(),
        "every mnemonic subcommand must declare --no-auto-repair (toolkit v5 \
         `global: true` projection); missing in: {missing:?}"
    );
}

#[test]
fn r7_removal_no_auto_repair_global_flag_set_per_subcommand() {
    // Each --no-auto-repair entry must carry `global: true` (the v5-schema
    // mirror). Drift here would diverge from the toolkit's gui-schema JSON.
    for sub in schema::mnemonic::SCHEMA.subcommands {
        let entry = sub
            .flags
            .iter()
            .find(|f| f.name == "--no-auto-repair")
            .unwrap_or_else(|| {
                panic!(
                    "subcommand {} missing --no-auto-repair (precondition for \
                     this cell — see r7_removal_no_auto_repair_declared_in_every_mnemonic_subcommand)",
                    sub.name,
                )
            });
        assert!(
            entry.global,
            "subcommand {} declares --no-auto-repair with `global: false`; \
             must be `global: true` per toolkit v5 schema",
            sub.name,
        );
        assert!(
            matches!(entry.kind, FlagKind::Boolean),
            "subcommand {} declares --no-auto-repair with wrong FlagKind \
             (expected Boolean for a global presence-only flag)",
            sub.name,
        );
        assert!(
            !entry.secret,
            "--no-auto-repair must NOT be marked secret (it's a presence-only \
             toggle); subcommand {} declares secret=true",
            sub.name,
        );
        assert!(
            entry.default_value.is_none(),
            "--no-auto-repair must have no default_value (clap-derive doesn't \
             declare one); subcommand {} declares default_value={:?}",
            sub.name,
            entry.default_value,
        );
    }
}

#[test]
fn r7_removal_no_auto_repair_emits_when_true() {
    // Standard Boolean emission: when state has the flag set true, argv
    // includes `--no-auto-repair` as a presence-only token (no value).
    // Replaces the pre-v0.10.0 `runner::prepend_no_auto_repair` path
    // (which spliced between argv[0] and argv[1..]); the standard path
    // emits in schema-declaration order (last in each subcommand's
    // FLAGS array).
    fn sub(name: &str) -> &'static schema::SubcommandSchema {
        schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .find(|s| s.name == name)
            .expect("subcommand exists")
    }
    let state = FormState::from_pairs(vec![
        ("--ms1", FlagValue::Text("ms1example".into())),
        ("--no-auto-repair", FlagValue::Boolean(true)),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, sub("repair"), &state);
    assert!(
        argv.iter().any(|a| a == "--no-auto-repair"),
        "--no-auto-repair must emit when Boolean(true); got {argv:?}"
    );
    // Boolean emission: name only, no value token after.
    let pos = argv.iter().position(|a| a == "--no-auto-repair").unwrap();
    // The next argv element (if any) must NOT be a value for this flag.
    // Since --no-auto-repair is the last schema entry, it's the last argv
    // token — easy check.
    assert_eq!(
        pos,
        argv.len() - 1,
        "--no-auto-repair should be the last token (last schema entry; \
         Boolean emits presence-only); got {argv:?}"
    );
}

#[test]
fn r7_removal_no_auto_repair_suppressed_when_false() {
    // Boolean(false) is trivially suppressed by emit_one — same as any
    // other Boolean flag. No special-case routing.
    fn sub(name: &str) -> &'static schema::SubcommandSchema {
        schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .find(|s| s.name == name)
            .expect("subcommand exists")
    }
    let state = FormState::from_pairs(vec![
        ("--ms1", FlagValue::Text("ms1example".into())),
        ("--no-auto-repair", FlagValue::Boolean(false)),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, sub("repair"), &state);
    assert!(
        !argv.iter().any(|a| a == "--no-auto-repair"),
        "--no-auto-repair must NOT emit when Boolean(false); got {argv:?}"
    );
}

#[test]
fn r7_removal_no_auto_repair_absent_state_omits() {
    // Default omission: when the form-state doesn't carry the flag at
    // all, argv does not include it. Anchors the "absent != global"
    // contract: `global: true` is a schema property, NOT an auto-emission
    // behavior.
    fn sub(name: &str) -> &'static schema::SubcommandSchema {
        schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .find(|s| s.name == name)
            .expect("subcommand exists")
    }
    let state = FormState::from_pairs(vec![(
        "--ms1",
        FlagValue::Text("ms1example".into()),
    )]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, sub("repair"), &state);
    assert!(
        !argv.iter().any(|a| a == "--no-auto-repair"),
        "--no-auto-repair must NOT emit when absent from form-state; got {argv:?}"
    );
}

#[test]
fn r7_removal_action_bar_checkbox_source_text_absent() {
    // Source-text anchor: the v0.9.0 action-bar checkbox text MUST NOT
    // appear in the app-shell source anymore. Catches accidental
    // re-introduction in a future cycle. gui_example_tutorial P0: the app
    // shell relocated verbatim from main.rs into src/app_window.rs — scan
    // BOTH so the anchor keeps guarding the surface wherever it lives.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut shell_src = std::fs::read_to_string(manifest.join("src/main.rs"))
        .expect("read src/main.rs");
    shell_src.push_str(
        &std::fs::read_to_string(manifest.join("src/app_window.rs"))
            .expect("read src/app_window.rs"),
    );
    assert!(
        !shell_src.contains("No auto-repair (--no-auto-repair)"),
        "the app shell (main.rs + app_window.rs) MUST NOT contain the \
         pre-v0.10.0 R7 action-bar checkbox label \
         `No auto-repair (--no-auto-repair)`; it was \
         deleted in lockstep with toolkit v5 schema emission. \
         A drift-equivalent label would silently re-introduce the \
         deleted UI surface; update this anchor only when the new label \
         is explicitly approved."
    );
    assert!(
        !shell_src.contains("self.no_auto_repair"),
        "the app shell (main.rs + app_window.rs) MUST NOT carry a \
         `no_auto_repair` field on MnemonicGuiApp; the field was deleted \
         in lockstep with the action-bar checkbox removal."
    );
}

#[test]
fn r7_removal_runner_prepend_helper_absent_from_source() {
    // The `runner::prepend_no_auto_repair` fn definition was deleted.
    // This cell anchors the removal so a future cycle can't accidentally
    // re-introduce it without updating the contract explicitly.
    let runner_rs = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runner.rs"),
    )
    .expect("read src/runner.rs");
    assert!(
        !runner_rs.contains("pub fn prepend_no_auto_repair"),
        "src/runner.rs MUST NOT define a `prepend_no_auto_repair` helper; \
         it was deleted in lockstep with the toolkit v5 schema's \
         per-subcommand --no-auto-repair emission. If you're \
         re-introducing it for a new use case, update this cell and \
         document the new contract."
    );
}
