//! v0.34.0 (audit I5 + I6) — persistence-redaction hardening pins.
//!
//! SPEC: `design/SPEC_gui_v0_34_0_persist_redaction_hardening.md`.
//!
//! - T1 — the redaction BELT: `redact_for_persistence` drops ALL
//!   `state.positionals` content (it has no subcommand context, so the
//!   belt is unconditional / fail-closed), and a secret positional typed
//!   via the widget path never appears in the serialized form state.
//! - T2 — secret positionals emit from `secret_widgets["positional:<name>"]`
//!   rows at argv end; blank rows emit nothing; a stale `state.positionals`
//!   entry for a secret positional does NOT emit (widget path authoritative).
//! - T3 — `should_confirm_run` fires on a filled secret positional.
//! - T4 — frozen secret-positional census (non-circular): exactly the 5
//!   ms.rs entries are `secret: true` across all four CLI schemas.
//! - T5 — the tree persist walk keeps ONLY extended-public-prefixed
//!   key/keys entries (positive allowlist; WIF / raw-hex / `Kpub…` /
//!   garbage all blanked — fail-closed, no `from_str` backstop exists on
//!   the persist path).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::{self, FormState};
use mnemonic_gui::secrets::should_confirm_run;

fn sub<'a>(s: &'a schema::Schema, name: &str) -> &'a schema::SubcommandSchema {
    s.subcommands
        .iter()
        .find(|x| x.name == name)
        .unwrap_or_else(|| panic!("subcommand `{name}` not found"))
}

// ── T1 — redaction belt ─────────────────────────────────────────────────────

#[test]
fn t1_redact_drops_all_positionals() {
    let mut state = FormState::default();
    state.positionals.push("md1qqqq-example-nonsecret".into());
    state.positionals.push("second".into());
    let redacted = redact_for_persistence(&state);
    assert!(
        redacted.positionals.is_empty(),
        "redact_for_persistence must drop ALL positionals (no subcommand \
         context at this layer → unconditional fail-closed belt; audit I5)"
    );
}

#[test]
fn t1b_secret_positional_widget_value_never_serializes() {
    let mut state = FormState::default();
    state.secret_widgets.insert(
        "positional:shares".into(),
        vec![SecretLineEdit::from_text("ms12fakesharevalueabcdef")],
    );
    let redacted = redact_for_persistence(&state);
    let json = serde_json::to_string(&redacted).expect("FormState serializes");
    assert!(
        !json.contains("ms12fakesharevalueabcdef"),
        "a secret positional typed via the widget path must never reach \
         the serialized form state (secret_widgets is #[serde(skip)])"
    );
}

// ── T2 — assembler routing ──────────────────────────────────────────────────

#[test]
fn t2_secret_positional_emits_from_widget_rows() {
    let combine = sub(&schema::ms::SCHEMA, "combine");
    let mut state = FormState::default();
    state.secret_widgets.insert(
        "positional:shares".into(),
        vec![
            SecretLineEdit::from_text("ms1share-a"),
            SecretLineEdit::new(), // blank row emits nothing
            SecretLineEdit::from_text("ms1share-b"),
        ],
    );
    let argv = assemble_argv(&schema::ms::SCHEMA, combine, &state);
    let a = argv.iter().position(|x| x == "ms1share-a");
    let b = argv.iter().position(|x| x == "ms1share-b");
    assert!(
        a.is_some() && b.is_some() && a < b,
        "filled secret-positional rows must emit in row order at argv end; got {argv:?}"
    );
    assert_eq!(
        argv.iter().filter(|x| x.starts_with("ms1share")).count(),
        2,
        "blank rows must emit nothing; got {argv:?}"
    );
}

#[test]
fn t2b_stale_state_positionals_do_not_emit_for_secret_positional() {
    let combine = sub(&schema::ms::SCHEMA, "combine");
    let mut state = FormState::default();
    // A values-style stale entry (e.g. hand-synthesized) for a SECRET
    // positional must be ignored — the widget path is authoritative.
    state.positionals.push("ms1stale-should-not-emit".into());
    let argv = assemble_argv(&schema::ms::SCHEMA, combine, &state);
    assert!(
        !argv.contains(&"ms1stale-should-not-emit".to_string()),
        "state.positionals content must NOT emit for a subcommand whose \
         positional is secret (widget path authoritative); got {argv:?}"
    );
}

#[test]
fn t2c_non_secret_positionals_keep_emitting_from_state() {
    let decode = sub(&schema::md::SCHEMA, "decode");
    let mut state = FormState::default();
    state.positionals.push("md1qexample".into());
    let argv = assemble_argv(&schema::md::SCHEMA, decode, &state);
    assert!(
        argv.contains(&"md1qexample".to_string()),
        "non-secret positionals keep the state.positionals path; got {argv:?}"
    );
}

// ── T3 — run-confirm ────────────────────────────────────────────────────────

#[test]
fn t3_filled_secret_positional_triggers_run_confirm() {
    let combine = sub(&schema::ms::SCHEMA, "combine");
    let mut state = FormState::default();
    assert!(
        !should_confirm_run(combine, &state),
        "empty form must not confirm"
    );
    state.secret_widgets.insert(
        "positional:shares".into(),
        vec![SecretLineEdit::from_text("ms1share-a")],
    );
    assert!(
        should_confirm_run(combine, &state),
        "a filled secret positional must trigger the run-confirm modal (audit I5)"
    );
}

// ── T4 — frozen census ──────────────────────────────────────────────────────

#[test]
fn t4_secret_positional_census_matches_frozen_literal() {
    const FROZEN: &[(&str, &str, &str)] = &[
        ("ms", "combine", "shares"),
        ("ms", "decode", "ms1"),
        ("ms", "derive", "ms1"),
        ("ms", "inspect", "ms1"),
        ("ms", "verify", "ms1"),
    ];
    let mut live: Vec<(String, String, String)> = Vec::new();
    for (cli, s) in [
        ("mnemonic", &schema::mnemonic::SCHEMA),
        ("md", &schema::md::SCHEMA),
        ("ms", &schema::ms::SCHEMA),
        ("mk", &schema::mk::SCHEMA),
    ] {
        for sc in s.subcommands {
            assert!(
                sc.positional_args.len() <= 1,
                "({cli}, {}): the assembler + render index logic assume \
                 every positional table has <= 1 entry — a 2-entry table \
                 (esp. mixed secret/non-secret) needs an assembler redesign",
                sc.name
            );
            for pos in sc.positional_args {
                if pos.secret {
                    live.push((cli.into(), sc.name.into(), pos.name.into()));
                }
            }
        }
    }
    live.sort();
    let frozen: Vec<(String, String, String)> = FROZEN
        .iter()
        .map(|(a, b, c)| ((*a).into(), (*b).into(), (*c).into()))
        .collect();
    assert_eq!(
        live, frozen,
        "secret-positional census diverged from the frozen literal — \
         update BOTH the schema tables and this list consciously"
    );
}

// ── T5 — tree persist allowlist ─────────────────────────────────────────────

#[test]
fn t5_tree_persist_keeps_only_extended_public_keys() {
    use mnemonic_gui::form::tree_model::{TreeNode, TreeState};

    const KEPT: &[&str] = &[
        "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8",
        "tpubD6NzVbkrYhZ4XgiXtGrdW5XDAPFCL9h7we1vwNCpn8tGbBcgfVYjXyhWo4E1xkh56hjod1RhGjxbaTLV3X4FyWuejifB9jusQ46QzG87VKp",
        "[deadbeef/84'/0'/0']xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8",
        "zpub6jftahH18ngZxLmXaKw3GSZzZsszmt9WqedkyZdezFtWRFBZqsQH5hyUmb4pCEeZGmVfQuP5bedXTB8is6fTv19U1GQRyQUKQGUTzyHACMF",
    ];
    // WIF / raw-hex / Kpub-form / xprv / garbage — ALL blanked. The two hex
    // rows are the documented fail-closed cost: a raw-hex x-only/compressed
    // PUBLIC key is shape-indistinguishable from a raw-hex PRIVATE key, and
    // the persist path has no from_str backstop — so hex blanks too.
    const BLANKED: &[&str] = &[
        "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi",
        "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn",
        "L1aW4aubDFB7yfras2S1mN3bqg9nwySY8nkoLmJebSLD5BWv3ENZ",
        "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN8rFTv2sfUK",
        "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ",
        "9213qJab2HNEpMpYNBa7wHGFKKbkDn24jpANDs2huN3yi4J11ko",
        "Kpub2HNEpMpYNBa7wHGFKKbkDn24jpANDs2huN3yi4J11koAbB7L",
        "0000000000000000000000000000000000000000000000000000000000000001",
        "02deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        "not-a-key",
    ];

    let child = TreeNode {
        key: BLANKED[0].to_string(),
        keys: BLANKED[1..].iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    };
    let root = TreeNode {
        key: KEPT[0].to_string(),
        keys: KEPT[1..].iter().map(|s| s.to_string()).collect(),
        children: vec![child],
        ..Default::default()
    };
    let state = TreeState {
        root,
        ..Default::default()
    };
    let redacted = state.redacted_for_persistence();

    assert_eq!(redacted.root.key, KEPT[0], "extended-public key must survive");
    assert_eq!(
        redacted.root.keys,
        KEPT[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "extended-public keys list must survive"
    );
    assert!(
        redacted.root.children[0].key.is_empty(),
        "xprv must blank"
    );
    for (i, entry) in redacted.root.children[0].keys.iter().enumerate() {
        assert!(
            entry.is_empty(),
            "BLANKED[{}] (`{}…`) survived the persist walk — the positive \
             allowlist must clear every non-extended-public key/keys entry",
            i + 1,
            &BLANKED[i + 1][..8.min(BLANKED[i + 1].len())]
        );
    }
}
