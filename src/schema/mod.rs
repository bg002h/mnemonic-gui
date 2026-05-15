//! Schema types — the static description of each CLI's flag surface, used
//! by the form renderer (Phase 2), conditional-visibility engine (Phase 5),
//! argv assembler (Phase 2 §6), and the schema-mirror CI gate (Phase 9).
//!
//! See SPEC §B.3 for the normative type definitions.

pub mod md;
pub mod mk;
pub mod mnemonic;
pub mod ms;

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
    pub secret: bool,
}

/// Widget shape + argv emission rule. See SPEC §6.7 for byte-exact emission
/// rules per variant.
pub enum FlagKind {
    /// Free-form text.
    Text,
    /// Integer with inclusive bounds.
    Number { min: i64, max: i64 },
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

/// Per-flag visibility decision returned by a `SubcommandSchema.conditional`
/// function. Phase 5 wires the conditionals; Phase 1 declares the shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

/// Map flag-name → visibility. Returned by `SubcommandSchema.conditional`.
/// `Vec<(name, vis)>` rather than `HashMap` to keep the struct `Copy`-
/// friendly downstream and the iteration order deterministic.
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
    /// FormState's `Clone` derive was removed because `SecretLineEdit`
    /// deliberately does not implement `Clone` (a clone is a second copy
    /// of the secret in memory). No v0.1.1 caller depended on
    /// `FormState::clone()`. See SPEC §3 R1 C-1 fold.
    #[serde(skip)]
    pub secret_widgets: std::collections::BTreeMap<String, crate::form::secret_widget::SecretLineEdit>,
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
        self.values
            .iter()
            .any(|(k, v)| k == name && flag_value_is_present(v))
            || self
                .secret_widgets
                .get(name)
                .is_some_and(|w| !w.is_empty())
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
    }
}

/// Typed value mirroring `FlagKind`. The form widget renderer holds these
/// per-flag; the argv assembler consumes them per the SPEC §6.7 emission
/// rules.
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
