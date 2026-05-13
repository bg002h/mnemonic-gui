//! v0.2 Phase B.1: secret-widget paste-warn modal trigger.
//!
//! R1 C-1 fold (Section C iterative-review log): this cell was originally
//! `widget_interaction::cell_3_paste_warn_modal_trigger` in Phase A.3.
//! Splitting it to a separate test binary lets A.3 land without a
//! compile-time dependency on `SecretLineEdit` (which Phase B.1
//! introduces); A.3's `widget_interaction.rs` cells stay observable RED
//! → GREEN on their own merits, and this file lands GREEN alongside the
//! `SecretLineEdit` implementation in B.1.
//!
//! The cell asserts:
//!   1. `SecretLineEdit::from_text("…")` populates the buffer.
//!   2. `secrets::should_warn_on_paste(flag, len)` returns `true` for
//!      a secret-class flag with paste length ≥ PASTE_WARN_THRESHOLD.
//!   3. The modal-state predicate transitions when a simulated paste
//!      crosses the threshold.
//!
//! Note: the live "modal state set on the app" check (`app.pending_paste_warn`)
//! requires driving `MnemonicGuiApp` through an egui frame and observing
//! its modal-state field. That field is currently private to main.rs and
//! cannot be queried from an integration test. The cell tests the
//! observable predicate (`should_warn_on_paste`) and the secret-widget
//! buffer transition; the higher-fidelity app-state assertion is deferred
//! to a future test scaffold change (out of B.1 scope).

use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::schema::{self, FlagSchema};
use mnemonic_gui::secrets::{should_warn_on_paste, PASTE_WARN_THRESHOLD};

fn passphrase_flag() -> &'static FlagSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "bundle")
        .expect("bundle subcommand present")
        .flags
        .iter()
        .find(|f| f.name == "--passphrase")
        .expect("--passphrase flag present on bundle")
}

#[test]
fn cell_paste_warn_modal_trigger() {
    let flag = passphrase_flag();

    // Threshold: paste of exactly PASTE_WARN_THRESHOLD chars triggers.
    assert!(
        should_warn_on_paste(flag, PASTE_WARN_THRESHOLD),
        "paste of {} chars on secret-class flag must trigger modal",
        PASTE_WARN_THRESHOLD
    );
    // Below threshold: no modal.
    assert!(
        !should_warn_on_paste(flag, PASTE_WARN_THRESHOLD - 1),
        "paste below threshold must NOT trigger modal"
    );

    // Buffer-side state transition: SecretLineEdit reflects the pasted
    // content. Construct a non-empty widget and assert is_empty()/len.
    let pasted = "p".repeat(PASTE_WARN_THRESHOLD);
    let widget = SecretLineEdit::from_text(&pasted);
    assert!(!widget.is_empty());
    // B.1 R1 I-2 fold: as_string() returns Zeroizing<String>.
    assert_eq!(widget.as_string().as_str(), pasted);

    // Zeroizing the widget clears the buffer (used by the run-confirm
    // post-cancel cleanup path).
    let mut widget = widget;
    widget.zeroize();
    assert!(widget.is_empty());
}
