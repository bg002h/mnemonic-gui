//! Fold 1 of the phase-2 whole-diff review
//! (`mnemonic-engrave/design/agent-reports/gui-impl-phase2-review.md`).
//!
//! - M1: `ms hashlock` Copy is gated exactly as Run is while `--kind` is
//!   "(choose)" — pure and through the window.
//! - M2: a private key pasted into a public md field (DESIGN §B2–B4) rides
//!   argv, so the Copy buttons say "— reveals secret" and Run opens the
//!   confirm dialog, which does not claim the run is private.
//! - N1: binding labels drop a nested subcommand's child word.

use std::path::PathBuf;

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::form::channels::copy::{copy_commands, copy_reveals_secret, CopyState};
use mnemonic_gui::form::channels::{self};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::path_detect::Detected;
use mnemonic_gui::schema::{self, FlagValue, FormState, Schema, SubcommandSchema};

const XPRV: &str = "xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu";
const XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

fn sub(schema: &'static Schema, name: &str) -> &'static SubcommandSchema {
    schema.subcommands.iter().find(|s| s.name == name).unwrap()
}

fn none(_: &str) -> Option<String> {
    None
}

fn hashlock(kind: &str) -> FormState {
    let mut st = FormState::from_pairs(vec![("--kind", FlagValue::Dropdown(kind.into()))]);
    st.secret_widgets.insert(
        "--hashlock-phrase".into(),
        vec![SecretLineEdit::from_text("pad")],
    );
    st
}

fn shape_key(descriptor: &str) -> FormState {
    FormState::from_pairs(vec![("--descriptor", FlagValue::Text(descriptor.into()))])
}

// ── M1 ────────────────────────────────────────────────────────────────────

#[test]
fn m1_hashlock_copy_is_gated_like_run_at_choose() {
    let ms = &schema::ms::SCHEMA;
    let (posix, windows) = copy_commands(ms, sub(ms, "hashlock"), &hashlock(""), &none);
    for c in [posix, windows] {
        match c {
            CopyState::Disabled(t) => assert!(t.contains("choose a --kind first"), "{t}"),
            other => panic!("Copy must be disabled at (choose): {other:?}"),
        }
    }
    let (posix, _) = copy_commands(ms, sub(ms, "hashlock"), &hashlock("sha256"), &none);
    assert!(
        posix.text().is_some_and(|t| t.contains("--kind sha256")),
        "{posix:?}"
    );
    let (posix, _) = copy_commands(
        ms,
        sub(ms, "hashlock"),
        &hashlock(schema::ms::HASHLOCK_KIND_ALL),
        &none,
    );
    assert!(
        posix.text().is_some(),
        "all kinds is a deliberate choice: {posix:?}"
    );
}

fn found() -> AppState {
    let f = |p: &str| Detected::Found(PathBuf::from(p));
    AppState {
        mnemonic_detect: f("/pinned/mnemonic"),
        md_detect: f("/pinned/md"),
        ms_detect: f("/pinned/ms"),
        mk_detect: f("/pinned/mk"),
        active_tab: CliTab::Mnemonic,
    }
}

fn app(tab: CliTab, sub: &str, st: FormState) -> Harness<'static, MnemonicGuiApp> {
    let mut app = MnemonicGuiApp::new_headless(found(), None, None);
    app.app_state.active_tab = tab;
    app.active_subcommand.insert(tab, sub.to_string());
    app.form_state
        .insert(format!("{}:{sub}", tab.bin_name()), st);
    Harness::builder()
        .with_size(egui::Vec2::new(1400.0, 2400.0))
        .with_max_steps(64)
        .build_state(|ctx, app: &mut MnemonicGuiApp| app.ui(ctx), app)
}

fn texts<S>(h: &Harness<'static, S>) -> String {
    h.query_all(egui_kittest::kittest::By::new())
        .filter_map(|n| n.value().or_else(|| n.label()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn m1_window_copy_buttons_are_disabled_at_choose() {
    let mut h = app(CliTab::Ms, "hashlock", hashlock(""));
    h.run();
    assert!(h.get_by_label("Copy command (POSIX)").is_disabled());
    assert!(h.get_by_label("Copy command (Windows)").is_disabled());
    assert!(h.get_by_label("Run").is_disabled());
    let mut h = app(CliTab::Ms, "hashlock", hashlock("sha256"));
    h.run();
    assert!(!h.get_by_label("Copy command (POSIX)").is_disabled());
}

// ── M2 ────────────────────────────────────────────────────────────────────

#[test]
fn m2_copy_reveals_a_pasted_private_key_and_says_so() {
    let md = &schema::md::SCHEMA;
    let s = sub(md, "shape-key");
    let priv_ = shape_key(&format!("wpkh({XPRV}/0/*)"));
    assert!(copy_reveals_secret(md, s, &priv_, &none));
    let (posix, _) = copy_commands(md, s, &priv_, &none);
    assert!(
        posix.text().is_some_and(|t| t.contains(XPRV)),
        "Copy carries it: {posix:?}"
    );
    let publ = shape_key(&format!("wpkh({XPUB}/0/*)"));
    assert!(!copy_reveals_secret(md, s, &publ, &none));
    // planner sources never make Copy reveal
    let ms = &schema::ms::SCHEMA;
    assert!(!copy_reveals_secret(
        ms,
        sub(ms, "hashlock"),
        &hashlock("sha256"),
        &none
    ));
}

#[test]
fn m2_window_labels_copy_and_confirms_run_without_claiming_privacy() {
    let mut h = app(
        CliTab::Md,
        "shape-key",
        shape_key(&format!("wpkh({XPRV}/0/*)")),
    );
    h.run();
    assert!(
        h.query_by_label("Copy command (POSIX) — reveals secret")
            .is_some(),
        "{}",
        texts(&h)
    );
    assert!(h
        .query_by_label("Copy command (Windows) — reveals secret")
        .is_some());
    h.get_by_label("Run").click();
    h.run();
    assert!(
        h.state().last_run.is_none(),
        "Run must stop at the confirm dialog"
    );
    let all = texts(&h);
    assert!(
        all.contains("This invocation passes secret-bearing arguments to md:"),
        "{all}"
    );
    assert!(!all.contains("privately"), "{all}");
    assert!(!all.contains(XPRV), "the key stays masked in the dialog");
    // a public descriptor: no reveal label, no dialog
    let mut h = app(
        CliTab::Md,
        "shape-key",
        shape_key(&format!("wpkh({XPUB}/0/*)")),
    );
    h.run();
    assert!(h.query_by_label("Copy command (POSIX)").is_some());
    assert!(h
        .query_by_label("Copy command (POSIX) — reveals secret")
        .is_none());
}

// ── N1 ────────────────────────────────────────────────────────────────────

#[test]
fn n1_binding_lines_drop_a_nested_subcommands_child_word() {
    let m = &schema::mnemonic::SCHEMA;
    let mut st = FormState::default();
    st.secret_widgets.insert(
        "--share".into(),
        vec![
            SecretLineEdit::from_text("share one"),
            SecretLineEdit::from_text("share two"),
        ],
    );
    let p = channels::plan(m, sub(m, "slip39-combine"), &st, &none, "linux").unwrap();
    let line = p.bindings[0].line();
    assert!(line.starts_with("--share ← env MNEMONIC_GUI_S0"), "{line}");
    // §A5 / T8 keep plan.describe's spelling
    assert_eq!(p.bindings[0].binding.source_label(), "combine --share");
    let (posix, _) = copy_commands(m, sub(m, "slip39-combine"), &st, &none);
    let t = posix.text().unwrap();
    assert!(
        t.contains("# --share: IFS= read -rs") && !t.contains("# combine --share"),
        "{t}"
    );
    // non-nested, positional and slot keys are unchanged
    let ms = &schema::ms::SCHEMA;
    let mut st = FormState::default();
    st.secret_widgets.insert(
        "positional:shares".into(),
        vec![
            SecretLineEdit::from_text("ms12a"),
            SecretLineEdit::from_text("ms12b"),
        ],
    );
    let p = channels::plan(ms, sub(ms, "combine"), &st, &none, "linux").unwrap();
    assert!(
        p.bindings[0].line().starts_with("<shares> ← "),
        "{}",
        p.bindings[0].line()
    );
}
