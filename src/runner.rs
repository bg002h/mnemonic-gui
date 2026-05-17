//! Subprocess runner — `wait_with_output` contract per SPEC §B.7.
//!
//! Spawns the CLI binary with the assembled argv, drains stdout + stderr
//! in parallel (stdlib `wait_with_output` does this for us so we don't
//! deadlock on >1MB outputs), and returns a `RunResult` with exit code
//! and both byte streams. OS-level spawn failures bubble as `Err`; CLI
//! non-zero exits return `Ok(RunResult)` with `exit_code: Some(n)` so the
//! GUI can render stderr to the user.

use std::ffi::OsStr;
use std::io;
use std::process::{Command, Stdio};

use tracing::{debug, warn};

/// Capture from one subprocess run.
#[derive(Debug)]
pub struct RunResult {
    /// The exact argv passed to spawn (including `argv[0]` = binary name).
    pub argv: Vec<String>,
    /// `Some(n)` for normal exit; `None` if killed by signal / no code.
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Pure-function helper: given a caller-assembled `argv` (binary name +
/// subcommand args) and the action-bar's `no_auto_repair` flag, return the
/// argv vector that should actually be passed to the OS spawn.
///
/// When `no_auto_repair == true`, `--no-auto-repair` is spliced in BETWEEN
/// `argv[0]` (the binary name) and `argv[1..]` (the subcommand + flags).
/// Toolkit's clap-derive accepts global flags either before or after the
/// subcommand, but the safer-by-convention position is between the binary
/// and the subcommand — matches how the toolkit's own `--help` output
/// renders the flag.
///
/// When `no_auto_repair == false` the argv is returned unchanged.
///
/// Empty argv (degenerate) is returned unchanged regardless of the flag,
/// so the caller's existing "empty argv" guard in `run` still fires.
///
/// This is the load-bearing R7 fallback for the schema-mirror gap surfaced
/// in Phase A.1 (`mnemonic gui-schema` JSON does NOT emit global flags
/// per-subcommand, so we cannot wire `--no-auto-repair` via per-subcommand
/// schemas; the action-bar checkbox is the workaround until FOLLOWUP
/// `gui-schema-global-flag-emission` lands toolkit-side).
pub fn prepend_no_auto_repair(argv: Vec<String>, no_auto_repair: bool) -> Vec<String> {
    if !no_auto_repair || argv.is_empty() {
        return argv;
    }
    let mut out = Vec::with_capacity(argv.len() + 1);
    out.push(argv[0].clone());
    out.push("--no-auto-repair".to_string());
    out.extend(argv.into_iter().skip(1));
    out
}

/// Spawn `argv[0]` with `argv[1..]` as args; pipe stdout + stderr; wait
/// for exit. SPEC §B.7 invariants:
///
/// - First argv element is the binary name (no absolute path). The OS
///   resolves it via `$PATH`. Caller is responsible for `path_detect`
///   on the GUI side so we can render a friendly "binary missing"
///   message before reaching this function.
/// - Stdout + stderr drained in parallel by stdlib `wait_with_output`,
///   so subprocesses emitting >1MB of either pipe do not deadlock.
/// - Non-zero exit → `Ok(RunResult)` with the code; the GUI surfaces
///   stderr verbatim in the output pane.
/// - OS spawn failure (`Err`) → caller renders error class 1 from
///   SPEC §8 ("Binary missing from $PATH" / permission denied).
///
/// # mnemonic-gui v0.9.0 D23 — spawn-time `MNEMONIC_FORCE_TTY=1`
///
/// The toolkit's auto-repair UX (introduced in `mnemonic-toolkit-v0.22.1`)
/// gates auto-fire emission of BCH repair reports on
/// `std::io::stdout().is_terminal() && !no_auto_repair`. GUI subprocesses
/// are spawned with `stdout(Stdio::piped())` (so the GUI can capture and
/// render the bytes), which means `is_terminal()` returns `false` — without
/// an override, GUI users would NEVER see the auto-fire short-circuit
/// behavior that terminal users get for free.
///
/// Toolkit v0.22.1 exposes `MNEMONIC_FORCE_TTY=1` as the "force-TTY-positive"
/// path: when set, the toolkit treats stdout as a terminal for auto-fire
/// gating purposes regardless of `is_terminal()`. This env-var is currently
/// classified as test-only in toolkit's `verify_bundle::run` doc-comment
/// (introduced for `cli_auto_repair.rs` test harnessing); GUI consumption
/// in production is a deliberate trade-off accepted per D23 user-lock.
///
/// FOLLOWUP `toolkit-mnemonic-force-tty-promote-from-test-only` (filed at
/// Phase A.4) tracks promoting this env-var to a first-class public
/// contract in a future toolkit minor release, with a companion mirror
/// entry GUI-side per CLAUDE.md mirror invariant.
///
/// # R7 action-bar opt-out
///
/// The GUI's top-level action bar carries a `--no-auto-repair` checkbox.
/// When the user checks it, `MnemonicGuiApp` calls `prepend_no_auto_repair`
/// before invoking `run`, so the resulting argv carries `--no-auto-repair`
/// between the binary name and the subcommand. The MNEMONIC_FORCE_TTY=1
/// override forces auto-fire ON by default; the checkbox forces it OFF
/// per-invocation. Both knobs are GUI-internal — they do not appear in
/// the toolkit's per-subcommand schemas (see Phase A.1 R7 finding +
/// FOLLOWUP `gui-schema-global-flag-emission`).
pub fn run<I, S>(argv: I) -> io::Result<RunResult>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let argv: Vec<String> = argv.into_iter().map(Into::into).collect();
    if argv.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "runner::run: argv must contain at least the binary name",
        ));
    }

    debug!(target: "mnemonic_gui::runner", argv = ?argv, "subprocess spawn");

    let output = Command::new(OsStr::new(&argv[0]))
        // mnemonic-gui v0.9.0 D23: see module-level doc above run().
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let exit_code = output.status.code();
    let result = RunResult {
        argv,
        exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    };

    match exit_code {
        Some(0) => debug!(target: "mnemonic_gui::runner", "subprocess exit 0"),
        Some(n) => warn!(target: "mnemonic_gui::runner", exit_code = n, "subprocess non-zero exit"),
        None => warn!(target: "mnemonic_gui::runner", "subprocess killed by signal or no exit code"),
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── prepend_no_auto_repair unit cells ────────────────────────────────

    #[test]
    fn prepend_no_auto_repair_off_is_identity() {
        let argv = vec![
            "mnemonic".to_string(),
            "convert".to_string(),
            "--ms1".to_string(),
            "ms1bad".to_string(),
        ];
        let out = prepend_no_auto_repair(argv.clone(), false);
        assert_eq!(out, argv, "no_auto_repair=false must leave argv unchanged");
    }

    #[test]
    fn prepend_no_auto_repair_on_splices_global_flag_after_binary() {
        let argv = vec![
            "mnemonic".to_string(),
            "convert".to_string(),
            "--ms1".to_string(),
            "ms1bad".to_string(),
        ];
        let out = prepend_no_auto_repair(argv, true);
        assert_eq!(
            out,
            vec![
                "mnemonic".to_string(),
                "--no-auto-repair".to_string(),
                "convert".to_string(),
                "--ms1".to_string(),
                "ms1bad".to_string(),
            ],
            "no_auto_repair=true must splice --no-auto-repair between argv[0] and argv[1..]"
        );
    }

    #[test]
    fn prepend_no_auto_repair_empty_argv_is_unchanged() {
        let out = prepend_no_auto_repair(vec![], true);
        assert!(
            out.is_empty(),
            "empty argv is preserved even when no_auto_repair=true \
             (the run() empty-argv guard still fires)"
        );
    }

    // ── env-var injection integration cell ───────────────────────────────

    /// D23 spawn-time env-var: spawn `/usr/bin/env` and assert the spawned
    /// child sees `MNEMONIC_FORCE_TTY=1` in its environment. This is the
    /// integration regression guard for D23 — if a future cycle removes the
    /// `.env("MNEMONIC_FORCE_TTY", "1")` call in `run()`, this cell fails.
    #[test]
    fn d23_run_injects_mnemonic_force_tty_into_subprocess_env() {
        if !std::path::Path::new("/usr/bin/env").exists() {
            eprintln!(
                "d23_run_injects_mnemonic_force_tty_into_subprocess_env: \
                 /usr/bin/env not available; skipping"
            );
            return;
        }
        let result =
            run(["/usr/bin/env".to_string()]).expect("env should spawn cleanly");
        assert_eq!(result.exit_code, Some(0));
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(
            stdout.lines().any(|line| line == "MNEMONIC_FORCE_TTY=1"),
            "D23 regression: MNEMONIC_FORCE_TTY=1 must appear in spawned \
             subprocess environment. stdout was:\n{stdout}"
        );
    }
}
