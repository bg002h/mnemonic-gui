//! SlotEditor composite widget for the `--slot @N.<subkey>=<value>`
//! repeating grammar (SPEC §B.4).
//!
//! P1 `gui`-feature split: the egui-FREE data model (`SlotSubkey` /
//! `SlotRow` / `SlotState` + the `remove_row` / `detect_slot_index_gaps`
//! helpers) lives in the non-gated [`crate::form::slot_model`] so the form
//! model + headless emit-mode build without egui; this gated module owns
//! only the `render(&mut egui::Ui, …)` widget. The model types are
//! re-exported below so existing `slot_editor::SlotState` etc. paths
//! continue to resolve under the `gui` feature.

use eframe::egui;

pub use crate::form::slot_model::{
    detect_slot_index_gaps, remove_row, SlotRow, SlotState, SlotSubkey,
};

/// Render the SlotEditor inside a vertical scroll area (SPEC §B.4 R1 L-2:
/// row-height ~32px, no virtualization in v0.1 — N ≤ 16 cosigners bounds
/// the row count below the threshold where virtualization matters).
///
/// **v0.8.1 F3 — `path_hint`:** when Some, the per-row text-edit widget
/// renders the hint string as a placeholder (`egui::TextEdit::hint_text`)
/// whenever `row.subkey == SlotSubkey::Path` AND `row.value.is_empty()`.
/// Pass None to preserve pre-v0.8.1 rendering. Main.rs computes the hint
/// from `descriptor_non_canonical_default_path_notice`'s underlying
/// machinery and threads it through here.
pub fn render(ui: &mut egui::Ui, state: &mut SlotState, path_hint: Option<&str>) {
    egui::ScrollArea::vertical()
        .max_height(320.0) // ~10 rows at default row-height before scroll
        .show(ui, |ui| {
            let mut remove_idx: Option<usize> = None;
            for (i, row) in state.rows.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("@");
                    ui.add(egui::DragValue::new(&mut row.index).range(0u8..=15));
                    ui.label(".");
                    egui::ComboBox::from_id_salt(("slot_subkey", i))
                        .selected_text(row.subkey.as_str())
                        .show_ui(ui, |ui| {
                            for opt in SlotSubkey::ALL {
                                ui.selectable_value(&mut row.subkey, *opt, opt.as_str());
                            }
                        });
                    ui.label("=");
                    // v0.38.0: mask secret-bearing slot values. Gate on
                    // is_secret_bearing() FIRST — Path is never secret, so
                    // the password edit and the (Path, hint) edit are
                    // mutually exclusive (.password never combines with
                    // hint_text).
                    if row.subkey.is_secret_bearing() {
                        // v0.57.0: secret slot rows get the reveal (👁) eye
                        // (site #2, secret arm ONLY — the `(Path, hint)` arm
                        // below stays eye-free). Per-row stable id via this
                        // row's `ui.id()`.
                        let ctx = ui.ctx().clone();
                        let field_id = ui.unique_id().with("slot_secret_reveal");
                        let reveal =
                            crate::form::secret_widget::reveal_toggle(ui, &ctx, field_id);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut row.value)
                                .id(field_id)
                                .password(!reveal),
                        );
                        crate::form::secret_widget::clear_reveal_on_blur(&ctx, field_id, &resp);
                    } else {
                        match (row.subkey, path_hint) {
                            (SlotSubkey::Path, Some(hint)) if row.value.is_empty() => {
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.value).hint_text(hint),
                                );
                            }
                            _ => {
                                ui.text_edit_singleline(&mut row.value);
                            }
                        }
                    }
                    if ui.button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }
            if let Some(i) = remove_idx {
                remove_row(state, i);
            }
            if ui.button("+ Add slot").clicked() {
                state.rows.push(SlotRow::default());
            }
            // v0.7.1 SPEC §6.6 row 8 — inline contiguity warning. Pre-checks
            // the CLI's mode-violation row 8 (`error: slot indices must be
            // contiguous starting at @0; missing @{i}`) so the user sees
            // the issue before hitting the CLI error. Renders nothing when
            // the slot set is contiguous (or empty). Option A pattern: no
            // toolkit wire-format change; the CLI is still authoritative.
            let gaps = detect_slot_index_gaps(&state.rows);
            if !gaps.is_empty() {
                let missing = gaps
                    .iter()
                    .map(|i| format!("@{i}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 165, 0),
                    format!(
                        "⚠ slot indices must be contiguous starting at @0; missing {missing}"
                    ),
                );
            }
        });
}
