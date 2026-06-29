//! Egui-free canonical form fixtures (P1 — SPEC §6 / plan I2).
//!
//! [`render_fixture`] is the ONE shared canonical-fixture source consumed by
//! BOTH the headless `gui-render` emit binary (P2) and the egui_kittest
//! faithfulness test (P3) — so the documented render and the gated
//! faithfulness check observe the SAME form state (no duplicated fixture
//! logic that could silently diverge).
//!
//! The canonical base is `FormState::default()`, which is a valid base for
//! ALL 61 forms (it is `sweep_candidate_bases`'s first element for every
//! form — the blank-form state, every flag Visible, no context needed). A
//! form with a MODE (e.g. `build-descriptor`: archetype vs tree vs generic)
//! renders its generic/default mode here (tree `None`, `--archetype` unset),
//! which is the canonical screen per SPEC §6.

use crate::app::CliTab;
use crate::schema::FormState;

/// The canonical fixture `FormState` for `(tab, sub)`.
///
/// Egui-free and deterministic (no RNG / timestamp / PATH): returns the
/// canonical base `FormState::default()` for every form today. `tab` / `sub`
/// are part of the signature so a future follow-on can return a per-form
/// fixture (e.g. a non-default mode base) without changing any call site;
/// they are reserved for now.
pub fn render_fixture(tab: CliTab, sub: &str) -> FormState {
    let _ = (tab, sub);
    FormState::default()
}
