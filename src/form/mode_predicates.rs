//! Egui-free build-descriptor mode predicates, extracted from the gated
//! `tree_form.rs` and `archetype_form.rs` in the P1 `gui`-feature split
//! (SPEC §3). These pure `&FormState`/`&str` predicates compute the tree-mode
//! and archetype-mode suppression that the headless `gui-render` emit-mode
//! must reproduce to derive each flag's visible/enabled/pinned state, so they
//! live in an egui-free home. The gated form-widget modules re-export them so
//! their existing call sites keep resolving under the `gui` feature.

use crate::schema::archetypes::{self, ArchetypeSpec, ARCHETYPE_PARAM_FLAGS};
use crate::schema::FormState;

/// The tree-mode predicate: `FormState.tree` exists AND its `enabled` bit
/// is set (the ONLY stored mode state — R0-r1 I4).
pub fn tree_enabled(state: &FormState) -> bool {
    state.tree.as_ref().is_some_and(|t| t.enabled)
}

/// True iff `flag_name` is suppressed from the generic flag loop in TREE
/// mode (SPEC §0 mode mutex render leg): `--spec` + `--archetype` (their
/// argv suppression is the conditional's `Disabled`; their ROWS don't
/// render — the mode selector + tree form replace them) + the 10
/// `requires = "archetype"` flags (9 params + `--emit-spec`; argv-
/// suppressed via the conditional's `Hidden`, listed here for the render
/// dispatch). The 6 mode-independent flags (`--spec-schema`, `--allow`,
/// `--format`, `--network`, `--json`, `--no-auto-repair`) render normally.
pub fn suppressed_in_tree_mode(flag_name: &str) -> bool {
    flag_name == "--spec"
        || flag_name == "--archetype"
        || flag_name == "--emit-spec"
        || ARCHETYPE_PARAM_FLAGS.contains(&flag_name)
}

/// True iff `flag_name` is suppressed from the generic flag loop in
/// archetype mode: the 9 archetype param flags (rendered by
/// `archetype_form::render` instead) + `--spec` (the conditional's
/// `Disabled` ghost row is replaced by a cleaner mode switch — the
/// conditional itself is UNCHANGED for `--spec`, R0-r1 M1). Full 18-flag
/// accounting (R0-r1 I1): 9 params + `--spec` suppressed; the 8
/// mode-independent flags (`--archetype`, `--spec-schema`, `--emit-spec`,
/// `--allow`, `--format`, `--network`, `--json`, `--no-auto-repair`) keep
/// their generic widgets.
pub fn suppressed_in_archetype_mode(flag_name: &str) -> bool {
    flag_name == "--spec" || ARCHETYPE_PARAM_FLAGS.contains(&flag_name)
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
