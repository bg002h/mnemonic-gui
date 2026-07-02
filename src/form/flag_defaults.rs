//! Egui-free flag default/placeholder resolvers (extracted from `widget.rs`
//! in the P1 `gui`-feature split — SPEC §3 / n-R4-1). The headless
//! `gui-render` emit-mode reuses these as the SINGLE SOURCE OF TRUTH for a
//! flag's initial/default `FlagValue` (rather than re-implementing the
//! FlagKind→default mapping, which would drift). The gated `widget.rs`
//! re-exports both so its existing call sites keep resolving.

use crate::schema::{FlagKind, FlagSchema, FlagValue};

/// Construct the default `FlagValue` for a given `FlagKind`, used as the
/// initial state-of-form entry the first time a flag's widget is rendered.
///
/// v0.6.0 P3: Number / Range / Timestamp / TaggedOrIndexed return `Unset`
/// rather than a seeded numeric value (was `Number(*min)`, `Range(0, 999)`,
/// etc.). Pre-P3, the first render of any of those widgets would push a
/// concrete value into `state.values`; the argv assembler would then emit
/// `--<flag> <min>` for any numeric flag the user hadn't touched, sending
/// bogus flags to the CLI. With Unset the widget renders a `Set` affordance
/// instead; the user must opt-in to a value before emission. Kinds with a
/// natural empty representation (Text / Dropdown / Path / NodeValueComposite
/// / Boolean) keep their empty-default behavior.
///
/// v0.10.0 B.3 (D31): kept as the kind-only fallback; flag-aware callers
/// should prefer `default_flag_value_for_flag(&FlagSchema)` which consults
/// the schema-declared `default_value` (toolkit v5 single source of truth)
/// for Dropdown / Text / Path kinds.
pub fn default_flag_value_for(kind: &FlagKind) -> FlagValue {
    match kind {
        FlagKind::Text => FlagValue::Text(String::new()),
        FlagKind::Dropdown(opts) => FlagValue::Dropdown(
            opts.first().map(|s| (*s).to_string()).unwrap_or_default(),
        ),
        FlagKind::Boolean => FlagValue::Boolean(false),
        FlagKind::NodeValueComposite(opts) => FlagValue::NodeValueComposite {
            node: opts.first().map(|s| (*s).to_string()).unwrap_or_default(),
            value: String::new(),
        },
        FlagKind::Path { .. } => FlagValue::Path(String::new()),
        // v0.6.0 P3 Unset-default kinds. Click-to-seed via `seeded_value_for`.
        FlagKind::Number { .. }
        | FlagKind::Range
        | FlagKind::Timestamp
        | FlagKind::TaggedOrIndexed(_) => FlagValue::Unset,
    }
}

/// v0.10.0 B.3 (D31) — flag-aware default constructor. Reads
/// `flag.default_value` (toolkit v5 schema's per-flag default) and maps
/// it onto a concrete `FlagValue` per the FlagKind dispatch table. Falls
/// back to `default_flag_value_for(&flag.kind)` for:
///   - flags without a schema-declared default (`default_value == None`),
///   - Text / Path (hint-text-defaults, see below),
///   - the four Unset-default kinds (Number / Range / Timestamp /
///     TaggedOrIndexed) which keep their click-to-seed UX regardless of
///     the schema default (the schema default is consulted only by the
///     argv assembler's `is_at_default` suppression predicate; the widget
///     still requires user opt-in to emit anything).
///   - parse failures (defensive: bad schema would otherwise crash).
///
/// Dropdown with a declared default uses the schema string directly —
/// eliminating the pre-v0.10.0 fragility where Dropdown widgets seeded
/// `opts[0]` which only coincidentally matched the toolkit's default
/// ordering.
///
/// Hint-text-defaults (SPEC_gui_hint_text_defaults.md §3.1): **Text/Path
/// schema defaults are DISPLAY-ONLY (`hint_text` ghost) + emission-time
/// (`is_at_default`); they never enter `state.values`.** Pre-fix these
/// arms seeded the default as REAL editable text, so typing without
/// clearing APPENDED (`--feerate` `1.0`+`5` → `1.05` — the
/// `gui-prefilled-default-text-appends-on-type` papercut). The empty seed
/// is argv-identical for an untouched field (D33 already omitted
/// at-default values); the widget renders the default as a ghost that
/// typing REPLACES. This one arm-change atomically moves ALL resolver
/// consumers: the widget seed (`widget.rs`), the emit-side
/// `seeded_fixture` + value column (`render_emit.rs`).
pub fn default_flag_value_for_flag(flag: &FlagSchema) -> FlagValue {
    let Some(default_str) = flag.default_value else {
        return default_flag_value_for(&flag.kind);
    };
    match flag.kind {
        FlagKind::Dropdown(_) => FlagValue::Dropdown(default_str.to_string()),
        // Text / Path: empty seed — the schema default is a hint_text ghost,
        // never buffer content (see fn doc).
        // Boolean / NodeValueComposite: no meaningful default-value mapping;
        // fall through to the kind-only default (Boolean(false), empty
        // composite). Toolkit v5 doesn't emit defaults for these in practice.
        FlagKind::Text
        | FlagKind::Path { .. }
        | FlagKind::Boolean
        | FlagKind::NodeValueComposite(_) => default_flag_value_for(&flag.kind),
        // Unset-default kinds: keep the click-to-seed UX. The argv
        // assembler's `is_at_default` consults the schema default
        // separately at emission time; the widget initial state stays
        // Unset so the user must opt in.
        FlagKind::Number { .. }
        | FlagKind::Range
        | FlagKind::Timestamp
        | FlagKind::TaggedOrIndexed(_) => FlagValue::Unset,
    }
}
