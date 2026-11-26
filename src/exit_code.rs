//! The constellation's exit codes (`SPEC_constellation_exit_codes.md` §3, in
//! mnemonic-engrave's `design/`), for decoding a CLI's status (R12) and for
//! the GUI's own process status.
//!
//! **A local copy.** The constants and [`Bits`] mirror the `exit` module of
//! mnemonic-io-lib (decision 4) until that crate is published with it; then the
//! GUI takes the shared one and this part of the file goes. The numbers are
//! pinned to the spec by the tests below. [`decode`] is the GUI's own: a CLI
//! classifies, a GUI only decodes.
//!
//! Codes 1 to 63 are sums of the six bits, and each sum is unique. 64, 101 and
//! 130 are single codes and are not bits.

// A mirror of the shared module: not every constant has a user in the GUI.
#![allow(dead_code)]

use std::ops::{BitOr, BitOrAssign};

/// 0: everything requested was done and every check passed.
pub const OK: u8 = 0;
/// Bit 1: a well-formed input was compared against something and came out no.
pub const NO: u8 = 1;
/// Bit 2: an input is wrong for this verb whatever the flags.
pub const INVALID: u8 = 2;
/// Bit 4: the tool checked part of the result and could not confirm it.
pub const UNVERIFIED: u8 = 4;
/// Bit 8: refused under a safety or policy rule that a flag, network, channel
/// or different destination would waive; also a network mismatch.
pub const REFUSED: u8 = 8;
/// Bit 16: a checksum correction was applied and the corrected value used.
pub const REPAIRED: u8 = 16;
/// Bit 32: something needed from the system failed or is missing.
pub const IO: u8 = 32;
/// 64, alone: the command line alone is wrong. Nothing ran.
pub const USAGE: u8 = 64;
/// 101, alone: a bug, or any failure no class covers (Rust's panic code).
pub const INTERNAL: u8 = 101;
/// 130, alone: the operator stopped it, at a prompt or with Ctrl-C.
pub const STOPPED: u8 = 130;

/// The legend every CLI's root `--help` ends with (spec R11), verbatim.
pub const HELP_LEGEND: &str =
    "Exit status: 0 = done, every check passed. 64 = bad command line, nothing \
ran. 101 = internal error. 130 = stopped by the operator. Other codes from \
128 up = killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; \
2 an input is invalid; 4 verify the result before trusting it; 8 refused \
(network mismatch, secret on the command line, unsafe output); 16 a correction \
was applied; 32 a file, device or helper failed.";

/// A set of the six combinable bits (1, 2, 4, 8, 16, 32). Its exit code is
/// their sum, so every combination is a unique code from 0 to 63.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Bits(u8);

impl Bits {
    /// No bit set: exit 0.
    pub const OK: Bits = Bits(0);
    /// [`NO`] (1).
    pub const NO: Bits = Bits(NO);
    /// [`INVALID`] (2).
    pub const INVALID: Bits = Bits(INVALID);
    /// [`UNVERIFIED`] (4).
    pub const UNVERIFIED: Bits = Bits(UNVERIFIED);
    /// [`REFUSED`] (8).
    pub const REFUSED: Bits = Bits(REFUSED);
    /// [`REPAIRED`] (16).
    pub const REPAIRED: Bits = Bits(REPAIRED);
    /// [`IO`] (32).
    pub const IO: Bits = Bits(IO);

    /// True when every bit of `other` is set here.
    pub const fn contains(self, other: Bits) -> bool {
        self.0 & other.0 == other.0
    }

    /// True when no bit is set (exit 0).
    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }

    /// The process exit status: the sum of the bits, 0 to 63.
    pub const fn to_exit_code(self) -> u8 {
        self.0
    }
}

impl BitOr for Bits {
    type Output = Bits;
    fn bitor(self, rhs: Bits) -> Bits {
        Bits(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bits {
    fn bitor_assign(&mut self, rhs: Bits) {
        self.0 |= rhs.0;
    }
}

/// The six bits in ascending order, each with the legend's words for it (R11).
pub const BIT_NAMES: [(Bits, &str); 6] = [
    (Bits::NO, "a check said no"),
    (Bits::INVALID, "an input is invalid"),
    (Bits::UNVERIFIED, "verify the result before trusting it"),
    (
        Bits::REFUSED,
        "refused (network mismatch, secret on the command line, unsafe output)",
    ),
    (Bits::REPAIRED, "a correction was applied"),
    (Bits::IO, "a file, device or helper failed"),
];

/// What a child's exit status says, read the way R12 tells a GUI to: bit by
/// bit for 1..=63, and by whole number only for 0, 64, 101 and 130.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decoded {
    /// 0: done, every check passed.
    Ok,
    /// 1..=63: the bits that are set.
    Bits(Bits),
    /// 64: bad command line, nothing ran.
    Usage,
    /// 101: internal error.
    Internal,
    /// 130: stopped by the operator (a prompt answer or Ctrl-C).
    Stopped,
    /// Anything else, including a Windows NTSTATUS crash value: the process
    /// ended abnormally.
    Abnormal,
}

/// Decode an exit status (spec §3, R12).
pub fn decode(code: i32) -> Decoded {
    match code {
        0 => Decoded::Ok,
        1..=63 => Decoded::Bits(Bits(code as u8)),
        64 => Decoded::Usage,
        101 => Decoded::Internal,
        130 => Decoded::Stopped,
        _ => Decoded::Abnormal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The numbers are the spec's (§3); a change here is a constellation change.
    #[test]
    fn numbers_match_the_spec() {
        assert_eq!(
            [OK, NO, INVALID, UNVERIFIED, REFUSED, REPAIRED, IO, USAGE, INTERNAL, STOPPED],
            [0, 1, 2, 4, 8, 16, 32, 64, 101, 130]
        );
        let all = [NO, INVALID, UNVERIFIED, REFUSED, REPAIRED, IO];
        assert_eq!(all.iter().fold(0u8, |a, b| a | b), 63);
        assert_eq!(all.iter().map(|b| b.count_ones()).sum::<u32>(), 6);
    }

    #[test]
    fn bits_sum_and_contain() {
        let b = Bits::REPAIRED | Bits::UNVERIFIED;
        assert_eq!(b.to_exit_code(), 20);
        assert!(b.contains(Bits::REPAIRED) && b.contains(Bits::UNVERIFIED));
        assert!(!b.contains(Bits::INVALID));
        let mut c = b;
        c |= Bits::INVALID;
        assert_eq!(c.to_exit_code(), 22);
        assert!(Bits::OK.is_ok() && Bits::default().is_ok());
    }

    #[test]
    fn bit_names_cover_the_six_bits_in_order() {
        let codes: Vec<u8> = BIT_NAMES.iter().map(|(b, _)| b.to_exit_code()).collect();
        assert_eq!(codes, [1, 2, 4, 8, 16, 32]);
        // Each name is the legend's own words.
        for (_, name) in BIT_NAMES {
            assert!(HELP_LEGEND.contains(name), "{name}");
        }
    }

    #[test]
    fn legend_is_the_spec_text() {
        assert_eq!(
            HELP_LEGEND,
            "Exit status: 0 = done, every check passed. 64 = bad command line, nothing ran. \
             101 = internal error. 130 = stopped by the operator. Other codes from 128 up = \
             killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; 2 an input \
             is invalid; 4 verify the result before trusting it; 8 refused (network mismatch, \
             secret on the command line, unsafe output); 16 a correction was applied; 32 a \
             file, device or helper failed."
        );
    }

    #[test]
    fn decode_keys_on_whole_numbers_only_for_0_64_101_130() {
        assert_eq!(decode(0), Decoded::Ok);
        for c in 1..=63 {
            assert_eq!(decode(c), Decoded::Bits(Bits(c as u8)), "{c}");
        }
        assert_eq!(decode(64), Decoded::Usage);
        assert_eq!(decode(101), Decoded::Internal);
        assert_eq!(decode(130), Decoded::Stopped);
        for c in (65..=100).chain(102..=129).chain(131..=255) {
            assert_eq!(decode(c), Decoded::Abnormal, "{c}");
        }
        // Windows NTSTATUS crash values arrive as large or negative i32s.
        assert_eq!(decode(-1_073_741_819), Decoded::Abnormal); // 0xC0000005
        assert_eq!(decode(-1), Decoded::Abnormal);
        assert_eq!(decode(256), Decoded::Abnormal);
    }
}
