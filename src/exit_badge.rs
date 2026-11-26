//! What the output panel's exit line says (`SPEC_constellation_exit_codes.md`
//! rev 5, R12 and §7j).
//!
//! The CLIs the GUI spawns exit with the constellation's codes: 0 done, 64 bad
//! command line, 101 internal error, 130 stopped by the operator, and
//! otherwise a sum (1 to 63) of six bits. The badge decodes the sum bit by bit
//! and names each bit; it never keys on a whole number other than 0, 64, 101
//! and 130, and it never depends on which command ran. Anything else outside
//! 0..=125 "ended abnormally"; a child killed by a signal is shown from the
//! platform's status. The badge says nothing about whether output exists: the
//! panel shows the output itself (R12).
//!
//! **Version floor (§7j, §9).** Every old code is a valid new sum with a
//! different meaning (old 5 would read as NO + UNVERIFIED), so a status is
//! decoded only when the binary reported a version at or above the first one
//! that speaks these codes ([`CodeScheme::Bits`], decided by
//! [`crate::cli_version::exit_scheme`] before every run). Below it the number
//! is shown undecoded, with the reason.
//!
//! Egui-free so it is testable without the `gui` feature.

use crate::exit_code::{decode, Bits, Decoded, BIT_NAMES};

/// How a run's exit status may be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeScheme {
    /// The binary speaks the constellation's exit codes: decode the bits.
    Bits,
    /// Do not decode: the binary predates the codes, or its version could not
    /// be confirmed. The text says why.
    Undecoded(String),
}

/// How the exit line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    /// Default label colour: 0, 130, or a status that is not decoded.
    Plain,
    /// Amber: bit 4 without a failure bit; verify the result before trusting it.
    VerifyMe,
    /// Green: bit 16 alone; a correction was applied (the plate is damaged).
    Repaired,
    /// Red: a "no" (1), an invalid input (2), a refusal (8), an IO failure
    /// (32), a bad command line (64), an internal error (101), a signal kill or
    /// an abnormal end.
    Failed,
}

/// The exit line: its colour class and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Badge {
    pub kind: BadgeKind,
    pub text: String,
}

fn badge(kind: BadgeKind, text: String) -> Badge {
    Badge { kind, text }
}

/// The badge for a run that ended with `exit_code` (`None` when the platform
/// gave none) or was killed by `signal` (Unix), read under `scheme`.
pub fn classify(exit_code: Option<i32>, signal: Option<i32>, scheme: &CodeScheme) -> Badge {
    let Some(code) = exit_code else {
        return match signal {
            Some(n) => badge(BadgeKind::Failed, format!("killed by signal {n}")),
            None => badge(
                BadgeKind::Failed,
                "ended abnormally (no exit status)".to_string(),
            ),
        };
    };
    if let CodeScheme::Undecoded(why) = scheme {
        return if code == 0 {
            badge(BadgeKind::Plain, "exit: 0".to_string())
        } else {
            badge(
                BadgeKind::Plain,
                format!("exit: {code} (not decoded: {why})"),
            )
        };
    }
    match decode(code) {
        Decoded::Ok => badge(
            BadgeKind::Plain,
            "exit: 0 — done, every check passed".to_string(),
        ),
        Decoded::Usage => badge(
            BadgeKind::Failed,
            "exit: 64 — bad command line, nothing ran".to_string(),
        ),
        Decoded::Internal => badge(
            BadgeKind::Failed,
            "exit: 101 — internal error (a bug; report it)".to_string(),
        ),
        Decoded::Stopped => badge(
            BadgeKind::Plain,
            "exit: 130 — stopped by the operator".to_string(),
        ),
        Decoded::Abnormal => badge(
            BadgeKind::Failed,
            format!("exit: {code} — ended abnormally"),
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
    let names: Vec<&str> = set.iter().map(|(_, n)| *n).collect();
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
