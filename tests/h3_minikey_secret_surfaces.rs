//! cycle-3 H3 — AUTHORITATIVE regression: a `--from minikey=<KEY>` composite
//! (a Casascius mini PRIVATE KEY) must be treated as secret across the GUI's
//! argv/confirm/persist surfaces, on parity with `phrase`/`xprv`.
//!
//! The GUI previously classified composite-node secrecy via the NARROW
//! `SECRET_NODE_TYPES`, which EXCLUDES `minikey`. So `minikey` fell through
//! four surfaces: argv-mask (unmasked in Preview / last-run), `should_confirm_run`
//! (no run-confirm), persist-redact (written PLAINTEXT to state.json), and
//! paste-warn (covered by the widget test in `tests/h3_minikey_paste_warn.rs`).
//! This file is the authoritative coverage for surfaces (i) argv-mask,
//! (ii) run-confirm, and (iv) persist-redact, because the `tests/persistence.rs`
//! fixture does NOT contain a minikey composite — so widening that file's
//! on-disk loop alone is vacuous for minikey.
//!
//! Negative controls throughout: a `--from xpub=…` composite (watch-only) is
//! NOT masked, does NOT confirm, and IS persisted.

use mnemonic_gui::form::invocation::assemble_argv_with_secret_mask;
use mnemonic_gui::persistence::{redact_for_persistence, save, PersistedState, SCHEMA_VERSION};
use mnemonic_gui::schema::{self, FlagValue, FormState};
use mnemonic_gui::secrets::should_confirm_run;

use std::collections::BTreeMap;

/// A realistic Casascius mini private key (30-char condensed form).
const MINIKEY: &str = "S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy";
/// A watch-only xpub (negative control).
const WATCH_XPUB: &str =
    "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet";

fn convert_subcommand() -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "convert")
        .expect("convert subcommand in schema")
}

fn from_composite(node: &str, value: &str) -> FormState {
    FormState::from_pairs(vec![(
        "--from",
        FlagValue::NodeValueComposite {
            node: node.into(),
            value: value.into(),
        },
    )])
}

// ── surface (i) — argv-mask ──────────────────────────────────────────────────

#[test]
fn surface_i_minikey_composite_value_token_is_masked() {
    let state = from_composite("minikey", MINIKEY);
    let (argv, mask) = assemble_argv_with_secret_mask(
        &schema::mnemonic::SCHEMA,
        convert_subcommand(),
        &state,
    );
    // Locate the `minikey=<KEY>` value token and assert its mask bit is true.
    let token = format!("minikey={MINIKEY}");
    let idx = argv
        .iter()
        .position(|a| *a == token)
        .unwrap_or_else(|| panic!("minikey value token not in argv: {argv:?}"));
    assert!(
        mask[idx],
        "the minikey composite value token must be masked=true (Preview/last-run \
         redaction). argv={argv:?} mask={mask:?}"
    );
}

#[test]
fn surface_i_negative_control_xpub_composite_value_token_is_not_masked() {
    let state = from_composite("xpub", WATCH_XPUB);
    let (argv, mask) = assemble_argv_with_secret_mask(
        &schema::mnemonic::SCHEMA,
        convert_subcommand(),
        &state,
    );
    let token = format!("xpub={WATCH_XPUB}");
    let idx = argv
        .iter()
        .position(|a| *a == token)
        .unwrap_or_else(|| panic!("xpub value token not in argv: {argv:?}"));
    assert!(
        !mask[idx],
        "a watch-only xpub composite value token must NOT be masked. \
         argv={argv:?} mask={mask:?}"
    );
}

// ── surface (ii) — run-confirm ───────────────────────────────────────────────

#[test]
fn surface_ii_minikey_composite_triggers_run_confirm() {
    let state = from_composite("minikey", MINIKEY);
    assert!(
        should_confirm_run(convert_subcommand(), &state),
        "a non-empty --from minikey=… must fire the run-confirm modal"
    );
}

#[test]
fn surface_ii_negative_control_xpub_composite_does_not_confirm() {
    let state = from_composite("xpub", WATCH_XPUB);
    assert!(
        !should_confirm_run(convert_subcommand(), &state),
        "a watch-only --from xpub=… must NOT fire the run-confirm modal"
    );
}

// ── surface (iv) — persist-redact (in-memory + on-disk bytes) ─────────────────

fn persisted(form: FormState) -> PersistedState {
    let mut map = BTreeMap::new();
    map.insert("mnemonic:convert".into(), form);
    PersistedState {
        schema_version: SCHEMA_VERSION,
        last_cli_tab: "mnemonic".into(),
        last_subcommand_per_tab: BTreeMap::new(),
        window_size: None,
        window_position: None,
        show_cmdline: true,
        show_stdout: true,
        show_stderr: true,
        form_state_per_subcommand: map,
    }
}

#[test]
fn surface_iv_minikey_composite_dropped_from_redacted_form() {
    let redacted = redact_for_persistence(&from_composite("minikey", MINIKEY));
    assert!(
        !redacted.values.iter().any(|(k, _)| k == "--from"),
        "redact_for_persistence must DROP the --from minikey=… composite \
         (minikey is a private key). redacted values: {:?}",
        redacted.values
    );
}

#[test]
fn surface_iv_minikey_value_absent_from_on_disk_state_json() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    save(&persisted(from_composite("minikey", MINIKEY)), &path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(
        !on_disk.contains(MINIKEY),
        "the minikey value leaked PLAINTEXT to state.json:\n{on_disk}"
    );
    assert!(
        !on_disk.contains("\"minikey\""),
        "the minikey node token leaked to state.json via NodeValueComposite:\n{on_disk}"
    );
}

#[test]
fn surface_iv_negative_control_xpub_composite_is_persisted() {
    let redacted = redact_for_persistence(&from_composite("xpub", WATCH_XPUB));
    assert!(
        redacted.values.iter().any(|(k, v)| k == "--from"
            && matches!(v, FlagValue::NodeValueComposite { node, .. } if node == "xpub")),
        "a watch-only --from xpub=… composite must SURVIVE persistence. \
         redacted values: {:?}",
        redacted.values
    );

    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("state.json");
    save(&persisted(from_composite("xpub", WATCH_XPUB)), &path).unwrap();
    let on_disk = std::fs::read_to_string(&path).unwrap();
    assert!(
        on_disk.contains(WATCH_XPUB),
        "the watch-only xpub value must be persisted to state.json:\n{on_disk}"
    );
}
