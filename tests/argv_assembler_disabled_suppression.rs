//! v0.10.0 Tranche B.2 — regression cells for the existing
//! Disabled-flag-suppression mechanism in `assemble_argv`.
//!
//! SPEC §6.10 / v0.16.0 contract: a flag whose effective `Visibility` is
//! `Disabled` (or `Hidden`) is suppressed from argv emission. The
//! mechanism lives at `src/form/invocation.rs:80,94-98` — see the
//! `suppresses` closure and the `if flag.name != "--slot" || …`
//! pre-emit guard.
//!
//! Pre-existing coverage at `tests/argv_assembler_visibility.rs` pins the
//! Disabled-suppression mechanism via real-schema conditional rules
//! (`bundle`'s single-sig template disables `--threshold`, etc.). This
//! file complements that by isolating the mechanism against a synthetic
//! one-subcommand schema, one cell per `FlagKind` variant, so a future
//! schema/conditional-rule refactor that accidentally drops a variant
//! cannot fall through the real-schema cells if the rule happens to not
//! target that variant. Test-only — no `src/` changes.
//!
//! Fixture convention: a single `static SCHEMA` declares a synthetic
//! `mock` CLI with one subcommand `do-thing` carrying five flags, one
//! per `FlagKind` variant we cover here. The subcommand's
//! `conditional` is a top-level `fn` (required by `SubcommandSchema`'s
//! `Option<fn(&FormState) -> FlagVisibility>` shape — closures and
//! non-`'static` fn-items can't coerce). The fn marks the targeted
//! flag `Disabled` when state carries the marker
//! `("--mark-disabled", FlagValue::Boolean(true))`; otherwise the
//! flag stays `Visible` (the negative-control path).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::schema::{
    FlagKind, FlagSchema, FlagValue, FlagVisibility, FormState, NumberMax, Schema,
    SubcommandSchema, TimestampValue, Visibility,
};

// ── synthetic schema ───────────────────────────────────────────────────

const DROPDOWN_OPTIONS: &[&str] = &["alpha", "beta", "gamma"];
const NODE_OPTIONS: &[&str] = &["phrase", "entropy"];

const FLAGS: &[FlagSchema] = &[
    FlagSchema {
        name: "--mark-disabled",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Test marker — when true, the conditional fn marks the \
               other five flags Disabled. Not asserted in argv.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-bool",
        kind: FlagKind::Boolean,
        required: false,
        repeating: false,
        help: "Boolean variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-text",
        kind: FlagKind::Text,
        required: false,
        repeating: false,
        help: "Text variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-number",
        kind: FlagKind::Number { min: 0, max: NumberMax::Static(100) },
        required: false,
        repeating: false,
        help: "Number variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-dropdown",
        kind: FlagKind::Dropdown(DROPDOWN_OPTIONS),
        required: false,
        repeating: false,
        help: "Dropdown variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-composite",
        kind: FlagKind::NodeValueComposite(NODE_OPTIONS),
        required: false,
        repeating: false,
        help: "NodeValueComposite variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-range",
        kind: FlagKind::Range,
        required: false,
        repeating: false,
        help: "Range variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-timestamp",
        kind: FlagKind::Timestamp,
        required: false,
        repeating: false,
        help: "Timestamp variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
    FlagSchema {
        name: "--my-path",
        kind: FlagKind::Path { stdio_sentinel: true },
        required: false,
        repeating: false,
        help: "Path variant under test.",
        secret: false,
        default_value: None,
        global: false,
    },
];

/// Marks all eight variant flags as `Disabled` when `--mark-disabled` is
/// true. Otherwise returns an empty `FlagVisibility` (all flags Visible
/// by default — see `invocation.rs:66-71` fallback).
fn mark_all_disabled(state: &FormState) -> FlagVisibility {
    if state.has_value("--mark-disabled") {
        vec![
            ("--my-bool", Visibility::Disabled),
            ("--my-text", Visibility::Disabled),
            ("--my-number", Visibility::Disabled),
            ("--my-dropdown", Visibility::Disabled),
            ("--my-composite", Visibility::Disabled),
            ("--my-range", Visibility::Disabled),
            ("--my-timestamp", Visibility::Disabled),
            ("--my-path", Visibility::Disabled),
        ]
    } else {
        Vec::new()
    }
}

const SUBCOMMAND: SubcommandSchema = SubcommandSchema {
    name: "do-thing",
    human_name: "Do Thing",
    flags: FLAGS,
    positional_args: &[],
    allows_slots: false,
    conditional: Some(mark_all_disabled),
};

const SUBCOMMANDS: &[SubcommandSchema] = &[SUBCOMMAND];

static SCHEMA: Schema = Schema {
    cli_name: "mock",
    pinned_version: "mock 0.0.0",
    subcommands: SUBCOMMANDS,
};

// ── cell helpers ───────────────────────────────────────────────────────

fn argv(state: &FormState) -> Vec<String> {
    assemble_argv(&SCHEMA, &SUBCOMMAND, state)
}

// ── 5 per-FlagKind Disabled-suppression cells ──────────────────────────

#[test]
fn boolean_disabled_is_suppressed() {
    // FlagKind::Boolean, value `true` — would normally emit `--my-bool`
    // (no value token; presence-only). With Disabled marker, suppressed.
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        ("--my-bool", FlagValue::Boolean(true)),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-bool"),
        "argv must NOT contain --my-bool when Disabled; got {argv:?}"
    );
}

#[test]
fn text_disabled_is_suppressed() {
    // FlagKind::Text — would normally emit `--my-text VAL`. With
    // Disabled marker, suppressed (flag name AND value).
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        ("--my-text", FlagValue::Text("hello".into())),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-text"),
        "argv must NOT contain --my-text when Disabled; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "hello"),
        "the text value must NOT leak as a positional; got {argv:?}"
    );
}

#[test]
fn number_disabled_is_suppressed() {
    // FlagKind::Number — would normally emit `--my-number 42`. With
    // Disabled marker, suppressed.
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        ("--my-number", FlagValue::Number(42)),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-number"),
        "argv must NOT contain --my-number when Disabled; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "42"),
        "the number value must NOT leak; got {argv:?}"
    );
}

#[test]
fn dropdown_disabled_is_suppressed() {
    // FlagKind::Dropdown — would normally emit `--my-dropdown alpha`.
    // With Disabled marker, suppressed.
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        ("--my-dropdown", FlagValue::Dropdown("alpha".into())),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-dropdown"),
        "argv must NOT contain --my-dropdown when Disabled; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "alpha"),
        "the dropdown value must NOT leak; got {argv:?}"
    );
}

#[test]
fn node_value_composite_disabled_is_suppressed() {
    // FlagKind::NodeValueComposite — would normally emit `--my-composite
    // phrase=foo`. With Disabled marker, suppressed.
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        (
            "--my-composite",
            FlagValue::NodeValueComposite {
                node: "phrase".into(),
                value: "foo".into(),
            },
        ),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-composite"),
        "argv must NOT contain --my-composite when Disabled; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "phrase=foo"),
        "the composite value must NOT leak; got {argv:?}"
    );
}

#[test]
fn range_disabled_is_suppressed() {
    // FlagKind::Range — would normally emit `--my-range 0,999`. With
    // Disabled marker, suppressed.
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        ("--my-range", FlagValue::Range(0, 999)),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-range"),
        "argv must NOT contain --my-range when Disabled; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "0,999"),
        "the range value must NOT leak; got {argv:?}"
    );
}

#[test]
fn timestamp_disabled_is_suppressed() {
    // FlagKind::Timestamp — would normally emit `--my-timestamp 1234567890`.
    // With Disabled marker, suppressed.
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        (
            "--my-timestamp",
            FlagValue::Timestamp(TimestampValue::Unix(1234567890)),
        ),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-timestamp"),
        "argv must NOT contain --my-timestamp when Disabled; got {argv:?}"
    );
    assert!(
        !argv.iter().any(|a| a == "1234567890"),
        "the timestamp value must NOT leak; got {argv:?}"
    );
}

#[test]
fn path_disabled_is_suppressed() {
    // FlagKind::Path with stdio_sentinel — would normally emit
    // `--my-path -`. With Disabled marker, suppressed (including the
    // sentinel token).
    let state = FormState::from_pairs(vec![
        ("--mark-disabled", FlagValue::Boolean(true)),
        ("--my-path", FlagValue::Path("-".into())),
    ]);
    let argv = argv(&state);
    assert!(
        !argv.iter().any(|a| a == "--my-path"),
        "argv must NOT contain --my-path when Disabled; got {argv:?}"
    );
    // The "-" sentinel must not leak as a positional either. Cargo's own
    // test binary argv carries a "-" in rare runners, so be strict: it
    // must not appear AFTER the subcommand-name token.
    let after_subcmd: Vec<&String> = argv.iter().skip_while(|a| *a != "do-thing").skip(1).collect();
    assert!(
        !after_subcmd.iter().any(|a| *a == "-"),
        "the stdio sentinel must NOT leak; got {argv:?}"
    );
}

// ── negative control: Visible flags DO emit ────────────────────────────

#[test]
fn visible_flags_emit_normally_negative_control() {
    // Same eight flags populated, but `--mark-disabled` absent — conditional
    // fn returns empty FlagVisibility → all flags default Visible →
    // assemble_argv MUST emit every populated flag per SPEC §6.7.
    //
    // Regression-anchors the Disabled-suppression mechanism's *symmetry*:
    // suppression must fire only on Disabled, not unconditionally.
    let state = FormState::from_pairs(vec![
        ("--my-bool", FlagValue::Boolean(true)),
        ("--my-text", FlagValue::Text("hello".into())),
        ("--my-number", FlagValue::Number(42)),
        ("--my-dropdown", FlagValue::Dropdown("alpha".into())),
        (
            "--my-composite",
            FlagValue::NodeValueComposite {
                node: "phrase".into(),
                value: "foo".into(),
            },
        ),
        ("--my-range", FlagValue::Range(0, 999)),
        (
            "--my-timestamp",
            FlagValue::Timestamp(TimestampValue::Unix(1234567890)),
        ),
        ("--my-path", FlagValue::Path("-".into())),
    ]);
    let argv = argv(&state);
    // Schema flag-declaration order: --mark-disabled, --my-bool, --my-text,
    // --my-number, --my-dropdown, --my-composite, --my-range, --my-timestamp,
    // --my-path. --mark-disabled is absent here so it does NOT emit.
    // Expected argv per SPEC §6.7 emission rules.
    assert_eq!(
        argv,
        vec![
            "mock",
            "do-thing",
            "--my-bool",
            "--my-text",
            "hello",
            "--my-number",
            "42",
            "--my-dropdown",
            "alpha",
            "--my-composite",
            "phrase=foo",
            "--my-range",
            "0,999",
            "--my-timestamp",
            "1234567890",
            "--my-path",
            "-",
        ],
        "Visible flags must emit per SPEC §6.7; got {argv:?}",
    );
}
