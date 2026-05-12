//! mnemonic-gui — binary entry. Thin wrapper over `mnemonic_gui` library.
//!
//! Phase 0 stub. Real eframe entry lands in Phase 4 (subprocess runner +
//! tracing init) and Phase 5+ (form wiring).

fn main() {
    // Reference the library crate so the binary depends on it (placeholder
    // surface; replaced by eframe::run_native(...) in Phase 4).
    let _ = mnemonic_gui::app::placeholder;
}
