//! Per-FlagKind widget renderer (SPEC §6, §B.3).
//!
//! Each `render_*` function takes an `egui::Ui` reference plus a mutable
//! `FlagValue` slot and draws the appropriate input. Phase 2 ships the
//! function surface; Phase 4 wires them into the eframe app loop and
//! Phase 5 layers conditional-visibility on top.

use eframe::egui;

use crate::schema::{
    FlagKind, FlagSchema, FlagValue, TaggedOrIndexedValue, TimestampValue,
};

/// Render the widget appropriate for `flag.kind`, mutating `value` in place.
pub fn render(ui: &mut egui::Ui, flag: &FlagSchema, value: &mut FlagValue) {
    ui.horizontal(|ui| {
        ui.label(flag.name).on_hover_text(flag.help);
        match (&flag.kind, value) {
            (FlagKind::Text, FlagValue::Text(s)) => {
                ui.text_edit_singleline(s);
            }
            (FlagKind::Number { min, max }, FlagValue::Number(n)) => {
                ui.add(egui::DragValue::new(n).range(*min..=*max));
            }
            (FlagKind::Dropdown(opts), FlagValue::Dropdown(sel)) => {
                egui::ComboBox::from_id_salt(("flag_dropdown", flag.name))
                    .selected_text(sel.as_str())
                    .show_ui(ui, |ui| {
                        for opt in *opts {
                            ui.selectable_value(sel, (*opt).to_string(), *opt);
                        }
                    });
            }
            (FlagKind::Boolean, FlagValue::Boolean(b)) => {
                ui.checkbox(b, "");
            }
            (FlagKind::Range, FlagValue::Range(a, b)) => {
                ui.add(egui::DragValue::new(a));
                ui.label(",");
                ui.add(egui::DragValue::new(b));
            }
            (FlagKind::Timestamp, FlagValue::Timestamp(t)) => {
                let mut is_now = matches!(t, TimestampValue::Now);
                ui.checkbox(&mut is_now, "now");
                if is_now {
                    *t = TimestampValue::Now;
                } else {
                    let mut n = match t {
                        TimestampValue::Now => 0u64,
                        TimestampValue::Unix(n) => *n,
                    };
                    ui.add(egui::DragValue::new(&mut n));
                    *t = TimestampValue::Unix(n);
                }
            }
            (
                FlagKind::NodeValueComposite(opts),
                FlagValue::NodeValueComposite { node, value },
            ) => {
                egui::ComboBox::from_id_salt(("flag_nodevalue", flag.name))
                    .selected_text(node.as_str())
                    .show_ui(ui, |ui| {
                        for opt in *opts {
                            ui.selectable_value(node, (*opt).to_string(), *opt);
                        }
                    });
                ui.label("=");
                ui.text_edit_singleline(value);
            }
            (FlagKind::TaggedOrIndexed(tags), FlagValue::TaggedOrIndexed(tv)) => {
                // v0.1: emit a free-form text field + radio for Tag/Indexed.
                // The full design (per-tag chooser + cosigner-index picker)
                // is Phase 5's conditional concern.
                let mut is_tag = matches!(tv, TaggedOrIndexedValue::Tag(_));
                ui.radio_value(&mut is_tag, true, "tag");
                ui.radio_value(&mut is_tag, false, "@N");
                if is_tag {
                    let mut s = match tv {
                        TaggedOrIndexedValue::Tag(s) => s.clone(),
                        TaggedOrIndexedValue::Indexed(_) => {
                            tags.first().map(|s| (*s).to_string()).unwrap_or_default()
                        }
                    };
                    egui::ComboBox::from_id_salt(("flag_tagged", flag.name))
                        .selected_text(s.as_str())
                        .show_ui(ui, |ui| {
                            for opt in *tags {
                                ui.selectable_value(&mut s, (*opt).to_string(), *opt);
                            }
                        });
                    *tv = TaggedOrIndexedValue::Tag(s);
                } else {
                    let mut n: u8 = match tv {
                        TaggedOrIndexedValue::Indexed(n) => *n,
                        TaggedOrIndexedValue::Tag(_) => 0,
                    };
                    ui.add(egui::DragValue::new(&mut n));
                    *tv = TaggedOrIndexedValue::Indexed(n);
                }
            }
            (FlagKind::Path { stdio_sentinel }, FlagValue::Path(p)) => {
                ui.text_edit_singleline(p);
                if *stdio_sentinel && ui.button("stdio").clicked() {
                    *p = "-".to_string();
                }
            }
            // FlagKind/FlagValue type mismatch — defensively render the
            // flag name as disabled (the form-state initializer normally
            // ensures matching shapes; this branch guards against bugs).
            _ => {
                ui.label("(value-shape mismatch — see form-state init)");
            }
        }
        if flag.required {
            ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
        }
    });
}
