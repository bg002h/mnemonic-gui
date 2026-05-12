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
    /// True for `bundle` / `verify-bundle` / `export-wallet` — subcommands
    /// that accept the `--slot @N.<subkey>=<value>` repeating grammar.
    pub allows_slots: bool,
    /// Optional conditional-visibility function. Phase 5 fills these in;
    /// Phase 1 leaves them all `None`.
    pub conditional: Option<fn(&FormState) -> FlagVisibility>,
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

/// Per-subcommand form state. Phase 2 fills in the typed-value carrier.
///
/// Repeating flags (`FlagSchema.repeating == true`) may appear multiple
/// times in `values`; ordering is preserved (slot-index ascending for the
/// SlotEditor path, row-add order for other repeating flags — see SPEC §6.3).
#[derive(Default, Debug, Clone)]
pub struct FormState {
    pub values: Vec<(String, FlagValue)>,
}

impl FormState {
    pub fn from_pairs<I, S>(iter: I) -> Self
    where
        I: IntoIterator<Item = (S, FlagValue)>,
        S: Into<String>,
    {
        Self {
            values: iter.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }
}

/// Typed value mirroring `FlagKind`. The form widget renderer holds these
/// per-flag; the argv assembler consumes them per the SPEC §6.7 emission
/// rules.
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimestampValue {
    Now,
    Unix(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaggedOrIndexedValue {
    Tag(String),
    Indexed(u8),
}
