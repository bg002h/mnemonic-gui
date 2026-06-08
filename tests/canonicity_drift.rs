//! v0.8.1 F2 drift gate — agreement between the GUI's regex classifier
//! (`classify_descriptor_canonicity`) and the toolkit's authoritative
//! canonical-origin table (via `mnemonic gui-schema --classify-descriptor`).
//!
//! Each fixture carries an explicit per-fixture `Expect` (Canonical /
//! NonCanonical / ParseFails) empirically captured against the CI-pinned
//! toolkit binary (`pinned-upstream.toml:22` → `mnemonic-toolkit-v0.47.3`).
//! The loop shells out per-fixture to `MNEMONIC_BIN gui-schema
//! --classify-descriptor <STR>` and asserts, for every fixture:
//!   * a `ParseFails` fixture STILL parse-fails (exit 2 → `None`); if it now
//!     parses, that's an actionable `newly_parsed` failure ("promote it");
//!   * a classify fixture STILL classifies (`regressed` catches a broad
//!     toolkit-parser regression — the case the old `classified >= len/2`
//!     floor silently tolerated);
//!   * the toolkit verdict equals the pinned `want` (`wrong_verdict` pins the
//!     absolute canonical↔non-canonical verdict, not just GUI/toolkit
//!     agreement); AND
//!   * the GUI regex verdict equals the live toolkit verdict (`disagreements`
//!     — the original drift check, keyed on the live toolkit verdict).
//!
//! Replaces the prior lenient floor (`classified >= FIXTURES.len() / 2`),
//! which let a broad toolkit-parser regression pass silently. See
//! `design/SPEC_canonicity_drift_per_fixture_table.md`.
//!
//! Skipped (returns early) when `MNEMONIC_BIN` is unset and the binary is
//! not on PATH — matches the convention at `gui_schema_conditional_drift.rs`.

use mnemonic_gui::form::conditional::{classify_descriptor_canonicity, Canonicity};
use std::path::PathBuf;
use std::process::Command;

fn mnemonic_bin() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("MNEMONIC_BIN") {
        return Some(PathBuf::from(p));
    }
    // Fallback: look up on PATH; return None if the binary isn't there
    // so the test silently skips in dev environments without an install.
    let probe = Command::new("mnemonic").arg("--version").output().ok()?;
    if probe.status.success() {
        Some(PathBuf::from("mnemonic"))
    } else {
        None
    }
}

/// Toolkit verdict via shell-out. None = parse failure (exit 2).
fn toolkit_classify(bin: &PathBuf, descriptor: &str) -> Option<Canonicity> {
    let out = Command::new(bin)
        .args(["gui-schema", "--classify-descriptor", descriptor])
        .output()
        .expect("failed to spawn mnemonic gui-schema --classify-descriptor");
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    match stdout.trim() {
        "canonical" => Some(Canonicity::Canonical),
        "non-canonical" => Some(Canonicity::NonCanonical),
        other => panic!(
            "unexpected toolkit verdict {other:?} for descriptor {descriptor:?}"
        ),
    }
}

/// Per-fixture expectation against the CI-pinned toolkit binary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expect {
    /// Toolkit returns `Some(Canonicity::Canonical)` (exit 0).
    Canonical,
    /// Toolkit returns `Some(Canonicity::NonCanonical)` (exit 0).
    NonCanonical,
    /// Toolkit refuses to parse (exit 2 → `None`). The BIP-388 `@N/**`
    /// shorthand: the toolkit's single-star lexer leaves the second `*` in
    /// place, corrupting the descriptor downstream (see toolkit
    /// `parse_descriptor.rs` single-star lexer regex).
    ParseFails,
}

impl Expect {
    /// The `Canonicity` a classify-expecting fixture must yield. `None` for
    /// `ParseFails` (which is handled before this is ever called).
    fn as_canonicity(self) -> Option<Canonicity> {
        match self {
            Expect::Canonical => Some(Canonicity::Canonical),
            Expect::NonCanonical => Some(Canonicity::NonCanonical),
            Expect::ParseFails => None,
        }
    }
}

/// Fixture set mirrors `tests/canonicity_classifier.rs`'s corpus shape, each
/// with its explicit toolkit expectation. Empty descriptor is excluded — the
/// GUI classifies empty as NonCanonical (banner-gating convenience); the
/// toolkit rejects empty with exit 2 (no `@N` placeholder). This is a
/// documented behavioral divergence at the empty-string edge — covered
/// separately at
/// `descriptor_non_canonical_default_path_notice.rs::empty_descriptor_returns_none`
/// which pins the banner-suppression invariant. Do NOT add an empty-string
/// fixture here.
///
/// NOTE: the `@N/**` rows are `ParseFails`, NOT Canonical — the GUI regex
/// *does* class them canonical, but this table records the *toolkit*
/// expectation. Group comments below deliberately omit "Canonical" so they
/// can't be read as labelling a ParseFails row.
const FIXTURES: &[(&str, Expect)] = &[
    // pkh single-key shapes.
    ("pkh(@0)", Expect::Canonical),
    ("pkh(@0/<0;1>/*)", Expect::Canonical),
    ("pkh(@0/**)", Expect::ParseFails), // → toolkit exit 2 (`/**` shorthand)
    ("pkh([deadbeef/44'/0'/0']@0/<0;1>/*)", Expect::Canonical),
    // wpkh single-key shapes.
    ("wpkh(@0)", Expect::Canonical),
    ("wpkh(@0/<0;1>/*)", Expect::Canonical),
    ("wpkh(@0/**)", Expect::ParseFails), // → toolkit exit 2 (`/**` shorthand)
    // tr keypath-only shapes (BIP-86).
    ("tr(@0)", Expect::Canonical),
    ("tr(@0/<0;1>/*)", Expect::Canonical),
    ("tr([deadbeef/86'/0'/0']@0/**)", Expect::ParseFails), // → toolkit exit 2 (`/**` shorthand)
    // wsh-multi / sh-wsh-multi shapes (BIP-48).
    ("wsh(multi(2,@0,@1,@2))", Expect::Canonical),
    ("wsh(sortedmulti(2,@0,@1))", Expect::Canonical),
    ("sh(wsh(multi(2,@0,@1)))", Expect::Canonical),
    ("sh(wsh(sortedmulti(2,@0,@1)))", Expect::Canonical),
    // Non-canonical: tr with taptree.
    ("tr(NUMS,and_v(v:pk(@0),after(12000000)))", Expect::NonCanonical),
    // Non-canonical: wsh(andor(...)) — user's flagship v0.19.0 invocation.
    ("wsh(andor(pkh(@0),after(12000000),pk(@1)))", Expect::NonCanonical),
    // Non-canonical: sh(sortedmulti(...)) / sh(multi(...)) legacy P2SH (no wsh wrap).
    ("sh(sortedmulti(2,@0,@1))", Expect::NonCanonical),
    ("sh(multi(2,@0,@1))", Expect::NonCanonical),
];
// 11 Canonical + 4 NonCanonical + 3 ParseFails = 18 (15 classify, 3 parse-fail).

#[test]
fn gui_classifier_matches_toolkit_for_all_fixtures() {
    let Some(bin) = mnemonic_bin() else {
        eprintln!("skip: MNEMONIC_BIN unset and `mnemonic` not on PATH");
        return;
    };

    // Four independent failure accumulators, each asserted empty below.
    // gui != tk (the original drift check, keyed on the LIVE toolkit verdict):
    let mut disagreements: Vec<(String, Canonicity, Canonicity)> = Vec::new();
    // a classify-expected fixture now parse-fails (broad toolkit regression):
    let mut regressed: Vec<String> = Vec::new();
    // the toolkit verdict drifted from the pinned `want`:
    let mut wrong_verdict: Vec<(String, Canonicity, Canonicity)> = Vec::new();
    // a ParseFails-expected fixture now parses (promote it):
    let mut newly_parsed: Vec<String> = Vec::new();

    for &(descriptor, expect) in FIXTURES {
        let gui = classify_descriptor_canonicity(descriptor);
        let tk = toolkit_classify(&bin, descriptor);
        match (expect, tk) {
            // Expected parse-fail, still parse-fails: the pinned state. OK.
            (Expect::ParseFails, None) => {}
            // Expected parse-fail, but the toolkit now ACCEPTS it. The
            // BIP-388 `@N/**` shorthand started parsing — promote it to its
            // real Canonical/NonCanonical expectation.
            (Expect::ParseFails, Some(_)) => newly_parsed.push(descriptor.to_string()),
            // Expected to classify, but parse-failed: the broad toolkit
            // regression the old `classified >= len/2` floor tolerated.
            (_want, None) => regressed.push(descriptor.to_string()),
            // Expected to classify, and it did: pin the absolute verdict
            // (want) AND the GUI/toolkit agreement (the original drift check).
            (want, Some(tk_verdict)) => {
                let want = want
                    .as_canonicity()
                    .expect("classify-expected fixture maps to a Canonicity");
                if tk_verdict != want {
                    wrong_verdict.push((descriptor.to_string(), want, tk_verdict));
                }
                if gui != tk_verdict {
                    disagreements.push((descriptor.to_string(), gui, tk_verdict));
                }
            }
        }
    }

    assert!(
        newly_parsed.is_empty(),
        "canonicity drift: the toolkit parser now ACCEPTS fixture(s) pinned as ParseFails:\n  {}\n\
         The BIP-388 `@N/**` shorthand started parsing at the toolkit — promote each to its real \
         Canonical/NonCanonical expectation in FIXTURES (re-capture via \
         `mnemonic gui-schema --classify-descriptor`).",
        newly_parsed.join("\n  ")
    );
    assert!(
        regressed.is_empty(),
        "canonicity drift: expected-classify fixture(s) now parse-fail at the toolkit (exit 2):\n  {}\n\
         A broad toolkit-parser regression — the exact failure the old `classified >= len/2` floor \
         silently tolerated. Source-of-truth: \
         descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs.",
        regressed.join("\n  ")
    );
    assert!(
        wrong_verdict.is_empty(),
        "canonicity drift: toolkit canonical-origin verdict drifted from the pinned expectation on:\n{}\n\
         Source-of-truth: descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs. \
         If md-codec's canonical table legitimately changed, update each fixture's Expect in FIXTURES.",
        wrong_verdict
            .iter()
            .map(|(d, want, tk)| format!("  {d:?}: want={want:?} toolkit={tk:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        disagreements.is_empty(),
        "canonicity drift: GUI regex disagrees with toolkit canonical-origin table on:\n{}\n\
         Source-of-truth: descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs (SPEC §4.12.a).\n\
         GUI regex: mnemonic-gui/src/form/conditional.rs::classify_descriptor_canonicity.",
        disagreements
            .iter()
            .map(|(d, gui, tk)| format!("  {d:?}: gui={gui:?} toolkit={tk:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
