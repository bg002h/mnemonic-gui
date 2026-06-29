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

impl SecretLineEdit {
    /// Render one egui frame. Mutates the internal buffer in place via a
    /// short-lived `String` copy. After the egui `TextEdit::singleline`
    /// call completes, the `String` is consumed back into the buffer and
    /// the temporary `String` is overwritten before drop.
    ///
    /// The `label` is the field label (rendered to the left, like the
    /// non-secret renderer). `help` becomes hover text.
    pub fn show(&mut self, ui: &mut egui::Ui, label: &str, help: &str) {
        ui.label(label).on_hover_text(help);
        // Take buffer contents into a transient String. Replacement of
        // buf contents with the freshly-typed bytes happens at end of
        // frame; `Zeroizing` re-engages there.
        let mut transient = String::from_utf8(self.buf.to_vec()).unwrap_or_default();
        let response = ui.add(egui::TextEdit::singleline(&mut transient).password(true));
        // v0.40.0 (Item 3): an `Event::Paste` stays in `input.events` after the
        // TextEdit reads it (egui 0.31 `filtered_events` clones; the only
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
    }
}

// `_doctest_wrap_pattern` removed in B.1 R1 I-2 fold — `as_string()` now
// returns `Zeroizing<String>` directly, making the wrap contract
// type-level rather than doc-only.
