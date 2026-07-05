//! mnemonic-gui — library crate root.
//!
//! Dual-target package: `lib.rs` declares modules so integration tests in
//! `tests/` can reach internals; `main.rs` is a thin binary wrapper that
//! calls into this library. Modules below mirror SPEC §B.2 source tree.

pub mod app;
// P0 spike (gui_example_tutorial SPEC §3.1(a)): the real application shell,
// relocated verbatim from `src/main.rs`. Gated like the egui form-widget
// modules — `--no-default-features` builds must not pull the egui stack.
#[cfg(feature = "gui")]
pub mod app_window;
pub mod form;
pub mod help;
pub mod path_detect;
pub mod persistence;
pub mod platform;
pub mod runner;
pub mod schema;
pub mod schema_check;
pub mod secrets;
