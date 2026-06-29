//! Form-layer modules: widget renderer, slot editor, conditional visibility,
//! argv assembler + copy-command shell-quoting.
//!
//! P1 `gui`-feature split (SPEC §3): the five egui-coupled form-widget
//! modules (`widget`, `slot_editor`, `secret_widget`, `tree_form`,
//! `archetype_form`) are gated behind the default-on `gui` feature. Their
//! egui-FREE data types + pure helpers were extracted into the unconditional
//! sibling modules below (`slot_model`, `secret_model`, `flag_defaults`,
//! `mode_predicates`) so the form model + the headless `gui-render` emit-mode
//! build WITHOUT egui. `fixtures` is the shared canonical-fixture source.

// ── Unconditional (egui-free) form model + helpers ──────────────────────────
pub mod conditional;
pub mod fixtures;
pub mod flag_defaults;
pub mod invocation;
pub mod mode_predicates;
pub mod render_emit;
pub mod secret_model;
pub mod slot_model;
pub mod tree_model;

// ── egui-gated form-widget renderers (default-on `gui` feature) ─────────────
#[cfg(feature = "gui")]
pub mod archetype_form;
#[cfg(feature = "gui")]
pub mod secret_widget;
#[cfg(feature = "gui")]
pub mod slot_editor;
#[cfg(feature = "gui")]
pub mod tree_form;
#[cfg(feature = "gui")]
pub mod widget;
