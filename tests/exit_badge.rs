//! The output panel's exit badge depends on which command ran (exit-code
//! survey §8, 2026-11-26). Exit 4 and 5 mean VERIFY-ME / Repair Applied only
//! where the tool says so; `md descriptor --verify-against` exits 5 for NOT
//! spend-equal, and `mk verify` / `ms verify` exit 4 for a mismatch.

use mnemonic_gui::exit_badge::{classify, BadgeKind};

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| s.to_string()).collect()
}

#[test]
fn md_descriptor_verify_against_5_is_not_spend_equal_not_repair() {
    let a = argv(&["md", "descriptor", "md1xyz", "--verify-against", "md1abc"]);
    let b = classify(&a, Some(5));
    assert_eq!(b.kind, BadgeKind::No);
    assert!(b.text.contains("NOT spend-equal"), "{}", b.text);
    assert!(!b.text.to_lowercase().contains("repair"), "{}", b.text);
}

#[test]
fn md_descriptor_by_path_is_recognised() {
    let a = argv(&["/usr/local/bin/md", "descriptor", "--verify-against", "x"]);
    assert_eq!(classify(&a, Some(5)).kind, BadgeKind::No);
}

#[test]
fn repair_5_stays_green_repair_applied() {
    for a in [
        argv(&["md", "repair", "md1..."]),
        argv(&["mk", "repair", "mk1..."]),
        argv(&["mnemonic", "repair", "--md1", "md1..."]),
        argv(&["mnemonic", "verify-bundle", "--md1", "x"]),
    ] {
        let b = classify(&a, Some(5));
        assert_eq!(b.kind, BadgeKind::RepairApplied, "{a:?}");
        assert!(b.text.contains("Repair Applied"), "{}", b.text);
    }
}

#[test]
fn mk_and_ms_verify_4_is_mismatch_not_verify_me() {
    for a in [
        argv(&["mk", "verify", "mk1..."]),
        argv(&["ms", "verify", "--in", "card.ms1", "--phrase", "-"]),
    ] {
        let b = classify(&a, Some(4));
        assert_eq!(b.kind, BadgeKind::No, "{a:?}");
        assert!(b.text.contains("MISMATCH"), "{}", b.text);
        assert!(!b.text.contains("VERIFY-ME"), "{}", b.text);
    }
}

#[test]
fn verify_me_4_stays_amber_where_the_tool_means_it() {
    for a in [
        argv(&["ms", "repair", "ms1..."]),
        argv(&["md", "decode", "md1..."]),
        argv(&["mnemonic", "repair", "--ms1", "ms1..."]),
        // The toolkit's project-wide exit 4 is VERIFY-ME (its SPEC §4),
        // mismatches included.
        argv(&["mnemonic", "verify-bundle", "--md1", "x"]),
    ] {
        let b = classify(&a, Some(4));
        assert_eq!(b.kind, BadgeKind::VerifyMe, "{a:?}");
        assert!(b.text.contains("VERIFY-ME"), "{}", b.text);
    }
}

#[test]
fn other_codes_are_plain() {
    let a = argv(&["md", "descriptor", "--verify-against", "x"]);
    assert_eq!(classify(&a, Some(0)).kind, BadgeKind::Plain);
    assert_eq!(classify(&a, Some(0)).text, "exit: 0");
    assert_eq!(classify(&a, Some(1)).text, "exit: 1");
    assert_eq!(classify(&a, None).text, "exit: (killed)");
    assert_eq!(classify(&[], Some(2)).text, "exit: 2");
}
