//! F1 — export-wallet `--template` UNSET affordance (gui_example P1.3).
//!
//! Ruling: the F1 pre-implementation mini-R0 (BINDING; persisted at
//! `mnemonic-toolkit/docs/manual-gui/design/agent-reports/gui-example-f1-mini-r0.md`)
//! chose **A1 with APPEND placement**: export-wallet's `--template` opts are
//! the 10 shared `TEMPLATES` values IN ORDER plus a trailing `""` UNSET
//! sentinel (a per-flag `EXPORT_WALLET_TEMPLATES` const — the shared
//! `TEMPLATES` const stays 10 and keeps feeding bundle/verify-bundle + the
//! SINGLE_SIG/MULTISIG partition consts).
//!
//! Pre-F1 the form was a trap: the form-loop materializes a virgin Dropdown
//! to `opts[0]` (`flag_defaults.rs`), so `--template` was ALWAYS present →
//! the template/descriptor mutex (`conditional.rs::export_wallet`) kept
//! `--descriptor` permanently Disabled — the descriptor arm was unreachable
//! from the GUI. Selecting the appended `(none)` row writes `Dropdown("")`
//! → `has_value` false → the mutex releases by construction (NO conditional
//! rule changed).
//!
//! APPEND (not the archetype's prepend) is load-bearing: `opts[0]` must stay
//! `bip44` so the virgin settled form still materializes `bip44`, keeping
//! the 61-form gallery PNG byte-stable and the taught "clear Template to
//! unlock Descriptor" flow honest. Test 5 below is the census guard pinning
//! that placement; `gui_render_emit.rs`'s exact-ASCII export-wallet pin is
//! the redundant structural guard.
//!
//! The mini-R0's 5 TDD cells (all always-run, not env-gated):
//!   1. Reachability — (none) → type a descriptor → `--descriptor` enabled,
//!      argv carries it, NO `--template` token (the F1 regression test).
//!   2. `(none)` never leaks `""` into argv or the masked copy-command;
//!      re-selecting a real template re-emits + re-disables `--descriptor`.
//!   3. Materialization/`has_value` — virgin true (bip44); after `(none)`
//!      false; the mutex tracks both transitions.
//!   4. `(none)` display — the sentinel renders `(none)` in the OPEN popup
//!      list AND the closed combo after selection, never a bare `""`.
//!   5. Virgin-default stability (the append-pin) — virgin settled state
//!      materializes `--template == "bip44"` and `--descriptor` Disabled on
//!      load; opts placement pinned first==bip44 / last=="" (the tripwire
//!      against a future silent prepend).

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::{
    assemble_argv, assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::widget::display_or;
use mnemonic_gui::schema::{FlagKind, FlagValue, FormState, Visibility};

use ui_harness::{
    effect_of, flag_of, label_disabled, render_flag_harness, render_whole_form_harness,
    schema_for, sub_of,
};

const DESCRIPTOR: &str = "wsh(sortedmulti(2,@0,@1))";

/// A settled virgin export-wallet whole-form harness: the construction frame
/// materializes `--template` to `opts[0]` (the form-loop write-back), the two
/// follow-up frames let `conditional::export_wallet` re-evaluate against the
/// materialized value (the on-load steady state the GUI shows).
fn settled_virgin_form() -> Harness<'static, FormState> {
    let sub = sub_of(CliTab::Mnemonic, "export-wallet");
    let mut h = render_whole_form_harness(CliTab::Mnemonic, sub, FormState::default());
    h.run();
    h.run();
    h
}

/// Select `opt` in the export-wallet `--template` combo. The combo BUTTON
/// exposes NO AccessKit label (egui paints the selected text without
/// projecting it — verified against the node dump), so the handle is TREE
/// ORDER: `--template` is the form's FIRST Dropdown flag (schema order,
/// byte-pinned by `gui_render_emit.rs`'s exact-ASCII render), hence the
/// first ComboBox-role node. A mis-target fails loudly downstream — no
/// other export-wallet combo lists template rows or a `(none)` row, so
/// `get_by_label(opt)` would panic. egui's ComboBox does NOT auto-close its
/// popup on a `selectable_value` click (see `ui_harness_i2_conditional.rs::
/// select_context`), so the combo is only toggled open when `opt` is not
/// already a visible popup row.
fn select_template(h: &mut Harness<'static, FormState>, opt: &str) {
    if h.query_by_label(opt).is_none() {
        h.get_all_by_role(Role::ComboBox)
            .next()
            .expect("export-wallet must render the --template ComboBox first")
            .click();
        h.run();
        h.run(); // settle: the popup's option nodes land in the AccessKit tree
    }
    h.get_by_label(opt).click();
    h.run();
    h.run(); // settle: selection write-back + conditional re-evaluation
}

/// Type `text` into the `--descriptor` TextEdit. The export-wallet form
/// renders four TextInput-role nodes (`--descriptor`, `--output`,
/// `--wallet-name`, `--from-import-json`); `--descriptor` is the FIRST in
/// tree order (schema order, byte-pinned by `gui_render_emit.rs`'s
/// exact-ASCII render). The caller's state/argv assertion (`--descriptor`
/// carries the typed text) makes a mis-target fail loudly, never silently.
fn type_into_descriptor(h: &mut Harness<'static, FormState>, text: &str) {
    h.get_all_by_role(Role::TextInput)
        .next()
        .expect("export-wallet must render a TextInput for --descriptor")
        .type_text(text);
    h.run();
    h.run(); // settle: buffer write-back lands at frame end
}

fn argv_of(h: &Harness<'static, FormState>) -> Vec<String> {
    assemble_argv(
        schema_for(CliTab::Mnemonic),
        sub_of(CliTab::Mnemonic, "export-wallet"),
        h.state(),
    )
}

fn template_value(h: &Harness<'static, FormState>) -> Option<FlagValue> {
    h.state()
        .values
        .iter()
        .find(|(k, _)| k == "--template")
        .map(|(_, v)| v.clone())
}

// ── 1. Reachability (the F1 regression test) ───────────────────────────────

#[test]
fn none_sentinel_unlocks_descriptor_arm_and_argv_carries_it() {
    let mut h = settled_virgin_form();
    // Virgin steady state: materialized bip44 holds the mutex — the
    // descriptor arm starts locked (this is exactly the pre-F1 trap).
    assert_eq!(
        label_disabled(&h, "--descriptor"),
        Some(true),
        "virgin export-wallet must render --descriptor Disabled (template materialized)"
    );

    select_template(&mut h, "(none)");
    assert_eq!(
        label_disabled(&h, "--descriptor"),
        Some(false),
        "selecting (none) must release the template/descriptor mutex"
    );

    type_into_descriptor(&mut h, DESCRIPTOR);
    let argv = argv_of(&h);
    assert!(
        argv.windows(2)
            .any(|w| w[0] == "--descriptor" && w[1] == DESCRIPTOR),
        "argv must carry the typed descriptor; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "--template"),
        "argv must carry NO --template token after (none); got {argv:?}"
    );
}

// ── 2. (none) never leaks `""`; re-selection restores the mutex ────────────

#[test]
fn none_selection_never_leaks_empty_string_into_argv_or_copy_command() {
    let mut h = settled_virgin_form();
    select_template(&mut h, "(none)");

    let (argv, mask) = assemble_argv_with_secret_mask(
        schema_for(CliTab::Mnemonic),
        sub_of(CliTab::Mnemonic, "export-wallet"),
        h.state(),
    );
    assert!(
        !argv.iter().any(|a| a == "--template"),
        "empty Dropdown must be presence-suppressed (no --template); got {argv:?}"
    );
    assert!(
        argv.iter().all(|t| !t.is_empty()),
        "no \"\" token may leak into the assembled argv; got {argv:?}"
    );
    // Masked copy-command echo (both shell flavors): no --template, no
    // empty-string quoting artifact (posix `''` / windows `""`).
    let posix = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    assert!(
        !posix.contains("--template") && !posix.contains("''"),
        "posix copy-command must carry neither --template nor an empty-string \
         artifact; got {posix:?}"
    );
    let windows = render_copy_command_masked(&argv, &mask, ShellFlavor::WindowsCmd);
    assert!(
        !windows.contains("--template") && !windows.contains("\"\""),
        "windows copy-command must carry neither --template nor an empty-string \
         artifact; got {windows:?}"
    );

    // Re-selecting a real template re-emits `--template <v>` and re-disables
    // the descriptor arm (the mutex re-engages).
    select_template(&mut h, "bip49");
    assert_eq!(
        label_disabled(&h, "--descriptor"),
        Some(true),
        "re-selecting a real template must re-disable --descriptor"
    );
    let argv2 = argv_of(&h);
    assert!(
        argv2.windows(2).any(|w| w[0] == "--template" && w[1] == "bip49"),
        "re-selection must re-emit --template bip49; got {argv2:?}"
    );
}

// ── 3. Materialization / has_value tracks both transitions ────────────────

#[test]
fn has_value_tracks_none_transitions_both_ways() {
    let sub = sub_of(CliTab::Mnemonic, "export-wallet");
    let mut h = settled_virgin_form();
    // Virgin: materialized opts[0] (= bip44) IS a present value.
    assert!(
        h.state().has_value("--template"),
        "virgin materialization must yield a present --template value"
    );
    assert_eq!(
        effect_of(sub, h.state(), "--descriptor"),
        Visibility::Disabled,
        "present template ⇒ mutex holds --descriptor Disabled"
    );

    // After (none): Dropdown("") ⇒ has_value false ⇒ the mutex releases and
    // the both-required arm marks the pair Required (the toolkit's projected
    // runtime pre-check).
    select_template(&mut h, "(none)");
    assert!(
        !h.state().has_value("--template"),
        "(none) must clear --template presence (Dropdown(\"\") is not a value)"
    );
    assert_eq!(
        effect_of(sub, h.state(), "--descriptor"),
        Visibility::Required,
        "neither set ⇒ both-required arm marks --descriptor Required"
    );

    // Back to a real value: presence + Disabled both re-engage.
    select_template(&mut h, "bip84");
    assert!(
        h.state().has_value("--template"),
        "re-selecting a real template must restore presence"
    );
    assert_eq!(
        effect_of(sub, h.state(), "--descriptor"),
        Visibility::Disabled,
        "the mutex must re-engage after re-selection"
    );
}

// ── 4. (none) display: open popup row AND closed combo, never a bare "" ────

#[test]
fn none_sentinel_displays_in_open_popup_and_closed_combo() {
    // Single-flag render (the P1 harness): exactly ONE ComboBox-role node, so
    // the combo button is uniquely targetable while its popup is open.
    let sub = sub_of(CliTab::Mnemonic, "export-wallet");
    let flag = flag_of(sub, "--template");
    let mut h = render_flag_harness(CliTab::Mnemonic, sub, flag, FormState::default());
    h.run();
    h.run();

    // Virgin closed combo: selected-text is bip44 — no "(none)" node exists.
    assert!(
        h.query_by_label("(none)").is_none(),
        "virgin closed combo must not display (none) — the default is bip44"
    );

    // Open the popup: the sentinel renders as a selectable "(none)" row
    // (the `display_or` mapping — never a bare "" sliver), alongside all 10
    // real template rows.
    h.get_by_role(Role::ComboBox).click();
    h.run();
    h.run();
    assert_eq!(
        h.get_all_by_label("(none)").count(),
        1,
        "the OPEN popup must list exactly one (none) row"
    );
    for t in [
        "bip44",
        "bip49",
        "bip84",
        "bip86",
        "wsh-multi",
        "wsh-sortedmulti",
        "sh-wsh-multi",
        "sh-wsh-sortedmulti",
        "tr-multi-a",
        "tr-sortedmulti-a",
    ] {
        assert!(
            h.query_by_label(t).is_some(),
            "the OPEN popup must still list the real template row {t}"
        );
    }

    // Select it: the stored value is the RAW "" sentinel — the (none)
    // mapping is display-only.
    h.get_by_label("(none)").click();
    h.run();
    h.run();
    assert_eq!(
        h.state()
            .values
            .iter()
            .find(|(k, _)| k == "--template")
            .map(|(_, v)| v.clone()),
        Some(FlagValue::Dropdown(String::new())),
        "the stored value must be the raw \"\" sentinel (display-only mapping)"
    );

    // Closed-combo display site: egui does NOT project the combo button's
    // painted selected-text into AccessKit (node dump: `label: ""`, no text
    // child — the same reason `select_template` targets by Role), so the
    // button text is asserted at its production seam: `widget.rs` feeds
    // `.selected_text(display_or("(none)", sel))`, and BOTH display sites
    // route through the shared `display_or` helper (v0.32.0 R0-r1 M2 — the
    // popup site is the kittest-asserted instance above). With the stored
    // value now "", the closed combo therefore renders "(none)", never a
    // bare "" sliver.
    assert_eq!(
        display_or("(none)", ""),
        "(none)",
        "the closed combo's selected-text seam must map \"\" to (none)"
    );
}

// ── 5. ★ Virgin-default stability — the APPEND census guard ────────────────

#[test]
fn virgin_default_stability_append_pin() {
    // Placement pin: the sentinel is APPENDED — opts[0] stays bip44 (the
    // virgin materialization seed), opts[last] is "". A future refactor
    // silently PREPENDING the sentinel would flip the virgin default to
    // unset, enable --descriptor + the multisig-only flags on load, move the
    // 61-form gallery PNG, and desync the taught "clear Template to unlock
    // Descriptor" flow — this cell is the tripwire (gui_render_emit.rs's
    // exact-ASCII pin is the redundant structural guard).
    let sub = sub_of(CliTab::Mnemonic, "export-wallet");
    let flag = flag_of(sub, "--template");
    let FlagKind::Dropdown(opts) = &flag.kind else {
        panic!("export-wallet --template must be a Dropdown");
    };
    assert_eq!(opts.len(), 11, "10 shared template values + the appended sentinel");
    assert_eq!(
        opts.first().copied(),
        Some("bip44"),
        "APPEND pin: opts[0] must stay bip44 (virgin materialization seed)"
    );
    assert_eq!(
        opts.last().copied(),
        Some(""),
        "APPEND pin: the \"\" unset sentinel must be the LAST option"
    );

    // The 10 real values are exactly bundle's shared TEMPLATES, in order.
    // bundle stays 10 — the 11-vs-10 asymmetry is INTENDED and scoped to
    // export-wallet (mini-R0 §5: do not \"fix\" bundle).
    let bundle_flag = flag_of(sub_of(CliTab::Mnemonic, "bundle"), "--template");
    let FlagKind::Dropdown(bundle_opts) = &bundle_flag.kind else {
        panic!("bundle --template must be a Dropdown");
    };
    assert_eq!(
        bundle_opts.len(),
        10,
        "bundle --template must stay 10 values — the sentinel is export-wallet-ONLY"
    );
    assert_eq!(
        &opts[..opts.len() - 1],
        *bundle_opts,
        "export-wallet's real values must mirror the shared TEMPLATES in order"
    );

    // Virgin settled form: --template materializes bip44 (NOT ""), presence
    // holds, and --descriptor is Disabled on load — byte-identical to the
    // pre-F1 steady state (the gallery PNG census cell).
    let h = settled_virgin_form();
    assert_eq!(
        template_value(&h),
        Some(FlagValue::Dropdown("bip44".into())),
        "virgin settled state must materialize --template == \"bip44\""
    );
    assert!(h.state().has_value("--template"));
    assert_eq!(
        label_disabled(&h, "--descriptor"),
        Some(true),
        "--descriptor must render Disabled on load (virgin steady state unchanged)"
    );
    assert_eq!(
        effect_of(sub, h.state(), "--descriptor"),
        Visibility::Disabled
    );
}
