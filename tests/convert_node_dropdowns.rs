//! L13 (constellation bughunt) — `convert --from` / `--to` node dropdowns.
//!
//! `convert --from seedqr=<digits>` was unreachable from the GUI: a single
//! shared `NODE_TYPES` const (seedqr-free) backed BOTH `--from`
//! (`NodeValueComposite`) and `--to` (`Dropdown`). The toolkit ACCEPTS
//! `--from seedqr` (`NodeType::as_str` lists `seedqr` at index 1) but REJECTS
//! `--to seedqr` (its `--to` `PossibleValuesParser` deliberately omits it —
//! seedqr is decode/input-only). The fix splits the list:
//!   * `CONVERT_FROM_NODES` (14, seedqr@1) backs `--from`;
//!   * `CONVERT_TO_NODES` (13, seedqr-free) backs `--to`.

use mnemonic_gui::schema::{self, FlagKind, FlagSchema};

fn convert_flag(name: &str) -> &'static FlagSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "convert")
        .expect("convert subcommand")
        .flags
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("convert {name} flag"))
}

/// Value list backing a `--from` NodeValueComposite flag.
fn from_values() -> &'static [&'static str] {
    match convert_flag("--from").kind {
        FlagKind::NodeValueComposite(vals) => vals,
        _ => panic!("--from is not a NodeValueComposite flag"),
    }
}

/// Value list backing a `--to` Dropdown flag.
fn to_values() -> &'static [&'static str] {
    match convert_flag("--to").kind {
        FlagKind::Dropdown(vals) => vals,
        _ => panic!("--to is not a Dropdown flag"),
    }
}

#[test]
fn convert_from_dropdown_includes_seedqr() {
    // RED at 0bbe3e1: `--from` shared the seedqr-free `NODE_TYPES`.
    let vals = from_values();
    assert!(
        vals.contains(&"seedqr"),
        "--from must offer `seedqr` (the toolkit accepts `--from seedqr`); got {vals:?}"
    );
    // Mirrors `NodeType::as_str` ordering: seedqr at index 1 (after phrase).
    assert_eq!(
        vals.get(1),
        Some(&"seedqr"),
        "`seedqr` should sit at index 1 (after `phrase`), mirroring NodeType::as_str; got {vals:?}"
    );
}

#[test]
fn convert_to_dropdown_excludes_seedqr() {
    // GUARD (GREEN before and after the split): the split must NOT leak
    // seedqr into `--to`. The toolkit's `--to` PossibleValuesParser refuses
    // seedqr (decode/input-only) — offering it would be a guaranteed-error
    // UI choice and would diverge from gui-schema's `--to` enum.
    let vals = to_values();
    assert!(
        !vals.contains(&"seedqr"),
        "--to must NOT offer `seedqr` (the toolkit refuses `--to seedqr`); got {vals:?}"
    );
}
