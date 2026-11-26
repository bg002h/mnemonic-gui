//! What the output panel's exit line says, by command (exit-code survey §8,
//! 2026-11-26).
//!
//! Exit 4 and 5 do not mean the same thing in every CLI the GUI spawns:
//!
//! - 5 is REPAIR_APPLIED for `md repair`, `mk repair` and the toolkit's
//!   auto-repair paths, but `md descriptor --verify-against` exits 5 when the
//!   two descriptors are NOT spend-equal (a "no" verdict, not a repair).
//! - 4 is VERIFY-ME for `ms repair`, `md` partial decodes and the toolkit
//!   (whose project-wide exit 4 is its VERIFY-ME class, mismatches included,
//!   toolkit SPEC §4), but `mk verify` and `ms verify` exit 4 on a mismatch:
//!   a definite "no", not a result to confirm.
//!
//! Egui-free so it is testable without the `gui` feature.

use std::path::Path;

/// How the exit line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    /// Default label colour.
    Plain,
    /// Amber: confirm the result out-of-band before trusting it.
    VerifyMe,
    /// Green: a BCH repair was applied and the corrected card is on stdout.
    RepairApplied,
    /// Red: the command's answer is "no" (a mismatch, NOT spend-equal).
    No,
}

/// The exit line: its colour class and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Badge {
    pub kind: BadgeKind,
    pub text: String,
}

/// `argv[0]`'s file name without a `.exe` suffix, and `argv[1]`.
fn cli_and_sub(argv: &[String]) -> (&str, &str) {
    let cli = argv
        .first()
        .and_then(|a| Path::new(a).file_name())
        .and_then(|n| n.to_str())
        .map(|n| n.strip_suffix(".exe").unwrap_or(n))
        .unwrap_or("");
    let sub = argv.get(1).map(String::as_str).unwrap_or("");
    (cli, sub)
}

/// The badge for a run of `argv` that ended with `exit_code`.
pub fn classify(argv: &[String], exit_code: Option<i32>) -> Badge {
    let (cli, sub) = cli_and_sub(argv);
    let (kind, text) = match exit_code {
        None => (BadgeKind::Plain, "exit: (killed)".to_string()),
        Some(5) if cli == "md" && sub == "descriptor" => (
            BadgeKind::No,
            "exit: 5 — NOT spend-equal (the two descriptors do not spend \
             alike; the verdict and the failing half are below)"
                .to_string(),
        ),
        Some(4) if matches!(cli, "mk" | "ms") && sub == "verify" => (
            BadgeKind::No,
            "exit: 4 — MISMATCH (the card does not match what it was \
             checked against; see stdout/stderr below)"
                .to_string(),
        ),
        Some(4) => (
            BadgeKind::VerifyMe,
            "exit: 4 — VERIFY-ME (confirm the result out-of-band before \
             trusting; see stdout/stderr below)"
                .to_string(),
        ),
        Some(5) => (
            BadgeKind::RepairApplied,
            "exit: 5 — Repair Applied (BCH auto-fire succeeded; \
             corrected chunk on stdout)"
                .to_string(),
        ),
        Some(n) => (BadgeKind::Plain, format!("exit: {n}")),
    };
    Badge { kind, text }
}
