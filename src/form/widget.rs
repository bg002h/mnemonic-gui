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
    disabled_options: &[String],
) {
    if crate::secrets::flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text) {
        // v0.31.1 (repeating-secrets SPEC §1): the secret branch splits on
        // `flag.repeating`. Scalar secrets keep byte-identical behavior
        // (one widget = the 1-element vec); repeating secrets mirror the
        // v0.30.0 non-secret repeating UX (header + per-row SecretLineEdit
        // + per-row ✕ + "+ add"), with the rows living in `secret_widgets`
        // — NEVER `state.values` — so the never-persist invariant stays
        // TYPE-level (#[serde(skip)]).
        if !flag.repeating {
            // Scalar (non-repeating) secret — behavior byte-identical to
            // pre-v0.31.1: one widget = vec[0].
            ui.horizontal(|ui| {
                let rows = state
                    .secret_widgets
                    .entry(flag.name.to_string())
                    .or_insert_with(|| vec![crate::form::secret_widget::SecretLineEdit::default()]);
                if rows.is_empty() {
                    // Defensive: an externally-inserted empty vec must not
                    // panic the render (e.g. test-constructed state).
                    rows.push(crate::form::secret_widget::SecretLineEdit::default());
                }
                rows[0].show(ui, flag.name, flag.help);
                render_help_icon(ui, tab, subcommand, flag);
                if flag.required {
                    ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
                }
            });
            return;
        }

        // Repeating secret (--ms1 on verify-bundle/import-wallet; --share
        // on slip39-combine/ms-shares-combine). Mirrors render_repeating's
        // layout, adapted to the secret_widgets rows:
        //   - Header row ALWAYS: label + `?` (repeating flags qualify per
        //     needs_help_icon) + required marker + "+ add" (appends an
        //     empty SecretLineEdit).
        //   - Per-frame required-seed (the v0.30.0 rule applied to the
        //     vec): an empty vec for a REQUIRED repeating secret flag
        //     seeds ONE empty widget (--share sites are required); optional
        //     (--ms1) seeds none. Removing the last required row respawns
        //     it next frame — intended.
        //   - One masked TextEdit per row + per-row ✕ (collect-then-apply;
        //     egui's positional auto-IDs keep the rows distinct — no salt
        //     work needed). Empty rows emit nothing (assembler-side guard).
        let rows = state.secret_widgets.entry(flag.name.to_string()).or_default();
        ui.horizontal(|ui| {
            ui.label(flag.name).on_hover_text(flag.help);
            render_help_icon(ui, tab, subcommand, flag);
            if flag.required {
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
            }
            if ui.button("+ add").clicked() {
                rows.push(crate::form::secret_widget::SecretLineEdit::new());
            }
        });
        if flag.required && rows.is_empty() {
            rows.push(crate::form::secret_widget::SecretLineEdit::new());
        }
        let mut remove_at: Option<usize> = None;
        for (row_idx, w) in rows.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // The header row owns the label/help chrome (the
                // render_repeating convention); rows render the masked
                // TextEdit only.
                w.show(ui, "", "");
                if ui.small_button("✕").on_hover_text("remove row").clicked() {
                    remove_at = Some(row_idx);
                }
            });
        }
        if let Some(i) = remove_at {
            rows.remove(i);
        }
        return;
    }

    // v0.30.0 SPEC §3: NON-secret repeating flags get the generic
    // multi-row widget (header + N rows + per-row remove + "+ add").
    // The branch order is load-bearing: the secret check above runs FIRST,
    // so secret repeating flags (import-wallet --ms1, --share) route to
    // the per-row secret widget above (v0.31.1), keeping their buffers in
    // `secret_widgets` rather than `state.values`.
    if flag.repeating {
        render_repeating(ui, tab, subcommand, flag, state, disabled_options, None);
        return;
    }

    // Non-secret path: look up FlagValue from state.values, render via
    // the existing FlagValue-based renderer, then write back. The Number
    // widget reads state.slot_count() via the NumberMax::FromSlotCount
    // resolve; the Dropdown widget consumes `disabled_options` directly
    // (extracted upstream from the vis map's DisableOptions entries for
    // this flag — orthogonal to the primary first-rule-wins Visibility
    // which decorates the label).
    //
    // v0.10.0 B.3 (D31): when the flag has a schema-declared `default_value`
    // (mirrored from the toolkit v5 schema), we consult that single source
    // of truth via `default_flag_value_for_flag(flag)`. Pre-v0.10.0 the
    // Dropdown widget seeded `opts[0]` which only coincidentally matched
    // the toolkit's default (fragile across schema reorderings).
    let idx = state.values.iter().position(|(k, _)| k == flag.name);
    let mut value = match idx {
        Some(i) => state.values[i].1.clone(),
        None => default_flag_value_for_flag(flag),
    };
    render(ui, tab, subcommand, flag, &mut value, state, disabled_options);
    match idx {
        Some(i) => state.values[i].1 = value,
        None => state.values.push((flag.name.to_string(), value)),
    }
}

/// v0.31.0 (archetype forms SPEC §4, R0-r1 M4 + R0-r2 m2) — named seam for
/// the repeating-header annotation. Carries BOTH the display label (e.g.
/// `"(min 2)"` for key-repeatable params, `"(exactly 1)"` for the C1
/// non-repeatable-with-surplus-rows arm) AND the add-suppression bit — one
/// seam serves both arms. `None` (every pre-v0.31.0 caller) renders the
/// header byte-unchanged.
pub(crate) struct RepeatAnnotation {
    /// Header label rendered after the required marker (weak text).
    pub label: String,
    /// When true the "+ add" button is suppressed (the C1 arm: surplus
    /// rows stay visible and removable, but no new rows can be added).
    pub suppress_add: bool,
}

/// v0.30.0 SPEC §3 — generic multi-row widget for NON-secret repeating
/// flags (`--key`, `--recovery-key`, `--allow`, `--md1`, `--cosigner`,
/// `--to`, `--group`, `--add-path`, `--target-address`, …; `--slot` keeps
/// its dedicated SlotEditor branch in main.rs). The argv assembler already
/// emits every matching `state.values` row for `flag.repeating`
/// (invocation.rs); this widget closes the render-side gap (pre-v0.30.0,
/// `render_with_dispatch` rendered at most ONE row per flag name).
///
/// Layout:
///   - Header row (R0-r1 I5) renders REGARDLESS of row count: flag label +
///     `?` help icon + required marker + "+ add" button — zero rows must
///     not make the flag invisible or drop the help affordance.
///   - One widget row per `(k, v)` entry in `state.values` with
///     `k == flag.name`, each with a per-row remove button. Remove-intents
///     are collected during the loop and applied AFTER it (R0-r1 M7 — the
///     existing `transition` pattern; mutating `state.values` mid-iteration
///     is a borrow error).
///   - Seed rule (R0-r1 C2 / R0-r3 M-i): ANY render observing zero rows for
///     a REQUIRED repeating flag seeds one row with
///     `default_flag_value_for_flag(flag)` — convert `--to` keeps seeding
///     `Dropdown("phrase")` exactly as the pre-v0.30.0 scalar path did, so
///     the default convert form stays runnable. Optional repeating flags
///     (`--key`/`--recovery-key`/`--allow`) seed NOTHING (never auto-emit
///     an allowance). Removing the LAST row of a required flag respawns it
///     next frame (the per-frame seed re-fires) — INTENDED (R0-r2 M-C):
///     a required flag always shows at least one row.
///   - "+ add" appends `add_row_value_for(&flag.kind)`: Text rows seed
///     `Text("")`; Dropdown rows seed `Dropdown("")` — NOT `opts[0]`
///     (R0-r1 I3: an added-but-untouched `--allow` row must emit NOTHING;
///     accidental emission of an allowance is a funds-safety opt-out).
///
/// v0.31.0: `pub(crate)` (was private) so `form/archetype_form.rs` can
/// render key params through the SAME widget with a header annotation
/// (R0-r2 m2); `annotation: None` keeps every v0.30.0 call site
/// byte-unchanged.
pub(crate) fn render_repeating(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    flag: &FlagSchema,
    state: &mut FormState,
    disabled_options: &[String],
    annotation: Option<RepeatAnnotation>,
) {
    // Header row: label + `?` + required marker + optional annotation +
    // "+ add" (suppressible via the annotation — R0-r1 C1).
    ui.horizontal(|ui| {
        ui.label(flag.name).on_hover_text(flag.help);
        render_help_icon(ui, tab, subcommand, flag);
        if flag.required {
            ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
        }
        if let Some(ann) = &annotation {
            ui.weak(&ann.label);
        }
        let suppress_add = annotation.as_ref().is_some_and(|a| a.suppress_add);
        if !suppress_add && ui.button("+ add").clicked() {
            state
                .values
                .push((flag.name.to_string(), add_row_value_for(&flag.kind)));
        }
    });

    // Per-frame required-row seed (see fn doc).
    let has_rows = state.values.iter().any(|(k, _)| k == flag.name);
    if flag.required && !has_rows {
        state
            .values
            .push((flag.name.to_string(), default_flag_value_for_flag(flag)));
    }

    // Render every matching row. `row_idx` (0-based position among THIS
    // flag's rows) is threaded into the egui ID salts (R0-r1 I4 — N
    // same-name rows sharing a ComboBox ID is the v0.1.1 popup-state-leak
    // bug class); `i` is the absolute `state.values` index used for
    // write-back + removal.
    let indices: Vec<usize> = state
        .values
        .iter()
        .enumerate()
        .filter(|(_, (k, _))| k == flag.name)
        .map(|(i, _)| i)
        .collect();
    let mut remove_at: Option<usize> = None;
    for (row_idx, &i) in indices.iter().enumerate() {
        let mut value = state.values[i].1.clone();
        ui.horizontal(|ui| {
            render_row(
                ui,
                tab,
                subcommand,
                flag,
                &mut value,
                state,
                disabled_options,
                Some(row_idx),
            );
            if ui.small_button("✕").on_hover_text("remove row").clicked() {
                remove_at = Some(i);
            }
        });
        state.values[i].1 = value;
    }
    if let Some(i) = remove_at {
        state.values.remove(i);
    }
}

/// v0.30.0 SPEC §3 — the "+ add" row seed. Text rows seed empty;
/// Dropdown rows seed `Dropdown("")` rather than
/// `default_flag_value_for`'s `opts[0]` (R0-r1 I3): `emit_one` skips an
/// empty Dropdown, so an added-but-untouched row emits NOTHING — on
/// `--allow`, accidental emission of `opts[0]` would be a silent
/// funds-safety opt-out. Other kinds fall through to the kind-only
/// default (`Unset` for Number/Range/Timestamp/TaggedOrIndexed).
fn add_row_value_for(kind: &FlagKind) -> FlagValue {
    match kind {
        FlagKind::Dropdown(_) => FlagValue::Dropdown(String::new()),
        other => default_flag_value_for(other),
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
///
/// v0.10.0 B.3 (D31): kept as the kind-only fallback; flag-aware callers
/// should prefer `default_flag_value_for_flag(&FlagSchema)` which consults
/// the schema-declared `default_value` (toolkit v5 single source of truth)
/// for Dropdown / Text / Path kinds.
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

/// v0.10.0 B.3 (D31) — flag-aware default constructor. Reads
/// `flag.default_value` (toolkit v5 schema's per-flag default) and maps
/// it onto a concrete `FlagValue` per the FlagKind dispatch table. Falls
/// back to `default_flag_value_for(&flag.kind)` for:
///   - flags without a schema-declared default (`default_value == None`),
///   - the four Unset-default kinds (Number / Range / Timestamp /
///     TaggedOrIndexed) which keep their click-to-seed UX regardless of
///     the schema default (the schema default is consulted only by the
///     argv assembler's `is_at_default` suppression predicate; the widget
///     still requires user opt-in to emit anything).
///   - parse failures (defensive: bad schema would otherwise crash).
///
/// Dropdown / Text / Path with a declared default use the schema string
/// directly — eliminating the pre-v0.10.0 fragility where Dropdown
/// widgets seeded `opts[0]` which only coincidentally matched the toolkit's
/// default ordering.
pub fn default_flag_value_for_flag(flag: &FlagSchema) -> FlagValue {
    let Some(default_str) = flag.default_value else {
        return default_flag_value_for(&flag.kind);
    };
    match flag.kind {
        FlagKind::Text => FlagValue::Text(default_str.to_string()),
        FlagKind::Dropdown(_) => FlagValue::Dropdown(default_str.to_string()),
        FlagKind::Path { .. } => FlagValue::Path(default_str.to_string()),
        // Boolean / NodeValueComposite: no meaningful default-value mapping;
        // fall through to the kind-only default (Boolean(false), empty
        // composite). Toolkit v5 doesn't emit defaults for these in practice.
        FlagKind::Boolean | FlagKind::NodeValueComposite(_) => {
            default_flag_value_for(&flag.kind)
        }
        // Unset-default kinds: keep the click-to-seed UX. The argv
        // assembler's `is_at_default` consults the schema default
        // separately at emission time; the widget initial state stays
        // Unset so the user must opt in.
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
///
/// v0.30.0: delegates to `render_row` with `row: None` — the scalar path
/// is byte-unchanged (same egui ID salts, same chrome). Repeating rows go
/// through `render_repeating` → `render_row(Some(row_idx))`.
pub fn render(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    flag: &FlagSchema,
    value: &mut FlagValue,
    state: &FormState,
    disabled_options: &[String],
) {
    render_row(ui, tab, subcommand, flag, value, state, disabled_options, None);
}

/// Per-row renderer behind [`render`] (v0.30.0 SPEC §3).
///
/// `row` semantics:
///   - `None` — the scalar (non-repeating) path. Behavior is byte-identical
///     to the pre-v0.30.0 `render`: per-flag chrome (label + `?` help icon +
///     required marker) renders here, and the egui ID salts keep the
///     two-element tuple shape (`("flag_dropdown", flag.name)`, …) so
///     existing egui widget state is not invalidated.
///   - `Some(row_idx)` — one row of a repeating flag. The header row in
///     `render_repeating` owns the chrome, so it is skipped here; the
///     ID salts gain the row index (`("flag_dropdown", flag.name, row_idx)`
///     — R0-r1 I4: N same-name rows sharing an ID is the v0.1.1
///     popup-state-leak bug class).
#[allow(clippy::too_many_arguments)]
fn render_row(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    flag: &FlagSchema,
    value: &mut FlagValue,
    state: &FormState,
    disabled_options: &[String],
    row: Option<usize>,
) {
    // v0.6.0 P3 — transition sentinel for Unset ↔ seeded swaps. Mutating
    // *value mid-match would conflict with the destructured borrow inside
    // each arm; collect any swap intent here and apply it after the match.
    let mut transition: Option<FlagValue> = None;
    ui.horizontal(|ui| {
        if row.is_none() {
            ui.label(flag.name).on_hover_text(flag.help);
            render_help_icon(ui, tab, subcommand, flag);
        }
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
                let resolved_max = max.resolve(state);
                ui.add(egui::DragValue::new(n).range(*min..=resolved_max));
                if ui.small_button("✕").on_hover_text("clear (Unset)").clicked() {
                    transition = Some(FlagValue::Unset);
                }
            }
            (FlagKind::Dropdown(opts), FlagValue::Dropdown(sel)) => {
                // v0.7.0 SPEC §6.10.4 v4 — `disabled_options` greys out the
                // listed option values + renders them non-selectable.
                // Orthogonal to the primary Visibility (which decorates the
                // label via Required/PinValue/etc and is consumed upstream
                // in main.rs's `add_enabled_ui` gate for Disabled). A flag
                // can simultaneously be Required (red asterisk on label) AND
                // have invalid Dropdown options greyed out (e.g., --template
                // when slot_count>=2 and no descriptor).
                //
                // v0.30.0 R0-r1 I4: repeating rows salt the row index in —
                // the scalar path keeps the 2-tuple salt byte-unchanged.
                let combo = match row {
                    Some(row_idx) => egui::ComboBox::from_id_salt((
                        "flag_dropdown",
                        flag.name,
                        row_idx,
                    )),
                    None => egui::ComboBox::from_id_salt(("flag_dropdown", flag.name)),
                };
                // v0.30.0 R0-r2 I-1 — empty-option display mapping: the ""
                // UNSET sentinel (ARCHETYPES[0]; also the "+ add" Dropdown
                // row seed) displays as "(none)" for BOTH the selected_text
                // and the popup row. DISPLAY-ONLY: the stored/emitted value
                // stays "" (`emit_one` / `is_at_default` / `has_value` all
                // key off the value). Without it the unset row is a ~4px
                // sliver and — with FormState persisted per subcommand and
                // no reset affordance — selecting an archetype once would
                // near-permanently trap the user out of `--spec`.
                let selected_label = if sel.is_empty() {
                    "(none)".to_string()
                } else {
                    sel.clone()
                };
                combo
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        for opt in *opts {
                            let is_disabled =
                                disabled_options.iter().any(|d| d == *opt);
                            let display = if opt.is_empty() { "(none)" } else { *opt };
                            ui.add_enabled_ui(!is_disabled, |ui| {
                                ui.selectable_value(sel, (*opt).to_string(), display);
                            });
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
                // v0.30.0 R0-r1 I4: row-salted for repeating rows (no
                // NodeValueComposite flag repeats today; defensive parity
                // with the Dropdown arm).
                let combo = match row {
                    Some(row_idx) => egui::ComboBox::from_id_salt((
                        "flag_nodevalue",
                        flag.name,
                        row_idx,
                    )),
                    None => egui::ComboBox::from_id_salt(("flag_nodevalue", flag.name)),
                };
                combo
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
                    // v0.30.0 R0-r1 I4: row-salted for repeating rows (no
                    // TaggedOrIndexed flag repeats today; defensive parity).
                    let combo = match row {
                        Some(row_idx) => egui::ComboBox::from_id_salt((
                            "flag_tagged",
                            flag.name,
                            row_idx,
                        )),
                        None => {
                            egui::ComboBox::from_id_salt(("flag_tagged", flag.name))
                        }
                    };
                    combo
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
            // recover by re-seeding to the default. v0.10.0 B.3 (D31): use
            // the flag-aware default constructor so the recovered value
            // honours the schema's `default_value`.
            _ => {
                if matches!(*value, FlagValue::Unset) {
                    transition = Some(default_flag_value_for_flag(flag));
                } else {
                    ui.label("(value-shape mismatch — see form-state init)");
                }
            }
        }
        // Required marker: scalar path only — for repeating rows the
        // header row in `render_repeating` owns the marker (R0-r1 I5).
        if row.is_none() && flag.required {
            ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "*");
        }
    });
    if let Some(new) = transition {
        *value = new;
    }
}
