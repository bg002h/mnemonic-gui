//! The output panel's exit badge (`SPEC_constellation_exit_codes.md` rev 5,
//! R12, §7j; review 2026-11-26).
//!
//! - A CLI that speaks the constellation's codes (its root `--help` ends with
//!   the R11 legend, `CodeScheme::Bits`): the badge names each bit set in
//!   1..=63, keys on a whole number only for 0, 64, 101 and 130, and reports
//!   anything else as ended abnormally. It does not depend on the command.
//! - A CLI that does not (`CodeScheme::Legacy`, every pin of 2026-11-26): the
//!   per-command meanings of the old codes (`a5fa42d`): `md descriptor
//!   --verify-against` 5 = NOT spend-equal, `mk`/`ms verify` 4 = MISMATCH,
//!   other 4 VERIFY-ME, other 5 Repair Applied.
//! - A CLI whose scheme could not be learned: the number, undecoded.
//! - A signal kill: from the platform's status, in every scheme.

use mnemonic_gui::exit_badge::{classify, BadgeKind, CodeScheme};

const BITS: CodeScheme = CodeScheme::Bits;
const LEGACY: CodeScheme = CodeScheme::Legacy;

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| s.to_string()).collect()
}

fn any() -> Vec<String> {
    argv(&["md", "decode", "md1..."])
}

fn bits(code: i32) -> mnemonic_gui::exit_badge::Badge {
    classify(&any(), Some(code), None, &BITS)
}

const NAMES: [(i32, &str); 6] = [
    (1, "a check said no"),
    (2, "an input is invalid"),
    (4, "verify the result before trusting it"),
    (8, "refused"),
    (16, "a correction was applied"),
    (32, "a file, device or helper failed"),
];

// ─── the constellation's codes ──────────────────────────────────────────────

#[test]
fn zero_is_done() {
    let b = bits(0);
    assert_eq!(b.kind, BadgeKind::Plain);
    assert_eq!(b.text, "exit: 0 — done, every check passed");
}

#[test]
fn every_sum_names_exactly_its_bits() {
    for code in 1..=63 {
        let b = bits(code);
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
    assert!(
        bits(20).text.starts_with("exit: 20 = 4 + 16 — "),
        "{}",
        bits(20).text
    );
    assert!(
        bits(16).text.starts_with("exit: 16 — "),
        "{}",
        bits(16).text
    );
}

#[test]
fn colour_follows_the_bits_not_the_command() {
    for code in [1, 2, 8, 32, 18, 22, 3, 24, 52] {
        assert_eq!(bits(code).kind, BadgeKind::Failed, "{code}");
    }
    assert_eq!(bits(4).kind, BadgeKind::VerifyMe);
    assert_eq!(bits(20).kind, BadgeKind::VerifyMe);
    assert_eq!(bits(16).kind, BadgeKind::Repaired);
    // The same code means the same thing whatever ran.
    for a in [
        argv(&["md", "descriptor", "--verify-against", "x"]),
        argv(&["mk", "verify", "mk1..."]),
        argv(&["mnemonic", "repair", "--md1", "x"]),
    ] {
        for code in [1, 4, 5, 16, 20] {
            assert_eq!(
                classify(&a, Some(code), None, &BITS),
                bits(code),
                "{a:?} {code}"
            );
        }
    }
}

#[test]
fn sixteen_alone_says_to_re_engrave() {
    // Review M3: green, but the §3 action is in the text.
    let b = bits(16);
    assert_eq!(b.kind, BadgeKind::Repaired);
    assert!(
        b.text.contains("re-engrave the damaged plate"),
        "{}",
        b.text
    );
}

#[test]
fn md_not_spend_equal_is_a_no_not_a_repair() {
    let b = classify(
        &argv(&["md", "descriptor", "--verify-against", "x"]),
        Some(1),
        None,
        &BITS,
    );
    assert_eq!(b.kind, BadgeKind::Failed);
    assert!(!b.text.to_lowercase().contains("repair"), "{}", b.text);
}

#[test]
fn the_badge_never_claims_output_exists() {
    for scheme in [BITS, LEGACY] {
        for code in 0..=255 {
            let t = classify(&any(), Some(code), None, &scheme).text;
            assert!(!t.contains("stdout"), "{code}: {t}");
        }
    }
}

#[test]
fn whole_numbers_64_101_130() {
    let u = bits(64);
    assert_eq!(u.kind, BadgeKind::Failed);
    assert_eq!(u.text, "exit: 64 — bad command line, nothing ran");
    let i = bits(101);
    assert_eq!(i.kind, BadgeKind::Failed);
    assert_eq!(i.text, "exit: 101 — internal error (a bug; report it)");
    // Review N1: a stop is its own kind, not the colour of success.
    let s = bits(130);
    assert_eq!(s.kind, BadgeKind::Stopped);
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
        let b = bits(code);
        assert_eq!(b.kind, BadgeKind::Failed, "{code}");
        // Review N2: §3 calls any such code a bug.
        assert_eq!(
            b.text,
            format!("exit: {code} — ended abnormally (a bug: please report)")
        );
    }
}

#[test]
fn a_signal_kill_is_shown_from_the_platform_status() {
    for scheme in [BITS, LEGACY, CodeScheme::Undecoded("x".into())] {
        let b = classify(&any(), None, Some(9), &scheme);
        assert_eq!(b.kind, BadgeKind::Failed);
        assert_eq!(b.text, "killed by signal 9");
    }
    assert_eq!(
        classify(&any(), None, None, &BITS).text,
        "ended abnormally (no exit status)"
    );
}

// ─── the old codes (a5fa42d), for a CLI without the legend ────────────────

#[test]
fn legacy_md_descriptor_5_is_not_spend_equal_not_repair() {
    for a in [
        argv(&["md", "descriptor", "md1xyz", "--verify-against", "md1abc"]),
        argv(&["/usr/local/bin/md", "descriptor", "--verify-against", "x"]),
    ] {
        let b = classify(&a, Some(5), None, &LEGACY);
        assert_eq!(b.kind, BadgeKind::Failed, "{a:?}");
        assert!(b.text.contains("NOT spend-equal"), "{}", b.text);
        assert!(!b.text.to_lowercase().contains("repair"), "{}", b.text);
    }
}

#[test]
fn legacy_repair_5_is_green_repair_applied() {
    for a in [
        argv(&["md", "repair", "md1..."]),
        argv(&["mk", "repair", "mk1..."]),
        argv(&["mnemonic", "repair", "--md1", "md1..."]),
        argv(&["mnemonic", "verify-bundle", "--md1", "x"]),
    ] {
        let b = classify(&a, Some(5), None, &LEGACY);
        assert_eq!(b.kind, BadgeKind::Repaired, "{a:?}");
        assert!(b.text.contains("Repair Applied"), "{}", b.text);
    }
}

#[test]
fn legacy_mk_and_ms_verify_4_is_mismatch() {
    for a in [
        argv(&["mk", "verify", "mk1..."]),
        argv(&["ms", "verify", "--in", "card.ms1", "--phrase", "-"]),
    ] {
        let b = classify(&a, Some(4), None, &LEGACY);
        assert_eq!(b.kind, BadgeKind::Failed, "{a:?}");
        assert!(b.text.contains("MISMATCH"), "{}", b.text);
        assert!(!b.text.contains("VERIFY-ME"), "{}", b.text);
    }
}

#[test]
fn legacy_other_4_is_amber_verify_me() {
    for a in [
        argv(&["ms", "repair", "ms1..."]),
        argv(&["md", "decode", "md1..."]),
        argv(&["mnemonic", "repair", "--ms1", "ms1..."]),
        argv(&["mnemonic", "verify-bundle", "--md1", "x"]),
    ] {
        let b = classify(&a, Some(4), None, &LEGACY);
        assert_eq!(b.kind, BadgeKind::VerifyMe, "{a:?}");
        assert!(b.text.contains("VERIFY-ME"), "{}", b.text);
    }
}

#[test]
fn legacy_other_codes_are_plain_numbers() {
    let a = argv(&["md", "descriptor", "--verify-against", "x"]);
    assert_eq!(classify(&a, Some(0), None, &LEGACY).text, "exit: 0");
    assert_eq!(classify(&a, Some(1), None, &LEGACY).text, "exit: 1");
    assert_eq!(classify(&a, Some(2), None, &LEGACY).kind, BadgeKind::Plain);
    assert_eq!(classify(&[], Some(64), None, &LEGACY).text, "exit: 64");
}

// ─── a CLI whose scheme is unknown ──────────────────────────────────────────

#[test]
fn an_unknown_scheme_is_not_decoded() {
    let why = CodeScheme::Undecoded("`md --help` did not finish within 5s".into());
    for code in [1, 4, 5, 16, 20, 64, 101] {
        let b = classify(&any(), Some(code), None, &why);
        assert_eq!(b.kind, BadgeKind::Plain, "{code}");
        assert_eq!(
            b.text,
            format!("exit: {code} (not decoded: `md --help` did not finish within 5s)")
        );
    }
    assert_eq!(classify(&any(), Some(0), None, &why).text, "exit: 0");
}
