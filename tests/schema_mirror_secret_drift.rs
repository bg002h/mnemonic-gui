//! v0.10.0 Tranche B.3 (D31) — secret-bool drift gate.
//!
//! For the `mnemonic` CLI: shells out to `mnemonic gui-schema`, collects
//! the `{(subcommand, flag_name) | flag.secret == true}` set from the v5
//! JSON, and asserts set-equality with the GUI's hand-coded
//! `FlagSchema.secret == true` set across the same subcommands.
//!
//! Toolkit-side ground truth: `mnemonic_toolkit::secrets::flag_is_secret`
//! is the union of NodeType + SlotSubkey + a hand-coded flag-name list.
//! The v5 schema's per-flag `secret` field is its projection. The GUI's
//! hand-coded `FlagSchema.secret` field must mirror this projection for
//! the existing `flag_is_secret` consumers (`secrets::flag_is_secret` at
//! `src/secrets.rs:134`).
//!
//! Drift here is HIGH-severity (was the v0.3.0..v0.3.2 BIP-39 persistence
//! leak class — see `src/secrets.rs` module doc + CHANGELOG v0.3.3). This
//! gate fires immediately on drift.
//!
//! Binary lookup mirrors `schema_check::json_flag_names`: `MNEMONIC_BIN`
//! env-var wins; otherwise `mnemonic` is invoked through `$PATH`. If the
//! binary cannot be located OR the output isn't v5+, the cell SKIPS (so
//! dev-laptop runs without the v5 binary don't spuriously fail).
//!
//! Companion to `tests/secret_taxonomy_pin.rs` (which pins the
//! NodeType / SlotSubkey constants) — this gate covers the per-flag
//! projection, the other gate covers the per-NodeType / per-SlotSubkey set.

use std::collections::BTreeSet;
use std::process::Command;

use mnemonic_gui::schema;

#[derive(Debug, serde::Deserialize)]
struct GuiSchemaV5Root {
    version: u32,
    cli: String,
    subcommands: Vec<GuiSchemaV5Subcommand>,
}

#[derive(Debug, serde::Deserialize)]
struct GuiSchemaV5Subcommand {
    name: String,
    #[serde(default)]
    flags: Vec<GuiSchemaV5Flag>,
}

#[derive(Debug, serde::Deserialize)]
struct GuiSchemaV5Flag {
    name: String,
    #[serde(default)]
    secret: bool,
}

fn resolve_mnemonic_bin() -> String {
    std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| "mnemonic".to_string())
}

/// Returns Some((cli, subcommands)) on success; None if the binary can't
/// be located, fails to produce gui-schema output, or emits a pre-v5
/// schema (the drift-gate is v5-only — pre-v5 docs lack per-flag `secret`).
fn fetch_v5_schema() -> Option<(String, Vec<GuiSchemaV5Subcommand>)> {
    let bin = resolve_mnemonic_bin();
    let output = Command::new(&bin).arg("gui-schema").output().ok()?;
    if !output.status.success() {
        eprintln!(
            "schema_mirror_secret_drift: `{bin} gui-schema` exited {:?}; skipping",
            output.status,
        );
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let root: GuiSchemaV5Root = serde_json::from_str(&stdout).ok()?;
    if root.version < 5 {
        eprintln!(
            "schema_mirror_secret_drift: gui-schema version {} (< 5); \
             skipping drift gate (v5 introduces the per-flag `secret` field)",
            root.version,
        );
        return None;
    }
    Some((root.cli, root.subcommands))
}

#[test]
fn secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode() {
    let Some((cli, subcommands)) = fetch_v5_schema() else {
        // SKIP (not failure): the gate is best-effort against a live
        // toolkit binary. CI gates this via pinned-upstream + the
        // schema-mirror workflow's binary install.
        return;
    };
    assert_eq!(cli, "mnemonic", "gui-schema cli field mismatch");

    // Collect from toolkit v5: { (subcommand, flag_name) | flag.secret }.
    let mut toolkit_secrets: BTreeSet<(String, String)> = BTreeSet::new();
    for sub in &subcommands {
        for flag in &sub.flags {
            if flag.secret {
                toolkit_secrets.insert((sub.name.clone(), flag.name.clone()));
            }
        }
    }

    // Collect from GUI hand-code: same shape.
    let mut gui_secrets: BTreeSet<(String, String)> = BTreeSet::new();
    for sub in schema::mnemonic::SCHEMA.subcommands {
        for flag in sub.flags {
            if flag.secret {
                gui_secrets.insert((sub.name.to_string(), flag.name.to_string()));
            }
        }
    }

    let only_in_toolkit: Vec<_> = toolkit_secrets.difference(&gui_secrets).collect();
    let only_in_gui: Vec<_> = gui_secrets.difference(&toolkit_secrets).collect();

    assert!(
        only_in_toolkit.is_empty() && only_in_gui.is_empty(),
        "secret-flag drift between toolkit v5 schema and GUI hand-code:\n  \
         only in toolkit (GUI must add secret=true): {only_in_toolkit:?}\n  \
         only in GUI (GUI must remove secret=true, OR file a FOLLOWUP if \
         the GUI's broader secret-class semantic should stay): {only_in_gui:?}"
    );
}
