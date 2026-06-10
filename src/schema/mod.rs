//! Schema types — the static description of each CLI's flag surface, used
//! by the form renderer (Phase 2), conditional-visibility engine (Phase 5),
//! argv assembler (Phase 2 §6), and the schema-mirror CI gate (Phase 9).
//!
//! See SPEC §B.3 for the normative type definitions.

pub mod archetypes;
pub mod md;
pub mod mk;
pub mod mnemonic;
pub mod ms;
pub mod nodes;

/// Static description of one CLI binary's flag surface.
pub struct Schema {
    /// Binary name (`"mnemonic"`, `"md"`, `"ms"`, `"mk"`).
    pub cli_name: &'static str,
    /// Pinned `--version` output string that the runtime soft-check
    /// (SPEC §11) compares against `<cli> --version`. Example for
    /// mnemonic-toolkit-v0.8.1: `"mnemonic 0.8.0"` (the v0.8.1 git tag
    /// did NOT bump the crate package version). The git-tag string for CI
    /// install lives separately in `pinned-upstream.toml`. R1 I-1 fold.
    pub pinned_version: &'static str,
    /// All subcommands the GUI surfaces. Subset of upstream — v0.1 covers
    /// the 5 most-used per Section A coverage table.
    pub subcommands: &'static [SubcommandSchema],
}

/// One subcommand (e.g. `mnemonic export-wallet`).
pub struct SubcommandSchema {
    /// Argv name (e.g. `"export-wallet"`).
    pub name: &'static str,
    /// Display label for the GUI subcommand picker.
    pub human_name: &'static str,
    /// Every flag the upstream clap-derive `Args` block declares.
    pub flags: &'static [FlagSchema],
    /// Positional args (no `--name` prefix), emitted at the end of argv
    /// after all flags. Phase 6 introduced this for `md inspect`,
    /// `ms inspect`, `mk inspect` which take an `<MD1>` / `[MS1]` /
    /// `[MK1_STRINGS]...` positional. mnemonic-toolkit's subcommands
    /// have zero positionals — they pass slot data via `--slot`. Empty
    /// slice for subcommands with no positionals.
    pub positional_args: &'static [PositionalArgSchema],
    /// True for `bundle` / `verify-bundle` / `export-wallet` — subcommands
    /// that accept the `--slot @N.<subkey>=<value>` repeating grammar.
    pub allows_slots: bool,
    /// Optional conditional-visibility function. Phase 5 fills these in;
    /// Phase 1 leaves them all `None`.
    pub conditional: Option<fn(&FormState) -> FlagVisibility>,
}

/// Positional argument schema (no `--name` prefix). Phase 6.
pub struct PositionalArgSchema {
    /// Human label for the form widget (e.g. `"md1-strings"`).
    pub name: &'static str,
    /// True for clap-required positionals (`<NAME>`), false for clap-
    /// optional (`[NAME]`).
    pub required: bool,
    /// True for `<NAME>...` / `[NAME]...` (clap `num_args = 1..`).
    pub repeating: bool,
    /// Tooltip text.
    pub help: &'static str,
}

/// One flag (e.g. `--template`).
///
/// v0.31.0 (SPEC §4 / R0-r1 I3a): derives `Clone` so the archetype param
/// form can synthesize per-mode variants of the static
/// `BUILD_DESCRIPTOR_FLAGS` entries (e.g. swap `--threshold`'s `max` to
/// `NumberMax::FromRowCount`, or render an archetype-non-repeatable key
/// as a scalar row). The static entries themselves are never mutated.
#[derive(Clone)]
pub struct FlagSchema {
    /// Argv form (e.g. `"--template"`).
    pub name: &'static str,
    /// Per-`FlagKind` widget + emission shape.
    pub kind: FlagKind,
    /// True if the flag is clap-level required.
    pub required: bool,
    /// True for clap `Append` / `num_args=1..` flags — `--to`,
    /// `--ms1` / `--mk1` / `--md1`, `--slot`.
    pub repeating: bool,
    /// Tooltip text for the form widget.
    pub help: &'static str,
    /// True if the flag is in the never-persist / paste-warn class
    /// (SPEC §9, §10). Phase 1 hand-codes; Phase 7 cross-checks against
    /// upstream `NodeType::is_secret_bearing()` via the source-level audit.
    /// v0.10.0 B.3 (D31): mirrored from the toolkit v5 schema's per-flag
    /// `secret` boolean. Hand-coded values reconciled with toolkit v5 ground
    /// truth (toolkit's `flag_is_secret` is the union of NodeType +
    /// SlotSubkey + hand-listed flag names).
    pub secret: bool,
    /// v0.10.0 B.3 (D31): clap-derive declared default-value, mirrored from
    /// the toolkit v5 schema's per-flag `default_value` field. Stored as a
    /// string-form representation that maps onto the argv emission token
    /// per-`FlagKind`:
    ///
    /// - Number: decimal integer (e.g. `"0"`, `"25"`).
    /// - Dropdown / Text / Path: literal string (e.g. `"mainnet"`, `"-"`).
    /// - Range: `<u32>,<u32>` form (e.g. `"0,999"`).
    /// - Timestamp: `"now"` for the Now sentinel; Epoch values never have
    ///   a meaningful schema default.
    /// - Boolean: ignored (Boolean defaults are always-false; presence-only
    ///   emission already trivially suppresses absent / `false` values).
    /// - NodeValueComposite / TaggedOrIndexed: no toolkit default emission.
    ///
    /// Consumed by `form/invocation::is_at_default` per D33's compare-predicate
    /// table for argv emission suppression. `None` means the flag has no
    /// clap-declared default (most flags); in that case `is_at_default` always
    /// returns false and the value emits if Set.
    pub default_value: Option<&'static str>,
    /// v0.10.0 B.3 (D31): mirrored from the toolkit v5 schema's per-flag
    /// `global` boolean. True for clap-global flags like `--no-auto-repair`
    /// that propagate across every subcommand. The argv assembler emits
    /// global flags the same way as non-global; this field is consumed by
    /// future GUI-side projection (e.g. action-bar promotion) and the
    /// drift gate.
    pub global: bool,
}

/// Widget shape + argv emission rule. See SPEC §6.7 for byte-exact emission
/// rules per variant.
///
/// v0.31.0 (SPEC §4 / R0-r1 I3a): derives `Clone, Copy` (every field is a
/// `&'static` slice / bool / `NumberMax`, all `Copy`) so the archetype param
/// form can synthesize bespoke `FlagSchema` clones.
#[derive(Clone, Copy)]
pub enum FlagKind {
    /// Free-form text.
    Text,
    /// Integer with inclusive bounds. v0.7.0: `max` is `NumberMax` (was
    /// `i64`) to support `FromSlotCount` (closes SPEC §6.6 row 9 via the
    /// GUI's `--threshold` widget binding its resolved max to the user's
    /// configured slot_count).
    Number { min: i64, max: NumberMax },
    /// One-of choice from a static list.
    Dropdown(&'static [&'static str]),
    /// Flag presence/absence.
    Boolean,
    /// Comma-separated `<u32>,<u32>` form (e.g. `--range 0,999`).
    Range,
    /// `now` sentinel or unix-seconds integer.
    Timestamp,
    /// `--name <node>=<value>` composite (e.g. `--from phrase=<v>`). The
    /// argv emits ONE token after `--name`: `node=value` fused with `=`.
    NodeValueComposite(&'static [&'static str]),
    /// `--name <tag>` OR `--name @N` choice (e.g. `--taproot-internal-key`).
    /// The static list names the tagged variants; `@N` is the indexed mode.
    TaggedOrIndexed(&'static [&'static str]),
    /// File path. When `stdio_sentinel == true`, the value `"-"` represents
    /// stdin/stdout and is emitted verbatim.
    Path { stdio_sentinel: bool },
}

/// Resolved-at-render-time upper bound for a `FlagKind::Number`. SPEC §6.6
/// row 9 closes GUI-side via this enum: `--threshold`'s max equals the
/// user's current slot_count, but slot_count is FormState-derived (not a
/// compile-time constant), so the enum has a `FromSlotCount` variant that
/// the widget renderer resolves against `state.slot_count()` at frame
/// boundary.
///
/// **Resolve discipline (defensive bound):** when slot_count == 0 (no
/// slots configured), the resolved max is `1` (matching the `min`
/// declared by the toolkit's clap-derive `value_parser!(1..)`). The
/// degenerate range `1..=1` is valid (the user can only enter `1`, which
/// is also the only valid threshold for a 1-slot configuration that hasn't
/// been built yet); CLI row 9 catches the residual case if the user emits
/// argv with slot_count==0.
///
/// **Toolkit wire-format:** the toolkit does NOT emit `FromSlotCount` over
/// the wire — this is a GUI-internal extension (Option A per the v0.7.0
/// design doc). The toolkit's gui-schema JSON `Flag` shape emits only
/// `kind: "number"` without min/max bounds (clap-derive's `value_parser!`
/// bounds live CLI-side only and never reach the GUI). The GUI's per-flag
/// `min`/`max: NumberMax` here is GUI-declared in `src/schema/mnemonic.rs`
/// etc., not toolkit-derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberMax {
    /// Concrete upper bound resolved at compile time (e.g., u32::MAX cast
    /// or a script-type-derived cap).
    Static(i64),
    /// Resolved at render time as `state.slot_count().max(1) as i64`.
    /// Used by `--threshold` only (SPEC §6.6 row 9).
    FromSlotCount,
    /// v0.31.0 (archetype forms, SPEC §4): resolved at render time as the
    /// `state.values` ROW COUNT for the named flag, floored `.max(1)` (the
    /// `FromSlotCount` degenerate-range discipline — R0-r2 m1). ALL rows
    /// count, including empty ones (R0-r1 I3b: an empty row emits nothing,
    /// so the resolved max can exceed the emitted key count — acceptable;
    /// the CLI gate validates). Used ONLY in the bespoke-synthesized
    /// `FlagSchema` the archetype form constructs for threshold params
    /// (`--threshold` max = `--key` row count, `--recovery-threshold` max =
    /// `--recovery-key` row count); the static `BUILD_DESCRIPTOR_FLAGS`
    /// entries keep `Static(20)` (generic-mode behavior unchanged —
    /// R0-r1 I3a).
    FromRowCount(&'static str),
}

impl NumberMax {
    /// Resolve to a concrete `i64` upper bound given the current FormState.
    /// `FromSlotCount` falls back to `1` when `slot_count() == 0` (degenerate
    /// but valid range `min..=1`; the CLI catches the residual case per
    /// SPEC §6.6 row 9 if argv is emitted from that state).
    pub fn resolve(self, state: &FormState) -> i64 {
        match self {
            NumberMax::Static(n) => n,
            NumberMax::FromSlotCount => state.slot_count().max(1) as i64,
            // v0.31.0: row count for the named flag, ALL rows incl. empty
            // (R0-r1 I3b), floored to 1 (R0-r2 m1 degenerate-range
            // discipline — `min..=1` stays a valid range when no rows
            // exist yet; the CLI catches the residual).
            NumberMax::FromRowCount(flag_name) => state
                .values
                .iter()
                .filter(|(k, _)| k == flag_name)
                .count()
                .max(1) as i64,
        }
    }
}

/// Per-flag visibility decision returned by a `SubcommandSchema.conditional`
/// function. Phase 5 wires the conditionals; Phase 1 declares the shape.
///
/// v0.6.0 SPEC §6.10.3 v3: gained `PinValue { value }` variant for the v3
/// pin_value Effect. v0.7.0 SPEC §6.10.3 v4: gained `DisableOptions {
/// values }` variant for the v4 disable_options Effect. Dropped `Copy`
/// because `serde_json::Value` isn't Copy; downstream consumers `.clone()`
/// instead. Argv emission semantics per §6.10.4: Hidden/Disabled suppress;
/// Required is decorative; PinValue REPLACES the user-typed value with
/// `value` and emits the pair; DisableOptions is schema-time only (does
/// NOT affect argv emission — stale state values still emit if their
/// option was disabled mid-frame; CLI rows 10/11 catch the residual).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// Render the flag normally.
    Visible,
    /// Hide the flag entirely (slot in the layout still reserved per SPEC
    /// §A I-7 focus-order invariant).
    Hidden,
    /// Render the flag with a required-marker (red asterisk).
    Required,
    /// Render the flag greyed out — exclusive sibling is populated
    /// (SPEC §5 R1 I-3 mutual-exclusion encoding).
    Disabled,
    /// Coerce the flag's argv value to `value`; render the widget as
    /// read-only with a tooltip. SPEC §6.10.3 / §6.10.4 (v3). Argv
    /// emission REPLACES whatever the user typed with this pinned value.
    PinValue { value: serde_json::Value },
    /// Dropdown-only: render the listed option values as greyed-out +
    /// non-selectable. Schema-time only per SPEC §6.10.4 (v4): does NOT
    /// affect argv emission. If the user's `state.values` still holds a
    /// now-disabled value from an earlier frame, argv emits it; CLI
    /// rows 10/11 catch the residual at run time.
    DisableOptions { values: Vec<String> },
}

/// Map flag-name → visibility. Returned by `SubcommandSchema.conditional`.
/// `Vec<(name, vis)>` rather than `HashMap` for deterministic iteration
/// order. (Pre-v0.6.0 the comment also cited Copy-friendliness; Copy on
/// Visibility was dropped in v0.6.0 with the v3 PinValue variant.)
pub type FlagVisibility = Vec<(&'static str, Visibility)>;

/// Per-subcommand form state. Phase 2 wires `values`; Phase 3 wires
/// `slots`.
///
/// Repeating flags (`FlagSchema.repeating == true`) may appear multiple
/// times in `values`; ordering is preserved (slot-index ascending for the
/// SlotEditor path, row-add order for other repeating flags — see SPEC §6.3).
///
/// `slots` is only consulted for subcommands where
/// `SubcommandSchema.allows_slots == true`. The SlotEditor (Phase 3) owns
/// the widget; `assemble_argv` (Phase 2) emits `--slot @N.subkey=value`
/// pairs from this field in slot-index ascending order at the position
/// where `--slot` appears in the schema's flag iteration.
#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct FormState {
    pub values: Vec<(String, FlagValue)>,
    pub slots: crate::form::slot_editor::SlotState,
    /// Positional args in `positional_args` declaration order. For
    /// repeating positionals, multiple entries may share the same
    /// schema index (the form widget renders multiple input rows).
    /// Empty strings are dropped at emit time (SPEC §6.7 parity).
    pub positionals: Vec<String>,
    /// SPEC §3 (v0.2 B.1): secret-bearing widgets keyed by flag name.
    /// Owned by FormState so the lifetime spans the form session; sweeped
    /// to zero on `secrets::zeroize_form_state`. `#[serde(skip)]` enforces
    /// the never-persist invariant by type — serde's deserialize codegen
    /// default-constructs the field to an empty BTreeMap.
    ///
    /// v0.31.1 (repeating-secrets SPEC §0/§1): the value is a
    /// `Vec<SecretLineEdit>` — one entry per ROW of a repeating secret
    /// Text flag (`--ms1`, `--share`); scalar secrets (`--passphrase`,
    /// `--bip38-passphrase`) are the 1-element vec. Row order = visual
    /// order = argv order. Secrets never enter `state.values`; the
    /// never-persist invariant stays TYPE-level.
    ///
    /// FormState's `Clone` derive was removed because `SecretLineEdit`
    /// deliberately does not implement `Clone` (a clone is a second copy
    /// of the secret in memory). No v0.1.1 caller depended on
    /// `FormState::clone()`. See SPEC §3 R1 C-1 fold.
    #[serde(skip)]
    pub secret_widgets:
        std::collections::BTreeMap<String, Vec<crate::form::secret_widget::SecretLineEdit>>,

    /// v0.32.0 (node-tree builder SPEC §1.3): the build-descriptor
    /// recursive tree-builder state. `None` = non-tree mode (and what a
    /// pre-v0.32.0 `state.json` with no `tree` field loads to —
    /// `#[serde(default)]` is the migration leg, R0-r1 M6a). Persisted —
    /// `redact_for_persistence` blanks xprv-matching `key`/`keys`
    /// entries via `TreeState::redacted_for_persistence`; the transient
    /// diagnostics / validate_ok never persist (`#[serde(skip)]` inside
    /// `TreeState`).
    #[serde(default)]
    pub tree: Option<crate::form::tree_model::TreeState>,

    /// v0.32.0 P3 (node-tree builder SPEC §3): transient error from the
    /// last archetype-mode "Edit as tree…" attempt (`--emit-spec` stderr,
    /// spawn failure, or spec-parse failure). Rendered as a label under
    /// the button (`tree_form::render_edit_as_tree`); overwritten or
    /// cleared by the next attempt; NEVER persisted (`#[serde(skip)]` —
    /// the diagnostics-never-persist discipline).
    #[serde(skip)]
    pub edit_as_tree_error: Option<String>,
}

impl FormState {
    pub fn from_pairs<I, S>(iter: I) -> Self
    where
        I: IntoIterator<Item = (S, FlagValue)>,
        S: Into<String>,
    {
        Self {
            values: iter.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            slots: crate::form::slot_editor::SlotState::new(),
            positionals: Vec::new(),
            secret_widgets: std::collections::BTreeMap::new(),
            tree: None,
            edit_as_tree_error: None,
        }
    }

    pub fn with_slots(
        mut self,
        slots: crate::form::slot_editor::SlotState,
    ) -> Self {
        self.slots = slots;
        self
    }

    pub fn with_positionals<I, S>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.positionals = iter.into_iter().map(Into::into).collect();
        self
    }

    /// True iff `name` appears in `values` with a "present" value. Used by
    /// the Phase 5 conditional-visibility engine to check upstream
    /// `conflicts_with` / `required_unless_present_any` constraints.
    /// Semantics:
    ///   - Text / Dropdown / Path: present iff non-empty.
    ///   - Boolean: present iff `true` (matches SPEC §6.7 emission rule:
    ///     Boolean(false) is omitted from argv, so it's "not present").
    ///   - NodeValueComposite: present iff `value` is non-empty.
    ///   - Number / Range / Timestamp / TaggedOrIndexed: always present
    ///     once in the map (no empty-form sentinel).
    pub fn has_value(&self, name: &str) -> bool {
        // v0.31.1 (SPEC §4, R0-r1 C2 — THE silent migration site): the
        // secret_widgets value is now a Vec of per-row widgets, and a bare
        // `!w.is_empty()` would be `Vec::is_empty` — compiling cleanly with
        // INVERTED meaning ("any rows exist" instead of "any row carries
        // bytes"; the required-seed rule keeps `--share` at >= 1 blank row,
        // so a blank form would read as present). Per-row semantics: present
        // iff ANY row's buffer is non-empty.
        self.values
            .iter()
            .any(|(k, v)| k == name && flag_value_is_present(v))
            || self
                .secret_widgets
                .get(name)
                .is_some_and(|rows| rows.iter().any(|w| !w.is_empty()))
    }

    /// v0.2 D.1 N-1: True iff positional `idx` is filled. Used by
    /// conditional fns that gate flags on positional presence
    /// (e.g. md_encode TEMPLATE XOR --from-policy).
    pub fn has_positional(&self, idx: usize) -> bool {
        self.positionals.get(idx).is_some_and(|s| !s.is_empty())
    }

    /// v0.2 D.1 N-2: Return the Dropdown value string for `name`, or
    /// `None` if the flag is absent / has a different FlagValue variant.
    /// Used by conditional fns that gate flags on Dropdown value-inspect
    /// (e.g. md_encode/md_compile gating --unspendable-key on --context).
    pub fn dropdown_value(&self, name: &str) -> Option<&str> {
        self.values.iter().find_map(|(k, v)| {
            if k == name {
                if let FlagValue::Dropdown(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// v0.8.0 Phase 6: Return the `Text` value string for `name`, or
    /// `None` if the flag is absent / has a different FlagValue variant.
    /// Mirrors `dropdown_value`'s shape; used by `is_descriptor_non_canonical`
    /// at `form/conditional.rs` to classify the descriptor string.
    pub fn text_value(&self, name: &str) -> Option<&str> {
        self.values.iter().find_map(|(k, v)| {
            if k == name {
                if let FlagValue::Text(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// v0.8.1 F3: Return the `Number` value (as i64) for `name`, or
    /// `None` if the flag is absent / has a different FlagValue variant.
    /// Mirrors `text_value` + `dropdown_value` shape; used by main.rs to
    /// extract `--account` when threading the non-canonical default-path
    /// hint through to `slot_editor::render`.
    pub fn number_value(&self, name: &str) -> Option<i64> {
        self.values.iter().find_map(|(k, v)| {
            if k == name {
                if let FlagValue::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// v0.6.0 SPEC §6.10.2 v3: total slot-row count. Used by
    /// `slot_count_eq` / `slot_count_gte` / `slot_count_lte` Predicate
    /// evaluation. Mirrors `slot_state.rows.len()` directly — no filter
    /// for empty / partial rows; the SPEC's predicate semantic is "slot
    /// row count" not "populated row count" (the slot-grid widget owns
    /// add/remove discipline).
    pub fn slot_count(&self) -> usize {
        self.slots.rows.len()
    }

    /// v0.3: Return the `node` token of a NodeValueComposite flag, or
    /// `None` if absent / different variant. Mirrors `dropdown_value`'s
    /// shape; used by `slip39_split` conditional to hide `--language`
    /// when `--from` node == entropy.
    pub fn composite_node(&self, name: &str) -> Option<&str> {
        self.values.iter().find_map(|(k, v)| {
            if k == name {
                if let FlagValue::NodeValueComposite { node, .. } = v {
                    Some(node.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

fn flag_value_is_present(v: &FlagValue) -> bool {
    match v {
        FlagValue::Text(s) | FlagValue::Dropdown(s) | FlagValue::Path(s) => !s.is_empty(),
        FlagValue::Boolean(b) => *b,
        FlagValue::NodeValueComposite { value, .. } => !value.is_empty(),
        FlagValue::Number(_)
        | FlagValue::Range(_, _)
        | FlagValue::Timestamp(_)
        | FlagValue::TaggedOrIndexed(_) => true,
        // v0.6.0 P3: Unset sentinel — explicit "no value" state for Number /
        // Range / Timestamp / TaggedOrIndexed widgets (which lack a natural
        // empty-string default). Conditional fns (`has_value`) and the argv
        // assembler treat Unset as absent.
        FlagValue::Unset => false,
    }
}

/// Typed value mirroring `FlagKind`. The form widget renderer holds these
/// per-flag; the argv assembler consumes them per the SPEC §6.7 emission
/// rules.
///
/// v0.6.0 P3 adds the `Unset` sentinel for Number / Range / Timestamp /
/// TaggedOrIndexed widgets. Previously those widgets auto-seeded to a
/// concrete value (e.g. `Number(min)`) the moment a user navigated the
/// form, which meant the argv assembler emitted `--<flag> <min>` for any
/// numeric flag the user hadn't touched — surfacing as bogus flags
/// downstream. With Unset, the same widget appears in the form with a
/// `Set` affordance and is only seeded when the user clicks. `#[serde(other)]`
/// keeps v0.6+ readers forward-compatible against future tag additions
/// (maps any unrecognized tag to Unset). Persistence-schema break for
/// older v0.5 readers — see CHANGELOG [0.6.0].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FlagValue {
    Text(String),
    Number(i64),
    Dropdown(String),
    Boolean(bool),
    /// Comma-separated `<u32>,<u32>` form. SPEC §6.7.
    Range(u32, u32),
    Timestamp(TimestampValue),
    /// `--name <node>=<value>` composite. SPEC §6.7.
    NodeValueComposite { node: String, value: String },
    TaggedOrIndexed(TaggedOrIndexedValue),
    /// File path. Empty → omitted at emit time. `"-"` is emitted verbatim
    /// when the flag's `FlagKind::Path { stdio_sentinel }` is true; the
    /// stdio-sentinel decision belongs to the schema, not the value.
    Path(String),
    /// v0.6.0 P3: explicit "no value" sentinel. Used as the initial form-
    /// state value for Number / Range / Timestamp / TaggedOrIndexed widgets
    /// (the four kinds without a natural empty-string default). The widget
    /// renders a `Set` affordance instead of an auto-seeded numeric value;
    /// clicking it transitions to the kind-specific seeded value. The argv
    /// assembler skips Unset (no emission); `flag_value_is_present` returns
    /// `false`; conditional fns see the flag as absent.
    ///
    /// `#[serde(other)]` makes v0.6+ readers map any unknown tag in
    /// state.json to Unset — protecting forward compat against future
    /// FlagValue additions.
    #[serde(other)]
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimestampValue {
    Now,
    Unix(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaggedOrIndexedValue {
    Tag(String),
    Indexed(u8),
}
