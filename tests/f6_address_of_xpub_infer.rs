//! F6 — `xpub-search-address-of-xpub` virgin-dropdown inference sentinel
//! (constellation-eval F6; SPEC `SPEC_f5_f6_gui_recovery_wiring.md`).
//!
//! Mirrors `tests/restore_template_none.rs`, but the OPPOSITE polarity: restore
//! APPENDED `""` (keeping a sensible `bip44` virgin default); F6 PREPENDS `""`
//! at `opts[0]` for `--address-type` / `--network` because there is NO safe
//! concrete default — the toolkit MUST infer the script-type / network from the
//! xpub's SLIP-0132 prefix. A virgin form must therefore emit NEITHER flag (a
//! forced `p2pkh` / `mainnet` overrides a correct inferred `p2wpkh`, turning an
//! ownership MATCH into a false-negative "no match" — a funds-safety bug).
//!
//! Cells:
//!   1. virgin whole form → argv carries NO `--address-type` / `--network`, and
//!      both dropdowns materialize `Dropdown("")` (the PREPEND steady state);
//!   2. virgin masked copy-command (both flavors) carries neither flag nor a
//!      `""` artifact;
//!   3. inverse — selecting a real `p2wpkh` DOES emit `--address-type p2wpkh`;
//!   4. ★ PREPEND census — `_INFER[0] == ""` for both dropdowns (the tripwire
//!      opposite of restore's APPEND pin), real values follow in order.

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::{
    assemble_argv, assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::schema::{FlagKind, FlagValue, FormState};

use ui_harness::{flag_of, render_flag_harness, render_whole_form_harness, schema_for, sub_of};

fn addr_sub() -> &'static mnemonic_gui::schema::SubcommandSchema {
    sub_of(CliTab::Mnemonic, "xpub-search-address-of-xpub")
}

/// A VIRGIN whole `address-of-xpub` form, settled so opts[0] materialization
/// has written the two inference dropdowns into `state.values`.
fn virgin_form() -> Harness<'static, FormState> {
    let mut h = render_whole_form_harness(CliTab::Mnemonic, addr_sub(), FormState::default());
    h.run();
    h.run(); // settle: virgin materialization of opts[0] into state.values
    h
}

fn argv_of(state: &FormState) -> Vec<String> {
    assemble_argv(schema_for(CliTab::Mnemonic), addr_sub(), state)
}

// ── 1. virgin form emits neither inference flag; both materialize Dropdown("") ─

#[test]
fn virgin_form_emits_no_inference_flags() {
    let h = virgin_form();
    // The two inference dropdowns materialized the PREPENDED sentinel.
    assert_eq!(
        h.state().dropdown_value("--address-type"),
        Some(""),
        "virgin --address-type must materialize the Dropdown(\"\") inference sentinel"
    );
    assert_eq!(
        h.state().dropdown_value("--network"),
        Some(""),
        "virgin --network must materialize the Dropdown(\"\") inference sentinel"
    );

    let argv = argv_of(h.state());
    assert!(
        !argv.iter().any(|a| a == "--address-type"),
        "virgin form must carry NO --address-type (toolkit infers); got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "--network"),
        "virgin form must carry NO --network (toolkit infers); got {argv:?}"
    );
    // No `""` sentinel token leaks into argv either.
    assert!(
        argv.iter().all(|t| !t.is_empty()),
        "no empty-string token may leak into argv; got {argv:?}"
    );
}

// ── 2. virgin masked copy-command (both flavors) — neither flag, no "" artifact ─

#[test]
fn virgin_masked_copy_carries_no_inference_flags_nor_empty_artifact() {
    let h = virgin_form();
    let (argv, mask) =
        assemble_argv_with_secret_mask(schema_for(CliTab::Mnemonic), addr_sub(), h.state());

    let posix = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    assert!(
        !posix.contains("--address-type") && !posix.contains("--network"),
        "posix copy must carry neither inference flag; got {posix:?}"
    );
    assert!(
        !posix.contains("''"),
        "posix copy must carry no empty-string artifact; got {posix:?}"
    );

    let windows = render_copy_command_masked(&argv, &mask, ShellFlavor::WindowsCmd);
    assert!(
        !windows.contains("--address-type") && !windows.contains("--network"),
        "windows copy must carry neither inference flag; got {windows:?}"
    );
    assert!(
        !windows.contains("\"\""),
        "windows copy must carry no empty-string artifact; got {windows:?}"
    );
}

// ── 3. inverse — a real selection DOES emit ─────────────────────────────────

/// Select `opt` in the single `--address-type` combo (unique ComboBox in the
/// single-flag render); mirrors restore_template_none's `select_template`.
fn select(h: &mut Harness<'static, FormState>, opt: &str) {
    if h.query_by_label(opt).is_none() {
        h.get_by_role(Role::ComboBox).click();
        h.run();
        h.run();
    }
    h.get_by_label(opt).click();
    h.run();
    h.run();
}

#[test]
fn selecting_a_real_address_type_emits() {
    let flag = flag_of(addr_sub(), "--address-type");
    let mut h = render_flag_harness(CliTab::Mnemonic, addr_sub(), flag, FormState::default());
    h.run();
    h.run(); // settle: virgin materialization of opts[0] (= "") into state.values

    // Virgin single-flag form emits nothing for --address-type.
    let argv0 = argv_of(h.state());
    assert!(
        !argv0.iter().any(|a| a == "--address-type"),
        "virgin --address-type must not emit; got {argv0:?}"
    );

    select(&mut h, "p2wpkh");
    let argv = argv_of(h.state());
    assert!(
        argv.windows(2).any(|w| w == ["--address-type", "p2wpkh"]),
        "selecting p2wpkh must emit --address-type p2wpkh; got {argv:?}"
    );
}

// ── 4. ★ PREPEND census — the tripwire opposite of restore's APPEND pin ──────

#[test]
fn infer_dropdowns_prepend_census() {
    let sub = addr_sub();

    let FlagKind::Dropdown(at_opts) = &flag_of(sub, "--address-type").kind else {
        panic!("--address-type must be a Dropdown");
    };
    assert_eq!(
        at_opts.first().copied(),
        Some(""),
        "PREPEND pin: --address-type opts[0] must be the \"\" inference sentinel"
    );
    assert_eq!(
        *at_opts,
        &["", "p2pkh", "p2sh-p2wpkh", "p2wpkh", "p2tr"],
        "--address-type INFER const must be \"\"-prepended + the 4 real types in order"
    );

    let FlagKind::Dropdown(net_opts) = &flag_of(sub, "--network").kind else {
        panic!("--network must be a Dropdown");
    };
    assert_eq!(
        net_opts.first().copied(),
        Some(""),
        "PREPEND pin: --network opts[0] must be the \"\" inference sentinel"
    );
    assert_eq!(
        *net_opts,
        &["", "mainnet", "testnet", "signet", "regtest"],
        "--network INFER const must be \"\"-prepended + the 4 real networks in order"
    );

    // Both flags keep default_value == None (the sentinel is the materialized
    // opts[0], NOT a declared default).
    assert!(
        flag_of(sub, "--address-type").default_value.is_none(),
        "--address-type must keep default_value: None"
    );
    assert!(
        flag_of(sub, "--network").default_value.is_none(),
        "--network must keep default_value: None"
    );

    // The virgin settled form materializes Dropdown("") for BOTH (not a
    // concrete p2pkh/mainnet) — the funds-safety steady state.
    let h = virgin_form();
    assert_eq!(
        h.state()
            .values
            .iter()
            .find(|(k, _)| k == "--address-type")
            .map(|(_, v)| v.clone()),
        Some(FlagValue::Dropdown(String::new())),
        "virgin settled --address-type must be Dropdown(\"\")"
    );
    assert_eq!(
        h.state()
            .values
            .iter()
            .find(|(k, _)| k == "--network")
            .map(|(_, v)| v.clone()),
        Some(FlagValue::Dropdown(String::new())),
        "virgin settled --network must be Dropdown(\"\")"
    );
}
