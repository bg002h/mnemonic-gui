//! Regression for v0.1.2: every `egui::ComboBox` in `src/form/widget.rs`
//! must use `from_id_salt(...)` with a unique key, never `from_label("")`
//! or `from_label(" ")`.
//!
//! Root cause of the v0.1.1 dropdown bug: `ComboBox::from_label(label)`
//! derives the egui widget ID from `label`. egui keys popup open-state,
//! hover-state, and selection-state by that ID. When multiple ComboBoxes
//! on the same page all use `""` (or `" "`), they share an ID and:
//!
//!   * The popup either fails to open, or
//!   * Hover/selection state leaks across every ComboBox on the page
//!     ("every list on the page gets highlighted when one is clicked").
//!
//! The convention used by `src/form/slot_editor.rs:160` is
//! `ComboBox::from_id_salt((const, varying_key))`. This audit pins that
//! convention into `widget.rs` so future edits that reach for the
//! quicker-typing `from_label("")` are caught at test-time.

use std::path::PathBuf;

fn widget_source() -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("src/form/widget.rs");
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("read {}: {}", p.display(), e))
}

#[test]
fn widget_combobox_never_uses_from_label() {
    let src = widget_source();
    assert!(
        !src.contains("ComboBox::from_label"),
        "src/form/widget.rs must not use ComboBox::from_label — \
         from_label(\"\") and from_label(\" \") cause egui widget-ID \
         collision across every dropdown on the page. Use \
         ComboBox::from_id_salt((const, flag.name)) — see \
         src/form/slot_editor.rs:160 for the convention."
    );
}

#[test]
fn widget_combobox_uses_from_id_salt() {
    let src = widget_source();
    assert!(
        src.contains("ComboBox::from_id_salt"),
        "src/form/widget.rs must use ComboBox::from_id_salt for every \
         ComboBox so each dropdown gets a unique egui widget ID."
    );
}
