//! P1.3 — restore `--template` `(none)` UNSET affordance (batch
//! `tutorial_surfaced_fixes_batch`; SPEC `restore_template_none_affordance`,
//! R0-GREEN). Mirrors `tests/export_wallet_template_none.rs` (F1 A1-APPEND).
//!
//! Ruling (restore-R0): a NEW per-flag `RESTORE_TEMPLATES` const = the 10 shared
//! `TEMPLATES` values IN ORDER + a trailing `""` UNSET sentinel (renders
//! "(none)" via `display_or`). `default_value` STAYS `None`; `opts[0]` stays
//! `bip44` (APPEND, never prepend — the virgin form-loop materializes `opts[0]`,
//! so a prepend would flip the virgin default to unset and move the gallery PNG).
//! The shared `TEMPLATES` const (bundle / verify-bundle / convert consumers)
//! is UNTOUCHED — the 11-vs-10 asymmetry is intended and now export-wallet ∪
//! restore-scoped.
//!
//! Restore semantics differ from export-wallet's (there is NO template/descriptor
//! mutex — restore's conditional keys ONLY on `--md1`): single-sig `(none)` =
//! the CLI emits all four `bip44/49/84/86`; md1 `(none)` = drops the single-sig
//! template the CLI REFUSES in `--md1` mode (exit 2). Selecting `(none)` writes
//! `Dropdown("")` ⇒ `has_value` false ⇒ no `--template` token — the clean md1
//! restore the tutorial now teaches.
//!
//! The 5 TDD cells (all always-run, not env-gated):
//!   1. `(none)` clears → no `--template` token, `has_value` false;
//!   2. `(none)` never leaks `""` into argv or the masked copy-command (both
//!      flavors); re-selecting `bip44` re-emits;
//!   3. materialization both ways + the restore conditional projection is
//!      UNCHANGED across bip44 / (none) / bip49 (it never reads `--template`);
//!   4. `(none)` display via `display_or` (open popup + closed seam), stored raw
//!      `Dropdown("")`, all 10 real rows still listed;
//!   5. ★ append census — `RESTORE_TEMPLATES.len()==11`, `opts[0]=="bip44"`,
//!      `opts.last()==Some("")`, `opts[..10]==TEMPLATES`, `TEMPLATES.len()==10`,
//!      virgin settled form materializes `bip44` (the PREPEND tripwire).

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::{
    assemble_argv, assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::widget::display_or;
use mnemonic_gui::schema::{FlagKind, FlagValue, FormState};

use ui_harness::{
    flag_of, render_flag_harness, render_whole_form_harness, schema_for, sub_of,
    visibility_projection,
};

fn restore_sub() -> &'static mnemonic_gui::schema::SubcommandSchema {
    sub_of(CliTab::Mnemonic, "restore")
}

/// A single-flag `--template` harness — restore renders `--format` (Dropdown) and
/// `--from` (composite) BEFORE `--template`, so `--template` is NOT the first
/// ComboBox in the whole form; the single-flag render surfaces exactly ONE
/// ComboBox, uniquely targetable (mirrors export-wallet cell 4's approach, used
/// here for the driving cells too).
fn template_harness() -> Harness<'static, FormState> {
    let flag = flag_of(restore_sub(), "--template");
    let mut h = render_flag_harness(CliTab::Mnemonic, restore_sub(), flag, FormState::default());
    h.run();
    h.run(); // settle: virgin materialization of opts[0] into state.values
    h
}

/// Select `opt` in the single `--template` combo (unique ComboBox in the
/// single-flag render). egui's ComboBox does not auto-close on a
/// `selectable_value` click, so only open when `opt` is not already visible.
fn select_template(h: &mut Harness<'static, FormState>, opt: &str) {
    if h.query_by_label(opt).is_none() {
        h.get_by_role(Role::ComboBox).click();
        h.run();
        h.run(); // settle: popup option nodes land in the AccessKit tree
    }
    h.get_by_label(opt).click();
    h.run();
    h.run(); // settle: selection write-back
}

fn argv_of(state: &FormState) -> Vec<String> {
    assemble_argv(schema_for(CliTab::Mnemonic), restore_sub(), state)
}

fn template_value(state: &FormState) -> Option<FlagValue> {
    state
        .values
        .iter()
        .find(|(k, _)| k == "--template")
        .map(|(_, v)| v.clone())
}

// ── 1. (none) clears → no --template token, has_value false ─────────────────

#[test]
fn none_sentinel_clears_template_from_argv() {
    let mut h = template_harness();
    // Virgin: materialized opts[0] (= bip44) IS a present value → argv carries it.
    let argv0 = argv_of(h.state());
    assert!(
        argv0.windows(2).any(|w| w[0] == "--template" && w[1] == "bip44"),
        "virgin restore must emit --template bip44 (opts[0] materialization); got {argv0:?}"
    );

    select_template(&mut h, "(none)");
    assert!(
        !h.state().has_value("--template"),
        "(none) must clear --template presence (Dropdown(\"\") is not a value)"
    );
    let argv = argv_of(h.state());
    assert!(
        !argv.iter().any(|a| a == "--template"),
        "argv must carry NO --template token after (none); got {argv:?}"
    );
}

// ── 2. (none) never leaks ""; re-selecting bip44 re-emits ───────────────────

#[test]
fn none_never_leaks_empty_string_and_reselection_reemits() {
    let mut h = template_harness();
    select_template(&mut h, "(none)");

    let (argv, mask) =
        assemble_argv_with_secret_mask(schema_for(CliTab::Mnemonic), restore_sub(), h.state());
    assert!(
        !argv.iter().any(|a| a == "--template"),
        "empty Dropdown must be presence-suppressed (no --template); got {argv:?}"
    );
    assert!(
        argv.iter().all(|t| !t.is_empty()),
        "no \"\" token may leak into the assembled argv; got {argv:?}"
    );
    let posix = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    assert!(
        !posix.contains("--template") && !posix.contains("''"),
        "posix copy-command must carry neither --template nor an empty-string artifact; got {posix:?}"
    );
    let windows = render_copy_command_masked(&argv, &mask, ShellFlavor::WindowsCmd);
    assert!(
        !windows.contains("--template") && !windows.contains("\"\""),
        "windows copy-command must carry neither --template nor an empty-string artifact; got {windows:?}"
    );

    // Re-selecting a real template re-emits `--template <v>`.
    select_template(&mut h, "bip44");
    let argv2 = argv_of(h.state());
    assert!(
        argv2.windows(2).any(|w| w[0] == "--template" && w[1] == "bip44"),
        "re-selection must re-emit --template bip44; got {argv2:?}"
    );
}

// ── 3. Materialization both ways + conditional UNCHANGED across states ───────

#[test]
fn has_value_tracks_transitions_and_conditional_is_template_independent() {
    let mut h = template_harness();
    assert!(
        h.state().has_value("--template"),
        "virgin materialization must yield a present --template value"
    );
    select_template(&mut h, "(none)");
    assert!(
        !h.state().has_value("--template"),
        "(none) must clear presence"
    );
    select_template(&mut h, "bip84");
    assert!(
        h.state().has_value("--template"),
        "re-selecting a real template must restore presence"
    );

    // The restore conditional keys ONLY on `--md1`, never `--template`, so the
    // visibility projection is IDENTICAL across bip44 / (none) / bip49 (the
    // "conditional rule set unchanged" invariant). It changes ONLY on --md1.
    let sub = restore_sub();
    let proj = |template: FlagValue| {
        let mut st = FormState::default();
        st.values.push(("--template".into(), template));
        visibility_projection(sub, &st)
    };
    let p_bip44 = proj(FlagValue::Dropdown("bip44".into()));
    let p_none = proj(FlagValue::Dropdown(String::new()));
    let p_bip49 = proj(FlagValue::Dropdown("bip49".into()));
    assert_eq!(
        p_bip44, p_none,
        "the restore conditional must not react to a (none) --template"
    );
    assert_eq!(
        p_bip49, p_none,
        "the restore conditional must not react to the --template value at all"
    );

    // And it DOES react to --md1 (non-vacuity: the projection is not a constant).
    let mut with_md1 = FormState::default();
    with_md1.values.push(("--md1".into(), FlagValue::Text("md1xyz".into())));
    with_md1
        .values
        .push(("--template".into(), FlagValue::Dropdown(String::new())));
    assert_ne!(
        visibility_projection(sub, &with_md1),
        p_none,
        "the restore conditional MUST react to --md1 (else the invariant is vacuous)"
    );
}

// ── 4. (none) display: open popup + closed seam, stored raw "" ──────────────

#[test]
fn none_sentinel_displays_in_open_popup_and_stores_raw_empty() {
    let mut h = template_harness();
    // Virgin closed combo: selected-text is bip44 — no "(none)" node exists.
    assert!(
        h.query_by_label("(none)").is_none(),
        "virgin closed combo must not display (none) — the default is bip44"
    );

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

    // Select it: the stored value is the RAW "" sentinel (display-only mapping).
    h.get_by_label("(none)").click();
    h.run();
    h.run();
    assert_eq!(
        template_value(h.state()),
        Some(FlagValue::Dropdown(String::new())),
        "the stored value must be the raw \"\" sentinel (display-only mapping)"
    );

    // Closed-combo seam: `widget.rs` feeds `.selected_text(display_or("(none)", sel))`.
    assert_eq!(
        display_or("(none)", ""),
        "(none)",
        "the closed combo's selected-text seam must map \"\" to (none)"
    );
}

// ── 5. ★ Append census — the PREPEND tripwire ───────────────────────────────

#[test]
fn restore_templates_append_census() {
    let sub = restore_sub();
    let flag = flag_of(sub, "--template");
    let FlagKind::Dropdown(opts) = &flag.kind else {
        panic!("restore --template must be a Dropdown");
    };
    assert_eq!(opts.len(), 11, "10 shared template values + the appended sentinel");
    assert_eq!(
        opts.first().copied(),
        Some("bip44"),
        "APPEND pin: opts[0] must stay bip44 (virgin materialization seed) — a \
         PREPEND would put \"\" here and flip the virgin default to unset"
    );
    assert_eq!(
        opts.last().copied(),
        Some(""),
        "APPEND pin: the \"\" unset sentinel must be the LAST option"
    );

    // The 10 real values are exactly bundle's shared TEMPLATES, in order; bundle
    // stays 10 (the sentinel is export-wallet ∪ restore-scoped, NOT shared).
    let bundle_flag = flag_of(sub_of(CliTab::Mnemonic, "bundle"), "--template");
    let FlagKind::Dropdown(bundle_opts) = &bundle_flag.kind else {
        panic!("bundle --template must be a Dropdown");
    };
    assert_eq!(
        bundle_opts.len(),
        10,
        "bundle --template must stay 10 values — the sentinel is not shared"
    );
    assert_eq!(
        &opts[..opts.len() - 1],
        *bundle_opts,
        "restore's real values must mirror the shared TEMPLATES in order"
    );

    // Virgin settled form materializes --template == "bip44" (NOT "") — the
    // gallery-PNG-stable steady state (mirrors export-wallet's append pin).
    let mut h = render_whole_form_harness(CliTab::Mnemonic, sub, FormState::default());
    h.run();
    h.run();
    assert_eq!(
        template_value(h.state()),
        Some(FlagValue::Dropdown("bip44".into())),
        "virgin settled restore must materialize --template == \"bip44\""
    );
}
