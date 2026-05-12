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
    /// Pinned upstream tag (e.g. `"mnemonic-toolkit-v0.8.1"`). Source of
    /// truth for the schema-mirror CI gate (SPEC §11).
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

/// Per-subcommand form state. Phase 5 elaborates the contents; Phase 1
/// declares the shape so `conditional` function pointers can typecheck.
#[derive(Default)]
pub struct FormState {
    /// One entry per flag the user has populated in the form. Phase 5 wires
    /// real population; Phase 1 keeps this empty.
    pub values: Vec<(&'static str, String)>,
}
