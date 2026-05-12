//! Subprocess-runner integration tests (SPEC §B.7 + IMPL_PLAN §C Phase 4).
//!
//! Three cells:
//!   1. `cell_1_mnemonic_export_wallet_byte_exact` — assemble argv via the
//!      GUI's `assemble_argv`, spawn via `runner::run`, assert stdout byte-
//!      identical to upstream's pinned Coldcard BIP-84 fixture.
//!   2. `cell_2_tracing_init_logs_subprocess_spawn` — install a capturing
//!      `tracing_subscriber`, call `runner::run`, assert a "subprocess
//!      spawn" debug event fired.
//!   3. `cell_3_runner_deadlock_safe_on_large_stdout` — spawn a child
//!      emitting >1MB of stdout (yes/dd-equivalent via a script-free
//!      Rust subprocess) and verify the runner does not hang.
//!
//! Binary lookup honors `MNEMONIC_BIN` env-var (set by Phase 1's
//! `tests/schema_mirror.rs`); fixture lookup honors
//! `MNEMONIC_GUI_UPSTREAM_ROOT` (set by Phase 9's CI workflow per SPEC
//! §B.11 resolution chain).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::runner;
use mnemonic_gui::schema::{self, FlagValue, FormState};

/// Upstream test constant from
/// `cli_export_wallet_coldcard.rs::TREZOR_24_BIP84_MAINNET_ZPUB`.
const TREZOR_24_BIP84_ZPUB: &str =
    "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";
const TREZOR_24_MASTER_FP: &str = "5436d724";

fn mnemonic_bin() -> String {
    std::env::var("MNEMONIC_BIN").unwrap_or_else(|_| "mnemonic".to_string())
}

fn upstream_root() -> PathBuf {
    if let Ok(p) = std::env::var("MNEMONIC_GUI_UPSTREAM_ROOT") {
        return PathBuf::from(p);
    }
    PathBuf::from("/scratch/code/shibboleth/mnemonic-toolkit")
}

fn coldcard_fixture_path() -> PathBuf {
    upstream_root()
        .join("crates/mnemonic-toolkit/tests/export_wallet")
        .join("coldcard_generic_bip84_mainnet.json")
}

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

#[test]
fn cell_1_mnemonic_export_wallet_byte_exact() {
    let fixture = coldcard_fixture_path();
    let expected = std::fs::read_to_string(&fixture).unwrap_or_else(|e| {
        panic!(
            "could not read fixture {:?}: {} \n(set MNEMONIC_GUI_UPSTREAM_ROOT to a local checkout \
             of mnemonic-toolkit at tag mnemonic-toolkit-v0.8.1)",
            fixture, e
        )
    });

    // Build the form state that produces the exact argv the upstream
    // fixture is pinned against (see cli_export_wallet_coldcard.rs cell_1).
    let slots = SlotState {
        rows: vec![
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Xpub,
                value: TREZOR_24_BIP84_ZPUB.into(),
            },
            SlotRow {
                index: 0,
                subkey: SlotSubkey::Fingerprint,
                value: TREZOR_24_MASTER_FP.into(),
            },
        ],
    };
    let state = FormState::from_pairs(vec![
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--format", FlagValue::Dropdown("coldcard".into())),
        ("--output", FlagValue::Path("-".into())),
    ])
    .with_slots(slots);

    let mut argv = assemble_argv(
        &schema::mnemonic::SCHEMA,
        subcommand("export-wallet"),
        &state,
    );
    // Replace argv[0] with the resolved binary so the test runs against the
    // configured installation rather than relying on $PATH lookup.
    argv[0] = mnemonic_bin();

    let result = runner::run(argv).expect("runner::run should not error on a known-good binary");
    assert_eq!(
        result.exit_code,
        Some(0),
        "non-zero exit; stderr:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );

    let stdout = String::from_utf8(result.stdout).expect("stdout must be UTF-8");
    assert_eq!(
        stdout, expected,
        "Coldcard BIP-84 mainnet emission must match fixture byte-exact"
    );
}

/// Custom `tracing_subscriber` writer that captures bytes into a shared
/// `Vec<u8>`. Used to assert the runner emits a "subprocess spawn" event.
#[derive(Clone)]
struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for CapturedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CapturedWriter {
    type Writer = CapturedWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn cell_2_tracing_init_logs_subprocess_spawn() {
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let writer = CapturedWriter(buf.clone());

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(writer)
        .with_ansi(false)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    // Spawn anything — even a bare `--version` proves the runner emits
    // the "subprocess spawn" debug event. We use the actual mnemonic
    // binary so the event payload is realistic.
    let _ = runner::run([mnemonic_bin(), "--version".into()])
        .expect("subprocess spawn should succeed");

    let captured = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert!(
        captured.contains("subprocess spawn"),
        "expected 'subprocess spawn' DEBUG event in captured tracing output:\n{}",
        captured
    );
    assert!(
        captured.contains("subprocess exit 0"),
        "expected 'subprocess exit 0' DEBUG event:\n{}",
        captured
    );
}

#[test]
fn cell_3_runner_deadlock_safe_on_large_stdout() {
    // Use the system `yes` (Unix) or a Rust-spawned `cargo run`-style
    // process to produce >1MB of stdout. Simpler: use `head -c 2097152`
    // on /dev/urandom, falling back to a Rust-side spawn of the `yes`
    // binary capped via `head -c`. The deadlock failure mode is: stdlib
    // wait_with_output drains in parallel; if it did NOT, the child would
    // block on a full pipe.
    if !std::path::Path::new("/usr/bin/yes").exists() {
        eprintln!("cell_3: /usr/bin/yes not available; skipping");
        return;
    }
    // `yes | head -c N` would require a pipeline; we approximate by
    // spawning `head -c` with `/dev/urandom` as input.
    if !std::path::Path::new("/usr/bin/head").exists() {
        eprintln!("cell_3: /usr/bin/head not available; skipping");
        return;
    }
    let result = runner::run([
        "/usr/bin/head".to_string(),
        "-c".to_string(),
        "2097152".to_string(),
        "/dev/urandom".to_string(),
    ])
    .expect("head -c 2MiB should not error");
    assert_eq!(result.exit_code, Some(0));
    assert_eq!(
        result.stdout.len(),
        2 * 1024 * 1024,
        "deadlock-safety: full 2 MiB stdout captured"
    );
}
