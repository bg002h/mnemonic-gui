//! T4 — `MNEMONIC_GUI_STATE_PATH` env seam for `default_state_path()`
//! (v0.35.0 SPEC Decision 6).
//!
//! ISOLATION RULE (R0-r1 I4): env mutation is process-global and cargo
//! runs `#[test]`s in parallel threads within one test binary — so this
//! file is DEDICATED to the env seam (its own process), and AT MOST ONE
//! test in this binary mutates `MNEMONIC_GUI_STATE_PATH`. Every other
//! persistence test uses explicit `&Path` args, never the env seam.

use mnemonic_gui::persistence::default_state_path;

#[test]
fn t4_env_var_overrides_default_state_path() {
    // The ONE test in this binary that mutates the var.
    let dir = tempfile::TempDir::new().unwrap();
    let override_path = dir.path().join("custom-state.json");
    std::env::set_var("MNEMONIC_GUI_STATE_PATH", &override_path);
    assert_eq!(
        default_state_path(),
        Some(override_path.clone()),
        "MNEMONIC_GUI_STATE_PATH must override the platform config dir"
    );

    // Empty string = unset (the non-empty guard; impl-review minor 1).
    std::env::set_var("MNEMONIC_GUI_STATE_PATH", "");
    assert_ne!(
        default_state_path(),
        Some(std::path::PathBuf::from("")),
        "an EMPTY env value must be ignored, not yield an empty path"
    );
    assert_ne!(
        default_state_path(),
        Some(override_path.clone()),
        "an empty env value must fall back, never the stale override"
    );

    std::env::remove_var("MNEMONIC_GUI_STATE_PATH");
    assert_ne!(
        default_state_path(),
        Some(override_path),
        "with the var unset, resolution must fall back to the platform \
         config dir (or None), never the stale override"
    );
}
