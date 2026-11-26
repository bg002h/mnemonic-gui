//! The output panel's exit badge decodes the constellation's exit codes bit by
//! bit (`SPEC_constellation_exit_codes.md` rev 5, R12): it names each bit set
//! in 1..=63, keys on a whole number only for 0, 64, 101 and 130, reports
//! anything else as "ended abnormally", shows a signal kill from the platform's
//! status, and never decodes a code from a binary older than the exit-code
//! floor (§7j), whose every code is a valid new sum with another meaning.

use mnemonic_gui::exit_badge::{classify, BadgeKind, CodeScheme};

const BITS: CodeScheme = CodeScheme::Bits;

fn old() -> CodeScheme {
    CodeScheme::Undecoded("`md 0.21.0` predates the constellation exit codes".into())
}

const NAMES: [(i32, &str); 6] = [
    (1, "a check said no"),
    (2, "an input is invalid"),
    (4, "verify the result before trusting it"),
    (8, "refused"),
    (16, "a correction was applied"),
    (32, "a file, device or helper failed"),
];

#[test]
fn zero_is_done() {
    let b = classify(Some(0), None, &BITS);
    assert_eq!(b.kind, BadgeKind::Plain);
    assert_eq!(b.text, "exit: 0 — done, every check passed");
}

#[test]
fn every_sum_names_exactly_its_bits() {
    for code in 1..=63 {
        let b = classify(Some(code), None, &BITS);
        for (bit, name) in NAMES {
            assert_eq!(
                b.text.contains(name),
                code & bit != 0,
                "code {code}, bit {bit}: {}",
                b.text
            );
        }
        assert!(b.text.starts_with(&format!("exit: {code} ")), "{}", b.text);
    }
}

#[test]
fn a_sum_shows_its_parts() {
    let b = classify(Some(20), None, &BITS);
    assert!(b.text.starts_with("exit: 20 = 4 + 16 — "), "{}", b.text);
    let b = classify(Some(16), None, &BITS);
    assert!(b.text.starts_with("exit: 16 — "), "{}", b.text);
}

#[test]
fn colour_follows_the_bits_not_the_command() {
    // Any of 1, 2, 8, 32 is red: a "no", an invalid input, a refusal, IO.
    for code in [1, 2, 8, 32, 18, 22, 3, 24, 52] {
        assert_eq!(
            classify(Some(code), None, &BITS).kind,
            BadgeKind::Failed,
            "{code}"
        );
    }
    // 4 without those is amber, 16 alone green.
    assert_eq!(classify(Some(4), None, &BITS).kind, BadgeKind::VerifyMe);
    assert_eq!(classify(Some(20), None, &BITS).kind, BadgeKind::VerifyMe);
    assert_eq!(classify(Some(16), None, &BITS).kind, BadgeKind::Repaired);
}

#[test]
fn md_not_spend_equal_is_a_no_not_a_repair() {
    // `md descriptor --verify-against` NOT spend-equal is 1 under the spec.
    let b = classify(Some(1), None, &BITS);
    assert_eq!(b.kind, BadgeKind::Failed);
    assert!(!b.text.to_lowercase().contains("repair"), "{}", b.text);
}

#[test]
fn the_badge_never_claims_output_exists() {
    // R12: whether output exists is read from the output, never the code.
    for code in 0..=255 {
        let t = classify(Some(code), None, &BITS).text;
        assert!(!t.contains("stdout"), "{code}: {t}");
    }
}

#[test]
fn whole_numbers_64_101_130() {
    let u = classify(Some(64), None, &BITS);
    assert_eq!(u.kind, BadgeKind::Failed);
    assert_eq!(u.text, "exit: 64 — bad command line, nothing ran");
    let i = classify(Some(101), None, &BITS);
    assert_eq!(i.kind, BadgeKind::Failed);
    assert_eq!(i.text, "exit: 101 — internal error (a bug; report it)");
    let s = classify(Some(130), None, &BITS);
    assert_eq!(s.kind, BadgeKind::Plain);
    assert_eq!(s.text, "exit: 130 — stopped by the operator");
}

#[test]
fn anything_else_ended_abnormally() {
    for code in [
        65,
        100,
        102,
        125,
        126,
        127,
        128,
        137,
        255,
        -1,
        -1_073_741_819,
    ] {
        let b = classify(Some(code), None, &BITS);
        assert_eq!(b.kind, BadgeKind::Failed, "{code}");
        assert_eq!(b.text, format!("exit: {code} — ended abnormally"));
    }
}

#[test]
fn a_signal_kill_is_shown_from_the_platform_status() {
    let b = classify(None, Some(9), &BITS);
    assert_eq!(b.kind, BadgeKind::Failed);
    assert_eq!(b.text, "killed by signal 9");
    // The same under an undecoded scheme: the signal is the platform's.
    assert_eq!(classify(None, Some(15), &old()).text, "killed by signal 15");
    assert_eq!(
        classify(None, None, &BITS).text,
        "ended abnormally (no exit status)"
    );
}

#[test]
fn a_binary_below_the_floor_is_not_decoded() {
    // Old `md descriptor --verify-against` 5 would read as NO + UNVERIFIED,
    // old `mk verify` 4 as UNVERIFIED: neither is decoded.
    for code in [1, 2, 3, 4, 5, 64, 101] {
        let b = classify(Some(code), None, &old());
        assert_eq!(b.kind, BadgeKind::Plain, "{code}");
        assert_eq!(
            b.text,
            format!(
                "exit: {code} (not decoded: `md 0.21.0` predates the constellation exit codes)"
            )
        );
    }
    // 0 is success in every scheme.
    assert_eq!(classify(Some(0), None, &old()).text, "exit: 0");
}
