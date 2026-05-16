//! Per-FlagKind widget renderer (SPEC §6, §B.3).
//!
//! Each `render_*` function takes an `egui::Ui` reference plus a mutable
//! `FlagValue` slot and draws the appropriate input. Phase 2 ships the
//! function surface; Phase 4 wires them into the eframe app loop and
//! Phase 5 layers conditional-visibility on top.
//!
//! manual-gui v1.0 (G-P2.2): `render` + `render_with_dispatch` gained
//! `tab` + `subcommand` parameters so the per-flag `?` help-icon button
//! can compose `url::manual_url_for_flag(tab, sub, flag.name)`. SPEC §2.4
//! render-site contract: the per-flag button MUST live inside this body
//! so the P1.4 kittest probe (`harness.query_by_label("?")` against a
//! single `render_with_dispatch` call) can locate it.

use eframe::egui;

use crate::app::CliTab;
use crate::help::url;
use crate::schema::{
    FlagKind, FlagSchema, FlagValue, FormState, TaggedOrIndexedValue, TimestampValue,
};

/// True iff `flag` is one of the Dropdown / NodeValueComposite /
/// TaggedOrIndexed / `repeating: true` shapes that earn a `?` help-icon
/// button per §1.6 Option C. Per §2.4 the button links to the flag
/// anchor (`manual_url_for_flag`), not per-variant.
fn needs_help_icon(flag: &FlagSchema) -> bool {
    matches!(
        flag.kind,
        FlagKind::Dropdown(_) | FlagKind::NodeValueComposite(_) | FlagKind::TaggedOrIndexed(_)
    ) || flag.repeating
}

/// Render the per-flag `?` help-icon button if `flag` is one of the four
/// shapes that earn one. The button's label is the ASCII U+003F `?`
/// character (NOT fullwidth `？` U+FF1F or emoji `❓` U+2753 — the P1.4
/// kittest `harness.query_by_label("?")` is byte-exact). Click triggers
/// `ctx.open_url(OpenUrl::new_tab(url::manual_url_for_flag(...)))`.
fn render_help_icon(ui: &mut egui::Ui, tab: CliTab, subcommand: &str, flag: &FlagSchema) {
    if !needs_help_icon(flag) {
        return;
    }
    let btn = egui::Button::new("?")
        .small()
        .fill(egui::Color32::from_gray(96));
    if ui.add(btn).clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(
            url::manual_url_for_flag(tab, subcommand, flag.name),
        ));
    }
}

/// Render the widget for `flag`, dispatching secret-class flags
/// (`secrets::flag_is_secret(flag) && FlagKind::Text`) to the
/// `SecretLineEdit` widget owned by `state.secret_widgets`, and non-secret
/// flags to the existing [`render`] FlagValue-based path (SPEC §3 / B.1).
///
/// The secret path does NOT write to `state.values`; the secret buffer
/// lives in `state.secret_widgets[flag.name]` and is consumed by
/// `assemble_argv` via the secret-flag branch. This preserves the
/// never-persist invariant by type — `secret_widgets` is `#[serde(skip)]`.
///
/// `tab` + `subcommand` are the call-site context passed through to the
/// per-flag `?` help-icon (G-P2.2 / §2.4 render-site contract). For the
/// secret path here, the icon only renders if the secret flag is itself
/// repeating (e.g., `--ms1`); FlagKind::Text non-repeating secret flags
/// are tooltip-only.
pub fn render_with_dispatch(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    flag: &FlagSchema,
    state: &mut FormState,
) {
    if crate::secrets::flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text) {
        ui.horizontal(|ui| {
            let widget = state.secret_widgets.entry(flag.name.to_string()).or_default();
            widget.show(ui, flag.name, flag.help);
            render_help_icon(ui, tab, subcommand, flag);
            if flag.required {
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
            }
        });
        return;
    }

    // Non-secret path: look up FlagValue from state.values, render via
    // the existing FlagValue-based renderer, then write back.
    let idx = state.values.iter().position(|(k, _)| k == flag.name);
    let mut value = match idx {
        Some(i) => state.values[i].1.clone(),
        None => default_flag_value_for(&flag.kind),
    };
    render(ui, tab, subcommand, flag, &mut value);
    match idx {
        Some(i) => state.values[i].1 = value,
        None => state.values.push((flag.name.to_string(), value)),
    }
}

/// Construct the default `FlagValue` for a given `FlagKind`, used as the
/// initial state-of-form entry the first time a flag's widget is rendered.
///
/// v0.6.0 P3: Number / Range / Timestamp / TaggedOrIndexed return `Unset`
/// rather than a seeded numeric value (was `Number(*min)`, `Range(0, 999)`,
/// etc.). Pre-P3, the first render of any of those widgets would push a
/// concrete value into `state.values`; the argv assembler would then emit
/// `--<flag> <min>` for any numeric flag the user hadn't touched, sending
/// bogus flags to the CLI. With Unset the widget renders a `Set` affordance
/// instead; the user must opt-in to a value before emission. Kinds with a
/// natural empty representation (Text / Dropdown / Path / NodeValueComposite
/// / Boolean) keep their empty-default behavior.
pub fn default_flag_value_for(kind: &FlagKind) -> FlagValue {
    match kind {
        FlagKind::Text => FlagValue::Text(String::new()),
        FlagKind::Dropdown(opts) => FlagValue::Dropdown(
            opts.first().map(|s| (*s).to_string()).unwrap_or_default(),
        ),
        FlagKind::Boolean => FlagValue::Boolean(false),
        FlagKind::NodeValueComposite(opts) => FlagValue::NodeValueComposite {
            node: opts.first().map(|s| (*s).to_string()).unwrap_or_default(),
            value: String::new(),
        },
        FlagKind::Path { .. } => FlagValue::Path(String::new()),
        // v0.6.0 P3 Unset-default kinds. Click-to-seed via `seeded_value_for`.
        FlagKind::Number { .. }
        | FlagKind::Range
        | FlagKind::Timestamp
        | FlagKind::TaggedOrIndexed(_) => FlagValue::Unset,
    }
}

/// v0.6.0 P3: kind-specific seeded value used when the user clicks the
/// `Set` affordance on an Unset numeric/range/timestamp/tagged widget. Always
/// returns a concrete (non-Unset) value for the four Unset-default kinds.
/// For non-Unset-default kinds, returns the same value as
/// `default_flag_value_for` (idempotent — the widget would never call this
/// for an already-seeded kind in practice).
pub fn seeded_value_for(kind: &FlagKind) -> FlagValue {
    match kind {
        FlagKind::Number { min, .. } => FlagValue::Number(*min),
        FlagKind::Range => FlagValue::Range(0, 999),
        FlagKind::Timestamp => FlagValue::Timestamp(TimestampValue::Now),
        FlagKind::TaggedOrIndexed(tags) => FlagValue::TaggedOrIndexed(
            TaggedOrIndexedValue::Tag(
                tags.first().map(|s| (*s).to_string()).unwrap_or_default(),
            ),
        ),
        // For kinds without a natural Unset state, fall through to the
        // default — `default_flag_value_for` returns the same concrete value
        // it always did pre-P3 for these.
        other => default_flag_value_for(other),
    }
}

/// Render the widget appropriate for `flag.kind`, mutating `value` in place.
///
/// `tab` + `subcommand` are passed through to the per-flag `?` help-icon
/// button (G-P2.2 / §2.4 render-site contract). The icon is rendered
/// inside the same `ui.horizontal` row as the flag label so the P1.4
/// kittest can find it via `harness.query_by_label("?")`.
pub fn render(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    flag: &FlagSchema,
    value: &mut FlagValue,
) {
    // v0.6.0 P3 — transition sentinel for Unset ↔ seeded swaps. Mutating
    // *value mid-match would conflict with the destructured borrow inside
    // each arm; collect any swap intent here and apply it after the match.
    let mut transition: Option<FlagValue> = None;
    ui.horizontal(|ui| {
        ui.label(flag.name).on_hover_text(flag.help);
        render_help_icon(ui, tab, subcommand, flag);
        match (&flag.kind, &mut *value) {
            (FlagKind::Text, FlagValue::Text(s)) => {
                ui.text_edit_singleline(s);
            }
            // v0.6.0 P3: Number / Range / Timestamp / TaggedOrIndexed
            // initial-Unset state — render a `Set` affordance that opts the
            // user into a seeded numeric value. Pre-P3, the seeded default
            // shipped automatically and emitted as `--<flag> <min>` even when
            // untouched (bogus argv noise).
            (
                FlagKind::Number { .. }
                | FlagKind::Range
                | FlagKind::Timestamp
                | FlagKind::TaggedOrIndexed(_),
                FlagValue::Unset,
            ) => {
                if ui
                    .button("Set")
                    .on_hover_text("seed default + edit")
                    .clicked()
                {
                    transition = Some(seeded_value_for(&flag.kind));
                }
            }
            (FlagKind::Number { min, max }, FlagValue::Number(n)) => {
                ui.add(egui::DragValue::new(n).range(*min..=*max));
                if ui.small_button("✕").on_hover_text("clear (Unset)").clicked() {
                    transition = Some(FlagValue::Unset);
                }
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
                if ui.small_button("✕").on_hover_text("clear (Unset)").clicked() {
                    transition = Some(FlagValue::Unset);
                }
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
                if ui.small_button("✕").on_hover_text("clear (Unset)").clicked() {
                    transition = Some(FlagValue::Unset);
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
                if ui.small_button("✕").on_hover_text("clear (Unset)").clicked() {
                    transition = Some(FlagValue::Unset);
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
            // v0.6.0 fold: a stray FlagValue::Unset for a non-Unset-default
            // kind (Text/Dropdown/Path/Composite/Boolean) also lands here —
            // recover by re-seeding to the default.
            _ => {
                if matches!(*value, FlagValue::Unset) {
                    transition = Some(default_flag_value_for(&flag.kind));
                } else {
                    ui.label("(value-shape mismatch — see form-state init)");
                }
            }
        }
        if flag.required {
            ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
        }
    });
    if let Some(new) = transition {
        *value = new;
    }
}
