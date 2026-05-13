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
        if response.changed() {
            // Overwrite the primary buffer with the new bytes; the old
            // buffer contents are zeroed via Zeroize when we reassign.
            self.buf.zeroize();
            *self.buf = transient.as_bytes().to_vec();
        }
        // Zero the transient before drop. allocator residue caveat
        // remains (one-frame scope; not a guarantee, just best-effort).
        transient.zeroize();
    }

    /// Extract the current value as a `String` for argv assembly.
    ///
    /// **The caller MUST wrap the returned `String` in `Zeroizing::new(...)`**
    /// (R1 N-1 fold). The transient String is heap-allocated, and
    /// although its scope is one argv-assembly call, allocator residue
    /// persists past drop. `assemble_argv` follows this contract.
    pub fn as_string(&self) -> String {
        String::from_utf8(self.buf.to_vec()).unwrap_or_default()
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

/// Compile-time SAFETY check that a `Zeroizing::new(String)` wrap is the
/// expected idiom for transient secrets extracted via [`as_string`].
/// Mostly documentation — the wrap site is in `invocation.rs`.
#[allow(dead_code)]
fn _doctest_wrap_pattern() {
    let w = SecretLineEdit::new();
    let _wrapped: Zeroizing<String> = Zeroizing::new(w.as_string());
}
