//! v0.6.0 P3 — FlagValue::Unset sentinel tests.
//!
//! Unset is the initial form-state value for Number / Range / Timestamp /
//! TaggedOrIndexed widgets (the four kinds without a natural empty-string
//! default). Pre-P3 these widgets auto-seeded to a concrete value
//! (e.g. `Number(min)`) the moment the form rendered, so the argv assembler
//! emitted `--<flag> <min>` for any numeric flag the user hadn't touched —
//! sending bogus flag/value pairs to the CLI. With Unset the widget renders
//! a `Set` affordance instead; the assembler skips Unset entries
//! (`flag_value_is_present` returns `false`).
//!
//! These cells assert:
//!   1. `flag_value_is_present(Unset)` is false (gates the conditional fn
//!      `has_value` checks + argv assembler emission).
//!   2. `default_flag_value_for` returns `Unset` for the four
//!      Unset-default kinds, and the prior-shape concrete default for the
//!      remaining kinds (no regression for Text/Dropdown/Boolean/Path/
//!      NodeValueComposite which already have natural empty representations).
//!   3. `seeded_value_for` returns the kind-specific concrete value used
//!      when the user clicks the `Set` affordance.
//!   4. `FlagValue::Unset` round-trips through serde.
//!   5. Forward-compat: unknown variant tags in state.json deserialize to
//!      `Unset` via `#[serde(other)]` (so a future GUI's serialized
//!      FlagValue tag would be readable by v0.6 without panic).
//!   6. The argv assembler emits NOTHING for a numeric flag whose value is
//!      Unset (the load-bearing P3 behavior fix).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::widget::{default_flag_value_for, seeded_value_for};
use mnemonic_gui::schema::{
    self, FlagKind, FlagValue, FormState, TaggedOrIndexedValue, TimestampValue,
};

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

// ── (1) flag_value_is_present(Unset) — invariant for has_value + emit ──

#[test]
fn flag_value_unset_is_not_present_via_has_value() {
    // `has_value` is the public surface that consumes
    // `flag_value_is_present` internally; assert at that boundary so the
    // contract is anchored to the consumer call site.
    let state = FormState::from_pairs(vec![("--account", FlagValue::Unset)]);
    assert!(
        !state.has_value("--account"),
        "Unset must NOT register as a present value for --account",
    );
    // Sanity: an explicit Number is present.
    let state = FormState::from_pairs(vec![("--account", FlagValue::Number(0))]);
    assert!(state.has_value("--account"));
}

// ── (2) default_flag_value_for — Unset for 4 kinds, concrete for others ──

#[test]
fn default_flag_value_for_number_kind_returns_unset() {
    assert_eq!(
        default_flag_value_for(&FlagKind::Number { min: 0, max: 100 }),
        FlagValue::Unset,
    );
}

#[test]
fn default_flag_value_for_range_kind_returns_unset() {
    assert_eq!(default_flag_value_for(&FlagKind::Range), FlagValue::Unset);
}

#[test]
fn default_flag_value_for_timestamp_kind_returns_unset() {
    assert_eq!(default_flag_value_for(&FlagKind::Timestamp), FlagValue::Unset);
}

#[test]
fn default_flag_value_for_tagged_kind_returns_unset() {
    assert_eq!(
        default_flag_value_for(&FlagKind::TaggedOrIndexed(&["nums", "explicit"])),
        FlagValue::Unset,
    );
}

#[test]
fn default_flag_value_for_text_kind_returns_empty_string_not_unset() {
    // Regression guard: Text/Dropdown/Boolean/Path already have natural
    // empty representations + their `flag_value_is_present` checks gate
    // emission correctly. They MUST NOT become Unset (would cascade
    // breakage to widget render arms expecting non-Unset for these kinds).
    assert_eq!(
        default_flag_value_for(&FlagKind::Text),
        FlagValue::Text(String::new()),
    );
    assert_eq!(
        default_flag_value_for(&FlagKind::Boolean),
        FlagValue::Boolean(false),
    );
    assert_eq!(
        default_flag_value_for(&FlagKind::Path { stdio_sentinel: false }),
        FlagValue::Path(String::new()),
    );
}

// ── (3) seeded_value_for — concrete kind-specific values ───────────────

#[test]
fn seeded_value_for_number_returns_min() {
    assert_eq!(
        seeded_value_for(&FlagKind::Number { min: 3, max: 100 }),
        FlagValue::Number(3),
    );
}

#[test]
fn seeded_value_for_range_returns_zero_to_999_default() {
    // Preserves the pre-P3 default range that the form-state initializer
    // used; click-to-set should reproduce it bit-identically.
    assert_eq!(seeded_value_for(&FlagKind::Range), FlagValue::Range(0, 999));
}

#[test]
fn seeded_value_for_timestamp_returns_now() {
    assert_eq!(
        seeded_value_for(&FlagKind::Timestamp),
        FlagValue::Timestamp(TimestampValue::Now),
    );
}

#[test]
fn seeded_value_for_tagged_returns_first_tag() {
    let kind = FlagKind::TaggedOrIndexed(&["nums", "explicit"]);
    assert_eq!(
        seeded_value_for(&kind),
        FlagValue::TaggedOrIndexed(TaggedOrIndexedValue::Tag("nums".into())),
    );
}

// ── (4) serde round-trip ───────────────────────────────────────────────

#[test]
fn flag_value_unset_serde_roundtrip() {
    let original = FlagValue::Unset;
    let json = serde_json::to_string(&original).expect("Unset must serialize");
    assert_eq!(json, r#""Unset""#);
    let back: FlagValue = serde_json::from_str(&json).expect("Unset must deserialize");
    assert_eq!(back, FlagValue::Unset);
}

// ── (5) #[serde(other)] forward-compat — load-bearing CANARY pair ──────
//
// The 3 cells below collectively pin the v0.6.0 empirically-observed
// behavior of `#[serde(other)]` on `FlagValue::Unset` (an externally-
// tagged enum). Per serde docs (https://serde.rs/variant-attrs.html):
// `#[serde(other)]` is "Only allowed on a unit variant inside of an
// internally tagged or adjacently tagged enum"; per serde-rs/serde#2010
// on externally-tagged enums it "compiles but mysteriously doesn't
// work" — a request was filed to make it a compile-time error.
// Empirically, on `FlagValue` it DOES correctly fall back unknown tags
// to Unset for BOTH bare-string and tagged-object serde branches, but
// this is undocumented and could silently change on a future serde
// upgrade. The 3 cells together cover the full forward-compat surface
// and fail LOUDLY if serde changes the behavior, forcing reconsideration
// of the v0.6.0 forward-compat claim in CHANGELOG [0.6.0]/[0.6.1].
// Tracks FOLLOWUP `gui-flag-value-unset-serde-other-externally-tagged-
// dependency` (filed v0.6.0 cycle close; resolved v0.6.1 via this canary
// pair).

#[test]
fn flag_value_unknown_tag_deserializes_to_unset_via_serde_other() {
    // CANARY (bare-string branch). A future GUI (v0.7+) might add a
    // FlagValue variant that v0.6's FlagValue doesn't know. `#[serde(
    // other)]` on Unset makes v0.6 map such a tag to Unset rather than
    // panic on deserialize. Surfaces as graceful state-restore: the
    // user's saved value is lost (becomes Unset = no value), but the
    // form-state file as a whole loads. THIS TEST FAILS LOUDLY if the
    // bare-string serde fallback breaks.
    let unknown_tag_json = r#""FutureKitchenSink""#;
    let parsed: FlagValue = serde_json::from_str(unknown_tag_json).expect(
        "CANARY: serde behavior changed — see FOLLOWUP \
         gui-flag-value-unset-serde-other-externally-tagged-dependency",
    );
    assert_eq!(parsed, FlagValue::Unset);
}

#[test]
fn flag_value_unset_canary_known_tags_still_deserialize_correctly() {
    // Regression guard for the canary pair: ensure `#[serde(other)]`
    // doesn't accidentally swallow known tags too. Every existing variant
    // tag must continue to deserialize to its specific variant, NOT to
    // Unset. If this fires, the fallback became over-broad — serde would
    // be silently mapping legitimate state.json contents to Unset, losing
    // every user-typed value across the form.
    assert_eq!(
        serde_json::from_str::<FlagValue>(r#"{"Text": ""}"#).unwrap(),
        FlagValue::Text(String::new())
    );
    assert_eq!(
        serde_json::from_str::<FlagValue>(r#"{"Number": 42}"#).unwrap(),
        FlagValue::Number(42)
    );
    assert_eq!(
        serde_json::from_str::<FlagValue>(r#""Unset""#).unwrap(),
        FlagValue::Unset
    );
    // (NOT exhaustive over all variants — sample-test the common ones; the
    // existing `flag_value_unset_serde_roundtrip` test above covers the
    // Unset-self round-trip; this cell focuses on the known-tag selectivity.)
}

#[test]
fn flag_value_unset_canary_unknown_tagged_object_currently_fails_to_deserialize() {
    // NEGATIVE CANARY (tagged-object branch). The companion bare-string
    // canary above passes — serde's `#[serde(other)]` DOES fall through
    // unknown unit-variant tags to Unset on externally-tagged FlagValue.
    // But for tagged-object payloads (a hypothetical future
    // `FutureWithPayload(SomeStruct)` variant serialized as
    // `{"FutureWithPayload": {...}}`), the v0.6 + current serde behavior
    // is to FAIL the whole deserialization rather than fall back — the
    // observed v0.6.1 limit on the forward-compat claim. Per
    // serde-rs/serde#2010 this is the documented "non-functional on
    // externally-tagged enums" behavior; empirically the bare-string
    // case works (the v0.6.1 P3 #4 discovery surfaced this asymmetry).
    //
    // This NEGATIVE canary fires if serde ever DOES make tagged-object
    // fallback work — at which point the v0.6.x forward-compat claim
    // can be broadened. Until then, the CHANGELOG/FOLLOWUP notes
    // qualify the forward-compat as "future unit-variant additions only".
    // Tracks FOLLOWUP `gui-flag-value-unset-serde-other-externally-
    // tagged-dependency`.
    let result: Result<FlagValue, _> =
        serde_json::from_str(r#"{"FutureWithPayload": {"x": 1, "y": 2}}"#);
    assert!(
        result.is_err(),
        "NEGATIVE CANARY fired: tagged-object unknown variant deserialized \
         successfully (was {result:?}). serde behavior changed — the v0.6.x \
         forward-compat claim can now be broadened to cover data-carrying \
         future variants. Update CHANGELOG + FOLLOWUP \
         gui-flag-value-unset-serde-other-externally-tagged-dependency.",
    );
}

// ── (6) argv assembler skips Unset (load-bearing P3 fix) ───────────────

#[test]
fn argv_does_not_emit_unset_numeric_flag() {
    // Pre-P3: state.values would carry --account → Number(0) the moment
    // the widget rendered, and the assembler would emit `--account 0`
    // unconditionally. Post-P3: state.values carries Unset and the
    // assembler's emit_one type-mismatch arm silently skips it.
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        // --account explicitly Unset — must NOT emit.
        ("--account", FlagValue::Unset),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert!(
        !argv.iter().any(|a| a == "--account"),
        "argv must NOT contain --account when its value is Unset; got {argv:?}",
    );
    // Sanity: non-Unset flags still emit.
    assert!(argv.iter().any(|a| a == "--network"));
    assert!(argv.iter().any(|a| a == "--template"));
}

#[test]
fn argv_does_not_emit_unset_range_flag() {
    // --range on `bundle` is a Range kind; same Unset semantic.
    let state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--range", FlagValue::Unset),
    ]);
    let argv = assemble_argv(&schema::mnemonic::SCHEMA, subcommand("bundle"), &state);
    assert!(
        !argv.iter().any(|a| a == "--range"),
        "argv must NOT contain --range when its value is Unset; got {argv:?}",
    );
}
