//! cycle-15 Lane G — run-holder secret-residue scrub (slugs 1 & 2).
//!
//! The app-level run holders (`last_run: RunResult`, `pending_confirm_argv:
//! PendingConfirm`) carry the secret as a cleartext token inside a larger
//! `Vec<String>` / `Vec<u8>`, OUTSIDE `FormState`, so the on-exit
//! `zeroize_form_state` sweep never reaches them. This cycle scrubs them
//! whole-holder on drop (zeroize-on-drop) + covers them in the exit sweep
//! via the pure `secrets::scrub_app_run_holders` lib seam.
//!
//! - T1 (pure logic): `RunResult::zeroize` empties argv/mask/stdout/stderr.
//! - T3 (pure logic): `PendingConfirm::zeroize` empties argv/mask/stdin.
//! - T4 (exit-sweep seam): `scrub_app_run_holders` takes both `Option`s to
//!   `None` (the taken values drop → their `Drop` scrubs).
//!
//! Seam discipline (D7): `main.rs` app-state fields are private to
//! integration tests (documented in `tests/widget_secret.rs`); each scrub is
//! a PUBLIC lib-crate seam the test asserts directly. `RunResult` +
//! `PendingConfirm` live in the public `runner` module.

use zeroize::Zeroize;

use mnemonic_gui::runner::{PendingConfirm, RunResult};
use mnemonic_gui::secrets;

fn secret_run_result() -> RunResult {
    RunResult {
        argv: vec![
            "mnemonic".into(),
            "--passphrase".into(),
            "abandon abandon abandon ... art".into(),
        ],
        mask: vec![false, false, true],
        exit_code: Some(0),
        stdout: b"secret-stdout".to_vec(),
        stderr: b"secret-stderr".to_vec(),
    }
}

fn secret_pending() -> PendingConfirm {
    PendingConfirm {
        argv: vec![
            "mnemonic".into(),
            "--passphrase".into(),
            "abandon abandon abandon ... art".into(),
        ],
        mask: vec![false, false, true],
        stdin: Some(b"secret-stdin-bytes".to_vec()),
    }
}

// ── T1 — RunResult whole-holder scrub ────────────────────────────────────────

#[test]
fn t1_run_result_zeroize_empties_all_secret_bearing_fields() {
    let mut result = secret_run_result();
    result.zeroize();
    assert!(result.argv.is_empty(), "argv must be cleared; got {:?}", result.argv);
    assert!(result.mask.is_empty(), "mask must be cleared; got {:?}", result.mask);
    assert!(result.stdout.is_empty(), "stdout must be cleared; got {:?}", result.stdout);
    assert!(result.stderr.is_empty(), "stderr must be cleared; got {:?}", result.stderr);
}

// ── T3 — PendingConfirm whole-holder scrub ───────────────────────────────────

#[test]
fn t3_pending_confirm_zeroize_empties_argv_mask_stdin() {
    let mut pending = secret_pending();
    pending.zeroize();
    assert!(pending.argv.is_empty(), "argv must be cleared; got {:?}", pending.argv);
    assert!(pending.mask.is_empty(), "mask must be cleared; got {:?}", pending.mask);
    assert!(
        pending.stdin.is_none() || pending.stdin.as_ref().is_some_and(|b| b.is_empty()),
        "stdin bytes must be cleared; got {:?}",
        pending.stdin
    );
}

// ── T4 — exit-sweep seam coverage ────────────────────────────────────────────

#[test]
fn t4_scrub_app_run_holders_takes_both_to_none() {
    let mut last_run = Some(secret_run_result());
    let mut pending = Some(secret_pending());
    secrets::scrub_app_run_holders(&mut last_run, &mut pending);
    assert!(last_run.is_none(), "last_run must be taken to None after scrub");
    assert!(pending.is_none(), "pending_confirm must be taken to None after scrub");
}

#[test]
fn t4b_scrub_app_run_holders_is_a_noop_on_empty_holders() {
    // Idempotent / no-panic on the already-empty case (the common exit path
    // where no run has happened).
    let mut last_run: Option<RunResult> = None;
    let mut pending: Option<PendingConfirm> = None;
    secrets::scrub_app_run_holders(&mut last_run, &mut pending);
    assert!(last_run.is_none() && pending.is_none());
}
