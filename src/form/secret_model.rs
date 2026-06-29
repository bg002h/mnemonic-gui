//! Egui-free secret-buffer data type (extracted from `secret_widget.rs` in
//! the P1 `gui`-feature split — SPEC §3 general invariant). `SecretLineEdit`
//! is a pure `struct { buf: Zeroizing<Vec<u8>> }` consumed non-gated by
//! `schema::FormState.secret_widgets` (+ `secrets`/`persistence` through that
//! field), so it must live in an egui-free home for the form model + the
//! headless `gui-render` emit-mode to build without egui. The egui-coupled
//! `show(&mut egui::Ui, …)` / `paste_warn_id() -> egui::Id` stay in the
//! gated `secret_widget.rs` (which re-exports this type).
//!
//! # Security
//! - Primary buffer: heap-allocated `Zeroizing<Vec<u8>>`; zeroed on drop
//!   by `Zeroizing`'s `Drop` impl.
//! - egui undo ring: NOT addressed (the gap is in the gated `show` widget,
//!   documented there + in FOLLOWUPS `gui-secret-buffer-allocator-residue`).
//!
//! # Clone is deliberately NOT implemented
//! A clone of a secret buffer is a second copy of the secret in memory.
//! The security boundary is "one live buffer per widget, zeroed on drop".
//! This forces `FormState` to drop its `Clone` derive too (transitive).

use zeroize::Zeroize;
use zeroize::Zeroizing;

/// Secret-bearing single-line text widget's data buffer. See module docs.
///
/// `buf` is `pub(crate)` so the egui-coupled `show` method (kept in the
/// gated `secret_widget.rs`, an inherent-impl split within this crate) can
/// mutate it; it is never exposed beyond the crate.
#[derive(Default)]
pub struct SecretLineEdit {
    pub(crate) buf: Zeroizing<Vec<u8>>,
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
    /// `show` (the gated `secret_widget.rs`).
    pub fn from_text(s: &str) -> Self {
        Self {
            buf: Zeroizing::new(s.as_bytes().to_vec()),
        }
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
