//! state.json round-trip + never-persist audit (SPEC §B.10 + Phase 8).
//!
//! Cells:
//!   - Round-trip: write PersistedState → read → assert equality.
//!   - Never-persist set: write a state with secret-class entries; read;
//!     assert NONE of `SECRET_FLAG_NAMES`, `SECRET_NODE_TYPES_ARGV`,
//!     `SECRET_SLOT_SUBKEYS` survives. The audit reads the SECRET_*
//!     constants DYNAMICALLY (not hard-coded literals) so adding new
//!     upstream secret-bearing variants automatically extends the audit.
//!   - Schema-version migration: write a state with the wrong version;
//!     assert load returns None AND renames the file to `<path>.bak`.
//!   - Schema-load round-trip / flag-name stability: serialize then
//!     deserialize a populated FormState; assert in-place stability.

use std::collections::BTreeMap;
use std::fs;

use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::persistence::{
    load, redact_for_persistence, redact_persisted_state, save, PersistedState, SCHEMA_VERSION,
};
use mnemonic_gui::schema::{FlagValue, FormState};
use mnemonic_gui::secrets::{SECRET_FLAG_NAMES, SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS};

fn watch_only_form() -> FormState {
    FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--account", FlagValue::Number(0)),
    ])
    .with_slots(SlotState {
        rows: vec![
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Xpub,
                value: "xpub...".into(),
            },
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Fingerprint,
                value: "00000000".into(),
            },
        ],
    })
}

fn mixed_form_with_secrets() -> FormState {
    FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--passphrase", FlagValue::Text("hunter2".into())), // SECRET
        ("--bip38-passphrase", FlagValue::Text("scrypt".into())), // SECRET
        ("--passphrase-stdin", FlagValue::Boolean(true)), // SECRET
        (
            "--from",
            FlagValue::NodeValueComposite {
                node: "phrase".into(), // SECRET
                value: "abandon...".into(),
            },
        ),
        (
            "--to",
            FlagValue::NodeValueComposite {
                node: "xpub".into(), // watch-only — persists
                value: "".into(),
            },
        ),
    ])
    .with_slots(SlotState {
        rows: vec![
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Phrase, // SECRET
                value: "abandon abandon...".into(),
            },
            SlotRow {
                index: 1,
                subkey: SlotSubkey::Xpub, // watch-only — persists
                value: "xpub...".into(),
            },
            SlotRow {
                index: 2,
                subkey: SlotSubkey::Wif, // SECRET
                value: "Kxxxxx...".into(),
            },
            SlotRow {
                index: 3,
                subkey: SlotSubkey::Fingerprint, // watch-only — persists
                value: "deadbeef".into(),
            },
        ],
    })
}

fn populated_persisted_state(form: FormState) -> PersistedState {
    let mut map = BTreeMap::new();
    map.insert("mnemonic:bundle".into(), form);
    let mut subs = BTreeMap::new();
    subs.insert("mnemonic".into(), "bundle".into());
    subs.insert("md".into(), "inspect".into());
    PersistedState {
        schema_version: SCHEMA_VERSION,
        last_cli_tab: "mnemonic".into(),
        last_subcommand_per_tab: subs,
        window_size: Some([1024.0, 768.0]),
        window_position: Some([100.0, 100.0]),
        show_cmdline: true,
        show_stdout: true,
        show_stderr: false,
        form_state_per_subcommand: map,
    }
}

#[test]
fn cell_1_round_trip_watch_only_state() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let original = populated_persisted_state(watch_only_form());
    save(&original, &path).unwrap();
    let loaded = load(&path).expect("load");
    assert_eq!(loaded.schema_version, SCHEMA_VERSION);
    assert_eq!(loaded.last_cli_tab, "mnemonic");
    assert_eq!(
        loaded.last_subcommand_per_tab.get("mnemonic").unwrap(),
        "bundle"
    );
    assert_eq!(loaded.window_size, Some([1024.0, 768.0]));
    assert!(loaded.show_cmdline);
    assert!(!loaded.show_stderr);
    let loaded_form = loaded.form_state_per_subcommand.get("mnemonic:bundle").unwrap();
    // Watch-only form has 3 values; all survive.
    assert_eq!(loaded_form.values.len(), 3);
    assert_eq!(loaded_form.slots.rows.len(), 2);
}

#[test]
fn cell_2_never_persist_audit_strips_all_secret_flags() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let mixed = populated_persisted_state(mixed_form_with_secrets());
    save(&mixed, &path).unwrap();

    // Read the raw on-disk JSON. NO secret-class flag name may appear
    // anywhere in the serialized representation.
    let on_disk = fs::read_to_string(&path).unwrap();

    for secret_flag in SECRET_FLAG_NAMES {
        assert!(
            !on_disk.contains(secret_flag),
            "secret flag {} leaked to disk:\n{}",
            secret_flag,
            on_disk
        );
    }
    // NodeValueComposite node tokens — quoted ("node-name") match.
    // cycle-3 H3: iterate the WIDE argv/redaction set (narrow + minikey) so the
    // authoritative redaction set is exercised here too (defense-in-depth). The
    // genuine minikey on-disk proof lives in tests/h3_minikey_secret_surfaces.rs
    // (this fixture has only phrase/xpub composites, so the minikey iteration is
    // a no-op against THIS state — the loop just tracks the authoritative set).
    for node in SECRET_NODE_TYPES_ARGV {
        let quoted = format!("\"{}\"", node);
        assert!(
            !on_disk.contains(&quoted),
            "secret node token {} leaked to disk via NodeValueComposite:\n{}",
            quoted,
            on_disk
        );
    }
    // R1 I-2 fold: `SlotSubkey` derives Serialize WITHOUT
    // #[serde(rename_all = "snake_case")], so serde emits PascalCase
    // variant names. The lowercase as_str() form (defence-in-depth)
    // passes vacuously because serde never produces it; the load-bearing
    // check is the PascalCase form, which exercises the actual
    // serialization path.
    const SECRET_SUBKEY_PASCAL: &[&str] = &["Phrase", "Entropy", "Wif", "Xprv"];
    for name in SECRET_SUBKEY_PASCAL {
        let quoted = format!("\"{}\"", name);
        assert!(
            !on_disk.contains(&quoted),
            "secret slot subkey {} (PascalCase) found in on-disk JSON — \
             redact_for_persistence is broken",
            quoted
        );
    }
    // Lowercase form retained as defence-in-depth: if a future
    // #[serde(rename_all = "snake_case")] annotation lands, this guard
    // catches the migration without rewriting the test.
    for subkey in SECRET_SLOT_SUBKEYS {
        let quoted_lc = format!("\"{}\"", subkey);
        assert!(
            !on_disk.contains(&quoted_lc),
            "secret slot subkey {} (lowercase) found in on-disk JSON",
            quoted_lc
        );
    }
    // Specifically: known secret VALUES from the fixture must not survive.
    for leak in [
        "hunter2",
        "scrypt",
        "abandon",
        "Kxxxxx",
    ] {
        assert!(
            !on_disk.contains(leak),
            "secret value `{}` leaked to disk:\n{}",
            leak,
            on_disk
        );
    }
}

#[test]
fn cell_3_watch_only_values_survive_redaction() {
    let mixed = mixed_form_with_secrets();
    let redacted = redact_for_persistence(&mixed);

    // Watch-only values that MUST survive:
    assert!(redacted
        .values
        .iter()
        .any(|(k, _)| k == "--network"));
    assert!(redacted
        .values
        .iter()
        .any(|(k, _)| k == "--template"));
    // The --to NodeValueComposite with node="xpub" must survive.
    assert!(redacted.values.iter().any(|(k, v)| k == "--to"
        && matches!(v, FlagValue::NodeValueComposite { node, .. } if node == "xpub")));

    // Secret-class values that MUST NOT survive:
    assert!(!redacted
        .values
        .iter()
        .any(|(k, _)| k == "--passphrase"));
    assert!(!redacted
        .values
        .iter()
        .any(|(k, _)| k == "--bip38-passphrase"));
    assert!(!redacted
        .values
        .iter()
        .any(|(k, _)| k == "--passphrase-stdin"));
    // The --from NodeValueComposite with node="phrase" must NOT survive.
    assert!(!redacted.values.iter().any(|(k, v)| k == "--from"
        && matches!(v, FlagValue::NodeValueComposite { node, .. } if node == "phrase")));

    // Slots: watch-only Xpub + Fingerprint survive; Phrase + Wif don't.
    let subkeys: Vec<SlotSubkey> = redacted.slots.rows.iter().map(|r| r.subkey).collect();
    assert!(subkeys.contains(&SlotSubkey::Xpub));
    assert!(subkeys.contains(&SlotSubkey::Fingerprint));
    assert!(!subkeys.contains(&SlotSubkey::Phrase));
    assert!(!subkeys.contains(&SlotSubkey::Wif));
}

#[test]
fn cell_4_schema_version_mismatch_renames_bak() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let bak = path.with_extension("json.bak");

    // Write a state.json with a stale schema_version.
    let mut state = populated_persisted_state(watch_only_form());
    state.schema_version = 999;
    let raw = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&path, &raw).unwrap();
    assert!(!bak.exists(), "bak must not pre-exist");

    let result = load(&path);
    assert!(result.is_none(), "stale-version load must return None");
    assert!(bak.exists(), "stale state.json must be renamed to .bak");
    assert!(!path.exists(), "primary state.json must be gone after rename");
}

#[test]
fn cell_5_missing_file_yields_none() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    assert!(load(&path).is_none(), "missing file → None");
}

// v0.35.0: load() now ALSO renames a malformed file to .bak — the rename
// leg is pinned by persistence_wiring_v0_35_0 T3; this cell pins only None.
#[test]
fn cell_6_malformed_json_yields_none() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    fs::write(&path, "{not valid json").unwrap();
    assert!(load(&path).is_none(), "malformed JSON → None");
}

#[test]
fn cell_7_form_state_round_trip_in_memory() {
    // Schema-load round-trip: serialize then deserialize a populated
    // FormState. Watch-only flags / slots survive byte-identically.
    let original = watch_only_form();
    let json = serde_json::to_string(&original).unwrap();
    let back: FormState = serde_json::from_str(&json).unwrap();

    assert_eq!(back.values.len(), original.values.len());
    assert_eq!(back.slots.rows.len(), original.slots.rows.len());
    for (orig, restored) in original.slots.rows.iter().zip(back.slots.rows.iter()) {
        assert_eq!(orig.index, restored.index);
        assert_eq!(orig.subkey, restored.subkey);
        assert_eq!(orig.value, restored.value);
    }
}

#[test]
fn cell_8_dynamic_secret_set_audit_no_hardcoded_literals() {
    // Sanity: the test reads SECRET_NODE_TYPES_ARGV + SECRET_SLOT_SUBKEYS
    // FROM THE LIBRARY (re-exported from
    // `mnemonic_toolkit::secret_taxonomy` since v0.4.0; was
    // build.rs-generated in v0.3.x) — not hard-coded literals here.
    // R1 I-4 fold: dynamic source so adding new upstream
    // is_secret_bearing variants automatically extends the audit.
    // cycle-3 H3: the wide argv/redaction set is the authoritative
    // composite-drop set used by `redact_for_persistence`.
    assert!(!SECRET_NODE_TYPES_ARGV.is_empty());
    assert!(!SECRET_SLOT_SUBKEYS.is_empty());
    // Every entry in these arrays must be excluded from redacted output.
    let mixed = mixed_form_with_secrets();
    let redacted = redact_for_persistence(&mixed);
    for sk in &mixed.slots.rows {
        if SECRET_SLOT_SUBKEYS.contains(&sk.subkey.as_str()) {
            assert!(
                !redacted
                    .slots
                    .rows
                    .iter()
                    .any(|r| r.subkey == sk.subkey),
                "subkey {:?} not stripped",
                sk.subkey
            );
        }
    }
}

#[test]
fn cell_9_redact_persisted_state_idempotent() {
    // Redacting twice gives the same result.
    let state = populated_persisted_state(mixed_form_with_secrets());
    let once = redact_persisted_state(&state);
    let twice = redact_persisted_state(&once);
    assert_eq!(
        serde_json::to_string(&once).unwrap(),
        serde_json::to_string(&twice).unwrap()
    );
}

#[test]
fn cell_11_save_stamps_current_schema_version_even_with_default_input() {
    // R1 I-1 fold regression guard. PersistedState::default() has
    // schema_version: 0; pre-fold save() would write 0 → load() rename
    // to .bak + None → state lost on every cold start.
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let default_state = PersistedState::default();
    assert_eq!(default_state.schema_version, 0, "default must be 0");
    save(&default_state, &path).unwrap();

    // Reload and verify schema_version got stamped to SCHEMA_VERSION.
    let loaded = load(&path).expect("default round-trip must succeed");
    assert_eq!(loaded.schema_version, SCHEMA_VERSION);
    let bak = path.with_extension("json.bak");
    assert!(
        !bak.exists(),
        "no .bak should be created — save() stamped current version"
    );
}

#[test]
fn cell_10_save_creates_parent_dirs() {
    // Config dir may not exist on first launch; save() must mkdir -p.
    let dir = tempfile::TempDir::new().unwrap();
    let nested = dir.path().join("a/b/c/state.json");
    let state = populated_persisted_state(watch_only_form());
    save(&state, &nested).unwrap();
    assert!(nested.exists());
}

// ─── v0.2 Phase B.1: secret_widgets never persist, both directions ───────

#[test]
fn secret_widgets_round_trip_never_persists_both_directions() {
    // SPEC §10 R1 N-4 fold: FormState::secret_widgets carries
    // #[serde(skip)], so the never-persist invariant is enforced by
    // type. Both directions must hold:
    //   (a) Serialize a populated FormState → secret_widgets absent.
    //   (b) Deserialize an arbitrary FormState JSON → secret_widgets
    //       default-constructs to empty.
    use mnemonic_gui::form::secret_widget::SecretLineEdit;

    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    let mut form = FormState::default();
    form.values.push((
        "--network".to_string(),
        FlagValue::Dropdown("mainnet".into()),
    ));
    form.secret_widgets
        .insert("--passphrase".into(), vec![SecretLineEdit::from_text("hunter2")]);
    let mut state = PersistedState::default();
    state
        .form_state_per_subcommand
        .insert("mnemonic:bundle".into(), form);
    save(&state, &path).unwrap();

    let on_disk = fs::read_to_string(&path).unwrap();
    assert!(
        !on_disk.contains("secret_widgets"),
        "secret_widgets key must not appear on disk: {}",
        on_disk
    );
    assert!(
        !on_disk.contains("--passphrase"),
        "secret-flag name must not appear on disk: {}",
        on_disk
    );
    assert!(
        !on_disk.contains("hunter2"),
        "secret value must not appear on disk: {}",
        on_disk
    );

    // Direction (b): load it back; secret_widgets is empty.
    let loaded = load(&path).expect("round-trip must succeed");
    let new_form = loaded
        .form_state_per_subcommand
        .get("mnemonic:bundle")
        .expect("subcommand state present");
    assert!(
        new_form.secret_widgets.is_empty(),
        "deserialized secret_widgets must be empty (serde skip + Default)"
    );
    assert!(new_form.values.iter().any(|(k, _)| k == "--network"));
}
