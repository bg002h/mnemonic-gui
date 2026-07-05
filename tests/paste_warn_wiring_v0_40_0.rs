//! v0.40.0 (Item 3) — the paste-warn modal is WIRED: an over-threshold paste
//! into a focused secret field raises the `paste_warn_id()` ctx-data bus flag,
//! which `update()` reads once and turns into the informational modal.
//!
//! SPEC: `design/SPEC_gui_v0_40_0_paste_warn_wiring.md`.
//!
//! - T-B2 (live wiring, kittest): inject an `Event::Paste` ≥ threshold into a
//!   focused `SecretLineEdit` → the bus flag is set; a `< threshold` paste →
//!   not set. (The missing `paste-warn-live-wiring-untested` live check.)
//! - T-B3 (read-once + clear, pure logic): `remove_temp` reads AND clears the
//!   flag (no leak into the next frame); the modal text is the constant.
//! - T-B4 (chokepoint pin): exactly 3 DIRECT `SecretLineEdit::show` sites.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::form::secret_widget::{paste_warn_id, SecretLineEdit};
use mnemonic_gui::secrets::{self, PASTE_WARN_THRESHOLD};

fn paste_harness() -> Harness<'static, SecretLineEdit> {
    Harness::new_ui_state(
        |ui, w: &mut SecretLineEdit| {
            w.show(ui, "secret", "help");
        },
        SecretLineEdit::new(),
    )
}

fn bus_flag(h: &mut Harness<'static, SecretLineEdit>) -> Option<bool> {
    h.ctx.data_mut(|d| d.get_temp::<bool>(paste_warn_id()))
}

// ── T-B2 — live wiring ──────────────────────────────────────────────────────

#[test]
fn t_b2_over_threshold_paste_into_focused_secret_sets_bus_flag() {
    let mut h = paste_harness();
    h.run();
    // Focus the masked field (click is processed on the next frame).
    h.get_by_role(egui::accesskit::Role::PasswordInput).focus();
    h.run();
    // Inject an over-threshold paste; the focused TextEdit consumes it
    // (buffer changes → response.changed()) and `show` raises the flag.
    let big = "x".repeat(PASTE_WARN_THRESHOLD + 4);
    h.input_mut().events.push(egui::Event::Paste(big));
    h.run();
    assert_eq!(
        bus_flag(&mut h),
        Some(true),
        "an over-threshold paste into a focused secret field must raise the paste-warn bus flag"
    );
}

#[test]
fn t_b2b_under_threshold_paste_does_not_set_bus_flag() {
    let mut h = paste_harness();
    h.run();
    h.get_by_role(egui::accesskit::Role::PasswordInput).focus();
    h.run();
    let small = "x".repeat(PASTE_WARN_THRESHOLD - 1);
    h.input_mut().events.push(egui::Event::Paste(small));
    h.run();
    assert_ne!(
        bus_flag(&mut h),
        Some(true),
        "an under-threshold paste must NOT raise the paste-warn flag"
    );
}

// ── T-B3 — read-once + clear (the load-bearing no-leak property) ─────────────

#[test]
fn t_b3_read_once_reads_then_clears_the_flag() {
    let ctx = egui::Context::default();
    // A secret widget raised the flag this frame.
    ctx.data_mut(|d| d.insert_temp(paste_warn_id(), true));
    // `update()`'s read-once uses remove_temp — it must observe true ONCE...
    let first = ctx.data_mut(|d| d.remove_temp::<bool>(paste_warn_id()).unwrap_or(false));
    // ...and clear it, so a stale flag can't leak into the next frame.
    let second = ctx.data_mut(|d| d.remove_temp::<bool>(paste_warn_id()).unwrap_or(false));
    assert!(first, "read-once must observe the raised flag");
    assert!(!second, "read-once must CLEAR the flag (no leak into the next frame)");
    // The modal renders the documented constant.
    assert!(
        secrets::PASTE_WARN_MODAL_TEXT.contains("pasting potentially secret material"),
        "the modal text constant must be the paste-warning copy"
    );
}

// ── T-B4 — chokepoint pin: exactly 3 DIRECT SecretLineEdit::show sites ───────

#[test]
fn t_b4_three_direct_secretlineedit_show_sites() {
    // The ctx-data bus is set INSIDE `show`, so `show` is the single
    // chokepoint covering every site (the 3 direct + every transitive
    // archetype path through render_with_dispatch → widget.rs). These are the
    // 3 DIRECT call sites: widget.rs:110 (scalar), widget.rs:153 (repeating),
    // app_window.rs (positional — relocated VERBATIM from main.rs:843 in the
    // gui_example_tutorial P0 app-shell extraction; main.rs stays in the scan
    // set so a re-added direct site there is still caught). A 4th is still
    // bus-covered, but is a deliberate surface change — this tripwire
    // documents it.
    //
    // SecretLineEdit::show calls pass STRING args (`.show(ui, label, help)`);
    // egui Window/ScrollArea/CollapsingHeader pass an `|ui|` closure
    // (`.show(ui, |ui| ...)`) or a `ctx` (`.show(ctx, ...)`) — excluded.
    let widget = include_str!("../src/form/widget.rs");
    let app_window = include_str!("../src/app_window.rs");
    let main = include_str!("../src/main.rs");
    let non_closure_show_ui = |src: &str| {
        // Collapse ALL whitespace first so a future multi-line call
        // (`w.show(\n  ui,\n  …)`) still matches — the impl-review M-1
        // hardening (a line-wrapped 4th site must not evade the tripwire).
        let flat: String = src.split_whitespace().collect();
        flat.matches(".show(ui,").count() - flat.matches(".show(ui,|").count()
    };
    let total =
        non_closure_show_ui(widget) + non_closure_show_ui(app_window) + non_closure_show_ui(main);
    assert_eq!(
        total, 3,
        "expected exactly 3 direct SecretLineEdit::show sites (widget.rs scalar+repeating, \
         app_window.rs positional); found {total} — a 4th direct site was added (still \
         bus-covered, but review the surface + update this tripwire)"
    );
}
