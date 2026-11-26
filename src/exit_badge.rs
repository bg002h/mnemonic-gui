//! What the output panel's exit line says (`SPEC_constellation_exit_codes.md`
//! rev 5, R12 and §7j).
//!
//! The CLIs move to one exit-code table: 0 done, 64 bad command line, 101
//! internal error, 130 stopped by the operator, and otherwise a sum (1 to 63)
//! of six bits. A CLI that speaks it is decoded bit by bit, naming each bit;
//! the badge never keys on a whole number other than 0, 64, 101 and 130, and
//! never depends on which command ran. Anything else outside 0..=125 "ended
//! abnormally". A child killed by a signal is shown from the platform's
//! status. The badge says nothing about whether output exists: the panel shows
//! the output itself (R12).
//!
//! **Which table a CLI speaks (§7j, §9).** Every old code is a valid new sum
//! with a different meaning (old 5 would read as NO + UNVERIFIED), and version
//! numbers cannot tell the two apart (`mk 0.18.0-dev` exists on both sides).
//! So the scheme is read from the binary: R11 makes every renumbered CLI's
//! root `--help` end with the legend ([`crate::cli_version::exit_scheme_cached`]).
//! - [`CodeScheme::Bits`]: the legend is there; decode the bits.
//! - [`CodeScheme::Legacy`]: it is not; the old codes, whose 4 and 5 mean
//!   different things per command (exit-code survey §8, `a5fa42d`): 5 is
//!   REPAIR_APPLIED for `md`/`mk repair` and the toolkit's auto-repair, but NOT
//!   spend-equal for `md descriptor --verify-against`; 4 is VERIFY-ME for `ms
//!   repair`, `md` partial decodes and the toolkit's project-wide class, but a
//!   MISMATCH for `mk verify` / `ms verify`.
//! - [`CodeScheme::Undecoded`]: the CLI could not be asked; the number alone.
//!
//! Egui-free so it is testable without the `gui` feature.

use std::path::Path;

use crate::exit_code::{decode, Bits, Decoded, BIT_NAMES};

/// How a run's exit status may be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeScheme {
    /// The binary speaks the constellation's exit codes: decode the bits.
    Bits,
    /// The binary answered `--help` without the legend: it speaks the old,
    /// per-tool codes.
    Legacy,
    /// The binary could not be asked which codes it speaks. The text says why.
    Undecoded(String),
}

/// How the exit line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    /// Default label colour: 0, or a status that is not decoded.
    Plain,
    /// Amber: bit 4 without a failure bit (old scheme: VERIFY-ME); verify the
    /// result before trusting it.
    VerifyMe,
    /// Green: bit 16 alone (old scheme: Repair Applied); a correction was
    /// applied and the plate is damaged.
    Repaired,
    /// Amber: 130, stopped by the operator. Not the colour of success.
    Stopped,
    /// Red: a "no" (1), an invalid input (2), a refusal (8), an IO failure
    /// (32), a bad command line (64), an internal error (101), a signal kill or
    /// an abnormal end; old scheme: NOT spend-equal, MISMATCH.
    Failed,
}

/// The exit line: its colour class and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Badge {
    pub kind: BadgeKind,
    pub text: String,
}

fn badge(kind: BadgeKind, text: impl Into<String>) -> Badge {
    Badge {
        kind,
        text: text.into(),
    }
}

/// The badge for a run of `argv` that ended with `exit_code` (`None` when the
/// platform gave none) or was killed by `signal` (Unix), read under `scheme`.
/// `argv` matters only under [`CodeScheme::Legacy`].
pub fn classify(
    argv: &[String],
    exit_code: Option<i32>,
    signal: Option<i32>,
    scheme: &CodeScheme,
) -> Badge {
    let Some(code) = exit_code else {
        return match signal {
            Some(n) => badge(BadgeKind::Failed, format!("killed by signal {n}")),
            None => badge(BadgeKind::Failed, "ended abnormally (no exit status)"),
        };
    };
    match scheme {
        CodeScheme::Bits => bits_scheme(code),
        CodeScheme::Legacy => legacy(argv, code),
        CodeScheme::Undecoded(_) if code == 0 => badge(BadgeKind::Plain, "exit: 0"),
        CodeScheme::Undecoded(why) => badge(
            BadgeKind::Plain,
            format!("exit: {code} (not decoded: {why})"),
        ),
    }
}

fn bits_scheme(code: i32) -> Badge {
    match decode(code) {
        Decoded::Ok => badge(BadgeKind::Plain, "exit: 0 — done, every check passed"),
        Decoded::Usage => badge(
            BadgeKind::Failed,
            "exit: 64 — bad command line, nothing ran",
        ),
        Decoded::Internal => badge(
            BadgeKind::Failed,
            "exit: 101 — internal error (a bug; report it)",
        ),
        Decoded::Stopped => badge(BadgeKind::Stopped, "exit: 130 — stopped by the operator"),
        Decoded::Abnormal => badge(
            BadgeKind::Failed,
            format!("exit: {code} — ended abnormally (a bug: please report)"),
        ),
        Decoded::Bits(bits) => bits_badge(code, bits),
    }
}

fn bits_badge(code: i32, bits: Bits) -> Badge {
    let set: Vec<(Bits, &str)> = BIT_NAMES
        .iter()
        .copied()
        .filter(|(b, _)| bits.contains(*b))
        .collect();
    let parts: Vec<String> = set
        .iter()
        .map(|(b, _)| b.to_exit_code().to_string())
        .collect();
    // Bit 16's §3 action goes with its name: the plate is damaged.
    let names: Vec<&str> = set
        .iter()
        .map(|(b, n)| {
            if *b == Bits::REPAIRED {
                "a correction was applied: re-engrave the damaged plate"
            } else {
                n
            }
        })
        .collect();
    let head = if parts.len() > 1 {
        format!("exit: {code} = {}", parts.join(" + "))
    } else {
        format!("exit: {code}")
    };
    let failed = [Bits::NO, Bits::INVALID, Bits::REFUSED, Bits::IO]
        .iter()
        .any(|b| bits.contains(*b));
    let kind = if failed {
        BadgeKind::Failed
    } else if bits.contains(Bits::UNVERIFIED) {
        BadgeKind::VerifyMe
    } else {
        BadgeKind::Repaired
    };
    badge(kind, format!("{head} — {}", names.join("; ")))
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

/// The old codes (`a5fa42d`), for a CLI whose `--help` has no legend.
fn legacy(argv: &[String], code: i32) -> Badge {
    let (cli, sub) = cli_and_sub(argv);
    match code {
        5 if cli == "md" && sub == "descriptor" => badge(
            BadgeKind::Failed,
            "exit: 5 — NOT spend-equal (the two descriptors do not spend \
             alike; the verdict and the failing half are below)",
        ),
        4 if matches!(cli, "mk" | "ms") && sub == "verify" => badge(
            BadgeKind::Failed,
            "exit: 4 — MISMATCH (the card does not match what it was \
             checked against; see the output below)",
        ),
        4 => badge(
            BadgeKind::VerifyMe,
            "exit: 4 — VERIFY-ME (confirm the result out-of-band before \
             trusting it; see the output below)",
        ),
        5 => badge(
            BadgeKind::Repaired,
            "exit: 5 — Repair Applied (BCH correction succeeded: \
             re-engrave the damaged plate)",
        ),
        n => badge(BadgeKind::Plain, format!("exit: {n}")),
    }
}
