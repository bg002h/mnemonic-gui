//! P2 — `gui-render` structural-form emit tests (SPEC §2 / §6).
//!
//! Pins the EXACT ASCII for representative forms (a secret-bearing form, a
//! mode form, a plain form), proves the secret-masking discipline, the
//! `--emit-all` census + count (dynamically derived — never a hard-coded 61),
//! byte-determinism, and ASCII-only stable encoding. The `gui-render` binary
//! itself is exercised as a subprocess (it must build + run headless under
//! `--no-default-features`; the `--no-default-features` build is gated in CI).

use std::collections::BTreeSet;
use std::process::Command;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::render_emit;

/// Locate `sub` on `tab`, panicking with a clear message if absent.
fn sub_of(tab: CliTab, name: &str) -> &'static mnemonic_gui::schema::SubcommandSchema {
    render_emit::schema_for(tab)
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no `{name}` subcommand on {:?}", tab))
}

fn render(tab: CliTab, name: &str) -> String {
    render_emit::render_form(tab, sub_of(tab, name))
}

// ─── exact-ASCII snapshots ──────────────────────────────────────────────────

#[test]
fn exact_render_secret_bearing_form_masks_secret() {
    // `mnemonic inspect`: a secret Text flag (`--ms1`) + checkboxes. The
    // secret value is the fixed `<masked>` sentinel — NEVER cleartext.
    let expected = "\
[ mnemonic > inspect ]
  --ms1             text  (required, secret) -> <masked>
  --mk1             text  (required, repeating) -> <empty>
  --md1             text  (required, repeating) -> <empty>
  --json            checkbox  -> [ ] off
  --reveal-secret   checkbox  -> [ ] off
  --no-auto-repair  checkbox  -> [ ] off
  [ Run ]
";
    assert_eq!(render(CliTab::Mnemonic, "inspect"), expected);
}

#[test]
fn exact_render_mode_form_shows_subsurface_placeholder() {
    // `mnemonic build-descriptor`: a MODE form. The canonical (generic)
    // fixture renders the mode-selector bespoke sub-surface as a SINGLE
    // placeholder line, not field-by-field.
    let expected = "\
[ mnemonic > build-descriptor ]
  [ build-descriptor mode selector: generic ]
  --spec                path(stdio)  -> <empty>
  --spec-schema         checkbox  -> [ ] off
  --archetype           dropdown[(none),decaying-multisig,hashlock-gated,kofn-recovery,simple-timelocked-inheritance,tiered-recovery]  -> <empty>
  --key                 text  (repeating) -> <empty>
  --threshold           number  -> <unset>
  --recovery-key        text  (repeating) -> <empty>
  --recovery-threshold  number  -> <unset>
  --final-key           text  -> <empty>
  --older               number  -> <unset>
  --recovery-older      number  -> <unset>
  --after               number  -> <unset>
  --hash                text  -> <empty>
  --emit-spec           checkbox  -> [ ] off
  --allow               dropdown[malleable,mixed-timelock,repeated-keys,resource-limit,sigless-branch]  (repeating) -> malleable
  --format              dropdown[descriptor,bip388]  -> descriptor
  --network             dropdown[mainnet,testnet,signet,regtest]  -> mainnet
  --json                checkbox  -> [ ] off
  --no-auto-repair      checkbox  -> [ ] off
  [ Run ]
";
    assert_eq!(render(CliTab::Mnemonic, "build-descriptor"), expected);
    // The bespoke sub-surface is a single labeled line, not a field dump.
    assert!(render(CliTab::Mnemonic, "build-descriptor")
        .contains("[ build-descriptor mode selector: generic ]"));
}

#[test]
fn exact_render_seeded_conditional_form_disables_multisig_only_flags() {
    // `mnemonic export-wallet`: the emit seeds flag defaults BEFORE evaluating
    // `conditional` (P3 R0 ruling A — the on-load steady state the GUI shows).
    // With `--template` auto-seeded to its single-sig default `bip44`, the
    // conditional (a) DISABLES `--descriptor` (template/descriptor mutex) and
    // the multisig-only `--threshold` / `--multisig-path-family`, and (b) DROPS
    // the spurious `(required)` markers on `--template` / `--descriptor` (the
    // "either template or descriptor" pre-check is already satisfied by the
    // seeded template). Pristine-fixture emit would falsely show all of these
    // enabled + the pair Required — a screen the user never sees.
    let expected = "\
[ mnemonic > export-wallet ]
  --template                dropdown[bip44,bip49,bip84,bip86,wsh-multi,wsh-sortedmulti,sh-wsh-multi,sh-wsh-sortedmulti,tr-multi-a,tr-sortedmulti-a]  -> bip44
  --descriptor              text  -> <empty> [disabled]
  --threshold               number  -> <unset> [disabled]
  --multisig-path-family    dropdown[bip48,bip87]  -> bip48 [disabled]
  --network                 dropdown[mainnet,testnet,signet,regtest]  -> mainnet
  --language                dropdown[english,simplifiedchinese,traditionalchinese,czech,french,italian,japanese,korean,portuguese,spanish]  -> english
  --account                 number  -> <unset>
  --format                  dropdown[bitcoin-core,bip388,coldcard,coldcard-multisig,jade,sparrow,specter,electrum,green,bsms,descriptor]  -> bitcoin-core
  --output                  path(stdio)  -> -
  --range                   range  -> <unset>
  --timestamp               timestamp  -> <unset>
  --bitcoin-core-version    number  -> <unset>
  --taproot-internal-key    tagged-or-indexed[nums]  -> <unset> [disabled]
  --wallet-name             text  -> <empty>
  --bsms-form               dropdown[2-line,4-line]  -> 4-line
  --from-import-json        path(stdio)  -> <empty>
  --from-import-json-index  number  -> <unset>
  --no-auto-repair          checkbox  -> [ ] off
  [ slot editor: 0 rows ]
  [ Run ]
";
    assert_eq!(render(CliTab::Mnemonic, "export-wallet"), expected);
}

#[test]
fn exact_render_plain_form() {
    // `mk inspect`: a plain form — one checkbox flag + one positional.
    let expected = "\
[ mk > inspect ]
  --json       checkbox  -> [ ] off
  mk1-strings  positional...  -> <empty>
  [ Run ]
";
    assert_eq!(render(CliTab::Mk, "inspect"), expected);
}

#[test]
fn slot_form_renders_slot_editor_placeholder() {
    // `mnemonic bundle` allows slots → the SlotEditor bespoke sub-surface is
    // a single placeholder line (0 rows for the canonical empty fixture).
    let out = render(CliTab::Mnemonic, "bundle");
    assert!(out.contains("[ slot editor: 0 rows ]"), "{out}");
    // `--slot` itself is suppressed from the flag grid (SlotEditor owns it).
    assert!(
        !out.lines().any(|l| l.trim_start().starts_with("--slot ")),
        "--slot must not render in the flag grid:\n{out}"
    );
    // A secret Text flag masks; a secret `*-stdin` Boolean stays a disabled
    // checkbox (no secret payload), NOT masked.
    assert!(out.contains("--passphrase            text  (secret) -> <masked>"), "{out}");
    assert!(
        out.contains("--passphrase-stdin      checkbox  (secret) -> [ ] off [disabled]"),
        "{out}"
    );
    // Seeded-conditional faithfulness (P3 R0 ruling A): the canonical fixture
    // auto-seeds `--template -> bip44` (single-sig), so the conditional greys
    // the multisig-only flags on load — exactly what the GUI shows.
    assert!(
        out.contains("--multisig-path-family  dropdown[bip48,bip87]  -> bip48 [disabled]"),
        "{out}"
    );
    assert!(out.contains("--threshold             number  -> <unset> [disabled]"), "{out}");
}

// ─── secret hygiene ─────────────────────────────────────────────────────────

#[test]
fn secret_positional_is_masked() {
    // `ms inspect`'s positional `ms1` is secret → `<masked>`, never a value.
    let out = render(CliTab::Ms, "inspect");
    assert!(out.contains("ms1     positional  (secret) -> <masked>"), "{out}");
}

#[test]
fn every_secret_flag_renders_masked_never_cleartext() {
    // For EVERY form, every secret value-bearing flag/positional renders the
    // masked sentinel; no secret Text/Path/etc. line carries a non-masked
    // value token after the `->`. (Secret `*-stdin` Booleans are checkboxes
    // with no secret payload and are excluded.)
    use mnemonic_gui::schema::FlagKind;
    for (tab, sub) in render_emit::all_forms() {
        let out = render_emit::render_form(tab, sub);
        for flag in sub.flags {
            let is_secret = mnemonic_gui::secrets::flag_is_secret(flag);
            if !is_secret || matches!(flag.kind, FlagKind::Boolean) {
                continue;
            }
            // Find the rendered line for this flag and assert it is masked.
            let line = out
                .lines()
                .find(|l| l.trim_start().starts_with(&format!("{} ", flag.name)));
            if let Some(line) = line {
                assert!(
                    line.contains("-> <masked>"),
                    "secret flag {} on {:?}/{} not masked:\n{line}",
                    flag.name,
                    tab,
                    sub.name
                );
            }
        }
        for pos in sub.positional_args {
            if pos.secret {
                let line = out.lines().find(|l| {
                    l.trim_start().starts_with(&format!("{} ", pos.name))
                });
                if let Some(line) = line {
                    assert!(
                        line.contains("-> <masked>"),
                        "secret positional {} on {:?}/{} not masked:\n{line}",
                        pos.name,
                        tab,
                        sub.name
                    );
                }
            }
        }
    }
}

// ─── structural invariants over every form ──────────────────────────────────

#[test]
fn every_form_has_header_and_action_bar_and_is_ascii() {
    for (tab, sub) in render_emit::all_forms() {
        let out = render_emit::render_form(tab, sub);
        assert!(
            out.starts_with(&format!("[ {} > {} ]\n", tab.bin_name(), sub.name)),
            "missing header for {:?}/{}",
            tab,
            sub.name
        );
        assert!(
            out.lines().last() == Some("  [ Run ]"),
            "missing action bar for {:?}/{}:\n{out}",
            tab,
            sub.name
        );
        // Stable byte encoding: strictly ASCII, LF line endings, no CR.
        assert!(out.is_ascii(), "non-ASCII render for {:?}/{}", tab, sub.name);
        assert!(!out.contains('\r'), "CR in render for {:?}/{}", tab, sub.name);
        assert!(out.ends_with('\n'), "no trailing LF for {:?}/{}", tab, sub.name);
        // No box-drawing / fancy glyphs leaked in (SPEC §2 ASCII-only).
        for bad in ['\u{25B8}', '\u{203A}', '\u{2300}', '\u{2502}', '\u{2514}'] {
            assert!(!out.contains(bad), "fancy glyph in {:?}/{}", tab, sub.name);
        }
    }
}

// ─── determinism ────────────────────────────────────────────────────────────

#[test]
fn render_is_byte_deterministic_across_runs() {
    for (tab, sub) in render_emit::all_forms() {
        let a = render_emit::render_form(tab, sub);
        let b = render_emit::render_form(tab, sub);
        assert_eq!(a, b, "non-deterministic render for {:?}/{}", tab, sub.name);
    }
}

#[test]
fn emit_all_is_byte_deterministic_across_dirs() {
    let d1 = tempfile::tempdir().unwrap();
    let d2 = tempfile::tempdir().unwrap();
    let n1 = render_emit::emit_all(d1.path()).unwrap();
    let n2 = render_emit::emit_all(d2.path()).unwrap();
    assert_eq!(n1, n2);
    for (tab, sub) in render_emit::all_forms() {
        let name = render_emit::render_file_name(tab, sub);
        let a = std::fs::read(d1.path().join(&name)).unwrap();
        let b = std::fs::read(d2.path().join(&name)).unwrap();
        assert_eq!(a, b, "non-deterministic emit for {name}");
    }
}

// ─── census + count (dynamically derived) ───────────────────────────────────

#[test]
fn emit_all_writes_one_file_per_subcommand() {
    // The count is DERIVED from the schema, never hard-coded: it must equal
    // the summed `schema_for(tab).subcommands.len()` across all four CLIs.
    let derived: usize = CliTab::ALL
        .iter()
        .map(|t| render_emit::schema_for(*t).subcommands.len())
        .sum();
    assert_eq!(render_emit::all_forms().len(), derived);

    let dir = tempfile::tempdir().unwrap();
    let written = render_emit::emit_all(dir.path()).unwrap();
    assert_eq!(written, derived, "emit_all count must equal schema sum");

    // Census: EVERY subcommand has a committed `<tab>-<sub>.gui`, including
    // zero-identity subs (every form renders, mirroring `sweep_census`).
    let on_disk: BTreeSet<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(on_disk.len(), derived);
    for (tab, sub) in render_emit::all_forms() {
        let name = render_emit::render_file_name(tab, sub);
        assert!(on_disk.contains(&name), "census gap: missing {name}");
    }
}

// ─── the actual `gui-render` binary (CLI) ───────────────────────────────────

/// Path to the built `gui-render` binary (Cargo sets this for integration
/// tests; the bin is non-gated so it builds under the default `cargo test`).
const GUI_RENDER_BIN: &str = env!("CARGO_BIN_EXE_gui-render");

#[test]
fn binary_form_mode_matches_library_render() {
    let out = Command::new(GUI_RENDER_BIN)
        .args(["--form", "mnemonic", "inspect"])
        .output()
        .expect("run gui-render --form");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, render(CliTab::Mnemonic, "inspect"));
}

#[test]
fn binary_emit_all_writes_every_form() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(GUI_RENDER_BIN)
        .args(["--emit-all"])
        .arg(dir.path())
        .output()
        .expect("run gui-render --emit-all");
    assert!(out.status.success());
    let derived: usize = CliTab::ALL
        .iter()
        .map(|t| render_emit::schema_for(*t).subcommands.len())
        .sum();
    let count = std::fs::read_dir(dir.path()).unwrap().count();
    assert_eq!(count, derived);
    // The binary's emit is byte-identical to the library's.
    for (tab, sub) in render_emit::all_forms() {
        let name = render_emit::render_file_name(tab, sub);
        let on_disk = std::fs::read_to_string(dir.path().join(&name)).unwrap();
        assert_eq!(on_disk, render_emit::render_form(tab, sub), "{name}");
    }
}

#[test]
fn binary_rejects_unknown_tab_and_subcommand() {
    let bad_tab = Command::new(GUI_RENDER_BIN)
        .args(["--form", "nope", "inspect"])
        .output()
        .unwrap();
    assert!(!bad_tab.status.success());

    let bad_sub = Command::new(GUI_RENDER_BIN)
        .args(["--form", "mnemonic", "no-such-sub"])
        .output()
        .unwrap();
    assert!(!bad_sub.status.success());

    let no_args = Command::new(GUI_RENDER_BIN).output().unwrap();
    assert!(!no_args.status.success());
}
