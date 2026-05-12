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
