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
    /// v0.39.0 — display secret-mask parallel to `argv` (`mask[i] == true` iff
    /// `argv[i]` is a secret value token). The runner layer is mask-oblivious:
    /// `run_with_stdin` initialises this `Vec::new()` and the GUI's
    /// `spawn_and_capture` overwrites it with the mask computed at assembly
    /// time before storing the result. Used only to mask the last-run `argv:`
    /// display; never affects what is spawned.
    pub mask: Vec<bool>,
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
/// # `--no-auto-repair` propagation (v0.10.0 B.3 / D33)
///
/// Pre-v0.10.0 the GUI carried an action-bar `--no-auto-repair` checkbox +
/// `prepend_no_auto_repair` helper as the load-bearing R7 fallback for
/// the v4 schema's missing global-flag emission. Toolkit v5 schema now
/// emits `--no-auto-repair` as a per-subcommand flag with `global: true`;
/// the GUI mirrors this via standard FlagSchema entries in each
/// subcommand. Users see the flag in the standard form widget per
/// subcommand and toggle it like any other Boolean. The MNEMONIC_FORCE_TTY=1
/// env-var still forces auto-fire ON by default for the GUI's piped-stdout
/// invocations.
pub fn run<I, S>(argv: I) -> io::Result<RunResult>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    // v0.32.0 (node-tree builder SPEC §2.1): `run` delegates with
    // `stdin: None` — byte-identical behavior (Stdio::null stdin,
    // MNEMONIC_FORCE_TTY spawn env, wait_with_output drain).
    run_with_stdin(argv, None)
}

/// v0.32.0 (node-tree builder SPEC §2.1 / §0 stdin discipline) — spawn
/// `argv[0]` with optional bytes piped to the child's stdin.
///
/// - `stdin: None` → `Stdio::null()` (the pre-v0.32.0 `run` behavior,
///   byte-identical; [`run`] delegates here).
/// - `stdin: Some(bytes)` → `Stdio::piped()`; the discipline is
///   `write_all` → explicitly DROP the `ChildStdin` (the toolkit reads to
///   EOF — an undropped handle deadlocks every run) → `wait_with_output`.
/// - A failed write (the child exited pre-EOF, e.g. a clap error →
///   `BrokenPipe`) DEGRADES to collect-output: never an error return for
///   that case — the child's stdout/stderr are still drained and surfaced.
/// - Unthreaded-writer license (SPEC §0): specs are ≤ ~2 KB ≪ the
///   64 KiB/4 KiB pipe buffers, so a synchronous `write_all` before the
///   output drain cannot deadlock on a child that reads to EOF.
pub fn run_with_stdin<I, S>(argv: I, stdin: Option<Vec<u8>>) -> io::Result<RunResult>
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

    // cycle-3 H2: NEVER Debug-format the cleartext argv — secret tokens
    // (BIP-39 phrase / entropy / WIF / minikey) are assembled INTO argv and
    // `--debug`/`RUST_LOG` would print them to stderr. Log only non-secret
    // shape fields. `argv[0]` is the resolved binary path/name (never secret).
    debug!(
        target: "mnemonic_gui::runner",
        program = %argv[0],
        argv_len = argv.len(),
        stdin = stdin.is_some(),
        "subprocess spawn",
    );

    let mut child = Command::new(OsStr::new(&argv[0]))
        // mnemonic-gui v0.9.0 D23: see module-level doc above run().
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(&argv[1..])
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(bytes) = stdin {
        // SPEC §2.1: write_all → drop ChildStdin → wait_with_output. The
        // explicit scope-drop closes the pipe so the child sees EOF.
        let mut handle = child.stdin.take().expect("stdin was requested piped");
        if let Err(e) = std::io::Write::write_all(&mut handle, &bytes) {
            // BrokenPipe (child exited pre-EOF, e.g. a clap error) degrades
            // to collect-output — NEVER an error return for that case. Any
            // other write-error kind is equally non-fatal to output
            // collection; log + degrade the same way.
            warn!(
                target: "mnemonic_gui::runner",
                error = %e,
                "stdin write failed; degrading to collect-output"
            );
        }
        drop(handle); // EOF for the child.
    }

    let output = child.wait_with_output()?;

    let exit_code = output.status.code();
    let result = RunResult {
        argv,
        // v0.39.0: runner stays mask-oblivious; the GUI caller
        // (`spawn_and_capture`) overwrites this with the assembly-time mask.
        mask: Vec::new(),
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

    // v0.10.0 B.3 (D33): the `prepend_no_auto_repair_*` cells covering the
    // pre-v0.10.0 R7 fallback helper have been DELETED in lockstep with
    // the helper's removal. `--no-auto-repair` is now a first-class
    // FlagSchema entry per subcommand (toolkit v5 `global: true`); the
    // standard argv-assembler path carries it. The `MNEMONIC_FORCE_TTY=1`
    // spawn-env integration cell below is preserved (D23 contract).

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

    // ── v0.32.0 run_with_stdin cells (SPEC §2.1) ─────────────────────────

    fn have(path: &str) -> bool {
        if std::path::Path::new(path).exists() {
            true
        } else {
            eprintln!("{path} not available; skipping run_with_stdin cell");
            false
        }
    }

    /// A child that reads stdin to EOF gets the bytes (cat echoes them).
    #[test]
    fn run_with_stdin_child_reading_to_eof_gets_the_bytes() {
        if !have("/bin/cat") {
            return;
        }
        let bytes = b"node-tree spec bytes \xf0\x9f\x8c\xb3".to_vec();
        let result = run_with_stdin(["/bin/cat".to_string()], Some(bytes.clone()))
            .expect("cat should spawn cleanly");
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.stdout, bytes, "cat must echo the piped stdin verbatim");
    }

    /// An immediately-exiting child does not error the parent — the
    /// BrokenPipe on the write degrades to collect-output (SPEC §2.1).
    /// 1 MiB ≫ the pipe buffer so the write deterministically hits the
    /// closed pipe.
    #[test]
    fn run_with_stdin_immediately_exiting_child_degrades_not_errors() {
        if !have("/bin/true") {
            return;
        }
        let big = vec![b'x'; 1024 * 1024];
        let result = run_with_stdin(["/bin/true".to_string()], Some(big))
            .expect("BrokenPipe must degrade to collect-output, never Err");
        assert_eq!(result.exit_code, Some(0));
    }

    /// Output is still collected when the child exits non-zero before
    /// consuming stdin (the clap-error shape).
    #[test]
    fn run_with_stdin_output_collected_on_early_nonzero_exit() {
        if !have("/bin/sh") {
            return;
        }
        let result = run_with_stdin(
            [
                "/bin/sh".to_string(),
                "-c".to_string(),
                "echo out; echo err >&2; exit 3".to_string(),
            ],
            Some(vec![b'y'; 256 * 1024]),
        )
        .expect("early exit must not error the parent");
        assert_eq!(result.exit_code, Some(3));
        assert_eq!(result.stdout, b"out\n");
        assert_eq!(result.stderr, b"err\n");
    }

    /// `run_with_stdin(.., None)` keeps the D23 spawn env (run() delegates
    /// here, so this also pins the delegation path's env behavior).
    #[test]
    fn run_with_stdin_none_keeps_force_tty_env() {
        if !have("/usr/bin/env") {
            return;
        }
        let result = run_with_stdin(["/usr/bin/env".to_string()], None)
            .expect("env should spawn cleanly");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(
            stdout.lines().any(|line| line == "MNEMONIC_FORCE_TTY=1"),
            "delegation path must keep the D23 env: {stdout}"
        );
    }
}
