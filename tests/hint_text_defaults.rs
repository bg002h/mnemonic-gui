//! Hint-text-defaults cycle (SPEC_gui_hint_text_defaults.md §7 — the fix for
//! `gui-prefilled-default-text-appends-on-type`).
//!
//! A Text/Path flag with a schema-declared `default_value` must NOT pre-fill
//! its widget with the default as real editable text (typing without clearing
//! APPENDED: `--feerate` `1.0`+`5` → `1.05`). Post-fix the default DISPLAYS
//! as an egui `hint_text` ghost, typing REPLACES, and an empty field means
//! "the CLI applies its own default" (argv-identical by D33 — the flag was
//! ALREADY omitted at-default before this fix; zero argv bytes change).
//!
//! Test classes here (spec §7):
//!  1. THE append regression: type into a defaulted field without clearing →
//!     the typed value REPLACES (argv binds the pure typed token).
//!  2. Schema-derived pre-fill invariant (sweep-style, not a 6-count
//!     change-detector): EVERY Text/Path flag with a `default_value` across
//!     all 61 subcommands starts with an empty first-render buffer AND is
//!     omitted from the assembled argv.
//!  3. Ghost presence, anti-tautology half: egui paints `hint_text` WITHOUT
//!     entering the text buffer, so the AccessKit text-input node's VALUE is
//!     empty on first render — while the snapshot-corpus PNG for the same
//!     form simultaneously shows the ghost. The pair ties the `.gui`
//!     `<hint:d>` notation (pinned in `tests/gui_render_emit.rs`) to real
//!     widget behavior rather than to the renderer's own output.
//!  4. Persistence normalization (SPEC §3.4 — load-time one-time migration):
//!     a persisted Text/Path value EQUAL to its flag's schema default is
//!     dropped on load; anything else (non-default values, `Path("")`,
//!     unknown subcommands/flags, non-Text/Path kinds) survives verbatim.

mod ui_harness;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::persistence::{self, PersistedState};
use mnemonic_gui::schema::{FlagKind, FlagValue, FormState};

use ui_harness::{flag_of, render_flag_harness, schema_for, sub_of};

// ─── 1. THE append regression ───────────────────────────────────────────────

/// Fresh `compare-cost` form, type `5` into `--feerate` WITHOUT clearing:
/// argv must contain `--feerate 5` — not `1.05` (the pre-fix append), not
/// `1.0` (the old pre-fill). The first render seeds the flag through the
/// production path (`render_with_dispatch` → `default_flag_value_for_flag`),
/// exactly the user's on-load state.
#[test]
fn typing_into_defaulted_feerate_replaces_never_appends() {
    let tab = CliTab::Mnemonic;
    let sub = sub_of(tab, "compare-cost");
    let flag = flag_of(sub, "--feerate");

    let mut h = render_flag_harness(tab, sub, flag, FormState::default());
    h.run();
    h.get_by_role(Role::TextInput).type_text("5");
    h.run();
    h.run(); // settle: buffer write-back lands at frame end

    let argv = assemble_argv(schema_for(tab), sub, h.state());
    let pos = argv
        .iter()
        .position(|t| t == "--feerate")
        .unwrap_or_else(|| panic!("--feerate 5 (non-default) must emit; got {argv:?}"));
    assert_eq!(
        argv.get(pos + 1).map(String::as_str),
        Some("5"),
        "typing into a defaulted field must REPLACE the ghost, never append; got {argv:?}"
    );
}

/// Typing the LITERAL default into the field is suppressed identically to
/// leaving it empty (the `is_at_default` emission gate is untouched) — the
/// reachable argv space is unchanged by this fix.
#[test]
fn typing_the_literal_default_is_suppressed_like_untouched() {
    let tab = CliTab::Mnemonic;
    let sub = sub_of(tab, "compare-cost");
    let flag = flag_of(sub, "--feerate");

    let mut h = render_flag_harness(tab, sub, flag, FormState::default());
    h.run();
    h.get_by_role(Role::TextInput).type_text("1.0");
    h.run();
    h.run();

    let argv = assemble_argv(schema_for(tab), sub, h.state());
    assert!(
        !argv.iter().any(|t| t == "--feerate"),
        "--feerate 1.0 (the literal schema default) must be suppressed; got {argv:?}"
    );
}

// ─── 2. schema-derived pre-fill invariant (all 61 subcommands) ──────────────

/// For EVERY Text/Path flag with a schema `default_value` across all four
/// CLIs: the first-render buffer is EMPTY (the `state.values` entry is
/// `Text("")`/`Path("")` or absent) AND the assembled argv omits the flag.
/// Schema-derived — a future defaulted Text/Path flag is covered on arrival,
/// and a re-introduced pre-fill fails here immediately.
#[test]
fn defaulted_text_path_flags_never_prefill_and_never_emit_untouched() {
    let mut census = 0usize;
    for tab in CliTab::ALL {
        for sub in schema_for(*tab).subcommands {
            for flag in sub.flags {
                if flag.default_value.is_none()
                    || !matches!(flag.kind, FlagKind::Text | FlagKind::Path { .. })
                {
                    continue;
                }
                census += 1;
                let coord = format!("{}/{}/{}", tab.bin_name(), sub.name, flag.name);

                let mut h = render_flag_harness(*tab, sub, flag, FormState::default());
                h.run();
                let state = h.state();
                match state.values.iter().find(|(k, _)| k == flag.name) {
                    None => {} // not rendered/seeded — absent is empty-equivalent
                    Some((_, FlagValue::Text(s))) | Some((_, FlagValue::Path(s))) => {
                        assert!(
                            s.is_empty(),
                            "[{coord}] first-render buffer must be EMPTY \
                             (ghost, not pre-fill); got {s:?}"
                        );
                    }
                    Some((_, other)) => {
                        panic!("[{coord}] unexpected first-render value shape {other:?}")
                    }
                }
                let argv = assemble_argv(schema_for(*tab), sub, state);
                assert!(
                    !argv.iter().any(|t| t == flag.name),
                    "[{coord}] untouched defaulted flag must be OMITTED from argv; got {argv:?}"
                );
            }
        }
    }
    // Vacuity guard only — NOT a change-detector (the exact census, 6 at spec
    // time, may legitimately grow with future schema additions).
    assert!(census > 0, "no defaulted Text/Path flags found — sweep is vacuous");
}

// ─── 3. ghost presence: AccessKit value stays empty ─────────────────────────

/// Anti-tautology anchor (spec §7): egui paints `hint_text` WITHOUT entering
/// the text buffer, so on first render the AccessKit text-input VALUE is
/// empty — while the regenerated snapshot PNG (`tests/snapshots/forms/
/// mnemonic-compare-cost.png`) simultaneously shows the `1.0` ghost. Together
/// with the `<hint:1.0>` exact-ASCII pin in `tests/gui_render_emit.rs`, this
/// ties the `.gui` hint notation to real widget behavior.
#[test]
fn ghost_hint_is_painted_not_stored_accesskit_value_empty() {
    let tab = CliTab::Mnemonic;
    let sub = sub_of(tab, "compare-cost");
    let flag = flag_of(sub, "--feerate");

    let mut h = render_flag_harness(tab, sub, flag, FormState::default());
    h.run();
    let node = h.get_by_role(Role::TextInput);
    let value = node.value().unwrap_or_default();
    assert!(
        value.is_empty(),
        "first-render AccessKit value must be empty (the hint is painted, \
         never stored in the buffer); got {value:?}"
    );
}

// ─── 4. persistence normalization (SPEC §3.4) ───────────────────────────────

/// Round-trip a `PersistedState` through `save` → `load` and return the
/// restored per-subcommand form map entry for `key`.
fn save_load_form(
    entries: Vec<(&str, FormState)>,
) -> std::collections::BTreeMap<String, FormState> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut ps = PersistedState {
        last_cli_tab: "mnemonic".into(),
        ..Default::default()
    };
    for (k, v) in entries {
        ps.form_state_per_subcommand.insert(k.to_string(), v);
    }
    persistence::save(&ps, &path).unwrap();
    persistence::load(&path)
        .expect("state.json must load")
        .form_state_per_subcommand
}

fn value_of<'a>(fs: &'a FormState, flag: &str) -> Option<&'a FlagValue> {
    fs.values.iter().find(|(k, _)| k == flag).map(|(_, v)| v)
}

/// Vector 1: a persisted `("--output", Path("-"))` — the pre-fix seeded
/// default — is DROPPED on load (the ghost now carries the display duty).
#[test]
fn migration_drops_persisted_value_equal_to_schema_default() {
    let restored = save_load_form(vec![(
        "mnemonic:export-wallet",
        FormState::from_pairs(vec![("--output", FlagValue::Path("-".into()))]),
    )]);
    let fs = restored.get("mnemonic:export-wallet").expect("key survives");
    assert!(
        value_of(fs, "--output").is_none(),
        "Path(\"-\") equals export-wallet --output's schema default — must be dropped; got {:?}",
        fs.values
    );
}

/// The Text twin of vector 1: import-wallet's `--select-descriptor` seeded
/// `Text("all")` is dropped on load.
#[test]
fn migration_drops_persisted_text_default() {
    let restored = save_load_form(vec![(
        "mnemonic:import-wallet",
        FormState::from_pairs(vec![(
            "--select-descriptor",
            FlagValue::Text("all".into()),
        )]),
    )]);
    let fs = restored.get("mnemonic:import-wallet").expect("key survives");
    assert!(
        value_of(fs, "--select-descriptor").is_none(),
        "Text(\"all\") equals the schema default — must be dropped; got {:?}",
        fs.values
    );
}

/// Vector 2: a persisted NON-default `Path("/tmp/x")` survives verbatim.
#[test]
fn migration_keeps_non_default_path_value() {
    let restored = save_load_form(vec![(
        "mnemonic:export-wallet",
        FormState::from_pairs(vec![("--output", FlagValue::Path("/tmp/x".into()))]),
    )]);
    let fs = restored.get("mnemonic:export-wallet").expect("key survives");
    assert_eq!(
        value_of(fs, "--output"),
        Some(&FlagValue::Path("/tmp/x".into())),
        "non-default user value must survive verbatim"
    );
}

/// Vector 3 (§3.4d): a persisted `Path("")` — a legitimate POST-fix autosave
/// entry — survives verbatim: empty does not equal the default and must NOT
/// be dropped.
#[test]
fn migration_keeps_empty_path_value() {
    let restored = save_load_form(vec![(
        "mnemonic:export-wallet",
        FormState::from_pairs(vec![("--output", FlagValue::Path(String::new()))]),
    )]);
    let fs = restored.get("mnemonic:export-wallet").expect("key survives");
    assert_eq!(
        value_of(fs, "--output"),
        Some(&FlagValue::Path(String::new())),
        "Path(\"\") does not equal the default \"-\" — must survive (§3.4d)"
    );
}

/// Fail-open (§3.4b): unknown subcommand keys and unknown flag names keep
/// their entries verbatim — the migration is never destructive on a lookup
/// miss.
#[test]
fn migration_fails_open_on_unknown_subcommand_and_flag() {
    let restored = save_load_form(vec![
        (
            "mnemonic:no-such-sub",
            FormState::from_pairs(vec![("--output", FlagValue::Path("-".into()))]),
        ),
        (
            "mnemonic:export-wallet",
            FormState::from_pairs(vec![("--no-such-flag", FlagValue::Text("all".into()))]),
        ),
    ]);
    let unknown_sub = restored.get("mnemonic:no-such-sub").expect("key survives");
    assert_eq!(
        value_of(unknown_sub, "--output"),
        Some(&FlagValue::Path("-".into())),
        "unknown subcommand: entry must survive verbatim (fail-open)"
    );
    let unknown_flag = restored.get("mnemonic:export-wallet").expect("key survives");
    assert_eq!(
        value_of(unknown_flag, "--no-such-flag"),
        Some(&FlagValue::Text("all".into())),
        "unknown flag name: entry must survive verbatim (fail-open)"
    );
}

/// Kind scope (§3.4c): Number-kind entries are untouched even when they equal
/// the schema default numerically — bundle's `--account` `Number(0)` hand-seed
/// (main.rs) must survive.
#[test]
fn migration_leaves_number_kind_entries_untouched() {
    let restored = save_load_form(vec![(
        "mnemonic:bundle",
        FormState::from_pairs(vec![("--account", FlagValue::Number(0))]),
    )]);
    let fs = restored.get("mnemonic:bundle").expect("key survives");
    assert_eq!(
        value_of(fs, "--account"),
        Some(&FlagValue::Number(0)),
        "the migration is Text/Path-scoped ONLY — Number entries survive"
    );
}
