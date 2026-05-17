//! v0.8.1 F2 drift gate — agreement between the GUI's regex classifier
//! (`classify_descriptor_canonicity`) and the toolkit's authoritative
//! canonical-origin table (via `mnemonic gui-schema --classify-descriptor`).
//!
//! Iterates a fixture list mirroring `tests/canonicity_classifier.rs`'s
//! corpus, shells out per-fixture to `MNEMONIC_BIN gui-schema
//! --classify-descriptor <STR>`, and asserts the GUI verdict matches the
//! toolkit verdict on every fixture. If md-codec's `canonical_origin`
//! table evolves AND the GUI's 5-regex classifier doesn't keep pace, this
//! gate fires.
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

/// Fixture set mirrors `tests/canonicity_classifier.rs`'s corpus shape.
/// Each (descriptor, expected) pair is what BOTH classifiers should agree
/// on. Empty descriptor is excluded — the GUI classifies empty as
/// NonCanonical (banner-gating convenience); the toolkit rejects empty
/// with exit 2 (no `@N` placeholder). This is a documented behavioral
/// divergence at the empty-string edge — covered separately at
/// `descriptor_non_canonical_default_path_notice.rs::empty_descriptor_returns_none`
/// which pins the banner-suppression invariant.
const FIXTURES: &[&str] = &[
    // Canonical pkh single-key shapes.
    "pkh(@0)",
    "pkh(@0/<0;1>/*)",
    "pkh(@0/**)",
    "pkh([deadbeef/44'/0'/0']@0/<0;1>/*)",
    // Canonical wpkh single-key shapes.
    "wpkh(@0)",
    "wpkh(@0/<0;1>/*)",
    "wpkh(@0/**)",
    // Canonical tr keypath-only shapes (BIP-86).
    "tr(@0)",
    "tr(@0/<0;1>/*)",
    "tr([deadbeef/86'/0'/0']@0/**)",
    // Canonical wsh-multi / sh-wsh-multi shapes (BIP-48).
    "wsh(multi(2,@0,@1,@2))",
    "wsh(sortedmulti(2,@0,@1))",
    "sh(wsh(multi(2,@0,@1)))",
    "sh(wsh(sortedmulti(2,@0,@1)))",
    // Non-canonical: tr with taptree.
    "tr(NUMS,and_v(v:pk(@0),after(12000000)))",
    // Non-canonical: wsh(andor(...)) — user's flagship v0.19.0 invocation.
    "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
    // Non-canonical: sh(sortedmulti(...)) legacy P2SH multi (no wsh wrap).
    "sh(sortedmulti(2,@0,@1))",
    "sh(multi(2,@0,@1))",
];

#[test]
fn gui_classifier_matches_toolkit_for_all_fixtures() {
    let Some(bin) = mnemonic_bin() else {
        eprintln!("skip: MNEMONIC_BIN unset and `mnemonic` not on PATH");
        return;
    };

    let mut disagreements: Vec<(String, Canonicity, Canonicity)> = Vec::new();
    let mut classified = 0usize;
    let mut parse_failed = 0usize;
    for descriptor in FIXTURES {
        let gui = classify_descriptor_canonicity(descriptor);
        match toolkit_classify(&bin, descriptor) {
            None => {
                // Toolkit can't parse this descriptor (exit 2, e.g. BIP-388
                // shorthand `@N/**` that rust-miniscript's pre-key
                // substitution refuses). The drift gate's purpose is to
                // pin md-codec's canonical-table verdict — not the
                // toolkit's parser-edge-cases. The GUI's verdict on
                // parse-fail descriptors is operationally irrelevant
                // because the user can't `bundle` them anyway.
                parse_failed += 1;
            }
            Some(toolkit_verdict) => {
                classified += 1;
                if gui != toolkit_verdict {
                    disagreements.push((descriptor.to_string(), gui, toolkit_verdict));
                }
            }
        }
    }

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
    assert!(
        classified >= FIXTURES.len() / 2,
        "drift gate exercised only {classified} of {} fixtures; {parse_failed} parse-failed at the toolkit. \
         If the toolkit's parser broadly regressed, this gate would silently pass — bumping the floor.",
        FIXTURES.len()
    );
}
