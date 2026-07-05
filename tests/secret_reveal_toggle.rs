//! v0.57.0 — secret-field reveal (👁) toggle hygiene cells
//! (`SPEC_gui_secret_reveal_toggle` §8; plan `tutorial_surfaced_fixes_batch` P1.2).
//!
//! The reveal toggle is a DELIBERATE secret-exposure affordance; the hygiene
//! model is the load-bearing part, so every one of these cells drives the real
//! widget path and asserts a hygiene invariant:
//!
//!   1. masking-default (unactuated → masked `PasswordInput`, eye present);
//!   2. reveal-flips via AccessKit `Click` (→ `TextInput`; buffer UNCHANGED);
//!   3. single-revealed-field invariant (revealing B re-masks A; ONE `Option<Id>`);
//!   4. auto-hide ×4 — (a) Run dispatch, (b) field blur, (c) window-focus-loss,
//!      (d) tab/subcommand switch;
//!   5. ruling-2 (reveal-R0 M-1): AccessKit/keyboard `Click` LATCHES; a pointer
//!      TAP does NOT latch (it gets only a momentary hold-reveal);
//!   6. never-persist orthogonality — with the latch ARMED, the persist / masked
//!      argv-preview / copy-command surfaces are UNCHANGED (masked/`••••`);
//!   8. slot-arm isolation ((Path,hint) arm eye-free) + composite gating (the eye
//!      tracks `is_secret_node`; a node switch secret→non-secret removes the eye
//!      AND clears the stale latch).
//!
//! (Cell #7 — the faithfulness both-sides + non-vacuity negative — lives in
//! `tests/gui_render_faithfulness.rs::reveal_eye_faithfulness_is_non_vacuous`.)
//!
//! ## Harness hygiene (secret-first-class)
//! FAKE, world-public / sentinel fixtures ONLY — never a real key. Failures are
//! coordinate-only where a payload could otherwise be echoed.

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::form::invocation::{
    assemble_argv_with_secret_mask, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::secret_widget::{
    clear_revealed_field, revealed_field, SecretLineEdit, REVEAL_EYE_GLYPH,
};
use mnemonic_gui::form::slot_editor::{self, SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::form::widget::render_with_dispatch;
use mnemonic_gui::path_detect::Detected;
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::{self, FlagValue, FormState};

use std::path::PathBuf;

// ─── helpers ─────────────────────────────────────────────────────────────────

fn count_role<S>(h: &Harness<'static, S>, role: Role) -> usize {
    h.query_all_by_role(role).count()
}

fn eye_count<S>(h: &Harness<'static, S>) -> usize {
    h.query_all_by_label(REVEAL_EYE_GLYPH).count()
}

/// One-secret-field harness rendering the real `SecretLineEdit::show`.
fn secret_field_harness(initial: &str) -> Harness<'static, SecretLineEdit> {
    Harness::new_ui_state(
        |ui, st: &mut SecretLineEdit| {
            st.show(ui, "secret", "the secret");
        },
        SecretLineEdit::from_text(initial),
    )
}

/// Two-secret-field state for the single-revealed-field invariant cell.
struct TwoSecrets {
    a: SecretLineEdit,
    b: SecretLineEdit,
}

/// Click the eye at tree-position `idx` (there may be several) then settle.
fn click_eye<S>(h: &mut Harness<'static, S>, idx: usize) {
    {
        let eyes: Vec<_> = h.query_all_by_label(REVEAL_EYE_GLYPH).collect();
        eyes[idx].click();
    }
    h.run();
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 1 — masking default
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell1_masking_is_the_unactuated_default() {
    let mut h = secret_field_harness("alpha bravo charlie");
    h.run();
    assert_eq!(
        count_role(&h, Role::PasswordInput),
        1,
        "a secret field renders MASKED (PasswordInput) by default, unactuated"
    );
    assert_eq!(
        count_role(&h, Role::TextInput),
        0,
        "no revealed (TextInput) field before any actuation"
    );
    assert_eq!(eye_count(&h), 1, "the reveal (👁) eye is present on the secret field");
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 2 — reveal flips the mask (AccessKit latch path); buffer UNCHANGED
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell2_accesskit_click_reveals_and_buffer_is_unchanged() {
    let mut h = secret_field_harness("alpha bravo charlie");
    h.run();
    let before = h.state().as_string().as_str().to_string();

    click_eye(&mut h, 0);

    assert_eq!(
        count_role(&h, Role::TextInput),
        1,
        "after an AccessKit Click the field is REVEALED (TextInput / password(false))"
    );
    assert_eq!(
        count_role(&h, Role::PasswordInput),
        0,
        "the revealed field is no longer masked"
    );
    // Display-only: reveal must NOT mutate the buffer.
    assert_eq!(
        h.state().as_string().as_str(),
        before,
        "reveal is display-only — the secret buffer must be UNCHANGED"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 3 — single-revealed-field invariant (revealing B re-masks A)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell3_single_revealed_field_invariant() {
    let mut h = Harness::new_ui_state(
        |ui, st: &mut TwoSecrets| {
            st.a.show(ui, "A", "");
            st.b.show(ui, "B", "");
        },
        TwoSecrets {
            a: SecretLineEdit::from_text("alpha aaaa"),
            b: SecretLineEdit::from_text("bravo bbbb"),
        },
    );
    h.run();
    assert_eq!(count_role(&h, Role::PasswordInput), 2, "both fields masked on load");
    assert_eq!(count_role(&h, Role::TextInput), 0);
    assert_eq!(eye_count(&h), 2, "one eye per secret field");
    assert!(revealed_field(&h.ctx).is_none(), "nothing revealed on load");

    // Reveal A (first eye).
    click_eye(&mut h, 0);
    let id_a = revealed_field(&h.ctx);
    assert!(id_a.is_some(), "field A is revealed after clicking its eye");
    assert_eq!(count_role(&h, Role::TextInput), 1, "exactly ONE field revealed");
    assert_eq!(count_role(&h, Role::PasswordInput), 1, "the other stays masked");

    // Reveal B (second eye) — A must re-mask (single-field invariant).
    click_eye(&mut h, 1);
    let id_b = revealed_field(&h.ctx);
    assert!(id_b.is_some(), "field B is revealed after clicking its eye");
    assert_ne!(id_a, id_b, "the single Option<Id> FLIPPED from A to B (holds exactly one)");
    assert_eq!(
        count_role(&h, Role::TextInput),
        1,
        "STILL exactly one revealed — revealing B re-masked A"
    );
    assert_eq!(count_role(&h, Role::PasswordInput), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 4b — auto-hide on field blur
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell4b_auto_hide_on_field_blur() {
    let mut h = secret_field_harness("alpha aaaa");
    h.run();
    click_eye(&mut h, 0); // latch armed → revealed
    assert_eq!(count_role(&h, Role::TextInput), 1, "revealed after arming");

    // Give the revealed field focus, then move focus away (to the eye) → blur.
    {
        h.get_by_role(Role::TextInput).focus();
    }
    h.run();
    {
        h.get_by_label(REVEAL_EYE_GLYPH).focus();
    }
    h.run();

    assert!(
        revealed_field(&h.ctx).is_none(),
        "the latch clears when the revealed field loses focus (§4.5-2)"
    );
    assert_eq!(count_role(&h, Role::PasswordInput), 1, "re-masked after blur");
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 4c — auto-hide on window-focus-loss (the OS-snapshot risk)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell4c_auto_hide_on_window_focus_loss() {
    let mut h = secret_field_harness("alpha aaaa");
    h.run();
    click_eye(&mut h, 0);
    assert_eq!(count_role(&h, Role::TextInput), 1, "revealed after arming");

    // Window loses focus (App-Switcher / Task-View thumbnail moment).
    h.input_mut().focused = false;
    h.run();

    assert!(
        revealed_field(&h.ctx).is_none(),
        "the latch clears the instant the window loses focus (§4.5-3)"
    );
    assert_eq!(
        count_role(&h, Role::PasswordInput),
        1,
        "the field is re-masked while the window is unfocused (snapshot captures masked)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 5 — ruling-2 (reveal-R0 M-1): AccessKit latches; pointer tap does NOT
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell5_accesskit_latches_pointer_tap_does_not() {
    let mut h = secret_field_harness("alpha aaaa");
    h.run();

    // ── AccessKit Click LATCHES (persists across frames without re-clicking) ──
    click_eye(&mut h, 0);
    assert_eq!(count_role(&h, Role::TextInput), 1, "AccessKit Click reveals");
    assert!(revealed_field(&h.ctx).is_some(), "AccessKit Click ARMS the latch");
    h.run(); // another frame, NO re-click
    assert_eq!(
        count_role(&h, Role::TextInput),
        1,
        "the reveal PERSISTS un-held → it is a LATCH, not a momentary hold"
    );
    // Toggle it back off via another AccessKit Click.
    click_eye(&mut h, 0);
    assert!(revealed_field(&h.ctx).is_none(), "a second AccessKit Click un-latches");
    assert_eq!(count_role(&h, Role::PasswordInput), 1, "re-masked");

    // ── A pointer TAP does NOT latch (only a momentary hold-reveal) ──
    // Manual press (hold) → release, so we can observe the momentary reveal
    // (non-vacuity: proves the pointer landed on the eye) AND that no latch
    // survives the release.
    let center = {
        let eye = h.get_by_label(REVEAL_EYE_GLYPH);
        let r = eye.raw_bounds().expect("eye has bounds");
        egui::Pos2::new(((r.x0 + r.x1) / 2.0) as f32, ((r.y0 + r.y1) / 2.0) as f32)
    };
    h.input_mut().events.push(egui::Event::PointerMoved(center));
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    h.run();
    assert_eq!(
        count_role(&h, Role::TextInput),
        1,
        "while the eye is physically HELD the field reveals (hold arm; proves the pointer hit the eye)"
    );
    assert!(
        revealed_field(&h.ctx).is_none(),
        "a HOLD is not a latch — no latched Id is set while held"
    );
    // Release → the tap is complete; NO latch must remain.
    h.input_mut().events.push(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    h.run();
    assert!(
        revealed_field(&h.ctx).is_none(),
        "a pointer TAP does NOT latch (reveal-R0 M-1)"
    );
    assert_eq!(count_role(&h, Role::PasswordInput), 1, "re-masked after the tap");
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 6 — never-persist orthogonality: reveal ARMED, surfaces still masked
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cell6_reveal_does_not_leak_into_persist_argv_or_copy() {
    // FAKE fixture (never a real key); alphanumeric so shell-quoting is inert.
    const FAKE: &str = "FAKE_REVEAL_ORTHOGONAL_000";
    let sub = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "bundle")
        .expect("bundle");
    let passphrase = sub
        .flags
        .iter()
        .find(|f| f.name == "--passphrase")
        .expect("bundle --passphrase");

    let mut h = Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_with_dispatch(ui, CliTab::Mnemonic, "bundle", passphrase, state, &[]);
        },
        FormState::default(),
    );
    h.run();
    // Type the FAKE secret into the masked field, then ARM the reveal.
    {
        h.get_by_role(Role::PasswordInput).type_text(FAKE);
    }
    h.run();
    h.run(); // settle buffer write-back
    click_eye(&mut h, 0);
    assert!(revealed_field(&h.ctx).is_some(), "reveal is ARMED for this cell");
    assert_eq!(
        count_role(&h, Role::TextInput),
        1,
        "the field is deliberately revealed on-screen"
    );

    let state = h.state();
    // The value genuinely reached the widget store (non-vacuity).
    assert!(
        state
            .secret_widgets
            .get("--passphrase")
            .map(|rows| rows.iter().any(|w| w.as_string().as_str() == FAKE))
            .unwrap_or(false),
        "the FAKE secret must have landed in secret_widgets (else the cell is vacuous)"
    );

    // Surface A — masked argv: every token bearing the fixture carries mask==true.
    let (argv, mask) = assemble_argv_with_secret_mask(&schema::mnemonic::SCHEMA, sub, state);
    let bearing: Vec<usize> = argv
        .iter()
        .enumerate()
        .filter(|(_, t)| t.contains(FAKE))
        .map(|(i, _)| i)
        .collect();
    assert!(!bearing.is_empty(), "the secret must reach argv (non-vacuous)");
    assert!(
        bearing.iter().all(|&i| mask[i]),
        "every argv token bearing the revealed secret is STILL mask==true"
    );

    // Surface B — masked copy-command (both flavors): `••••`, never the fixture.
    for flavor in [ShellFlavor::Posix, ShellFlavor::WindowsCmd] {
        let cmd = render_copy_command_masked(&argv, &mask, flavor);
        assert!(!cmd.contains(FAKE), "masked copy-command leaked the revealed secret ({flavor:?})");
    }
    let posix = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
    assert!(posix.contains("••••"), "the masked preview still redacts to ••••");

    // Surface C — persistence: the redacted+serialized state must not carry it.
    let redacted = redact_for_persistence(state);
    let json = serde_json::to_string(&redacted).expect("FormState serializes");
    assert!(!json.contains(FAKE), "the revealed secret leaked into persisted state");
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 8 — slot-arm isolation + composite gating
// ═══════════════════════════════════════════════════════════════════════════

fn slot_harness(rows: Vec<SlotRow>) -> Harness<'static, SlotState> {
    Harness::new_ui_state(
        |ui, st: &mut SlotState| {
            slot_editor::render(ui, st, None);
        },
        SlotState { rows },
    )
}

#[test]
fn cell8a_slot_secret_arm_has_eye_path_arm_does_not() {
    // A secret-bearing slot value (phrase) → masked + eye.
    let mut secret = slot_harness(vec![SlotRow {
        index: 0,
        subkey: SlotSubkey::Phrase,
        value: String::new(),
    }]);
    secret.run();
    assert_eq!(count_role(&secret, Role::PasswordInput), 1, "secret slot value is masked");
    assert_eq!(eye_count(&secret), 1, "the secret slot arm carries the reveal eye");

    // A non-secret (Path) slot value → NO mask, NO eye (the (Path,hint) arm).
    let mut path = slot_harness(vec![SlotRow {
        index: 0,
        subkey: SlotSubkey::Path,
        value: String::new(),
    }]);
    path.run();
    assert_eq!(count_role(&path, Role::PasswordInput), 0, "a Path slot value is not masked");
    assert_eq!(eye_count(&path), 0, "the (Path, hint) arm is eye-free (slot-arm isolation)");
}

#[test]
fn cell8b_composite_eye_tracks_is_secret_node_and_clears_stale_latch() {
    // `convert --from` is a NodeValueComposite over secret nodes (phrase) and
    // public nodes (xpub).
    let sub = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "convert")
        .expect("convert");
    let from = sub
        .flags
        .iter()
        .find(|f| f.name == "--from")
        .expect("convert --from");

    let mut state = FormState::default();
    state.values.push((
        "--from".to_string(),
        FlagValue::NodeValueComposite {
            node: "phrase".to_string(), // argv-secret
            value: "alpha aaaa".to_string(),
        },
    ));
    let mut h = Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            render_with_dispatch(ui, CliTab::Mnemonic, "convert", from, state, &[]);
        },
        state,
    );
    h.run();
    // Secret node → masked value field + eye.
    assert_eq!(count_role(&h, Role::PasswordInput), 1, "a secret composite node masks its value");
    assert_eq!(eye_count(&h), 1, "the secret composite arm carries the eye");

    // Arm the latch on the composite field.
    click_eye(&mut h, 0);
    assert!(revealed_field(&h.ctx).is_some(), "composite reveal armed");
    assert_eq!(count_role(&h, Role::TextInput), 1, "revealed");

    // Switch the node to a non-secret (xpub) — the eye must disappear AND the
    // stale latch must clear (composite gating, spec §8 test #8).
    {
        let st = h.state_mut();
        if let Some((_, FlagValue::NodeValueComposite { node, .. })) =
            st.values.iter_mut().find(|(k, _)| k == "--from")
        {
            *node = "xpub".to_string();
        }
    }
    h.run();
    assert_eq!(eye_count(&h), 0, "a non-secret (xpub) node has NO reveal eye");
    assert_eq!(
        count_role(&h, Role::PasswordInput),
        0,
        "the xpub value field is plainly readable (not masked)"
    );
    assert!(
        revealed_field(&h.ctx).is_none(),
        "the stale latch for the now-non-secret field is cleared"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell 4a / 4d — auto-hide via the REAL app window (Run dispatch; form switch)
// ═══════════════════════════════════════════════════════════════════════════

fn appstate_all_found() -> AppState {
    AppState {
        mnemonic_detect: Detected::Found(PathBuf::from("/pinned/mnemonic")),
        md_detect: Detected::Found(PathBuf::from("/pinned/md")),
        ms_detect: Detected::Found(PathBuf::from("/pinned/ms")),
        mk_detect: Detected::Found(PathBuf::from("/pinned/mk")),
        active_tab: CliTab::Mnemonic,
    }
}

/// The real app window, seeded to a given mnemonic subcommand.
fn app_on_sub(sub: &str) -> Harness<'static, MnemonicGuiApp> {
    let mut app = MnemonicGuiApp::new_headless(appstate_all_found(), None, None);
    app.app_state.active_tab = CliTab::Mnemonic;
    app.active_subcommand.insert(CliTab::Mnemonic, sub.to_string());
    Harness::builder()
        .with_size(egui::Vec2::new(1100.0, 1600.0))
        .with_max_steps(64)
        .build_state(|ctx, app: &mut MnemonicGuiApp| app.ui(ctx), app)
}

#[test]
fn cell4a_auto_hide_on_run_dispatch() {
    // `mnemonic inspect` has one secret scalar field (`--ms1` → one eye).
    let mut h = app_on_sub("inspect");
    h.run();
    assert_eq!(eye_count(&h), 1, "inspect renders exactly one reveal eye (--ms1)");

    click_eye(&mut h, 0);
    assert!(revealed_field(&h.ctx).is_some(), "the --ms1 eye armed the latch");

    // Click Run — a secret is present so this opens the (masked) confirm modal;
    // the reveal must clear on Run DISPATCH regardless (§4.5-1).
    {
        h.get_by_label("Run").click();
    }
    h.run();
    assert!(
        revealed_field(&h.ctx).is_none(),
        "the latch clears on Run dispatch (§4.5-1) — nothing stays revealed behind the modal"
    );
}

#[test]
fn cell4d_auto_hide_on_subcommand_switch() {
    let mut h = app_on_sub("inspect");
    h.run();
    click_eye(&mut h, 0);
    assert!(revealed_field(&h.ctx).is_some(), "armed on inspect");

    // Switch to another subcommand (the user picks a different form).
    {
        h.state_mut()
            .active_subcommand
            .insert(CliTab::Mnemonic, "compare-cost".to_string());
    }
    h.run();
    assert!(
        revealed_field(&h.ctx).is_none(),
        "the latch clears on a subcommand switch (§4.5-4)"
    );
}

// Silence the unused `clear_revealed_field` import when only the app path uses
// it indirectly — reference it in a trivial invariant so the API stays exported
// and the coverage is explicit.
#[test]
fn clear_revealed_field_is_idempotent_on_empty() {
    let ctx = egui::Context::default();
    assert!(revealed_field(&ctx).is_none());
    clear_revealed_field(&ctx); // no panic on an already-clear state
    assert!(revealed_field(&ctx).is_none());
}
