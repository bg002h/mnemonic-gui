//! SecretLineEdit — secret-bearing single-line text widget (SPEC §3).
//!
//! Owns a `Zeroizing<Vec<u8>>` primary buffer. On each egui frame, renders
//! a password-masked `egui::TextEdit` backed by a transient `String` copy
//! of the buffer; on frame completion the transient `String` is consumed
//! back into the buffer and dropped (and zeroed by `Zeroizing::Drop`).
//!
//! # Security
//! - Primary buffer: heap-allocated `Zeroizing<Vec<u8>>`; zeroed on drop
//!   by `Zeroizing`'s `Drop` impl.
//! - egui undo ring: NOT addressed. egui's `TextEditState` holds `String`
//!   snapshots in the undo buffer. This is a second-tier gap documented
//!   in Section A R-1 / FOLLOWUPS `gui-secret-buffer-allocator-residue`
//!   and deferred beyond v0.2.
//! - Transient-frame `String`: heap-allocated for the scope of one egui
//!   `update()` call; dropped at end of frame. Allocator-residue caveat
//!   remains; the transient is wrapped in `Zeroizing::new(...)` only
//!   when extracted via [`as_string`] (R1 N-1 fold — `assemble_argv`
//!   does the wrap at its call site).
//!
//! # Clone is deliberately NOT implemented
//! A clone of a secret buffer is a second copy of the secret in memory.
//! The security boundary is "one live buffer per widget, zeroed on drop".
//! This forces `FormState` to drop its `Clone` derive too (transitive).

use eframe::egui;
use zeroize::Zeroize;
use zeroize::Zeroizing;

/// v0.40.0 (Item 3) — the egui Context-data key under which any
/// `SecretLineEdit::show` raises an over-threshold-paste signal for the frame.
/// `update()` reads+clears it once (`remove_temp`) and fires the paste-warn
/// modal. This bus is the SINGLE chokepoint for every `show` call site (the 3
/// direct + transitive archetype paths) — do NOT reintroduce per-site
/// return-value wiring.
pub fn paste_warn_id() -> egui::Id {
    egui::Id::new("gui_secret_paste_warn_pending")
}

/// Secret-bearing single-line text widget. See module docs.
#[derive(Default)]
pub struct SecretLineEdit {
    buf: Zeroizing<Vec<u8>>,
}

impl std::fmt::Debug for SecretLineEdit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretLineEdit")
            .field("len", &self.buf.len())
            .finish()
    }
}

impl SecretLineEdit {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a `SecretLineEdit` pre-populated with the given text.
    /// Mirrors `secrets::SecretBuffer::from_text` shape. The bytes are
    /// owned by the `Zeroizing<Vec<u8>>` buffer immediately; the input
    /// `&str`'s backing storage is not zeroed (caller's responsibility).
    /// Primary use is test setup; production form mutation flows through
    /// [`show`].
    pub fn from_text(s: &str) -> Self {
        Self {
            buf: Zeroizing::new(s.as_bytes().to_vec()),
        }
    }

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

    /// Extract the current value as a `Zeroizing<String>` for argv
    /// assembly. The `Zeroizing` wrap engages `Drop` to zero the
    /// transient `String`'s heap allocation past the call's end; this
    /// is a best-effort guarantee, the allocator-residue caveat
    /// (FOLLOWUPS `gui-secret-buffer-allocator-residue`) still applies.
    ///
    /// B.1 R1 I-2 fold: the return type was previously `String` with a
    /// doc-only "caller MUST wrap" obligation. Type-level enforcement
    /// makes the contract compile-time-checked.
    pub fn as_string(&self) -> Zeroizing<String> {
        Zeroizing::new(String::from_utf8(self.buf.to_vec()).unwrap_or_default())
    }

    /// True iff the buffer is non-empty. Mirrors `FlagValue::Text`
    /// present-check semantics, used by `secrets::should_confirm_run`.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Explicit zero — called from `secrets::zeroize_form_state` on exit.
    pub fn zeroize(&mut self) {
        self.buf.zeroize();
    }
}

// `_doctest_wrap_pattern` removed in B.1 R1 I-2 fold — `as_string()` now
// returns `Zeroizing<String>` directly, making the wrap contract
// type-level rather than doc-only.
