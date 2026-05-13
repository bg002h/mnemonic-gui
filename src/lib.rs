//! mnemonic-gui — library crate root.
//!
//! Dual-target package: `lib.rs` declares modules so integration tests in
//! `tests/` can reach internals; `main.rs` is a thin binary wrapper that
//! calls into this library. Modules below mirror SPEC §B.2 source tree.

pub mod app;
pub mod form;
pub mod path_detect;
pub mod persistence;
pub mod platform;
pub mod runner;
pub mod schema;
pub mod schema_check;
pub mod secrets;
