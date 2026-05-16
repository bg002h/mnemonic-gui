//! Runtime soft-check + `--gui-schema` JSON consumer.
//!
//! v0.1: `--version` runtime soft-check stub (placeholder).
//! v0.2 Phase C.1 (SPEC §7): `--gui-schema` JSON consumer. The GUI
//! shells out to `<cli> gui-schema` when `pinned-upstream.toml`'s
//! per-CLI `gui-schema-capable` field is `true`, parses the emitted
//! JSON per SPEC §7, and returns the per-subcommand flag-name set.
//! When `gui-schema-capable` is `false`, returns `None` and the
//! caller falls back to the v0.1 regex-on-`--help` path
//! (`tests/schema_mirror.rs::help_text_flag_names`).

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

/// v0.1 stub kept for backward compatibility — no caller relies on
/// this function and it can be removed in a future cleanup phase.
pub fn placeholder() {}

/// Per-CLI metadata from `pinned-upstream.toml` (subset — only the
/// fields C.1 needs).
#[derive(Debug, Deserialize)]
struct PinnedCliEntry {
    /// Path-resolution: `bin` is invoked through `$PATH`. The
    /// `<CLI_uppercase>_BIN` env var override (e.g., `MD_BIN`) wins.
    bin: String,
    /// SPEC §7: when `true`, `json_flag_names` consults `<cli>
    /// gui-schema`; when `false`, returns `None`. `#[serde(default)]`
    /// makes pre-v0.2 `pinned-upstream.toml` files (which lack this
    /// key) deserialize cleanly to `false` — no migration required
    /// for v0.1.1 → v0.2.0 readers.
    #[serde(default, rename = "gui-schema-capable")]
    gui_schema_capable: bool,
}

#[derive(Debug, Deserialize)]
struct PinnedRoot {
    mnemonic: Option<PinnedCliEntry>,
    md: Option<PinnedCliEntry>,
    ms: Option<PinnedCliEntry>,
    mk: Option<PinnedCliEntry>,
}

impl PinnedRoot {
    fn entry(&self, cli_name: &str) -> Option<&PinnedCliEntry> {
        match cli_name {
            "mnemonic" => self.mnemonic.as_ref(),
            "md" => self.md.as_ref(),
            "ms" => self.ms.as_ref(),
            "mk" => self.mk.as_ref(),
            _ => None,
        }
    }
}

/// Read `pinned-upstream.toml` from the repo root. Returns `None` on
/// any I/O or parse error (the caller treats `None` as
/// "fall back to regex path").
fn load_pinned_upstream() -> Option<PinnedRoot> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("pinned-upstream.toml");
    let body = std::fs::read_to_string(&path).ok()?;
    toml::from_str::<PinnedRoot>(&body).ok()
}

/// SPEC §7 / §6.10 JSON shape for `<cli> gui-schema` output. The struct
/// describes both v1 and v2 docs (v2 adds `conditional_rules` per subcommand;
/// older fields unchanged). Version gating is done in the parse fns.
#[derive(Debug, Deserialize)]
struct GuiSchemaRoot {
    version: u32,
    #[allow(dead_code)]
    cli: String,
    subcommands: Vec<GuiSchemaSubcommand>,
}

#[derive(Debug, Deserialize)]
struct GuiSchemaSubcommand {
    name: String,
    #[serde(default)]
    flags: Vec<GuiSchemaFlag>,
    /// SPEC §6.10 conditional-applicability projection (v2+). `#[serde(default)]`
    /// makes v1 docs (which lack this field) deserialize cleanly to an empty
    /// Vec — preserving the parse_gui_schema_json fast path.
    #[serde(default)]
    conditional_rules: Vec<ConditionalRule>,
}

#[derive(Debug, Deserialize)]
struct GuiSchemaFlag {
    name: String,
    // Other fields (required, kind, choices) intentionally not
    // deserialized — C.1 only needs the flag-name set for the
    // schema-mirror invariant. The bootstrap-import code path (which
    // does consume kind/choices) is a separate concern.
}

/// SPEC §6.10 ConditionalRule — projection of a single CLI mutex / conditional
/// rule into the GUI's per-frame visibility computation. v2+ schema.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ConditionalRule {
    pub rationale: String,
    pub spec_ref: String,
    pub when: Predicate,
    pub effect: Effect,
}

/// SPEC §6.10.2 Predicate AST (tagged JSON union).
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Predicate {
    FlagPresent {
        flag: String,
    },
    DropdownValueIn {
        flag: String,
        values: Vec<String>,
    },
    CompositeNodeIs {
        flag: String,
        node: String,
    },
    PositionalPresent {
        index: usize,
    },
    AllOf {
        predicates: Vec<Predicate>,
    },
    AnyOf {
        predicates: Vec<Predicate>,
    },
    Not {
        predicate: Box<Predicate>,
    },
}

/// SPEC §6.10.3 Effect.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Effect {
    pub flag: String,
    pub visibility: VisibilityProjection,
}

/// SPEC §6.10.3 VisibilityProjection. `Visible` is the implicit default and
/// never appears as an Effect value.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum VisibilityProjection {
    Hidden,
    Disabled,
    Required,
}

/// Parse `<cli> gui-schema` JSON for a specific subcommand. Returns
/// the per-subcommand flag-name set. Returns `None` if:
///   - The JSON does not parse.
///   - `version < 1` (illegal value; GUI rejects).
///   - The subcommand is not present in the JSON.
///
/// v2 (SPEC §6.10.6) bumps the schema version 1 → 2 additively. v1 consumers
/// of THIS function continue to work on v2 docs — flag-name extraction
/// ignores the new `conditional_rules` field.
///
/// Pure function — does NOT shell out. Testable from unit tests with
/// synthetic JSON.
pub fn parse_gui_schema_json(
    json: &str,
    subcommand_name: &str,
) -> Option<BTreeSet<String>> {
    let root: GuiSchemaRoot = serde_json::from_str(json).ok()?;
    if root.version < 1 {
        tracing::warn!(
            "gui-schema JSON version unsupported: got {}, expected >= 1; falling back",
            root.version
        );
        return None;
    }
    let sub = root.subcommands.iter().find(|s| s.name == subcommand_name)?;
    Some(sub.flags.iter().map(|f| f.name.clone()).collect())
}

/// Parse `<cli> gui-schema` JSON for the per-subcommand SPEC §6.10
/// `conditional_rules` array. Returns `None` if:
///   - The JSON does not parse.
///   - `version < 2` (the field is only emitted by v2+ producers).
///   - The subcommand is not present in the JSON.
///
/// Companion to `parse_gui_schema_json` (the flag-name extractor), gated at
/// v2 minimum because pre-v2 docs lack the `conditional_rules` field
/// entirely. v0.16.0 toolkit emits v2 docs; the drift gate test consumes
/// this output.
pub fn parse_gui_schema_conditional_rules(
    json: &str,
    subcommand_name: &str,
) -> Option<Vec<ConditionalRule>> {
    let root: GuiSchemaRoot = serde_json::from_str(json).ok()?;
    if root.version < 2 {
        return None;
    }
    let sub = root.subcommands.iter().find(|s| s.name == subcommand_name)?;
    Some(sub.conditional_rules.clone())
}

/// SPEC §7 / Phase C.1: shell out to `<cli> gui-schema` (when the
/// CLI's `gui-schema-capable` flag is `true` in `pinned-upstream.toml`),
/// parse the JSON output, and return the per-subcommand flag-name set.
/// Returns `None` if:
///   - `gui-schema-capable = false` for the CLI (caller falls back to
///     the v0.1 regex-on-`--help` path).
///   - The CLI is not in `pinned-upstream.toml`.
///   - The binary cannot be invoked.
///   - The output does not parse (logged at `tracing::warn!`).
///   - `version != 1` in the JSON (logged at `tracing::warn!`).
///
/// Binary lookup: `<CLI_uppercase>_BIN` env var wins; otherwise the
/// `bin` field from `pinned-upstream.toml` is invoked through `$PATH`.
pub fn json_flag_names(
    cli_name: &str,
    subcommand_name: &str,
) -> Option<BTreeSet<String>> {
    let pinned = load_pinned_upstream()?;
    let entry = pinned.entry(cli_name)?;
    if !entry.gui_schema_capable {
        return None;
    }

    let bin_env = format!("{}_BIN", cli_name.to_uppercase());
    let bin = std::env::var(&bin_env).unwrap_or_else(|_| entry.bin.clone());

    let output = Command::new(&bin).arg("gui-schema").output().ok()?;
    if !output.status.success() {
        tracing::warn!(
            "{} gui-schema exited non-zero ({}); falling back to regex path",
            bin,
            output.status
        );
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    parse_gui_schema_json(&stdout, subcommand_name)
}
