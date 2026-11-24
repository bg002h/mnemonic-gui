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

// ─── FIX sec-secret-drift-gate-fail-closed: the gate's input contract ───────
//
// Each cell re-runs the gate test above in a child process (this same test
// binary, `--exact`), with `MNEMONIC_BIN` / `PATH` pointed at a fake
// `mnemonic`, and asserts the child's exit and its printed line. The fakes are
// `/bin/sh` scripts, so the cells are unix-only (CI runs them on ubuntu).

#[cfg(unix)]
mod gate_contract {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const GATE: &str = "secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode";

    fn fake_bin(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    /// Run the gate in a child; returns (exit success, stdout + stderr).
    fn run_gate(setup: impl FnOnce(&mut Command)) -> (bool, String) {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args(["--exact", GATE, "--nocapture"]);
        setup(&mut cmd);
        let out = cmd.output().unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.success(), text)
    }

    fn assert_fails_with(ok: bool, text: &str, needle: &str) {
        assert!(
            !ok && text.contains(needle),
            "gate must FAIL printing {needle:?}; child success={ok}, output:\n{text}"
        );
    }

    #[test]
    fn drift_gate_fails_when_bin_set_and_absent() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("absent-mnemonic");
        let (ok, text) = run_gate(|c| {
            c.env("MNEMONIC_BIN", &bin);
        });
        assert_fails_with(
            ok,
            &text,
            &format!("schema_mirror_secret_drift: cannot run `{} gui-schema`: ", bin.display()),
        );
    }

    #[test]
    fn drift_gate_fails_when_bin_exits_nonzero() {
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "mnemonic", "exit 3");
        let (ok, text) = run_gate(|c| {
            c.env("MNEMONIC_BIN", &bin);
        });
        assert_fails_with(
            ok,
            &text,
            &format!(
                "schema_mirror_secret_drift: `{} gui-schema` failed: exit status: 3",
                bin.display()
            ),
        );
    }

    #[test]
    fn drift_gate_fails_when_stdout_not_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "mnemonic", "printf '\\377'");
        let (ok, text) = run_gate(|c| {
            c.env("MNEMONIC_BIN", &bin);
        });
        assert_fails_with(
            ok,
            &text,
            &format!(
                "schema_mirror_secret_drift: `{} gui-schema` stdout is not UTF-8",
                bin.display()
            ),
        );
    }

    #[test]
    fn drift_gate_fails_when_stdout_not_json() {
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "mnemonic", "echo not-json");
        let (ok, text) = run_gate(|c| {
            c.env("MNEMONIC_BIN", &bin);
        });
        assert_fails_with(
            ok,
            &text,
            &format!(
                "schema_mirror_secret_drift: `{} gui-schema` stdout is not a gui-schema JSON document: ",
                bin.display()
            ),
        );
    }

    #[test]
    fn drift_gate_fails_when_schema_version_below_5() {
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(
            dir.path(),
            "mnemonic",
            r#"echo '{"version":4,"cli":"mnemonic","subcommands":[]}'"#,
        );
        let (ok, text) = run_gate(|c| {
            c.env("MNEMONIC_BIN", &bin);
        });
        assert_fails_with(
            ok,
            &text,
            &format!(
                "schema_mirror_secret_drift: `{} gui-schema` reports schema version 4; \
                 this gate needs version 5 or later (per-flag `secret` field)",
                bin.display()
            ),
        );
    }

    #[test]
    fn drift_gate_fails_when_bin_unset_and_path_mnemonic_is_pre_v5() {
        let dir = tempfile::tempdir().unwrap();
        fake_bin(
            dir.path(),
            "mnemonic",
            r#"echo '{"version":4,"cli":"mnemonic","subcommands":[]}'"#,
        );
        let (ok, text) = run_gate(|c| {
            c.env_remove("MNEMONIC_BIN").env("PATH", dir.path());
        });
        assert_fails_with(
            ok,
            &text,
            "schema_mirror_secret_drift: `mnemonic gui-schema` reports schema version 4; \
             this gate needs version 5 or later (per-flag `secret` field)",
        );
    }

    #[test]
    fn drift_gate_skips_loudly_when_bin_unset_and_not_on_path() {
        let dir = tempfile::tempdir().unwrap();
        let (ok, text) = run_gate(|c| {
            c.env_remove("MNEMONIC_BIN").env("PATH", dir.path());
        });
        let needle = "schema_mirror_secret_drift: SKIP: MNEMONIC_BIN is unset and `mnemonic` \
                      is not on PATH; set MNEMONIC_BIN to the pinned toolkit binary to run this gate";
        assert!(
            ok && text.contains(needle),
            "gate must PASS printing {needle:?}; child success={ok}, output:\n{text}"
        );
    }

    #[test]
    fn drift_gate_passes_on_matching_v5_document() {
        let subs: Vec<serde_json::Value> = mnemonic_gui::schema::mnemonic::SCHEMA
            .subcommands
            .iter()
            .map(|s| {
                let flags: Vec<serde_json::Value> = s
                    .flags
                    .iter()
                    .map(|f| serde_json::json!({ "name": f.name, "secret": f.secret }))
                    .collect();
                serde_json::json!({ "name": s.name, "flags": flags })
            })
            .collect();
        let doc = serde_json::json!({ "version": 5, "cli": "mnemonic", "subcommands": subs });
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("schema.json");
        std::fs::write(&json, doc.to_string()).unwrap();
        let bin = fake_bin(dir.path(), "mnemonic", &format!("cat '{}'", json.display()));
        let (ok, text) = run_gate(|c| {
            c.env("MNEMONIC_BIN", &bin);
        });
        assert!(ok, "gate must PASS on a matching v5 document; output:\n{text}");
    }
}
