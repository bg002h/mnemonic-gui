//! eval §2 #13 — per-flag `default_value` / dropdown `choices` drift gate.
//!
//! The `schema_mirror` suite gates flag NAMES only; the toolkit `gui-schema`
//! v5 JSON ALSO carries each flag's `default_value` and dropdown `choices`.
//! This gate asserts the hand-maintained `mnemonic` mirror's per-flag
//! `default_value` (`FlagSchema.default_value`) and `choices`
//! (`FlagKind::Dropdown` option list) match the live pinned `gui-schema` JSON.
//! It is what makes an F6-class silent-materialization drift (a GUI dropdown
//! whose materialized `opts[0]` diverges from the toolkit's real default/
//! choices) detectable going forward.
//!
//! ## Deliberate GUI-only divergences the comparison accounts for
//! - **`""` display sentinels** — the GUI PREPENDS/APPENDS a `""` "(none)" /
//!   inference option to some dropdowns (`RESTORE_TEMPLATES`,
//!   `EXPORT_WALLET_TEMPLATES`, `ARCHETYPES`, and the F6 `_INFER` consts).
//!   The comparison STRIPS `""` from BOTH `choices` and `default_value` before
//!   comparing (an empty-string default is a "(none)/infer" sentinel, never a
//!   real toolkit default).
//! - **text-kind rendered as a Dropdown** — the GUI renders some `text`-kind
//!   toolkit flags as a small Dropdown (e.g. `--separator`
//!   `[space|hyphen|comma]` over a toolkit `text` kind whose JSON `choices` is
//!   `null`). The choices comparison is SCOPED to flags whose pinned JSON
//!   carries NON-NULL `choices`, so those kind-divergences are ignored.
//! - **value-format divergences** — a tiny explicit allowlist for defaults that
//!   differ only in string form (`compare-cost --feerate`: hand `"1.0"` vs the
//!   toolkit's `"1"`; numerically identical).
//!
//! ## Scope
//! `mnemonic` only (the CLI where the eval finding + F6 live). Extending to
//! `md`/`ms`/`mk` is a natural follow-on (their pinned binaries + any allowlist
//! entries) — deliberately out of this cycle to stay a bounded add.
//!
//! ## Binary resolution + LOUD-SKIP (mirrors `schema_mirror.rs`)
//! `MNEMONIC_BIN` env wins; else `mnemonic` on `$PATH`; else LOUD SKIP (a dev
//! box with no toolkit installed does not red). The required `schema-mirror
//! gate` CI job points `MNEMONIC_BIN` at the pinned v0.75.0 binary.

use std::process::Command;

use mnemonic_gui::schema::{self, FlagKind};
use mnemonic_gui::schema_check::{
    json_flag_choices, json_flag_defaults, parse_gui_schema_choices, parse_gui_schema_defaults,
};

/// Known default-value string-form divergences (numerically identical; not
/// drift). Keyed `(subcommand, flag)`. Kept EXPLICIT + tiny — if this grows,
/// the mirror or the toolkit has real drift to reconcile, not to allowlist.
const DEFAULT_VALUE_ALLOWLIST: &[(&str, &str)] = &[
    // `--feerate` is a decimal-sats-per-vbyte Text flag: the hand-mirror stores
    // the human default "1.0"; the toolkit's gui-schema emits the number 1
    // (→ "1"). Numerically identical.
    ("compare-cost", "--feerate"),
];

/// The hand-mirror default string for a flag, with the `""` "(none)" sentinel
/// normalized to `None` (an empty-string default is a GUI display sentinel,
/// never a real toolkit default — mirrors the `choices` `""` strip).
fn hand_default(flag: &schema::FlagSchema) -> Option<String> {
    match flag.default_value {
        Some("") | None => None,
        Some(s) => Some(s.to_string()),
    }
}

/// The hand-mirror dropdown choices for a flag (empty for non-Dropdown kinds),
/// with `""` sentinels stripped.
fn hand_choices(flag: &schema::FlagSchema) -> Vec<String> {
    match &flag.kind {
        FlagKind::Dropdown(opts) => opts
            .iter()
            .filter(|c| !c.is_empty())
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

fn strip_empty(v: &[String]) -> Vec<String> {
    v.iter().filter(|c| !c.is_empty()).cloned().collect()
}

/// True iff `mnemonic` is invocable (MNEMONIC_BIN set, or on `$PATH`).
fn resolvable() -> bool {
    if std::env::var("MNEMONIC_BIN")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    Command::new("mnemonic").arg("--help").output().is_ok()
}

#[test]
fn mnemonic_defaults_and_choices_match_pinned_gui_schema() {
    if !resolvable() {
        eprintln!(
            "SKIP schema_mirror_defaults_drift: MNEMONIC_BIN unset + `mnemonic` \
             not on PATH (install the pinned toolkit or set MNEMONIC_BIN)"
        );
        return;
    }

    let mut default_drift: Vec<String> = Vec::new();
    let mut choices_drift: Vec<String> = Vec::new();

    for sub in schema::mnemonic::SCHEMA.subcommands {
        let jd = json_flag_defaults("mnemonic", sub.name).unwrap_or_else(|| {
            panic!(
                "gui-schema defaults for `mnemonic {}` returned None despite a \
                 resolvable, gui-schema-capable binary",
                sub.name
            )
        });
        let jc = json_flag_choices("mnemonic", sub.name).unwrap_or_else(|| {
            panic!(
                "gui-schema choices for `mnemonic {}` returned None",
                sub.name
            )
        });

        for flag in sub.flags {
            // ── default_value ──
            if !DEFAULT_VALUE_ALLOWLIST.contains(&(sub.name, flag.name)) {
                let hand = hand_default(flag);
                let json = jd.get(flag.name).cloned().flatten();
                if hand != json {
                    default_drift.push(format!(
                        "  {} {} :: mirror={:?} gui-schema={:?}",
                        sub.name, flag.name, hand, json
                    ));
                }
            }

            // ── choices (scoped to flags whose JSON carries NON-NULL choices) ──
            if let Some(json_choices) = jc.get(flag.name).cloned().flatten() {
                let hand = hand_choices(flag);
                let json = strip_empty(&json_choices);
                if hand != json {
                    choices_drift.push(format!(
                        "  {} {} :: mirror={:?} gui-schema={:?}",
                        sub.name, flag.name, hand, json
                    ));
                }
            }
        }
    }

    assert!(
        default_drift.is_empty(),
        "default_value drift between the hand mirror and the pinned gui-schema:\n{}",
        default_drift.join("\n")
    );
    assert!(
        choices_drift.is_empty(),
        "dropdown choices drift between the hand mirror and the pinned gui-schema:\n{}",
        choices_drift.join("\n")
    );
}

// ── binary-free unit guard for the accessor's typed-default normalization ────

#[test]
fn parse_accessors_normalize_typed_defaults_and_choices() {
    // A synthetic v5 doc: a Number flag with a numeric default (`0`), a
    // Dropdown flag with a string default + choices, and a bare flag with no
    // default. `default_value` is captured typed and canonicalized to a string.
    let json = r#"{
        "version": 5,
        "cli": "mnemonic",
        "subcommands": [
            { "name": "demo", "flags": [
                { "name": "--account", "required": false, "kind": "number", "choices": null, "default_value": 0 },
                { "name": "--network", "required": true,  "kind": "dropdown", "choices": ["mainnet","testnet"], "default_value": "mainnet" },
                { "name": "--flag",    "required": false, "kind": "boolean", "choices": null }
            ], "positionals": [] }
        ]
    }"#;

    let defaults = parse_gui_schema_defaults(json, "demo").expect("defaults parse");
    assert_eq!(defaults.get("--account"), Some(&Some("0".to_string())), "numeric default → \"0\"");
    assert_eq!(defaults.get("--network"), Some(&Some("mainnet".to_string())));
    assert_eq!(defaults.get("--flag"), Some(&None), "absent default_value → None");

    let choices = parse_gui_schema_choices(json, "demo").expect("choices parse");
    assert_eq!(
        choices.get("--network"),
        Some(&Some(vec!["mainnet".to_string(), "testnet".to_string()]))
    );
    assert_eq!(choices.get("--account"), Some(&None), "null choices → None");
}
