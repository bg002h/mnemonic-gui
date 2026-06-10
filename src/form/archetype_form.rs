//! v0.31.0 (SPEC §3/§4) — the data-driven archetype param form.
//!
//! When the build-descriptor form's `--archetype` dropdown holds a
//! non-empty `ARCHETYPE_SPECS` id, the generic flag grid is REPLACED (for
//! the 9 param flags + `--spec`) by this schema-driven form: only the
//! selected archetype's params render, in schema order, each with a
//! kind-appropriate widget (the §4 ParamKind → widget map), a required
//! marker, a `(min N)` annotation for repeatable params, and the
//! archetype's summary line directly under the dropdown.
//!
//! Lib seam (R0-r1 I2): this module is the REAL form — `src/main.rs` only
//! dispatches (the SlotEditor precedent), so kittests drive these `pub`
//! fns rather than re-implementing main.rs logic. The generic-loop skip
//! for DECLARED params is a name-set `continue` in the host loop
//! ([`suppressed_in_archetype_mode`]) — NOT `Visibility::Hidden`, which
//! would suppress argv emission; declared params must emit. The conditional
//! (`conditional::build_descriptor`) marks UNdeclared params `Hidden` so
//! the assembler skips them declaratively (SPEC §3/§5).
//!
//! Data preservation: params not in the selected archetype keep their
//! `state.values` rows (left intact, not rendered) — switching archetypes
//! back and forth never destroys entered data. The C1 corollary: an
//! archetype-NON-repeatable key with >1 carried-over rows renders through
//! the repeating widget (add suppressed, `(exactly 1)`) so what emits is
//! always what renders.

use eframe::egui;

use crate::app::CliTab;
use crate::form::widget::{self, RepeatAnnotation};
use crate::schema::archetypes::{self, ArchetypeParamSpec, ArchetypeSpec};
use crate::schema::{FlagKind, FlagSchema, FormState, NumberMax};

/// True iff `flag_name` is suppressed from the generic flag loop in
/// archetype mode: the 9 archetype param flags (rendered by [`render`]
/// instead) + `--spec` (the conditional's `Disabled` ghost row is replaced
/// by a cleaner mode switch — the conditional itself is UNCHANGED for
/// `--spec`, R0-r1 M1). Full 18-flag accounting (R0-r1 I1): 9 params +
/// `--spec` suppressed; the 8 mode-independent flags (`--archetype`,
/// `--spec-schema`, `--emit-spec`, `--allow`, `--format`, `--network`,
/// `--json`, `--no-auto-repair`) keep their generic widgets.
pub fn suppressed_in_archetype_mode(flag_name: &str) -> bool {
    flag_name == "--spec" || archetypes::ARCHETYPE_PARAM_FLAGS.contains(&flag_name)
}

/// The archetype-mode predicate: `Some(spec)` iff `--archetype` holds a
/// non-empty value matching an `ARCHETYPE_SPECS` id ("" is the GUI-side
/// UNSET sentinel — generic mode).
pub fn active_archetype(state: &FormState) -> Option<&'static ArchetypeSpec> {
    let id = state.dropdown_value("--archetype")?;
    if id.is_empty() {
        return None;
    }
    archetypes::find(id)
}

/// Positional threshold↔key pairing convention from the toolkit registry:
/// `--threshold` applies to `--key`, `--recovery-threshold` to
/// `--recovery-key`. Unit-tested against `ARCHETYPE_SPECS` (every
/// archetype declaring a threshold also declares the paired key).
pub fn paired_key_flag(threshold_flag: &str) -> Option<&'static str> {
    match threshold_flag {
        "--threshold" => Some("--key"),
        "--recovery-threshold" => Some("--recovery-key"),
        _ => None,
    }
}

/// Static `BUILD_DESCRIPTOR_FLAGS` entry lookup (the schema-declared
/// widget shapes the param form clones from — R0-r2 m3: the Number `min`
/// and the kind come from here; only `max`/`required`/`repeating` are
/// swapped per-mode).
fn static_flag(name: &str) -> &'static FlagSchema {
    crate::schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "build-descriptor")
        .expect("build-descriptor must appear in mnemonic SUBCOMMANDS")
        .flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("build-descriptor flag {name} missing"))
}

/// Render the archetype param form for `spec`: the summary line (directly
/// under the `--archetype` dropdown — the host calls this right after the
/// dropdown's generic widget) followed by every declared param in schema
/// order via the §4 ParamKind → widget map.
pub fn render(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    spec: &'static ArchetypeSpec,
    state: &mut FormState,
) {
    // Summary line under the dropdown (SPEC §3).
    ui.weak(spec.summary);
    for param in spec.params {
        render_param(ui, tab, subcommand, param, state);
    }
}

/// One param row per the §4 ParamKind → widget map. All arms reuse the
/// v0.30.0 machinery on a bespoke-synthesized clone of the static
/// `FlagSchema` entry (the static table is never mutated — R0-r1 I3a);
/// `required` comes from the param spec so the red asterisk renders per
/// archetype.
fn render_param(
    ui: &mut egui::Ui,
    tab: CliTab,
    subcommand: &str,
    param: &'static ArchetypeParamSpec,
    state: &mut FormState,
) {
    let mut flag = static_flag(param.flag).clone();
    flag.required = param.required;
    match param.kind {
        // key (repeatable): the v0.30.0 repeating-row widget with a
        // `(min N)` header annotation (the archetype `min` is an
        // occurrence COUNT — it feeds ONLY this annotation, R0-r2 m3).
        "key" if param.repeatable => {
            widget::render_repeating(
                ui,
                tab,
                subcommand,
                &flag,
                state,
                &[],
                Some(RepeatAnnotation {
                    label: format!("(min {})", param.min),
                    suppress_add: false,
                }),
            );
        }
        // key (non-repeatable per the ARCHETYPE spec; the underlying
        // FlagSchema stays `repeating: true` for clap): single Text row
        // when ≤1 row exists; when >1 row exists (carried over from
        // another archetype — R0-r1 C1) render through the repeating
        // widget with "+ add" SUPPRESSED and an `(exactly 1)` annotation —
        // surplus rows stay visible and removable, because `assemble_argv`
        // emits EVERY matching row off the static `FlagSchema.repeating`;
        // hiding them would emit invisible argv. What emits is always
        // what renders.
        "key" => {
            let rows = state
                .values
                .iter()
                .filter(|(k, _)| k == param.flag)
                .count();
            if rows > 1 {
                widget::render_repeating(
                    ui,
                    tab,
                    subcommand,
                    &flag,
                    state,
                    &[],
                    Some(RepeatAnnotation {
                        label: "(exactly 1)".to_string(),
                        suppress_add: true,
                    }),
                );
            } else {
                // Scalar single-row path: repeating=false routes
                // render_with_dispatch to the scalar renderer (first
                // matching row, label + tooltip + required marker;
                // tooltip-only per R0-r1 M3 status quo).
                flag.repeating = false;
                widget::render_with_dispatch(ui, tab, subcommand, &flag, state, &[]);
            }
        }
        // threshold: Number widget cloning the static entry — `min` stays
        // the static FlagSchema's (R0-r2 m3); `max` binds to the LIVE
        // row count of the paired key param via NumberMax::FromRowCount
        // (bespoke-only — the static entries keep Static(20), R0-r1 I3a).
        // Clamp-on-render is the existing egui DragValue behavior.
        "threshold" => {
            let paired = paired_key_flag(param.flag).unwrap_or_else(|| {
                panic!("threshold param {} has no paired key flag", param.flag)
            });
            if let FlagKind::Number { min, .. } = flag.kind {
                flag.kind = FlagKind::Number {
                    min,
                    max: NumberMax::FromRowCount(paired),
                };
            }
            widget::render_with_dispatch(ui, tab, subcommand, &flag, state, &[]);
        }
        // blocks / absolute_locktime: the static entries' Number widgets
        // as-is (bounds already mirror the gate: `older < 2³¹`,
        // `after ≤ u32::MAX`).
        "blocks" | "absolute_locktime" => {
            widget::render_with_dispatch(ui, tab, subcommand, &flag, state, &[]);
        }
        // hex_digest: Text widget + a NON-blocking 64-hex live-validity
        // hint (the toolkit gate is the validator; the hint is a warning
        // label, never an input block).
        "hex_digest" => {
            widget::render_with_dispatch(ui, tab, subcommand, &flag, state, &[]);
            if let Some(v) = state.text_value(param.flag) {
                if !v.is_empty() && !is_64_hex(v) {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 165, 0), // warning amber
                        "⚠ expected 64 hex chars",
                    );
                }
            }
        }
        // Unknown kind (a future toolkit vocabulary addition that slipped
        // past the drift gate + vocabulary pin): defensively fall through
        // to the generic widget for the static entry.
        _ => {
            widget::render_with_dispatch(ui, tab, subcommand, &flag, state, &[]);
        }
    }
}

fn is_64_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::archetypes::ARCHETYPE_SPECS;

    #[test]
    fn every_threshold_param_pairs_with_a_declared_key_param() {
        // SPEC §4: the threshold↔key pairing is positional convention from
        // the toolkit registry; every archetype that declares a threshold
        // param must also declare its paired key param (probed against the
        // v0.52.0 binary — no counter-case).
        for spec in ARCHETYPE_SPECS {
            for p in spec.params.iter().filter(|p| p.kind == "threshold") {
                let paired = paired_key_flag(p.flag).unwrap_or_else(|| {
                    panic!("{}: threshold param {} has no pairing", spec.id, p.flag)
                });
                assert!(
                    spec.params
                        .iter()
                        .any(|q| q.flag == paired && q.kind == "key"),
                    "{} declares {} but not its paired key param {}",
                    spec.id,
                    p.flag,
                    paired,
                );
            }
        }
    }

    #[test]
    fn suppressed_set_is_params_plus_spec_and_nothing_else() {
        // R0-r1 I1 accounting: 9 params + --spec suppressed; the 8
        // mode-independent flags keep their generic widgets.
        let sub = crate::schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .find(|s| s.name == "build-descriptor")
            .expect("build-descriptor");
        let (suppressed, independent): (Vec<&str>, Vec<&str>) = sub
            .flags
            .iter()
            .map(|f| f.name)
            .partition(|n| suppressed_in_archetype_mode(n));
        assert_eq!(suppressed.len(), 10, "9 params + --spec: {suppressed:?}");
        assert_eq!(
            independent,
            [
                "--spec-schema",
                "--archetype",
                "--emit-spec",
                "--allow",
                "--format",
                "--network",
                "--json",
                "--no-auto-repair",
            ],
            "the 8 mode-independent flags (R0-r1 I1)"
        );
    }
}
