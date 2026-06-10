//! Phase-8 persistence WIRING (v0.35.0 — SPEC_gui_v0_35_0_persistence_wiring).
//!
//! Cells:
//!   - T1: `CliTab::from_bin_name` round-trips all 4 tabs + unknown → None.
//!   - T2: `restore_selections` validation — stale tab → Mnemonic, stale
//!     subcommand → per-tab default, valid entries restore, unavailable
//!     tab → default.
//!   - T3: `.bak`-on-malformed symmetry — malformed JSON renames the file
//!     to `.json.bak` and returns None (the version-mismatch leg already
//!     did; this cell pins the new symmetric behavior + re-asserts the
//!     version-mismatch leg).
//!   - T5: end-to-end round-trip — save → load → restore_selections; all
//!     six persisted field groups land; a seeded secret_widgets value
//!     does NOT survive.
//!
//! Env-seam isolation rule (SPEC Decision 6 / R0-r1 I4): NOTHING in this
//! file touches `MNEMONIC_GUI_STATE_PATH` — T5 uses explicit `&Path`
//! args. The env-seam cell lives in its own dedicated tests file
//! (`tests/persistence_env_seam.rs`, own process).

use std::collections::BTreeMap;
use std::fs;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::persistence::{
    load, restore_selections, save, PersistedState, SCHEMA_VERSION,
};
use mnemonic_gui::schema::{FlagValue, FormState};

// ─── T1: CliTab::from_bin_name ───────────────────────────────────────────

#[test]
fn t1_from_bin_name_round_trips_all_tabs_and_rejects_unknown() {
    for tab in CliTab::ALL {
        assert_eq!(
            CliTab::from_bin_name(tab.bin_name()),
            Some(*tab),
            "from_bin_name must invert bin_name for {:?}",
            tab
        );
    }
    assert_eq!(CliTab::from_bin_name("floppy"), None);
    assert_eq!(CliTab::from_bin_name(""), None);
    // Case-sensitive: bin_name() emits lowercase only.
    assert_eq!(CliTab::from_bin_name("Mnemonic"), None);
}

// ─── T2: restore_selections validation ───────────────────────────────────

fn selections_state(tab: &str, subs: &[(&str, &str)]) -> PersistedState {
    PersistedState {
        last_cli_tab: tab.to_string(),
        last_subcommand_per_tab: subs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}

#[test]
fn t2_valid_tab_and_subcommands_restore() {
    let state = selections_state(
        "md",
        &[("mnemonic", "export-wallet"), ("md", "encode"), ("ms", "verify")],
    );
    let (tab, subs) = restore_selections(&state, |_| true);
    assert_eq!(tab, CliTab::Md);
    assert_eq!(subs.get(&CliTab::Mnemonic).unwrap(), "export-wallet");
    assert_eq!(subs.get(&CliTab::Md).unwrap(), "encode");
    assert_eq!(subs.get(&CliTab::Ms).unwrap(), "verify");
    // Missing entry → per-tab default.
    assert_eq!(subs.get(&CliTab::Mk).unwrap(), "inspect");
}

#[test]
fn t2_stale_tab_falls_back_to_mnemonic() {
    let state = selections_state("floppy", &[]);
    let (tab, _) = restore_selections(&state, |_| true);
    assert_eq!(tab, CliTab::Mnemonic);
}

#[test]
fn t2_unavailable_tab_falls_back_to_mnemonic() {
    // Restored tab parses fine but its binary is gone → default.
    let state = selections_state("ms", &[]);
    let (tab, _) = restore_selections(&state, |t| t != CliTab::Ms);
    assert_eq!(tab, CliTab::Mnemonic);
}

#[test]
fn t2_stale_subcommand_falls_back_to_per_tab_default() {
    let state = selections_state(
        "mnemonic",
        &[("mnemonic", "no-such-subcommand"), ("md", "also-gone")],
    );
    let (tab, subs) = restore_selections(&state, |_| true);
    assert_eq!(tab, CliTab::Mnemonic);
    // Stale entries → the hardcoded defaults (bundle for mnemonic,
    // inspect for the codec tabs); every tab has an entry.
    assert_eq!(subs.get(&CliTab::Mnemonic).unwrap(), "bundle");
    assert_eq!(subs.get(&CliTab::Md).unwrap(), "inspect");
    assert_eq!(subs.get(&CliTab::Ms).unwrap(), "inspect");
    assert_eq!(subs.get(&CliTab::Mk).unwrap(), "inspect");
}

#[test]
fn t2_default_persisted_state_yields_hardcoded_defaults() {
    // PersistedState::default() (empty tab string, empty map) must map to
    // exactly the pre-wiring hardcoded selections — the None-loaded and
    // default-loaded paths converge.
    let (tab, subs) = restore_selections(&PersistedState::default(), |_| true);
    assert_eq!(tab, CliTab::Mnemonic);
    assert_eq!(subs.len(), 4, "every tab gets a default entry");
    assert_eq!(subs.get(&CliTab::Mnemonic).unwrap(), "bundle");
    assert_eq!(subs.get(&CliTab::Mk).unwrap(), "inspect");
}

// ─── T3: `.bak`-on-malformed symmetry ────────────────────────────────────

#[test]
fn t3_malformed_json_renames_to_bak_and_returns_none() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let bak = path.with_extension("json.bak");
    fs::write(&path, "{not valid json").unwrap();
    assert!(!bak.exists(), "bak must not pre-exist");

    let result = load(&path);
    assert!(result.is_none(), "malformed JSON → None");
    assert!(
        bak.exists(),
        "malformed state.json must be renamed to .bak (preserved for \
         diagnosis, symmetric with the version-mismatch leg)"
    );
    assert!(
        !path.exists(),
        "primary state.json must be gone after the .bak rename"
    );
    let preserved = fs::read_to_string(&bak).unwrap();
    assert_eq!(preserved, "{not valid json", "corrupt content preserved");
}

#[test]
fn t3_version_mismatch_leg_still_renames_to_bak() {
    // Re-assert the pre-existing version-mismatch leg alongside the new
    // malformed leg (tests/persistence.rs cell_4 stays the canonical
    // cell; this re-pins symmetry from the wiring suite's vantage).
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let bak = path.with_extension("json.bak");
    let state = PersistedState {
        schema_version: SCHEMA_VERSION + 999,
        ..Default::default()
    };
    fs::write(&path, serde_json::to_string_pretty(&state).unwrap()).unwrap();

    assert!(load(&path).is_none(), "stale-version load must return None");
    assert!(bak.exists(), "stale state.json must be renamed to .bak");
    assert!(!path.exists());
}

#[test]
fn t3_missing_file_stays_plain_none_no_bak() {
    // Missing file must stay the quiet leg: None, and no stray .bak.
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    assert!(load(&path).is_none(), "missing file → None");
    assert!(
        !path.with_extension("json.bak").exists(),
        "missing file must not conjure a .bak"
    );
}

// ─── T5: end-to-end round-trip (explicit paths, NEVER the env seam) ──────

#[test]
fn t5_end_to_end_round_trip_all_six_groups_land_secrets_do_not() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");

    // Build the save-side state exactly as on_exit would: selections +
    // toggles off + non-secret form value + geometry, plus a seeded
    // secret_widgets value that must NOT survive.
    let mut form = FormState::default();
    form.values.push((
        "--network".to_string(),
        FlagValue::Dropdown("testnet".into()),
    ));
    form.secret_widgets.insert(
        "--passphrase".into(),
        vec![SecretLineEdit::from_text("hunter2")],
    );
    let mut forms = BTreeMap::new();
    forms.insert("mnemonic:bundle".to_string(), form);

    let state = PersistedState {
        schema_version: SCHEMA_VERSION,
        last_cli_tab: "md".into(),
        last_subcommand_per_tab: [
            ("mnemonic".to_string(), "export-wallet".to_string()),
            ("md".to_string(), "decode".to_string()),
        ]
        .into_iter()
        .collect(),
        window_size: Some([800.0, 600.0]),
        window_position: Some([10.0, 20.0]),
        show_cmdline: false,
        show_stdout: false,
        show_stderr: false,
        form_state_per_subcommand: forms,
    };

    save(&state, &path).unwrap();
    let loaded = load(&path).expect("round-trip load must succeed");

    // Group 1+2: tab + per-tab subcommand, via the restore mapping.
    let (tab, subs) = restore_selections(&loaded, |_| true);
    assert_eq!(tab, CliTab::Md);
    assert_eq!(subs.get(&CliTab::Mnemonic).unwrap(), "export-wallet");
    assert_eq!(subs.get(&CliTab::Md).unwrap(), "decode");
    assert_eq!(subs.get(&CliTab::Ms).unwrap(), "inspect");
    assert_eq!(subs.get(&CliTab::Mk).unwrap(), "inspect");

    // Group 3: geometry (size + position).
    assert_eq!(loaded.window_size, Some([800.0, 600.0]));
    assert_eq!(loaded.window_position, Some([10.0, 20.0]));

    // Group 4: the 3 output-pane toggles.
    assert!(!loaded.show_cmdline);
    assert!(!loaded.show_stdout);
    assert!(!loaded.show_stderr);

    // Group 5: non-secret form value survives, under the same key.
    let restored_form = loaded
        .form_state_per_subcommand
        .get("mnemonic:bundle")
        .expect("form state restored under the cli:sub key");
    assert!(restored_form
        .values
        .iter()
        .any(|(k, v)| k == "--network"
            && matches!(v, FlagValue::Dropdown(s) if s == "testnet")));

    // Group 6 (the inverted group): the seeded secret did NOT survive —
    // neither on disk nor in the restored FormState (type-level
    // #[serde(skip)] + redaction at save).
    let on_disk = fs::read_to_string(&path).unwrap();
    assert!(!on_disk.contains("hunter2"), "secret leaked to disk");
    assert!(!on_disk.contains("--passphrase"), "secret flag name leaked");
    assert!(
        restored_form.secret_widgets.is_empty(),
        "secret_widgets must come back empty"
    );
}
