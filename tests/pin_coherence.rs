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
