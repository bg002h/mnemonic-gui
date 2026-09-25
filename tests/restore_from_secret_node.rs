//! FOLLOWUP `restore-from-secret-node-unmasked-and-persisted`.
//!
//! `restore --from` is a plain `FlagKind::Text` flag (`secret: false`, upstream
//! and here): its secrecy depends on the NODE the user types (`ms1=<card>` is a
//! master secret, `xpub=<key>` is not). Before this fix a Text value of shape
//! `<node>=<v>` with an argv-secret node was treated as public at all four
//! secret-handling sites:
//!
//! 1. Preview / argv mask   — `assemble_argv_with_secret_mask` left it unmasked;
//! 2. run-confirm           — `should_confirm_run` returned false;
//! 3. persistence           — `redact_for_persistence` KEPT it (state.json);
//! 4. widget                — the field rendered cleartext.
//!
//! Each site has a positive test (a real secret node IS treated as secret) and
//! a negative control (a public node — `xpub=` — is NOT masked or redacted:
//! over-masking a public value hides what the user needs to check).

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::{
    assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::widget;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::{self, FlagSchema, FlagValue, FormState, SubcommandSchema};
use mnemonic_gui::secrets::{should_confirm_run, SECRET_NODE_TYPES_ARGV};

/// A real (valid-checksum) all-zero-entropy codex32 secret.
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// A watch-only xpub (negative control).
const XPUB: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet";

fn restore() -> &'static SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "restore")
        .expect("restore subcommand")
}

fn restore_from_flag() -> &'static FlagSchema {
    restore()
        .flags
        .iter()
        .find(|f| f.name == "--from")
        .expect("restore --from")
}

fn from_text(v: &str) -> FormState {
    FormState::from_pairs(vec![("--from", FlagValue::Text(v.into()))])
}

#[test]
fn precondition_restore_from_is_a_non_secret_text_flag() {
    // The fix exists BECAUSE of this shape; if upstream ever marks the flag
    // secret or gives it a composite widget, this file's premise changes.
    let f = restore_from_flag();
    assert!(
        matches!(f.kind, schema::FlagKind::Text),
        "restore --from must be Text"
    );
    assert!(!f.secret, "restore --from must be secret: false");
}

// ── site 1: Preview / argv mask ──────────────────────────────────────────────

fn from_token_mask(v: &str) -> (String, bool, String) {
    let state = from_text(v);
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, restore(), &state);
    let i = argv
        .iter()
        .position(|t| t == v)
        .expect("--from value token in argv");
    assert_eq!(argv[i - 1], "--from");
    let preview = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    (argv[i].clone(), mask[i], preview)
}

#[test]
fn site1_secret_node_value_is_masked_in_preview() {
    let (_, bit, preview) = from_token_mask(&format!("ms1={MS1}"));
    assert!(
        bit,
        "restore --from ms1=<card> must carry a secret mask bit"
    );
    assert!(
        !preview.contains(MS1),
        "the Preview must not show the ms1 card: {preview}"
    );
}

#[test]
fn site1_every_argv_secret_node_is_masked() {
    for node in SECRET_NODE_TYPES_ARGV {
        let (_, bit, _) = from_token_mask(&format!("{node}=somevalue"));
        assert!(bit, "restore --from {node}=… must be masked");
    }
}

#[test]
fn site1_public_node_is_not_masked() {
    let (_, bit, preview) = from_token_mask(&format!("xpub={XPUB}"));
    assert!(!bit, "restore --from xpub=… must NOT be masked");
    assert!(
        preview.contains(XPUB),
        "the Preview must show a public xpub: {preview}"
    );
}

#[test]
fn site1_c1_a_channel_spelling_is_a_source_that_c1_resolves_or_refuses() {
    // Phase 1a pinned `ms1=-` as "not masked, no material". Under the
    // secret-channel design (§A3a, §A4.1) `restore --from ms1=<v>` is a secret
    // SOURCE whatever `<v>` is, and C1 decides what `-` / `@env:VAR` mean: `-`
    // refuses on every OS (the GUI has no stdin to forward), `@env:VAR` is
    // resolved GUI-side and argv carries only the planner's reference.
    let (_, bit, _) = from_token_mask("ms1=-");
    assert!(bit, "ms1=- is a secret source (C1 handles its meaning)");
    let none = |_: &str| None;
    for os in ["linux", "macos", "windows"] {
        let r = mnemonic_gui::form::channels::plan(&schema::mnemonic::SCHEMA, restore(), &from_text("ms1=-"), &none, os);
        assert_eq!(r.err().map(|e| e.code), Some("C1-dash"), "{os}");
    }
    let env = |k: &str| (k == "SEED").then(|| MS1.to_string());
    let p = mnemonic_gui::form::channels::plan(&schema::mnemonic::SCHEMA, restore(), &from_text("ms1=@env:SEED"), &env, "linux")
        .expect("resolved");
    assert!(p.argv.iter().any(|t| t == "ms1=@env:MNEMONIC_GUI_S0"), "{:?}", p.argv);
    assert!(!p.argv.iter().any(|t| t.contains(MS1) || t.contains("SEED")));
    assert_eq!(p.env[0].1.as_str(), MS1);
    // interim: the RESOLVED card on argv, masked, never the typed `@env:` text
    let p = mnemonic_gui::form::channels::plan(&schema::mnemonic::SCHEMA, restore(), &from_text("ms1=@env:SEED"), &env, "macos")
        .expect("resolved");
    let at = p.argv.iter().position(|t| *t == format!("ms1={MS1}")).expect("resolved card on interim argv");
    assert!(p.mask[at]);
    assert!(!p.argv.iter().any(|t| t.contains("@env:")));
}

// ── site 2: run-confirm ──────────────────────────────────────────────────────

#[test]
fn site2_secret_node_value_requires_run_confirm() {
    assert!(
        should_confirm_run(restore(), &from_text(&format!("ms1={MS1}"))),
        "restore --from ms1=<card> must go through the run-confirm modal"
    );
}

#[test]
fn site2_public_node_does_not_require_run_confirm() {
    assert!(
        !should_confirm_run(restore(), &from_text(&format!("xpub={XPUB}"))),
        "restore --from xpub=… must not trigger the run-confirm modal"
    );
}

// ── site 3: persistence redaction ────────────────────────────────────────────

fn persisted_from(v: &str) -> Option<FlagValue> {
    redact_for_persistence(&from_text(v))
        .values
        .into_iter()
        .find(|(k, _)| k == "--from")
        .map(|(_, v)| v)
}

#[test]
fn site3_secret_node_value_is_not_persisted() {
    assert_eq!(
        persisted_from(&format!("ms1={MS1}")),
        None,
        "restore --from ms1=<card> must never reach state.json"
    );
    let json = serde_json::to_string(&redact_for_persistence(&from_text(&format!("ms1={MS1}"))))
        .expect("serialize");
    assert!(
        !json.contains(MS1),
        "the ms1 card leaked into the persisted JSON: {json}"
    );
}

#[test]
fn site3_public_node_value_is_persisted() {
    let v = format!("xpub={XPUB}");
    assert_eq!(
        persisted_from(&v),
        Some(FlagValue::Text(v.clone())),
        "restore --from xpub=… is public and must persist"
    );
}

// ── site 4: widget masking ───────────────────────────────────────────────────

fn text_harness(initial: &str) -> Harness<'static, FlagValue> {
    Harness::new_ui_state(
        move |ui, value: &mut FlagValue| {
            let state = FormState::default();
            widget::render(
                ui,
                CliTab::Mnemonic,
                "restore",
                restore_from_flag(),
                value,
                &state,
                &[],
            );
        },
        FlagValue::Text(initial.into()),
    )
}

#[test]
fn site4_secret_node_value_renders_as_password_field() {
    let mut h = text_harness(&format!("ms1={MS1}"));
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput)
            .is_some(),
        "restore --from ms1=<card> must render masked (PasswordInput)"
    );
}

#[test]
fn site4_public_node_value_is_not_masked() {
    let mut h = text_harness(&format!("xpub={XPUB}"));
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput)
            .is_none(),
        "restore --from xpub=… must stay readable"
    );
}

#[test]
fn site4_typing_into_the_mask_keeps_focus_and_every_character() {
    // The field flips from plain to masked mid-typing (at `ms1=m`), and a reveal
    // eye appears in front of it. If that changed the field's widget id, focus
    // would drop and the rest of the card would be lost. Type it key by key.
    let mut h = text_harness("");
    h.run();
    h.get_by_role(egui::accesskit::Role::TextInput).focus();
    h.run();
    let typed = format!("ms1={MS1}");
    for ch in typed.chars() {
        h.input_mut().events.push(egui::Event::Text(ch.to_string()));
        h.run();
    }
    assert_eq!(
        h.state(),
        &FlagValue::Text(typed),
        "every typed character must land"
    );
    assert!(h
        .query_by_role(egui::accesskit::Role::PasswordInput)
        .is_some());
}
