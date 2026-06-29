//! Egui-free structural form renderer (P2 — SPEC §2).
//!
//! Produces the deterministic, ASCII-only text render of a GUI subcommand
//! form: a header, the flag grid (name / kind / required-secret-repeating
//! markers / visible-enabled-pinned state / value), the bespoke sub-surface
//! placeholder lines (SlotEditor / mode selector / archetype param form /
//! tree builder — a SINGLE labeled line each, NOT field-by-field), the
//! positionals, and the `[ Run ]` action bar.
//!
//! Reused by BOTH the headless `gui-render` binary (`src/bin/gui_render.rs`)
//! and — in P3 — the egui_kittest faithfulness test, via the SAME shared
//! [`crate::form::fixtures::render_fixture`] source, so the documented render
//! and the gated faithfulness check observe one canonical form state.
//!
//! **Determinism (SPEC §6):** no RNG / timestamp / `$PATH` — the only inputs
//! are the static schema + the fixed fixture. Output is strictly ASCII with
//! `\n` (LF) line endings, so `gui-render --emit-all` is byte-reproducible
//! across machines.
//!
//! **Secret hygiene (SPEC §6, first-class):** a secret flag's value is the
//! fixed [`MASKED`] sentinel — gated on [`crate::secrets::flag_is_secret`],
//! never the cleartext value, even when a fixture would carry one. (A secret
//! `*-stdin` Boolean has no secret payload — it renders as the disabled
//! checkbox the GUI shows, mirroring `widget::render_with_dispatch`.)

use std::path::Path;

use crate::app::CliTab;
use crate::form::fixtures::render_fixture;
use crate::form::flag_defaults::default_flag_value_for_flag;
use crate::form::mode_predicates;
use crate::schema::{
    FlagKind, FlagSchema, FlagValue, FormState, PositionalArgSchema, Schema,
    SubcommandSchema, TaggedOrIndexedValue, TimestampValue, Visibility,
};
use crate::secrets::flag_is_secret;

/// The fixed sentinel for a secret value — NEVER the real value (SPEC §6).
pub const MASKED: &str = "<masked>";

/// Resolve the static [`Schema`] for a [`CliTab`]. Egui-free; mirrors the
/// bin-private `MnemonicGuiApp::schema_for` (`main.rs`) + the lib-private
/// `persistence::schema_for`, exposed publicly here so the headless emit
/// binary + the census test reach every subcommand without the gated bin.
pub fn schema_for(tab: CliTab) -> &'static Schema {
    match tab {
        CliTab::Mnemonic => &crate::schema::mnemonic::SCHEMA,
        CliTab::Md => &crate::schema::md::SCHEMA,
        CliTab::Ms => &crate::schema::ms::SCHEMA,
        CliTab::Mk => &crate::schema::mk::SCHEMA,
    }
}

/// The full `(tab, subcommand)` form set across all four CLIs, in
/// `CliTab::ALL` × schema-declared order — the iteration the census +
/// `--emit-all` both derive their count from (no hard-coded 61).
pub fn all_forms() -> Vec<(CliTab, &'static SubcommandSchema)> {
    let mut out = Vec::new();
    for tab in CliTab::ALL.iter().copied() {
        for sub in schema_for(tab).subcommands {
            out.push((tab, sub));
        }
    }
    out
}

/// Render one form using the [`seeded_fixture`] on-load steady state.
pub fn render_form(tab: CliTab, sub: &SubcommandSchema) -> String {
    render_form_from_state(tab, sub, &seeded_fixture(tab, sub))
}

/// The **render-gated monotone fixed-point seed** — the on-load steady state the
/// user actually sees, NOT the pristine `FormState::default()` (P3 R0 ruling A).
///
/// The GUI auto-seeds every CURRENTLY-RENDERED non-secret flag's schema default
/// into `state.values` on each frame and re-evaluates `conditional` at the top
/// of the next frame, so the screen the user sees on load is the FIXED POINT of
/// that loop — e.g. `bundle` seeds `--template -> bip44` (single-sig), and the
/// conditional then greys the multisig-only `--threshold`/`--multisig-path-family`.
/// The emit MUST evaluate `conditional` over THIS seeded state, not the unseeded
/// blank form, or the depicted `[disabled]` column contradicts the value column.
///
/// This mirrors `widget::render_with_dispatch` (`widget.rs:220-229`) and
/// `render_repeating`'s required-row seed (`widget.rs:309-315`) EXACTLY — and
/// deliberately does NOT over-seed:
///   - **Non-secret, non-repeating** flags that are RENDERED (not mode-suppressed,
///     not conditional-`Hidden`) seed `default_flag_value_for_flag` (all kinds;
///     a numeric default is `Unset`, which `has_value` reads as ABSENT — no
///     fabricated number). A `Disabled` flag IS still rendered (the GUI greys but
///     still calls the widget), so it IS seeded.
///   - **Required REPEATING** non-secret flags seed ONE row (the GUI's
///     per-frame required-row seed); **optional repeating** flags seed NOTHING.
///   - **Secret** flags are NEVER seeded (secret Text → `secret_widgets`, secret
///     `*-stdin` Boolean → early return); **mode-suppressed** + conditional-`Hidden`
///     flags are not rendered, hence not seeded.
///
/// `state.values` only GROWS (monotone — a flag that a later pass hides keeps its
/// already-stored value), so the loop converges in ≤ `sub.flags.len()` passes.
fn seeded_fixture(tab: CliTab, sub: &SubcommandSchema) -> FormState {
    let mut state = render_fixture(tab, sub.name);
    loop {
        let vis_map = sub.conditional.map(|f| f(&state)).unwrap_or_default();
        let visibility_of = |name: &str| -> Visibility {
            vis_map
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.clone())
                .unwrap_or(Visibility::Visible)
        };
        let mut changed = false;
        for flag in sub.flags {
            // Mode-suppressed + conditional-`Hidden` flags are NOT rendered,
            // hence NOT seeded (the GUI `continue`s them before the widget).
            if is_render_suppressed(sub, flag.name, &state) {
                continue;
            }
            if matches!(visibility_of(flag.name), Visibility::Hidden) {
                continue;
            }
            // The widget dispatch routes only secret TEXT/BOOLEAN away from
            // `state.values` (`widget.rs:181`/`:202`); a secret flag of any other
            // kind (e.g. the secret Composite `seed-xor-combine --share`) falls
            // through to `render_repeating` and IS seeded one row
            // (`widget.rs:309-315`). Mirror that exactly — skip seeding ONLY
            // secret Text/Boolean, not all secrets (P3-R0-r2 I1: a blanket skip
            // under-seeds the secret Composite).
            if flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text | FlagKind::Boolean) {
                continue;
            }
            let already = state.values.iter().any(|(k, _)| k == flag.name);
            if already {
                continue;
            }
            if flag.repeating {
                // Required-repeating: seed ONE row; optional-repeating: nothing.
                if flag.required {
                    state
                        .values
                        .push((flag.name.to_string(), default_flag_value_for_flag(flag)));
                    changed = true;
                }
                continue;
            }
            // Non-secret scalar: mirror the `widget.rs:220-229` None-arm push.
            state
                .values
                .push((flag.name.to_string(), default_flag_value_for_flag(flag)));
            changed = true;
        }
        if !changed {
            break;
        }
    }
    state
}

/// `<tab>-<sub>.gui` file name for a form (the committed-render naming the
/// manual's `include=` + census gate key off).
pub fn render_file_name(tab: CliTab, sub: &SubcommandSchema) -> String {
    format!("{}-{}.gui", tab.bin_name(), sub.name)
}

/// Emit a `<tab>-<sub>.gui` render for EVERY subcommand into `dir`,
/// returning the number of files written (== summed
/// `schema_for(tab).subcommands.len()`). Creates `dir` if absent.
pub fn emit_all(dir: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(dir)?;
    let mut count = 0;
    for (tab, sub) in all_forms() {
        let content = render_form(tab, sub);
        std::fs::write(dir.join(render_file_name(tab, sub)), content)?;
        count += 1;
    }
    Ok(count)
}

// ─── render core ────────────────────────────────────────────────────────────

/// One body-bearing row (flag or positional). The `name` column is
/// left-aligned to the form's widest name; `body` is everything after it.
struct Row {
    name: String,
    body: String,
}

/// A line of the render: an aligned `Row`, or a free-standing `Raw` line
/// (sub-surface placeholder / action bar — indented but not name-aligned).
enum Item {
    Row(Row),
    Raw(String),
}

/// One ordered structural element of a rendered form — the SINGLE pass
/// [`form_elements`] both the ASCII renderer ([`render_form_from_state`]) and
/// the P3 tree-observable projection ([`project_form`]) consume, so the
/// documented render and the gated faithfulness check can never drift
/// (plan P3: "ASCII and the test's projection must come from the one
/// `render_emit` core, not a re-implementation").
enum FormElement {
    /// A flag the form RENDERS (not suppressed, not `Hidden`), carrying the
    /// per-frame `Visibility` so the ASCII body + the projection's
    /// disabled-state both read the same decision.
    Flag {
        flag: &'static FlagSchema,
        vis: Visibility,
    },
    /// A positional argument (rendered unconditionally — no visibility gate,
    /// main.rs:832).
    Positional {
        pos: &'static PositionalArgSchema,
    },
    /// A bespoke sub-surface / mode-selector placeholder line (out of the P3
    /// faithfulness gate per SPEC §2 — a single labeled line, not field-level).
    Raw(String),
    /// The `[ Run ]` action bar (always last).
    Run,
}

/// The single structural pass over `sub` under `state`. The flag-grid GATE is
/// byte-identical to `main.rs`'s per-flag render loop (mirrored by the PR-#24
/// harness `render_whole_form`): the `--slot` / mode `continue`s, the
/// `Hidden → skip`, and the per-flag `Visibility`. The bespoke sub-surfaces
/// are single placeholder lines.
fn form_elements(sub: &SubcommandSchema, state: &FormState) -> Vec<FormElement> {
    let tree_mode =
        sub.name == "build-descriptor" && mode_predicates::tree_enabled(state);
    let archetype_mode = sub.name == "build-descriptor"
        && !tree_mode
        && mode_predicates::active_archetype(state).is_some();

    let vis_map = sub.conditional.map(|f| f(state)).unwrap_or_default();
    let visibility_of = |name: &str| -> Visibility {
        vis_map
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or(Visibility::Visible)
    };

    let mut els: Vec<FormElement> = Vec::new();

    // The build-descriptor mode selector (Generic / Archetype / Tree) is a
    // bespoke sub-surface rendered ABOVE the flag grid (main.rs:601).
    if sub.name == "build-descriptor" {
        let mode = if tree_mode {
            "tree"
        } else if archetype_mode {
            "archetype"
        } else {
            "generic"
        };
        els.push(FormElement::Raw(format!(
            "[ build-descriptor mode selector: {mode} ]"
        )));
    }

    // Flag grid — same suppression / visibility gate as main.rs.
    for flag in sub.flags {
        if is_render_suppressed(sub, flag.name, state) {
            continue;
        }
        let v = visibility_of(flag.name);
        if matches!(v, Visibility::Hidden) {
            continue;
        }
        els.push(FormElement::Flag { flag, vis: v });
        // The archetype param sub-form renders inline directly under the
        // `--archetype` dropdown (main.rs:690), as a single placeholder line.
        if flag.name == "--archetype" && archetype_mode {
            els.push(FormElement::Raw("[ archetype param form ]".to_string()));
        }
    }

    // SlotEditor bespoke sub-surface (main.rs:718).
    if sub.allows_slots {
        els.push(FormElement::Raw(format!(
            "[ slot editor: {} rows ]",
            state.slot_count()
        )));
    }

    // Positionals (main.rs:832), after the slot editor.
    for pos in sub.positional_args {
        els.push(FormElement::Positional { pos });
    }

    // Tree-builder bespoke sub-surface (main.rs:886).
    if tree_mode {
        els.push(FormElement::Raw("[ descriptor tree builder ]".to_string()));
    }

    els.push(FormElement::Run);
    els
}

/// Render `sub`'s form under `state` as the deterministic ASCII grid, derived
/// from the single [`form_elements`] structural pass.
pub fn render_form_from_state(
    tab: CliTab,
    sub: &SubcommandSchema,
    state: &FormState,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("[ {} > {} ]\n", tab.bin_name(), sub.name));

    // ASCII rows from the structured elements (the SAME pass the P3
    // faithfulness projection reads via `project_form`).
    let items: Vec<Item> = form_elements(sub, state)
        .iter()
        .map(|el| match el {
            FormElement::Flag { flag, vis } => Item::Row(Row {
                name: flag.name.to_string(),
                body: flag_body(flag, vis),
            }),
            FormElement::Positional { pos } => Item::Row(Row {
                name: pos.name.to_string(),
                body: positional_body(pos),
            }),
            FormElement::Raw(s) => Item::Raw(s.clone()),
            FormElement::Run => Item::Raw("[ Run ]".to_string()),
        })
        .collect();

    let name_w = items
        .iter()
        .filter_map(|i| match i {
            Item::Row(r) => Some(r.name.len()),
            Item::Raw(_) => None,
        })
        .max()
        .unwrap_or(0);

    for item in &items {
        match item {
            Item::Row(r) => {
                out.push_str(&format!("  {:<name_w$}  {}\n", r.name, r.body));
            }
            Item::Raw(s) => out.push_str(&format!("  {s}\n")),
        }
    }
    out
}

// ─── P3 tree-observable projection (the faithfulness gate's emit side) ───────
//
// `project_form` reduces the SAME `form_elements` pass to the AccessKit-tree-
// observable facets the P3 egui_kittest test reads off the REAL rendered form:
// per-flag presence / disabled / control-class, positional presence + secret-
// masking, and the action bar. The ASCII render and this projection are two
// reductions of ONE structural pass — they cannot drift (plan P3). What the
// projection deliberately OMITS (per SPEC §2 / plan m4, NOT AccessKit-
// recoverable; covered by P5 regen-determinism + schema_mirror): path-vs-text,
// the `(required)` marker, default/placeholder TEXT, and the sub-surface
// placeholder INTERNALS.

/// The tree-observable input-control class of a flag under the canonical
/// fixture — the coarse AccessKit shape its widget presents. The emit side
/// PREDICTS it from `(kind, secret, repeating)`; the P3 test reads the GROUND
/// TRUTH off the real egui widget tree and asserts they agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlClass {
    /// `Role::TextInput` — non-secret scalar `Text` / `Path`.
    TextInput,
    /// `Role::PasswordInput` (masked) — secret scalar `Text`.
    Secret,
    /// `Role::ComboBox` — scalar `Dropdown`.
    ComboBox,
    /// `Role::ComboBox` + an adjacent text/password field — scalar
    /// `NodeValueComposite` (`<node> = <value>`).
    Composite,
    /// `Role::CheckBox` — `Boolean` (incl. the always-disabled secret
    /// `*-stdin` toggle).
    CheckBox,
    /// A `"Set"` button (no value-bearing input yet) — a scalar
    /// `Number` / `Range` / `Timestamp` / `TaggedOrIndexed` whose canonical
    /// default is `Unset` (v0.6.0 click-to-seed UX).
    SetButton,
    /// The multi-row repeating widget — a header label + `"+ add"` button
    /// (any repeating flag, regardless of inner kind / secret).
    Repeating,
}

/// Whether a flag / positional is RENDERED in the canonical form (`Present`)
/// or absent from it — either mode-`Suppressed` into a bespoke sub-surface or
/// `Hidden` by the conditional. The P3 gate compares only Present-vs-Absent
/// (both "absent from the rendered grid"), so the two absent reasons collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Present,
    Absent,
}

/// The tree-observable projection of one flag.
#[derive(Debug, Clone)]
pub struct FlagProjection {
    pub name: &'static str,
    pub presence: Presence,
    /// The flag's disabled state over the [`seeded_fixture`] on-load steady
    /// state — conditional `Disabled` under the seeded fixed point, or the
    /// always-greyed secret `*-stdin` toggle. Mirrors the ASCII `[disabled]`
    /// markers (pinned by P2). Because the emit now seeds defaults before
    /// evaluating `conditional` (P3 R0 ruling A), this equals the SETTLED GUI's
    /// per-flag `is_disabled()`, so the P3 faithfulness gate cross-checks it
    /// against the real render — that re-gate self-guards the seed simulator
    /// (a seed drifting from `widget.rs` REDs the disabled axis).
    pub disabled: bool,
    /// Meaningful only when `Present`.
    pub control: ControlClass,
}

/// The tree-observable projection of one positional argument (always rendered).
#[derive(Debug, Clone)]
pub struct PositionalProjection {
    pub name: &'static str,
    /// Secret positionals render as a masked `PasswordInput`; non-secret as a
    /// plain `TextInput`.
    pub secret: bool,
}

/// The full tree-observable projection of a form: every schema flag (with its
/// presence), the positionals, and whether the action bar renders.
#[derive(Debug, Clone)]
pub struct FormProjection {
    pub flags: Vec<FlagProjection>,
    pub positionals: Vec<PositionalProjection>,
    pub has_run: bool,
}

/// Predict a flag's [`ControlClass`] under the canonical fixture, mirroring
/// the `widget::render_with_dispatch` branch order: repeating flags route to
/// the multi-row widget FIRST; a secret `*-stdin` Boolean is the disabled
/// checkbox; a secret scalar `Text` is the masked field; otherwise the kind
/// maps directly (the four `Unset`-default kinds render their `Set`
/// affordance). This is the emit's PREDICTION — the P3 test verifies it
/// against the real AccessKit tree, so it is not a self-fulfilling assertion.
fn control_class(flag: &FlagSchema) -> ControlClass {
    if flag.repeating {
        return ControlClass::Repeating;
    }
    let secret = flag_is_secret(flag);
    if secret && matches!(flag.kind, FlagKind::Boolean) {
        return ControlClass::CheckBox;
    }
    if secret && matches!(flag.kind, FlagKind::Text) {
        return ControlClass::Secret;
    }
    match flag.kind {
        FlagKind::Text | FlagKind::Path { .. } => ControlClass::TextInput,
        FlagKind::Dropdown(_) => ControlClass::ComboBox,
        FlagKind::Boolean => ControlClass::CheckBox,
        FlagKind::NodeValueComposite(_) => ControlClass::Composite,
        FlagKind::Number { .. }
        | FlagKind::Range
        | FlagKind::Timestamp
        | FlagKind::TaggedOrIndexed(_) => ControlClass::SetButton,
    }
}

/// The emit-side [`FormProjection`] for `(tab, sub)` under the shared canonical
/// [`render_fixture`] — the projection the P3 faithfulness gate compares the
/// real egui render against.
pub fn project_form(tab: CliTab, sub: &SubcommandSchema) -> FormProjection {
    let state = seeded_fixture(tab, sub);
    let elements = form_elements(sub, &state);

    // The flags the single pass actually RENDERS, with their per-frame
    // visibility (present-vs-absent is decided ONLY here, the one core).
    let present: Vec<(&'static str, Visibility)> = elements
        .iter()
        .filter_map(|el| match el {
            FormElement::Flag { flag, vis } => Some((flag.name, vis.clone())),
            _ => None,
        })
        .collect();

    let flags = sub
        .flags
        .iter()
        .map(|flag| match present.iter().find(|(n, _)| *n == flag.name) {
            Some((_, vis)) => {
                let is_secret_bool =
                    flag_is_secret(flag) && matches!(flag.kind, FlagKind::Boolean);
                FlagProjection {
                    name: flag.name,
                    presence: Presence::Present,
                    disabled: matches!(vis, Visibility::Disabled) || is_secret_bool,
                    control: control_class(flag),
                }
            }
            None => FlagProjection {
                name: flag.name,
                presence: Presence::Absent,
                disabled: false,
                control: control_class(flag),
            },
        })
        .collect();

    let positionals = sub
        .positional_args
        .iter()
        .map(|pos| PositionalProjection {
            name: pos.name,
            secret: pos.secret,
        })
        .collect();

    let has_run = elements.iter().any(|el| matches!(el, FormElement::Run));

    FormProjection {
        flags,
        positionals,
        has_run,
    }
}

/// The render-suppression gate, mirroring `main.rs`'s three `continue`s +
/// the PR-#24 harness `is_render_suppressed` (the egui-free `mode_predicates`
/// source of truth both reuse).
fn is_render_suppressed(
    sub: &SubcommandSchema,
    flag_name: &str,
    state: &FormState,
) -> bool {
    if flag_name == "--slot" && sub.allows_slots {
        return true;
    }
    let tree_mode =
        sub.name == "build-descriptor" && mode_predicates::tree_enabled(state);
    if tree_mode && mode_predicates::suppressed_in_tree_mode(flag_name) {
        return true;
    }
    let archetype_mode = sub.name == "build-descriptor"
        && !tree_mode
        && mode_predicates::active_archetype(state).is_some();
    if archetype_mode && mode_predicates::suppressed_in_archetype_mode(flag_name)
    {
        return true;
    }
    false
}

/// `{kind}  {markers}-> {value}{state}` for a flag row.
fn flag_body(flag: &FlagSchema, vis: &Visibility) -> String {
    let is_secret = flag_is_secret(flag);
    // A secret `*-stdin` Boolean carries no secret payload — the GUI renders
    // it as a DISABLED checkbox (widget::render_with_dispatch), not a masked
    // field. Only value-bearing secret kinds get the masked sentinel.
    let is_secret_bool = is_secret && matches!(flag.kind, FlagKind::Boolean);

    let kind = kind_label(&flag.kind);

    let required = flag.required || matches!(vis, Visibility::Required);
    let mut markers: Vec<&str> = Vec::new();
    if required {
        markers.push("required");
    }
    if is_secret {
        markers.push("secret");
    }
    if flag.repeating {
        markers.push("repeating");
    }
    let markers_str = if markers.is_empty() {
        String::new()
    } else {
        format!("({}) ", markers.join(", "))
    };

    let value = flag_value_str(flag, vis, is_secret, is_secret_bool);

    let mut state_suffix = String::new();
    if matches!(vis, Visibility::Disabled) || is_secret_bool {
        state_suffix.push_str(" [disabled]");
    }
    if let Visibility::DisableOptions { values } = vis {
        let mut v = values.clone();
        v.sort();
        state_suffix.push_str(&format!(" [disabled-options: {}]", v.join(", ")));
    }

    format!("{kind}  {markers_str}-> {value}{state_suffix}")
}

/// The rendered value for a flag under the canonical fixture. Secrets are
/// the [`MASKED`] sentinel; a `PinValue` shows the pinned (coerced) value;
/// everything else is the schema-default `FlagValue` (the SINGLE source of
/// truth `default_flag_value_for_flag`, never re-implemented).
fn flag_value_str(
    flag: &FlagSchema,
    vis: &Visibility,
    is_secret: bool,
    is_secret_bool: bool,
) -> String {
    if is_secret && !is_secret_bool {
        return MASKED.to_string();
    }
    if let Visibility::PinValue { value } = vis {
        return format!("<pinned: {}>", json_plain(value));
    }
    match default_flag_value_for_flag(flag) {
        FlagValue::Boolean(b) => {
            if b { "[x] on".into() } else { "[ ] off".into() }
        }
        FlagValue::Text(s) | FlagValue::Dropdown(s) | FlagValue::Path(s) => {
            if s.is_empty() {
                "<empty>".into()
            } else {
                s
            }
        }
        FlagValue::NodeValueComposite { node, value } => {
            if value.is_empty() {
                format!("{node}=<empty>")
            } else {
                format!("{node}={value}")
            }
        }
        FlagValue::Number(n) => n.to_string(),
        FlagValue::Range(a, b) => format!("{a},{b}"),
        FlagValue::Timestamp(TimestampValue::Now) => "now".into(),
        FlagValue::Timestamp(TimestampValue::Unix(n)) => n.to_string(),
        FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Tag(t)) => t,
        FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Indexed(i)) => {
            format!("@{i}")
        }
        FlagValue::Unset => "<unset>".into(),
    }
}

/// `{kind}  {markers}-> {value}` for a positional row.
fn positional_body(pos: &PositionalArgSchema) -> String {
    let kind = if pos.repeating {
        "positional..."
    } else {
        "positional"
    };
    let mut markers: Vec<&str> = Vec::new();
    if pos.required {
        markers.push("required");
    }
    if pos.secret {
        markers.push("secret");
    }
    let markers_str = if markers.is_empty() {
        String::new()
    } else {
        format!("({}) ", markers.join(", "))
    };
    let value = if pos.secret { MASKED } else { "<empty>" };
    format!("{kind}  {markers_str}-> {value}")
}

/// ASCII kind label per `FlagKind`. Booleans render as `checkbox` (the
/// SPEC §2 example glyph); Dropdown / composite / tagged kinds list their
/// option set inline so the documented render shows the choices.
fn kind_label(kind: &FlagKind) -> String {
    match kind {
        FlagKind::Text => "text".to_string(),
        FlagKind::Number { .. } => "number".to_string(),
        FlagKind::Dropdown(opts) => format!("dropdown[{}]", join_opts(opts)),
        FlagKind::Boolean => "checkbox".to_string(),
        FlagKind::Range => "range".to_string(),
        FlagKind::Timestamp => "timestamp".to_string(),
        FlagKind::NodeValueComposite(opts) => {
            format!("composite[{}]", join_opts(opts))
        }
        FlagKind::TaggedOrIndexed(opts) => {
            format!("tagged-or-indexed[{}]", join_opts(opts))
        }
        FlagKind::Path { stdio_sentinel } => {
            if *stdio_sentinel {
                "path(stdio)".to_string()
            } else {
                "path".to_string()
            }
        }
    }
}

/// Comma-join an option list; the `""` UNSET sentinel displays as `(none)`
/// (mirroring the GUI's `display_or`), so the render stays ASCII + readable.
fn join_opts(opts: &[&str]) -> String {
    opts.iter()
        .map(|o| if o.is_empty() { "(none)" } else { *o })
        .collect::<Vec<_>>()
        .join(",")
}

/// A flat, ASCII-friendly rendering of a pinned JSON value (PinValue coerces
/// `--account` to `0` etc.). Strings drop their quotes; scalars stringify.
fn json_plain(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
