//! v0.16.0 P3 — `assemble_argv` SPEC §6.10 visibility-gate tests.
//!
//! For each NEW Hidden- or Disabled-effecting rule introduced in the
//! v0.16.0 GUI conditional-applicability cycle, asserts the target flag is
//! ABSENT from the assembled argv when the rule's predicate is satisfied.
//! Also covers:
//!   - Composition: simultaneous template-single-sig (Disabled threshold)
//!     AND --descriptor present (Disabled threshold via different rule);
//!     first-rule-wins per SPEC §6.10.4 / `main.rs:391-394`. Both rules
//!     suppress emission.
//!   - Required decoration: a flag marked Required by its conditional fn
//!     STILL emits.
//!   - Mutex-pair argv-emission regression (LATENT-bug fix): user types
//!     --passphrase=foo then sets --passphrase-stdin; argv contains
//!     --passphrase-stdin but NOT --passphrase foo. Pre-v0.16.0 the typed
//!     value would emit and trigger clap conflicts_with rejection.

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::schema::{self, FlagValue, FormState};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

fn argv_for(sub: &str, state: &FormState) -> Vec<String> {
    assemble_argv(&schema::mnemonic::SCHEMA, subcommand(sub), state)
}

fn contains_pair(argv: &[String], flag: &str, value: &str) -> bool {
    argv.windows(2)
        .any(|w| w[0] == flag && w[1] == value)
}

// ── §6.10.7 bundle visibility gate ─────────────────────────────────────

#[test]
fn bundle_threshold_suppressed_when_single_sig_template() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--threshold", FlagValue::Number(2)),
    ]);
    let argv = argv_for("bundle", &state);
    assert!(
        !argv.iter().any(|a| a == "--threshold"),
        "argv must NOT contain --threshold when template is single-sig; \
         got {argv:?}"
    );
    // Sanity: --template DOES emit.
    assert!(contains_pair(&argv, "--template", "bip84"));
}

#[test]
fn bundle_multisig_path_family_suppressed_when_single_sig_template() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip49".into())),
        (
            "--multisig-path-family",
            FlagValue::Dropdown("bip48".into()),
        ),
    ]);
    let argv = argv_for("bundle", &state);
    assert!(!argv.iter().any(|a| a == "--multisig-path-family"));
}

#[test]
fn bundle_template_threshold_path_family_all_suppressed_when_descriptor() {
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--template", FlagValue::Dropdown("wsh-sortedmulti".into())),
        ("--threshold", FlagValue::Number(2)),
        (
            "--multisig-path-family",
            FlagValue::Dropdown("bip48".into()),
        ),
    ]);
    let argv = argv_for("bundle", &state);
    // --descriptor emits; the three others are suppressed by visibility gate.
    assert!(argv.iter().any(|a| a == "--descriptor"));
    assert!(!argv.iter().any(|a| a == "--template"));
    assert!(!argv.iter().any(|a| a == "--threshold"));
    assert!(!argv.iter().any(|a| a == "--multisig-path-family"));
}

#[test]
fn bundle_compose_descriptor_and_single_sig_template_both_suppress() {
    // Both rules target --threshold (descriptor-mode rule fires first
    // priority, single-sig rule fires second). With both predicates true,
    // first-rule-wins picks descriptor-mode; observable outcome is the
    // same (Disabled). Drift gate would fail if the JSON's emission order
    // didn't match this.
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--threshold", FlagValue::Number(1)),
    ]);
    let argv = argv_for("bundle", &state);
    assert!(!argv.iter().any(|a| a == "--threshold"));
}

// ── §6.10.7 export-wallet visibility gate ──────────────────────────────

#[test]
fn export_wallet_taproot_internal_key_suppressed_when_non_taproot_template() {
    use mnemonic_gui::schema::TaggedOrIndexedValue;
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
        (
            "--taproot-internal-key",
            FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Tag("nums".into())),
        ),
    ]);
    let argv = argv_for("export-wallet", &state);
    assert!(!argv.iter().any(|a| a == "--taproot-internal-key"));
}

#[test]
fn export_wallet_taproot_internal_key_emits_for_taproot_template() {
    use mnemonic_gui::schema::TaggedOrIndexedValue;
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("tr-sortedmulti-a".into())),
        (
            "--taproot-internal-key",
            FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Tag("nums".into())),
        ),
    ]);
    let argv = argv_for("export-wallet", &state);
    assert!(contains_pair(&argv, "--taproot-internal-key", "nums"));
}

#[test]
fn export_wallet_threshold_suppressed_when_single_sig() {
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip86".into())),
        ("--threshold", FlagValue::Number(2)),
    ]);
    let argv = argv_for("export-wallet", &state);
    assert!(!argv.iter().any(|a| a == "--threshold"));
}

// ── §6.10.4 Required-decoration regression: still emits ────────────────

#[test]
fn required_decoration_does_not_suppress_emission() {
    // verify-bundle: --mk1 is Required-unless-bundle-json; the conditional
    // fn marks it Required in the no-bundle-json branch. Required-marking
    // MUST NOT suppress emission — the user just hasn't supplied a value
    // yet. With a value present, it MUST emit.
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--mk1", FlagValue::Text("mk1abc".into())),
    ]);
    let argv = argv_for("verify-bundle", &state);
    assert!(contains_pair(&argv, "--mk1", "mk1abc"));
}

#[test]
fn derive_child_dice_sides_required_but_still_emits_when_supplied() {
    let mut state = FormState::from_pairs(vec![
        ("--application", FlagValue::Dropdown("dice".into())),
        ("--dice-sides", FlagValue::Number(6)),
        ("--length", FlagValue::Number(24)),
        ("--index", FlagValue::Number(0)),
        ("--from", FlagValue::Text("phrase=abandon abandon".into())),
    ]);
    // Add a non-secret way to bypass the from-text complication: skip
    // emission shape check, just confirm the Required-marked --dice-sides
    // emits its value.
    let _ = &mut state;
    let argv = argv_for("derive-child", &state);
    assert!(contains_pair(&argv, "--dice-sides", "6"));
}

// ── Mutex-pair argv-emission regression (latent-bug fix) ───────────────

#[test]
fn passphrase_typed_then_stdin_set_does_not_emit_typed_value() {
    // LATENT BUG FIX: pre-v0.16.0, the GUI would emit a typed --passphrase
    // value EVEN AFTER the user toggled --passphrase-stdin. clap would
    // reject with conflicts_with. v0.16.0 visibility gate suppresses the
    // typed --passphrase value when --passphrase-stdin is set (the
    // conditional fn marks --passphrase Disabled in that case).
    //
    // (Note: --passphrase-stdin's own emission goes through a separate
    // code path orthogonal to this fix; this test scope is the
    // --passphrase suppression only.)
    let mut state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--passphrase-stdin", FlagValue::Boolean(true)),
    ]);
    // Simulate the user having typed a value into --passphrase BEFORE
    // toggling --passphrase-stdin. The widget retains the value
    // internally so toggling back restores it.
    state.secret_widgets.insert(
        "--passphrase".into(),
        SecretLineEdit::from_text("typed-but-not-meant-to-emit"),
    );
    let argv = argv_for("bundle", &state);
    assert!(
        !argv.iter().any(|a| a == "--passphrase"),
        "argv MUST NOT contain --passphrase (typed value retained but \
         visibility-gated); got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "typed-but-not-meant-to-emit"),
        "the typed value itself MUST NOT leak"
    );
}

#[test]
fn bip38_passphrase_typed_then_stdin_set_does_not_emit_typed_value() {
    // Mirror of the passphrase test for the bip38 pair in convert.
    let mut state = FormState::default();
    state.secret_widgets.insert(
        "--bip38-passphrase".into(),
        SecretLineEdit::from_text("bip38-typed"),
    );
    state.values.push((
        "--bip38-passphrase-stdin".into(),
        FlagValue::Boolean(true),
    ));
    let argv = argv_for("convert", &state);
    assert!(
        !argv.iter().any(|a| a == "--bip38-passphrase"),
        "--bip38-passphrase must NOT emit (visibility-gated); got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "bip38-typed"),
        "the typed value must NOT leak"
    );
}

// ── v0.6.0 SPEC §6.10.4 v3 pin_value emission ──────────────────────────

#[test]
fn bundle_pin_value_account_replaces_user_value_when_descriptor() {
    // SPEC §6.10.7 row 12 / §6.10.4 emission table: PinValue REPLACES the
    // user-typed value with the pinned value (vs Hidden/Disabled which
    // suppress entirely).
    let state = FormState::from_pairs(vec![
        ("--descriptor", FlagValue::Text("wpkh(@0/**)".into())),
        ("--account", FlagValue::Number(5)),
    ]);
    let argv = argv_for("bundle", &state);
    assert!(
        contains_pair(&argv, "--account", "0"),
        "argv must contain --account 0 (pinned per §6.10.7 row 12); got {argv:?}",
    );
    assert!(
        !contains_pair(&argv, "--account", "5"),
        "argv must NOT contain --account 5 (user value REPLACED by pin); got {argv:?}",
    );
}

#[test]
fn bundle_pin_value_account_emits_zero_even_when_user_omits() {
    // PinValue applies whether the user typed a value or not; the rule
    // fires unconditionally on --descriptor presence.
    let state = FormState::from_pairs(vec![(
        "--descriptor",
        FlagValue::Text("wpkh(@0/**)".into()),
    )]);
    let argv = argv_for("bundle", &state);
    assert!(
        contains_pair(&argv, "--account", "0"),
        "argv must contain --account 0 even without user input; got {argv:?}",
    );
}

// ── v0.7.0 SPEC §6.10.4 v4 disable_options no-impact argv contract ─────

fn state_with_slot_count(count: usize) -> FormState {
    let mut state = FormState::default();
    while state.slots.rows.len() < count {
        state.slots.rows.push(mnemonic_gui::form::slot_editor::SlotRow::default());
    }
    state.slots.rows.truncate(count);
    state
}

#[test]
fn disable_options_does_not_suppress_argv_emission() {
    // SPEC §6.10.4 v4 emission table: `disable_options` is SCHEMA-TIME
    // ONLY — does NOT join the suppress set. If `state.values` already
    // holds a now-disabled value (e.g., user picked bip84 with 1 slot,
    // then added a 2nd slot which fires row 10 disabling single-sig
    // templates), argv still emits `--template bip84`. CLI row 10 is
    // the residual safety net.
    //
    // This test pins that contract: a state with slot_count=2 + a
    // single-sig template value emits `--template bip84` regardless of
    // the row-10 disable_options visibility.
    let mut state = state_with_slot_count(2);
    state.values.push(("--template".into(), FlagValue::Dropdown("bip84".into())));
    let argv = argv_for("bundle", &state);
    assert!(
        contains_pair(&argv, "--template", "bip84"),
        "argv must STILL contain --template bip84 even when row-10 \
         disable_options has greyed it out (schema-time only contract); \
         got {argv:?}",
    );
}

#[test]
fn threshold_widget_stale_value_above_slot_count_emits_argv_unchanged() {
    // SPEC §6.10.4 v4 / R0 I4: when --threshold = 5 with 5 slots, then
    // user deletes 2 → slot_count = 3 → state.values still has Number(5).
    // egui::DragValue's .range() clamps the in-widget value on next
    // user interaction but does NOT auto-mutate the underlying state.
    // argv emits `--threshold 5` regardless; CLI row 9 rejects.
    //
    // Plan choice: NO auto-clamp at frame boundary (matches the v0.6.0
    // P3 "preserve user values" discipline). This test pins the
    // contract — auto-clamp would silently lose user intent and is
    // YAGNI for v0.7.0.
    let mut state = state_with_slot_count(3);
    state.values.push(("--threshold".into(), FlagValue::Number(5)));
    // Add a multisig template so --threshold isn't suppressed by the
    // pre-existing v0.16.0 "single-sig disables threshold" rule.
    state.values.push(("--template".into(), FlagValue::Dropdown("wsh-multi".into())));
    let argv = argv_for("bundle", &state);
    assert!(
        contains_pair(&argv, "--threshold", "5"),
        "argv must emit --threshold 5 even when slot_count=3 (no auto-\
         clamp at frame boundary); CLI row 9 catches the residual. \
         Got: {argv:?}",
    );
}
