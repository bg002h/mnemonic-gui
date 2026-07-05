//! SecretLineEdit — secret-bearing single-line text widget (SPEC §3).
//!
//! P1 `gui`-feature split: the egui-FREE buffer type [`SecretLineEdit`]
//! (struct + `new`/`from_text`/`as_string`/`is_empty`/`zeroize`) lives in
//! the non-gated [`crate::form::secret_model`] — it is a field of the
//! non-gated `schema::FormState`, so it must build without egui. This gated
//! module owns only the egui-coupled surface: the per-frame `show` widget
//! (an inherent-impl split on the same type — legal within one crate) and
//! the `paste_warn_id` ctx-data key. `SecretLineEdit` is re-exported below
//! so existing `secret_widget::SecretLineEdit` paths keep resolving under
//! the `gui` feature.
//!
//! On each egui frame, `show` renders a password-masked `egui::TextEdit`
//! backed by a transient `String` copy of the buffer; on frame completion
//! the transient `String` is consumed back into the buffer and dropped
//! (and zeroed by `Zeroizing::Drop`).
//!
//! # Security
//! - egui undo ring: NOT addressed. egui's `TextEditState` holds `String`
//!   snapshots in the undo buffer. This is a second-tier gap documented
//!   in Section A R-1 / FOLLOWUPS `gui-secret-buffer-allocator-residue`
//!   and deferred beyond v0.2.
//! - Transient-frame `String`: heap-allocated for the scope of one egui
//!   `update()` call; dropped at end of frame. Allocator-residue caveat
//!   remains; the transient is wrapped in `Zeroizing::new(...)` only
//!   when extracted via [`SecretLineEdit::as_string`] (R1 N-1 fold —
//!   `assemble_argv` does the wrap at its call site).

use eframe::egui;
use zeroize::Zeroize;

pub use crate::form::secret_model::SecretLineEdit;

/// v0.40.0 (Item 3) — the egui Context-data key under which any
/// `SecretLineEdit::show` raises an over-threshold-paste signal for the frame.
/// `update()` reads+clears it once (`remove_temp`) and fires the paste-warn
/// modal. This bus is the SINGLE chokepoint for every `show` call site (the 3
/// direct + transitive archetype paths) — do NOT reintroduce per-site
/// return-value wiring.
pub fn paste_warn_id() -> egui::Id {
    egui::Id::new("gui_secret_paste_warn_pending")
}

// ─── v0.57.0: secret-field reveal (👁) toggle (SPEC_gui_secret_reveal_toggle) ──
//
// A DELIBERATE secret-exposure affordance so a user can proofread a hand-typed
// seed (and the tutorial can show the PUBLIC demo phrase). The hygiene model is
// the load-bearing part (R0-ruled):
//   - hold-to-reveal (pointer) PRIMARY: reveal only while the eye is physically
//     held (`is_pointer_button_down_on()`); re-masks on release.
//   - bounded LATCH fallback for keyboard / AccessKit / kittest / tutorial
//     capture: an AccessKit `Click` (or keyboard Space/Enter) arms a latched
//     reveal for ONE field; a pointer TAP does NOT latch (reveal-R0 M-1 — egui's
//     `clicked()` is true for pointer AND FAKE_PRIMARY clicks, `clicked_by(Primary)`
//     ONLY for pointer, so the latch arms iff `clicked() && !clicked_by(Primary)`).
//   - single-revealed-field invariant: ONE Context-transient `Option<egui::Id>`
//     (NEVER a `FormState` field → the I3 never-persist net is structurally
//     unaffected; nothing about reveal is serialized).
//   - auto-hide: Run dispatch + tab/subcommand switch (app-window seams,
//     `clear_revealed_field` / `clear_reveal_on_form_change`), field blur +
//     window-focus-loss (per-frame inside `reveal_toggle` / `clear_reveal_on_blur`).
//   - NO wall-clock timeout in v1 (determinism; `gui-secret-reveal-latch-timeout`).
// Reveal is DISPLAY-ONLY on the input widget — every masked/redacted surface
// (run-confirm modal, argv echo, copy-command, paste-warn, persistence, exit
// sweep) stays masked UNCONDITIONALLY, independent of reveal state.

/// The eye glyph — U+1F441, present in egui's bundled `emoji-icon-font`
/// (the egui-demo password-reveal glyph). kittest queries the reveal button
/// by this EXACT label; the visual gallery renders it.
pub const REVEAL_EYE_GLYPH: &str = "👁";

/// ctx-data key holding the single currently-revealed secret field's stable
/// `egui::Id` (single-revealed-field invariant). ABSENT = nothing revealed.
/// Sibling of [`paste_warn_id`]; NEVER a `FormState` field.
pub fn reveal_field_key() -> egui::Id {
    egui::Id::new("gui_secret_reveal_field")
}

/// The currently-revealed secret field's `Id`, if any (auto-hide + test oracle).
pub fn revealed_field(ctx: &egui::Context) -> Option<egui::Id> {
    ctx.data(|d| d.get_temp::<egui::Id>(reveal_field_key()))
}

fn set_revealed_field(ctx: &egui::Context, id: egui::Id) {
    ctx.data_mut(|d| d.insert_temp(reveal_field_key(), id));
}

/// Force-clear any latched reveal (→ masked). The Run-dispatch (§4.5-1) and
/// tab/subcommand-switch (§4.5-4) auto-hide triggers call this at the app-window
/// seam; blur / window-focus-loss are handled per-frame in [`reveal_toggle`] /
/// [`clear_reveal_on_blur`].
pub fn clear_revealed_field(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<egui::Id>(reveal_field_key()));
}

fn reveal_last_form_key() -> egui::Id {
    egui::Id::new("gui_secret_reveal_last_form")
}

/// Auto-hide trigger §4.5-4: clear any latched reveal when the active form
/// (tab / subcommand) changed since last frame. Call ONCE per frame from the
/// app window with the current `"<cli>:<sub>"` form key.
pub fn clear_reveal_on_form_change(ctx: &egui::Context, current_form_key: &str) {
    let changed = ctx
        .data(|d| d.get_temp::<String>(reveal_last_form_key()))
        .as_deref()
        != Some(current_form_key);
    if changed {
        clear_revealed_field(ctx);
        ctx.data_mut(|d| d.insert_temp(reveal_last_form_key(), current_form_key.to_string()));
    }
}

/// Auto-hide trigger §4.5-2: clear the latched reveal when the revealed field
/// itself loses keyboard focus. Call with the field's `Response` right AFTER
/// rendering it (it needs `Response::lost_focus()`). Only clears when THIS
/// field is the currently-revealed one (a sibling field's blur must not clear
/// an unrelated reveal).
pub fn clear_reveal_on_blur(ctx: &egui::Context, field_id: egui::Id, response: &egui::Response) {
    if response.lost_focus() && revealed_field(ctx) == Some(field_id) {
        clear_revealed_field(ctx);
    }
}

/// Render the reveal (👁) toggle for a secret field and return whether the field
/// must be REVEALED this frame — the predicate for `.password(!reveal)`.
///
/// MUST be called inside the field's row, and the eye is added BEFORE the
/// field's `TextEdit` so the pointer HOLD arm (`is_pointer_button_down_on()`) is
/// known in-frame. `field_id` is the field's stable per-frame `egui::Id` (used
/// as both the latch key AND the `TextEdit`'s explicit id — reveal-R0 M-6).
///
/// Interaction (SPEC §4.3, reveal-R0 M-1): pointer HOLD = reveal-while-held;
/// AccessKit/keyboard Click = arm/toggle the bounded LATCH; pointer TAP = NO
/// latch. Auto-hide §4.5-3 (window-focus-loss) is applied here (so an
/// app-switch thumbnail captures a re-masked field — the platform.rs risk).
pub fn reveal_toggle(ui: &mut egui::Ui, ctx: &egui::Context, field_id: egui::Id) -> bool {
    let window_focused = ui.input(|i| i.focused);
    let mut latched = revealed_field(ctx) == Some(field_id);
    // §4.5-3: window-focus-loss force-clears the latch for the revealed field.
    if !window_focused && latched {
        clear_revealed_field(ctx);
        latched = false;
    }
    let eye = ui
        .add(egui::Button::new(REVEAL_EYE_GLYPH).small())
        .on_hover_text("Hold to reveal (masked by default; keyboard/AT: click to latch)");
    // Latch arm: keyboard / AccessKit activation ONLY (FAKE_PRIMARY_CLICKED) —
    // a pointer TAP (`clicked_by(Primary)`) does NOT latch (reveal-R0 M-1); the
    // HOLD arm already gives a tap its momentary reveal.
    let accesskit_or_keyboard = eye.clicked() && !eye.clicked_by(egui::PointerButton::Primary);
    if accesskit_or_keyboard {
        if latched {
            clear_revealed_field(ctx);
            latched = false;
        } else {
            set_revealed_field(ctx, field_id);
            latched = true;
        }
    }
    // Pointer HOLD arm: reveal only for the frames the eye is physically held.
    let hold = eye.is_pointer_button_down_on();
    window_focused && (latched || hold)
}

impl SecretLineEdit {
    /// Render one egui frame. Mutates the internal buffer in place via a
    /// short-lived `String` copy. After the egui `TextEdit::singleline`
    /// call completes, the `String` is consumed back into the buffer and
    /// the temporary `String` is overwritten before drop.
    ///
    /// The `label` is the field label (rendered to the left, like the
    /// non-secret renderer). `help` becomes hover text.
    pub fn show(&mut self, ui: &mut egui::Ui, label: &str, help: &str) {
        // v0.57.0: wrap in a single row so the label + the reveal (👁) eye +
        // the masked field share one horizontal band (reveal-R0 M-6). Existing
        // call sites already wrap `show` in their own `ui.horizontal`; the inner
        // horizontal keeps the eye adjacent regardless of caller layout.
        let ctx = ui.ctx().clone();
        ui.horizontal(|ui| {
            ui.label(label).on_hover_text(help);
            // Per-field id: `ui.unique_id()` is globally unique per Ui instance
            // (position-derived; `ui.id()` is the STABLE id that COLLIDES between
            // sibling `horizontal`s — two secret fields would share it). Used as
            // BOTH the latch key and the `TextEdit`'s explicit id, so
            // `response.id == field_id` (reveal-R0 M-6).
            let field_id = ui.unique_id().with("secret_reveal_field");
            // The eye is rendered by `reveal_toggle` BEFORE the field so the
            // pointer-hold arm resolves in-frame. The default (unactuated) render
            // is masked (SPEC §4.1); reveal flips ONLY this frame's `.password`.
            let reveal = reveal_toggle(ui, &ctx, field_id);
            // Take buffer contents into a transient String. Replacement of
            // buf contents with the freshly-typed bytes happens at end of
            // frame; `Zeroizing` re-engages there.
            let mut transient = String::from_utf8(self.buf.to_vec()).unwrap_or_default();
            let response = ui.add(
                egui::TextEdit::singleline(&mut transient)
                    .id(field_id)
                    .password(!reveal),
            );
            // §4.5-2: a latched reveal on THIS field clears when it loses focus.
            clear_reveal_on_blur(&ctx, field_id, &response);
            // v0.40.0 (Item 3): an `Event::Paste` stays in `input.events` after
            // the TextEdit reads it (egui 0.31 `filtered_events` clones; the only
            // `events.retain` purges `Event::Ime` only) — so we can scan for it.
            let pasted_len = ui.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Paste(s) => Some(s.chars().count()),
                    _ => None,
                })
            });
            if response.changed() {
                // Overwrite the primary buffer with the new bytes; the old
                // buffer contents are zeroed via Zeroize when we reassign.
                self.buf.zeroize();
                *self.buf = transient.as_bytes().to_vec();
                // `response.changed()` attributes the paste to THIS focused field
                // (a paste into another field doesn't change this buffer), so no
                // false-positive and no multi-row double-trigger. The flag-is-secret
                // half is structural (this widget renders only for secret fields).
                if let Some(len) = pasted_len {
                    if len >= crate::secrets::PASTE_WARN_THRESHOLD {
                        ui.ctx().data_mut(|d| d.insert_temp(paste_warn_id(), true));
                    }
                }
            }
            // Zero the transient before drop. allocator residue caveat
            // remains (one-frame scope; not a guarantee, just best-effort).
            transient.zeroize();
        });
    }
}

// `_doctest_wrap_pattern` removed in B.1 R1 I-2 fold — `as_string()` now
// returns `Zeroizing<String>` directly, making the wrap contract
// type-level rather than doc-only.
