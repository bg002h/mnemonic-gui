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

// ── S2 (#6) — md/ms/mk CHOICES-ONLY gate extension ──────────────────────
//
// The shipped gate above is `mnemonic`-only (docstring `:28-31` scopes it
// there as "a bounded add"). md/ms/mk's `gui-schema` is v1 (ZERO
// `default_value` fields; only `mnemonic` emits v5) — a two-sided defaults
// gate for them is infeasible now: it would spurious-RED every correct
// mirror entry (mirror=Some, JSON=None) AND be blind to real mirror
// omissions comparing None==None GREEN. So this extension is CHOICES-ONLY
// for md/ms/mk, plus a SELF-ARMING one-sided defaults guard ("IF the JSON
// carries a default_value, it must equal the mirror") that is vacuously
// green today (JSON default_value is None everywhere for md/ms/mk) but
// arms the moment a sibling starts emitting v5. See FOLLOWUP
// `sibling-gui-schema-v5-default-value-emission` (filed in this repo's
// FOLLOWUPS.md) for the producer-side half.

/// Per-CLI binary resolver for the S2 md/ms/mk resolvability probe.
/// Adapted from `tests/schema_mirror.rs:47 resolve_bin` (kept in `tests/`, NOT
/// `src/schema_check.rs` — a `src/` change would violate this batch's
/// NO-BUMP acceptance criterion; `json_flag_choices`/`json_flag_defaults`
/// already resolve `<CLI>_BIN` internally for the actual shell-out, this
/// resolver is only for the up-front invocability probe below). NOTE: this
/// simplified copy omits that mirror's `.replace('-', '_')` env-var
/// normalization — `md`/`ms`/`mk` have no hyphen in their name, so it is a
/// no-op here; re-add it if a hyphenated CLI name is ever gated.
fn resolve_bin(cli_name: &str) -> String {
    let env_var = format!("{}_BIN", cli_name.to_ascii_uppercase());
    std::env::var(&env_var).unwrap_or_else(|_| cli_name.to_string())
}

/// True iff `cli_name`'s binary is invocable (`<CLI>_BIN` env set, or the
/// bare name resolves on `$PATH`). Generalizes `resolvable()` above
/// per-CLI for md/ms/mk.
fn cli_resolvable(cli_name: &str) -> bool {
    let env_var = format!("{}_BIN", cli_name.to_ascii_uppercase());
    if std::env::var(&env_var).map(|v| !v.is_empty()).unwrap_or(false) {
        return true;
    }
    Command::new(resolve_bin(cli_name)).arg("--help").output().is_ok()
}

/// Known md/ms/mk `choices` divergences (numerically/semantically
/// identical; not drift). Keyed `(cli, subcommand, flag)` — NOT
/// `(subcommand, flag)` like `DEFAULT_VALUE_ALLOWLIST` above, because
/// subcommand names COLLIDE across CLIs (md/ms/mk all have `encode`; md +
/// mnemonic both have `repair`). Day-one drift is ZERO (R0-verified), so
/// this stays empty — kept as scaffolding so a future genuine
/// value-form divergence has a keying convention to slot into rather than
/// inventing one under time pressure.
const CHOICES_ALLOWLIST: &[(&str, &str, &str)] = &[];

#[test]
fn md_ms_mk_choices_and_defaults_match_pinned_gui_schema() {
    let mut choices_drift: Vec<String> = Vec::new();
    let mut one_sided_default_violations: Vec<String> = Vec::new();
    let mut any_checked = false;

    for (cli, cli_schema) in [
        ("md", &schema::md::SCHEMA),
        ("ms", &schema::ms::SCHEMA),
        ("mk", &schema::mk::SCHEMA),
    ] {
        if !cli_resolvable(cli) {
            eprintln!(
                "SKIP schema_mirror_defaults_drift ({cli}): {}_BIN unset + `{cli}` \
                 not on PATH (install the pinned {cli}-cli or set {}_BIN)",
                cli.to_ascii_uppercase(),
                cli.to_ascii_uppercase()
            );
            continue;
        }
        any_checked = true;

        for sub in cli_schema.subcommands {
            let jd = json_flag_defaults(cli, sub.name).unwrap_or_else(|| {
                panic!(
                    "gui-schema defaults for `{cli} {}` returned None despite a \
                     resolvable, gui-schema-capable binary",
                    sub.name
                )
            });
            let jc = json_flag_choices(cli, sub.name).unwrap_or_else(|| {
                panic!(
                    "gui-schema choices for `{cli} {}` returned None",
                    sub.name
                )
            });

            for flag in sub.flags {
                let allowlisted = CHOICES_ALLOWLIST.contains(&(cli, sub.name, flag.name));

                // ── (a) choices gate — scoped to flags whose JSON carries
                // non-null `choices` (mirrors the mnemonic-only cell above). ──
                if !allowlisted {
                    if let Some(json_choices) = jc.get(flag.name).cloned().flatten() {
                        let hand = hand_choices(flag);
                        let json = strip_empty(&json_choices);
                        if hand != json {
                            choices_drift.push(format!(
                                "  {cli} {} {} :: mirror={:?} gui-schema={:?}",
                                sub.name, flag.name, hand, json
                            ));
                        }
                    }
                }

                // ── (b) one-sided defaults guard — self-arming: only fires
                // once md/ms/mk start emitting a non-null `default_value`
                // (v1 JSON today never does; vacuously green). ──
                if let Some(Some(json_default)) = jd.get(flag.name).cloned() {
                    let hand = hand_default(flag);
                    if hand.as_deref() != Some(json_default.as_str()) {
                        one_sided_default_violations.push(format!(
                            "  {cli} {} {} :: mirror={:?} gui-schema=Some({:?})",
                            sub.name, flag.name, hand, json_default
                        ));
                    }
                }
            }
        }
    }

    if !any_checked {
        eprintln!(
            "SKIP md_ms_mk_choices_and_defaults_match_pinned_gui_schema: none of \
             md/ms/mk resolvable (set MD_BIN/MS_BIN/MK_BIN or install the pinned \
             sibling CLIs on PATH)"
        );
        return;
    }

    assert!(
        choices_drift.is_empty(),
        "dropdown choices drift between the md/ms/mk hand mirrors and the pinned \
         gui-schema:\n{}",
        choices_drift.join("\n")
    );
    assert!(
        one_sided_default_violations.is_empty(),
        "md/ms/mk gui-schema now carries a non-null default_value that diverges \
         from the hand mirror (the sibling has started emitting v5-style \
         defaults — see FOLLOWUP `sibling-gui-schema-v5-default-value-emission`):\n{}",
        one_sided_default_violations.join("\n")
    );
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
