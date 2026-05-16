//! GUI help affordances (manual-gui v1.0 cycle).
//!
//! Container for the `?`-button URL helpers per SPEC §2.4. The
//! widget-integration (P2.2) edits live in `crate::form::widget`; this
//! module owns the URL construction so the form-widget code and the
//! P1.4 kittest cell both reach a single source of truth for the
//! anchor scheme.

pub mod url;
