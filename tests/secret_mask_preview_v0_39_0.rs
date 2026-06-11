//! v0.39.0 (Item 1) — secret VALUES are masked in every on-screen command
//! display (preview / run-confirm modal / last-run argv); the copy buttons
//! still reveal the real runnable command (the deliberate-reveal half of
//! decision (d)).
//!
//! SPEC: `design/SPEC_gui_v0_39_0_mask_preview_paste_warn.md`.
//!
//! The mask covers the FOUR secret argv-token sources `should_confirm_run`
//! classifies — proven per-source here:
//!   (a) secret Text flag value      — `bundle --passphrase`
//!   (b) secret slot row value token — `bundle --slot @0.phrase=…`
//!   (c) secret positional value     — `ms combine <shares>`
//!   (d) secret composite value      — `seed-xor-combine --share phrase=…`
//!   (e) value-dependent composite   — `convert --from phrase=…` (masked) vs
//!                                      `convert --from xpub=…` (NOT masked)
//!
//! - T-A1: per-source mask correctness (index masked true; flag name + a
//!   co-present non-secret value masked false; `mask.len()==argv.len()`).
//! - T-A2: the masked render hides every secret + shows `SECRET_MASK`; the
//!   real render still reveals it (reveal path intact). `SECRET_MASK` is
//!   un-quoted (a display sentinel).
//! - T-A3: the DANGEROUS direction — every state with `mask.any()` also has
//!   `should_confirm_run == true` (no secret renders masked while the run
//!   skips the confirm). Plus the documented safe asymmetry.
//! - T-A4: the preview STRING the action-bar label renders (main.rs:990) is
//!   masked. The bin-crate action bar is not harness-isolable, so this asserts
//!   at the `render_copy_command_masked` seam (recorded per SPEC T-A4).

use mnemonic_gui::form::invocation::{
    assemble_argv_with_secret_mask, render_copy_command, render_copy_command_masked, ShellFlavor,
    SECRET_MASK,
};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::schema::{self, FlagValue, FormState, SubcommandSchema};
use mnemonic_gui::secrets::should_confirm_run;

// Distinctive, space-free sentinel secret values (so `contains` checks are
// unambiguous and shell-quoting never splits/quotes them).
const PASS_SECRET: &str = "PASSPHRASEsecret0001";
const SEED_SECRET: &str = "SLOTPHRASEsecret0002";
const SHARE_POS_SECRET: &str = "ms1POSITIONALsecret0003";
const SHARE_COMPOSITE_SECRET: &str = "SHAREcompositesecret0004";
const CONVERT_SEED_SECRET: &str = "CONVERTphrasesecret0005";
const WATCH_XPUB: &str = "xpubWATCHONLY0006";

fn sub_mnemonic(name: &str) -> &'static SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("mnemonic subcommand {name} not in schema"))
}

fn sub_ms(name: &str) -> &'static SubcommandSchema {
    schema::ms::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("ms subcommand {name} not in schema"))
}

/// Index of the unique argv token equal to `needle` (the value tokens here are
/// whole tokens, never substrings of another).
fn idx_eq(argv: &[String], needle: &str) -> usize {
    let hits: Vec<usize> = argv
        .iter()
        .enumerate()
        .filter(|(_, t)| t.as_str() == needle)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(hits.len(), 1, "expected exactly one token == {needle:?}; argv={argv:?}");
    hits[0]
}

/// Index of the unique argv token CONTAINING `needle` (for composite/slot
/// tokens like `@0.phrase=<value>` / `phrase=<value>`).
fn idx_contains(argv: &[String], needle: &str) -> usize {
    let hits: Vec<usize> = argv
        .iter()
        .enumerate()
        .filter(|(_, t)| t.contains(needle))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(hits.len(), 1, "expected exactly one token containing {needle:?}; argv={argv:?}");
    hits[0]
}

// ── Fixture builders (return the populated FormState) ───────────────────────

/// (a) secret Text `--passphrase` + a co-present non-secret `--account=9`.
fn fixture_secret_text() -> FormState {
    let mut state = FormState::default();
    state
        .secret_widgets
        .insert("--passphrase".into(), vec![SecretLineEdit::from_text(PASS_SECRET)]);
    state.values.push(("--account".to_string(), FlagValue::Number(9)));
    state
}

/// (b) a secret slot row (Phrase @0) + a watch-only slot row (Xpub @1).
fn fixture_secret_slot() -> FormState {
    FormState::default().with_slots(SlotState {
        rows: vec![
            SlotRow { index: 0, subkey: SlotSubkey::Phrase, value: SEED_SECRET.into() },
            SlotRow { index: 1, subkey: SlotSubkey::Xpub, value: WATCH_XPUB.into() },
        ],
    })
}

/// (c) secret positional `shares` (ms combine).
fn fixture_secret_positional() -> FormState {
    let mut state = FormState::default();
    state
        .secret_widgets
        .insert("positional:shares".into(), vec![SecretLineEdit::from_text(SHARE_POS_SECRET)]);
    state
}

/// (d) secret composite flag `--share` (seed-xor-combine; PHRASE_ONLY).
fn fixture_secret_composite_flag() -> FormState {
    FormState::from_pairs(vec![(
        "--share",
        FlagValue::NodeValueComposite {
            node: "phrase".into(),
            value: SHARE_COMPOSITE_SECRET.into(),
        },
    )])
}

/// (e1) value-dependent composite `--from phrase=<seed>` (secret node).
fn fixture_composite_node_secret() -> FormState {
    FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: "phrase".into(),
            value: CONVERT_SEED_SECRET.into(),
        },
    )])
}

/// (e2) value-dependent composite `--from xpub=<xpub>` (NON-secret node).
fn fixture_composite_node_watch_only() -> FormState {
    FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: "xpub".into(),
            value: WATCH_XPUB.into(),
        },
    )])
}

// ── T-A1 — per-source mask correctness ──────────────────────────────────────

#[test]
fn t_a1a_secret_text_value_masked_name_and_nonsecret_not() {
    let state = fixture_secret_text();
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("bundle"), &state);
    assert_eq!(argv.len(), mask.len(), "mask must track argv length");

    let vi = idx_eq(&argv, PASS_SECRET);
    assert!(mask[vi], "the secret --passphrase VALUE token must be masked");
    let ni = idx_eq(&argv, "--passphrase");
    assert!(!mask[ni], "the --passphrase flag NAME token must not be masked");
    // co-present non-secret --account 9
    assert!(!mask[idx_eq(&argv, "--account")], "--account name not masked");
    assert!(!mask[idx_eq(&argv, "9")], "non-secret --account value not masked");
}

#[test]
fn t_a1b_secret_slot_token_masked_watchonly_slot_not() {
    let state = fixture_secret_slot();
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("bundle"), &state);
    assert_eq!(argv.len(), mask.len());

    let secret_tok = idx_contains(&argv, SEED_SECRET); // @0.phrase=<seed>
    assert!(mask[secret_tok], "a secret-bearing slot value token must be masked");
    let watch_tok = idx_contains(&argv, WATCH_XPUB); // @1.xpub=<xpub>
    assert!(!mask[watch_tok], "a watch-only slot value token must NOT be masked");
    // The `--slot` flag-name tokens are never masked.
    for (i, t) in argv.iter().enumerate() {
        if t == "--slot" {
            assert!(!mask[i], "the --slot name token must not be masked");
        }
    }
}

#[test]
fn t_a1c_secret_positional_value_masked() {
    let state = fixture_secret_positional();
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::ms::SCHEMA, sub_ms("combine"), &state);
    assert_eq!(argv.len(), mask.len());
    let vi = idx_eq(&argv, SHARE_POS_SECRET);
    assert!(mask[vi], "the secret positional value must be masked");
}

#[test]
fn t_a1d_secret_composite_flag_value_masked() {
    let state = fixture_secret_composite_flag();
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("seed-xor-combine"), &state);
    assert_eq!(argv.len(), mask.len());
    let vi = idx_contains(&argv, SHARE_COMPOSITE_SECRET); // phrase=<value>
    assert!(mask[vi], "a secret composite flag value token must be masked");
    assert!(!mask[idx_eq(&argv, "--share")], "--share name token not masked");
}

#[test]
fn t_a1e_composite_secret_node_masked_watchonly_node_not() {
    // e1: --from phrase=<seed> → masked (node secret, flag secret:false).
    let s1 = fixture_composite_node_secret();
    let (argv1, mask1) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("convert"), &s1);
    let vi1 = idx_contains(&argv1, CONVERT_SEED_SECRET);
    assert!(mask1[vi1], "--from phrase=<seed> value must be masked (secret node)");

    // e2: --from xpub=<xpub> → NOT masked (neither flag nor node secret).
    let s2 = fixture_composite_node_watch_only();
    let (argv2, mask2) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("convert"), &s2);
    let vi2 = idx_contains(&argv2, WATCH_XPUB);
    assert!(!mask2[vi2], "--from xpub=<xpub> value must NOT be masked (watch-only node)");
}

#[test]
fn t_a1f_no_secret_subcommand_mask_all_false() {
    let mut state = FormState::default();
    state.values.push(("--account".to_string(), FlagValue::Number(3)));
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("bundle"), &state);
    assert_eq!(argv.len(), mask.len());
    assert!(mask.iter().all(|&m| !m), "a no-secret form must produce an all-false mask; argv={argv:?}");
}

// ── T-A2 — masked render hides every secret; real render reveals ────────────

#[test]
fn t_a2_masked_render_hides_secrets_real_render_reveals() {
    // Multi-source on one subcommand: secret Text + secret slot + watch-only
    // slot + non-secret value.
    let mut state = fixture_secret_text();
    state.slots = SlotState {
        rows: vec![
            SlotRow { index: 0, subkey: SlotSubkey::Phrase, value: SEED_SECRET.into() },
            SlotRow { index: 1, subkey: SlotSubkey::Xpub, value: WATCH_XPUB.into() },
        ],
    };
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("bundle"), &state);

    let masked = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    let real = render_copy_command(&argv, ShellFlavor::Posix);

    // Masked: both secrets gone, SECRET_MASK present, watch-only + flag names present.
    assert!(!masked.contains(PASS_SECRET), "masked render leaked the passphrase: {masked}");
    assert!(!masked.contains(SEED_SECRET), "masked render leaked the slot seed: {masked}");
    assert!(masked.contains(SECRET_MASK), "masked render must show the SECRET_MASK sentinel: {masked}");
    assert!(masked.contains("--passphrase"), "masked render keeps the flag name: {masked}");
    assert!(masked.contains(WATCH_XPUB), "masked render keeps the watch-only xpub: {masked}");
    // SECRET_MASK appears literally (un-quoted — never shell-quoted).
    assert!(masked.contains(SECRET_MASK) && !masked.contains(&format!("'{SECRET_MASK}'")),
        "SECRET_MASK must be un-quoted: {masked}");

    // Real (reveal): both secrets present (the copy/Run path is intact).
    assert!(real.contains(PASS_SECRET), "real render must reveal the passphrase");
    assert!(real.contains(SEED_SECRET), "real render must reveal the slot seed");
}

#[test]
fn t_a2_composite_and_positional_masked_render_hides() {
    // composite secret node
    let s1 = fixture_composite_node_secret();
    let (a1, m1) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("convert"), &s1);
    let masked1 = render_copy_command_masked(&a1, &m1, ShellFlavor::Posix);
    assert!(!masked1.contains(CONVERT_SEED_SECRET), "masked render leaked --from phrase seed: {masked1}");
    assert!(masked1.contains(SECRET_MASK), "masked composite render must show SECRET_MASK: {masked1}");
    assert!(render_copy_command(&a1, ShellFlavor::Posix).contains(CONVERT_SEED_SECRET), "real render reveals it");

    // secret positional
    let s2 = fixture_secret_positional();
    let (a2, m2) = assemble_argv_with_secret_mask(&schema::ms::SCHEMA, sub_ms("combine"), &s2);
    let masked2 = render_copy_command_masked(&a2, &m2, ShellFlavor::Posix);
    assert!(!masked2.contains(SHARE_POS_SECRET), "masked render leaked the positional share: {masked2}");
    assert!(masked2.contains(SECRET_MASK), "masked positional render must show SECRET_MASK: {masked2}");
}

// ── T-A3 — dangerous direction: mask.any() ⟹ should_confirm_run ─────────────

#[test]
fn t_a3_every_masked_state_also_confirms_run() {
    // Each (subcommand, state) where a secret token is masked MUST also gate
    // the run-confirm modal. The reverse is NOT required (a greyed secret
    // `*-stdin` toggle confirms but emits no token → mask all-false; safe,
    // nothing to leak) — see the assembler's Boolean-suppression arm.
    let cases: Vec<(&'static SubcommandSchema, FormState, &str)> = vec![
        (sub_mnemonic("bundle"), fixture_secret_text(), "secret Text"),
        (sub_mnemonic("bundle"), fixture_secret_slot(), "secret slot"),
        (sub_ms("combine"), fixture_secret_positional(), "secret positional"),
        (sub_mnemonic("seed-xor-combine"), fixture_secret_composite_flag(), "secret composite flag"),
        (sub_mnemonic("convert"), fixture_composite_node_secret(), "secret composite node"),
    ];
    for (sub, state, label) in &cases {
        let cli = if sub.name == "combine" { &schema::ms::SCHEMA } else { &schema::mnemonic::SCHEMA };
        let (_argv, mask) = assemble_argv_with_secret_mask(cli, sub, state);
        assert!(mask.iter().any(|&m| m), "{label}: a secret token must be masked");
        assert!(
            should_confirm_run(sub, state),
            "{label}: DANGEROUS direction — a token is masked but should_confirm_run is false \
             (secret would render masked yet the run skips the confirm modal)"
        );
    }
}

#[test]
fn t_a3_no_secret_state_neither_masks_nor_confirms() {
    let mut state = FormState::default();
    state.values.push(("--account".to_string(), FlagValue::Number(3)));
    let sub = sub_mnemonic("bundle");
    let (_argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub, &state);
    assert!(mask.iter().all(|&m| !m), "no-secret form: mask all-false");
    assert!(!should_confirm_run(sub, &state), "no-secret form: no run-confirm");
}

// ── T-A4 — the preview STRING the action-bar label renders is masked ────────

#[test]
fn t_a4_preview_string_is_masked() {
    // main.rs:990 renders `format!("Preview: {preview}")` where `preview =
    // render_copy_command_masked(&argv, &mask, Posix)`. The bin-crate action
    // bar is not harness-isolable, so this pins the exact STRING it shows.
    let state = fixture_secret_text();
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub_mnemonic("bundle"), &state);
    let preview = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    assert!(!preview.contains(PASS_SECRET), "the Preview label must not show the secret: {preview}");
    assert!(preview.contains(SECRET_MASK), "the Preview label must show the mask sentinel: {preview}");
}
