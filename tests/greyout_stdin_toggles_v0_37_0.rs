//! v0.37.0 — the suppressed `*-stdin` secret toggles render DISABLED (greyed)
//! so the dead checkbox isn't a lie (user decision: grey out, not emit).
//!
//! SPEC: `design/SPEC_gui_v0_37_0_greyout_stdin_toggles.md`.
//!
//! - T1 (pure logic): the grey-out predicate (`flag_is_secret && Boolean`)
//!   is EXACTLY the assembler's suppressed set — a checked instance emits
//!   nothing — AND no secret non-Text/non-Composite/non-Boolean flag exists
//!   (the converse-closure: a future secret Path/Number would be suppressed
//!   but NOT greyed → a live-but-dead control → must trip RED here).
//! - T2 (kittest): the rendered checkbox is disabled (not interactable).

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::widget;
use mnemonic_gui::schema::{self, FlagKind, FlagValue, FormState};
use mnemonic_gui::secrets::flag_is_secret;

const SCHEMAS: &[(&str, &schema::Schema)] = &[
    ("mnemonic", &schema::mnemonic::SCHEMA),
    ("md", &schema::md::SCHEMA),
    ("ms", &schema::ms::SCHEMA),
    ("mk", &schema::mk::SCHEMA),
];

// ── T1 — predicate == suppression, + converse-closure ───────────────────────

#[test]
fn t1_greyout_predicate_equals_assembler_suppression() {
    for (_cli, s) in SCHEMAS {
        for sub in s.subcommands {
            for flag in sub.flags {
                let secret = flag_is_secret(flag);

                // Converse-closure: a secret flag that is NOT Text and NOT
                // NodeValueComposite MUST be Boolean — else it would be
                // assembler-suppressed (the `else { continue }` arm) but the
                // Boolean-only grey-out would leave it a live-but-dead
                // control. A future secret Path/Number trips this RED.
                if secret
                    && !matches!(flag.kind, FlagKind::Text)
                    && !matches!(flag.kind, FlagKind::NodeValueComposite(_))
                {
                    assert!(
                        matches!(flag.kind, FlagKind::Boolean),
                        "({}, {}): a secret non-Text/non-Composite flag is \
                         assembler-suppressed but the grey-out branch only covers \
                         Boolean — widen the predicate (else it renders a \
                         live-but-dead control)",
                        sub.name,
                        flag.name
                    );
                }

                // For the greyed set (secret Boolean), a checked instance
                // must emit NOTHING — the suppression the grey-out reflects.
                if secret && matches!(flag.kind, FlagKind::Boolean) {
                    let mut state = FormState::default();
                    state
                        .values
                        .push((flag.name.to_string(), FlagValue::Boolean(true)));
                    let argv = assemble_argv(s, sub, &state);
                    assert!(
                        !argv.contains(&flag.name.to_string()),
                        "({}, {}): a checked secret stdin toggle must NOT emit \
                         (the grey-out reflects this suppression); argv={argv:?}",
                        sub.name,
                        flag.name
                    );
                }
            }
        }
    }
}

// ── T2 — the toggle renders disabled ────────────────────────────────────────

#[test]
fn t2_stdin_toggle_checkbox_renders_disabled() {
    let sub_name = "bundle";
    let flag = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|x| x.name == sub_name)
        .unwrap()
        .flags
        .iter()
        .find(|f| f.name == "--passphrase-stdin")
        .expect("bundle carries --passphrase-stdin");
    assert!(
        flag_is_secret(flag) && matches!(flag.kind, FlagKind::Boolean),
        "fixture must be a secret Boolean stdin toggle"
    );

    let mut harness = Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            widget::render_with_dispatch(ui, CliTab::Mnemonic, sub_name, flag, state, &[]);
        },
        FormState::default(),
    );
    harness.run();
    // The checkbox is labeled with the flag name (it IS the label), so this
    // targets the CheckBox node directly.
    let toggle = harness.get_by_label("--passphrase-stdin");
    assert!(
        toggle.is_disabled(),
        "the suppressed `*-stdin` toggle must render disabled (greyed)"
    );
}
