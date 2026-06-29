//! Phase P1 — I1 form→argv wiring round-trip on a VERTICAL SLICE.
//!
//! Plan: `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` Phase P1.
//! Spec:  `design/SPEC_gui_automated_ui_test_harness.md` §4 (WIRING oracle),
//!        §5 I1 (render-via-kittest MANDATORY + injection discipline).
//!
//! The I1 invariant: render the under-test flag through the REAL form-level
//! per-flag path (`render_with_dispatch`), **widget-inject** a distinguishable
//! value through the rendered widget (base `FormState` seeds CONTEXT only,
//! never the asserted value — §5 IMP-3), run to stable, `assemble_argv`, and
//! assert the **injected value is argv-bound to its flag**.
//!
//! Anti-tautology (§4): the assertion's substance is the **value** the widget
//! produced, NOT the flag-NAME set (owned by `schema_mirror`). The flag name is
//! used only as the anchor to locate the value token (and, for `Boolean`, the
//! before/after absent→present check proves the *toggle wires to emission*, not
//! that the name is a valid schema entry).
//!
//! ## Slice (≥1 subcommand per CLI; together exercise all 5 identity kinds)
//! | CLI       | subcommand   | flag             | kind     |
//! |-----------|--------------|------------------|----------|
//! | mnemonic  | `addresses`  | `--from`         | Text     |
//! | mnemonic  | `addresses`  | `--account`      | Number   |
//! | mnemonic  | `addresses`  | `--network`      | Dropdown |
//! | mnemonic  | `addresses`  | `--json`         | Boolean  |
//! | md        | `vectors`    | `--out`          | Path     |
//! | ms        | `decode`     | `--language`     | Dropdown |
//! | mk        | `address`    | `--address-type` | Dropdown |
//!
//! `mnemonic addresses`, `md vectors`, `ms decode`, `mk address` all declare
//! `conditional: None`, so `assemble_argv`'s visibility gate cannot suppress
//! the under-test flag — the round-trip observes the pure render→store→argv
//! wiring. (Whole-form conditional *interaction* is P2/I2.)

mod ui_harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;

use ui_harness::{
    base_state, drive, flag_of, identity_flags, identity_kind, schema_for, sub_of, IdentityKind,
    Injected,
};

/// The §5 I1 round-trip on one `(tab, subcommand, identity-flag)` cell.
fn assert_roundtrip(tab: CliTab, sub_name: &str, flag_name: &str, injected: Injected) {
    let schema = schema_for(tab);
    let sub = sub_of(tab, sub_name);
    let flag = flag_of(sub, flag_name);

    // Tie the cell to the ENUMERATOR: the under-test flag must be one
    // `identity_flags` yields — proving it is identity-mapped, non-secret and
    // non-repeating (so the values-routed drive below is the right path).
    let kind = identity_flags(sub)
        .find(|(f, _)| f.name == flag_name)
        .map(|(_, k)| k)
        .unwrap_or_else(|| {
            panic!("enumerator does not yield `{}` `{flag_name}` as an identity flag", sub.name)
        });

    // Base seeds CONTEXT only; strip the under-test flag so its value is solely
    // widget-injected (§5 IMP-3 injection discipline).
    let mut base = base_state(tab, sub_name);
    base.values.retain(|(k, _)| k != flag_name);
    base.secret_widgets.remove(flag_name);

    let mut harness = ui_harness::render_flag_harness(tab, sub, flag, base);
    harness.run();

    let argv_before = assemble_argv(schema, sub, harness.state());

    // Widget-inject through the rendered control.
    drive(&mut harness, kind, &injected);

    let argv = assemble_argv(schema, sub, harness.state());

    match injected.expected_token() {
        // Value kinds (Text / Path / Number / Dropdown): the injected value
        // must appear bound immediately after the flag token.
        Some(expected) => {
            let pos = argv
                .iter()
                .position(|t| t == flag_name)
                .unwrap_or_else(|| panic!("argv has no `{flag_name}` token: {argv:?}"));
            assert_eq!(
                argv.get(pos + 1),
                Some(&expected),
                "`{flag_name}` must bind the widget-injected value `{expected}` in argv: {argv:?}"
            );
        }
        // Boolean: presence-only emission. Prove the checkbox TOGGLE wires to
        // emission — absent before driving, present after.
        None => {
            assert!(
                !argv_before.contains(&flag_name.to_string()),
                "`{flag_name}` must be absent before the checkbox is driven: {argv_before:?}"
            );
            assert!(
                argv.contains(&flag_name.to_string()),
                "driving the checkbox on must emit `{flag_name}`: {argv:?}"
            );
        }
    }
}

// ─── the 7 slice cells (one #[test] each for failure localization) ──────────

#[test]
fn mnemonic_addresses_from_text_roundtrip() {
    assert_roundtrip(
        CliTab::Mnemonic,
        "addresses",
        "--from",
        Injected::Text("seed-source-ref-xyz"),
    );
}

#[test]
fn mnemonic_addresses_account_number_roundtrip() {
    // 4242 ≠ the schema default "0" and is well within `--account`'s
    // [0, 2_147_483_647] range, so it survives DragValue clamping AND the
    // `is_at_default` suppression. A GREEN result here is empirical proof the
    // AccessKit `SetValue` flowed through the DragValue's own value-writing
    // into `FormState` (which is all `assemble_argv` reads) — NOT a bypass.
    assert_roundtrip(
        CliTab::Mnemonic,
        "addresses",
        "--account",
        Injected::Number(4242),
    );
}

#[test]
fn mnemonic_addresses_network_dropdown_roundtrip() {
    // opts[0] == "mainnet" (the rendered default); inject the distinct "regtest".
    assert_roundtrip(
        CliTab::Mnemonic,
        "addresses",
        "--network",
        Injected::Dropdown("regtest"),
    );
}

#[test]
fn mnemonic_addresses_json_boolean_roundtrip() {
    assert_roundtrip(
        CliTab::Mnemonic,
        "addresses",
        "--json",
        Injected::Boolean(true),
    );
}

#[test]
fn md_vectors_out_path_roundtrip() {
    assert_roundtrip(
        CliTab::Md,
        "vectors",
        "--out",
        Injected::Text("/tmp/vectors-out-dir"),
    );
}

#[test]
fn ms_decode_language_dropdown_roundtrip() {
    // LANG_MS opts[0] == "english"; inject the distinct "korean".
    assert_roundtrip(
        CliTab::Ms,
        "decode",
        "--language",
        Injected::Dropdown("korean"),
    );
}

#[test]
fn mk_address_address_type_dropdown_roundtrip() {
    // opts[0] == "p2pkh"; inject the distinct "p2tr".
    assert_roundtrip(
        CliTab::Mk,
        "address",
        "--address-type",
        Injected::Dropdown("p2tr"),
    );
}

// ─── slice-level coverage + enumerator sanity ───────────────────────────────

/// The slice must collectively exercise ALL FIVE identity kinds (the P0
/// spike-approved reach). This pins the slice against silent kind-coverage
/// regression.
#[test]
fn slice_covers_all_five_identity_kinds() {
    let slice: &[(CliTab, &str, &str)] = &[
        (CliTab::Mnemonic, "addresses", "--from"),
        (CliTab::Mnemonic, "addresses", "--account"),
        (CliTab::Mnemonic, "addresses", "--network"),
        (CliTab::Mnemonic, "addresses", "--json"),
        (CliTab::Md, "vectors", "--out"),
        (CliTab::Ms, "decode", "--language"),
        (CliTab::Mk, "address", "--address-type"),
    ];
    let mut kinds: std::collections::BTreeSet<IdentityKind> = Default::default();
    let mut clis: std::collections::BTreeSet<&str> = Default::default();
    for (tab, sub_name, flag_name) in slice {
        let sub = sub_of(*tab, sub_name);
        let flag = flag_of(sub, flag_name);
        kinds.insert(
            identity_kind(&flag.kind).expect("slice flags must be identity-mapped"),
        );
        clis.insert(tab.bin_name());
    }
    assert_eq!(
        kinds,
        [
            IdentityKind::Text,
            IdentityKind::Number,
            IdentityKind::Dropdown,
            IdentityKind::Boolean,
            IdentityKind::Path,
        ]
        .into_iter()
        .collect(),
        "the P1 slice must exercise all five identity kinds"
    );
    assert_eq!(clis.len(), 4, "the slice must cover all four CLI tabs");
}

/// The enumerator must yield ONLY identity-mapped, non-secret, non-repeating
/// flags — across every subcommand of all four CLIs (cheap; no rendering).
#[test]
fn enumerator_yields_only_identity_nonsecret_nonrepeating() {
    let mut total = 0usize;
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            for (flag, kind) in identity_flags(sub) {
                total += 1;
                assert_eq!(
                    identity_kind(&flag.kind),
                    Some(kind),
                    "{}/{} `{}`: enumerator kind disagrees with classifier",
                    tab.bin_name(),
                    sub.name,
                    flag.name
                );
                assert!(
                    !flag.repeating,
                    "{}/{} `{}`: enumerator must not yield repeating flags",
                    tab.bin_name(),
                    sub.name,
                    flag.name
                );
                assert!(
                    !mnemonic_gui::secrets::flag_is_secret(flag),
                    "{}/{} `{}`: enumerator must not yield secret flags",
                    tab.bin_name(),
                    sub.name,
                    flag.name
                );
            }
        }
    }
    assert!(total > 0, "enumerator yielded no identity flags across the constellation");
}

/// The enumerator must EXCLUDE known secret flags even where they are present
/// (e.g. `mnemonic addresses --passphrase`) — the secret routing/persistence
/// surface is I3/P3, not the values-routed I1 drive.
#[test]
fn enumerator_excludes_secret_passphrase() {
    let sub = sub_of(CliTab::Mnemonic, "addresses");
    assert!(
        sub.flags.iter().any(|f| f.name == "--passphrase"),
        "precondition: `mnemonic addresses` declares `--passphrase`"
    );
    assert!(
        identity_flags(sub).all(|(f, _)| f.name != "--passphrase"),
        "enumerator must exclude the secret `--passphrase`"
    );
}
