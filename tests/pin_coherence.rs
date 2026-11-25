//! Guards the bug class "Cargo toolkit pin and pinned-upstream.toml drift apart"
//! (CHANGELOG v0.22.0). pinned-upstream.toml:20-21 already declares the two MUST
//! bump in lockstep; this promotes that prose to a gate. Pure-logic; no binary,
//! no network.
//!
//! The bug class — "schema updated, `pinned-upstream.toml`/Cargo pin NOT bumped,
//! masked by a local-binary schema_mirror run" — has fired TWICE (K-of-N v0.40.0;
//! the `gui-ms1-slot-subkey-pending-pin-bump` FOLLOWUP). The existing
//! schema_mirror / schema_mirror_secret_drift / gui_schema_conditional_drift gates
//! all run a LIVE binary via `*_BIN` (skipping when absent) and have NO knowledge
//! of the declared pins, so they cannot catch it. This pure-logic test can.
//!
//! Scope: guards only that the two TOOLKIT pins agree. The three sibling pins
//! (ms/mk/md) rely on the standing paired-PR discipline + the live schema_mirror
//! gate (acceptable per SPEC §6).
use std::fs;
use std::path::Path;

fn read(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap()
}

#[test]
fn cargo_toolkit_pin_matches_pinned_upstream_mnemonic_tag() {
    let cargo: toml::Value = toml::from_str(&read("Cargo.toml")).unwrap();
    let cargo_tag = cargo["dependencies"]["mnemonic-toolkit"]["tag"]
        .as_str()
        .expect("Cargo.toml [dependencies].mnemonic-toolkit.tag");
    let pinned: toml::Value = toml::from_str(&read("pinned-upstream.toml")).unwrap();
    let pinned_tag = pinned["mnemonic"]["tag"]
        .as_str()
        .expect("pinned-upstream.toml [mnemonic].tag");
    assert_eq!(
        cargo_tag, pinned_tag,
        "pin drift: Cargo.toml toolkit tag {cargo_tag:?} != pinned-upstream [mnemonic].tag \
         {pinned_tag:?}; bump BOTH in lockstep (CHANGELOG v0.22.0 bug class)"
    );
}

/// Bug hunt 2026-11-25 G5: each `pinned-upstream.toml` tag's version equals
/// the schema's `pinned_version`. Since G1 that string is the minimum
/// `cli_version` admits for a secret-bearing run, so a schema version left
/// behind (or pushed ahead of) the tag the README/CI install is a functional
/// drift, not a cosmetic one.
#[test]
fn g5_every_pinned_tag_matches_the_schema_pinned_version() {
    use mnemonic_gui::cli_version::{parse_version, parse_version_line, pinned_version_for};
    let pinned: toml::Value = toml::from_str(&read("pinned-upstream.toml")).unwrap();
    for cli in ["mnemonic", "md", "ms", "mk"] {
        let tag = pinned[cli]["tag"].as_str().expect("tag");
        let tag_version = tag.rsplit("-v").next().and_then(parse_version);
        let schema = pinned_version_for(cli).unwrap();
        let (name, schema_version) = parse_version_line(schema).unwrap();
        assert_eq!(name, cli);
        assert_eq!(
            tag_version,
            Some(schema_version),
            "[{cli}].tag {tag:?} vs schema pinned_version {schema:?}"
        );
    }
}

/// G5: the header must not promise a runtime soft-check or banner that the
/// app does not have, and must name the check that exists.
#[test]
fn g5_pinned_upstream_header_describes_the_real_runtime_check() {
    let t = read("pinned-upstream.toml");
    assert!(!t.contains("src/schema_check.rs runtime soft-check"));
    assert!(t.contains("src/cli_version.rs"));
}
