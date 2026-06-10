//! Window-state + form-state round-trip via `state.json` (SPEC §B.10).
//!
//! Persisted (SPEC §B.10):
//!   - `last_cli_tab` + `last_subcommand_per_tab`
//!   - `window_size`, `window_position`
//!   - `show_cmdline` / `show_stdout` / `show_stderr` toggles
//!   - Per-subcommand non-secret form values
//!   - Watch-only slot rows (subkey NOT in `SECRET_SLOT_SUBKEYS`)
//!
//! NEVER persisted (SPEC §10 R1 I-4; v0.4.0+ taxonomy imported from
//! `mnemonic_toolkit::secret_taxonomy`):
//!   - Any flag whose name is in `SECRET_FLAG_NAMES`
//!   - Any `values` entry whose flag name is schema-`secret: true`
//!     anywhere across the 4 CLI schemas (v0.31.1 SPEC §3 — the
//!     `secrets::schema_secret_flag_names()` name-level net)
//!   - `NodeValueComposite` entries where `node` is in `SECRET_NODE_TYPES`
//!   - Slot rows whose subkey is in `SECRET_SLOT_SUBKEYS`
//!
//! Schema version mismatch → rename `state.json` to `state.json.bak` and
//! a fresh default is loaded (non-fatal).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app::CliTab;
use crate::form::slot_editor::SlotState;
use crate::schema::{FlagValue, FormState};
use crate::secrets::{SECRET_FLAG_NAMES, SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};

/// Current persistence schema version. Bump when the on-disk shape
/// changes incompatibly; loader writes `.bak` on mismatch.
pub const SCHEMA_VERSION: u32 = 1;

/// On-disk shape — what's actually written to `state.json`.
///
/// `Clone` was removed in v0.2 Phase B.1 (SPEC §3 R2 C-1 fold).
/// `BTreeMap<String, FormState>: Clone` requires `V: Clone`, and `FormState`
/// dropped its `Clone` derive when `SecretLineEdit` (which deliberately
/// does not implement `Clone`) was added. No v0.1.1 call site depended on
/// `PersistedState::clone()` (verified by source scan); `redact_for_persistence`
/// constructs the redacted value field-by-field rather than cloning.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PersistedState {
    pub schema_version: u32,
    pub last_cli_tab: String,
    pub last_subcommand_per_tab: BTreeMap<String, String>,
    pub window_size: Option<[f32; 2]>,
    pub window_position: Option<[f32; 2]>,
    #[serde(default = "default_show_true")]
    pub show_cmdline: bool,
    #[serde(default = "default_show_true")]
    pub show_stdout: bool,
    #[serde(default = "default_show_true")]
    pub show_stderr: bool,
    /// Key form: `"<cli>:<subcommand>"`, e.g. `"mnemonic:bundle"`.
    pub form_state_per_subcommand: BTreeMap<String, FormState>,
}

fn default_show_true() -> bool {
    true
}

/// Filter a `FormState` to drop every entry the GUI must never persist.
/// SPEC §B.10 never-persist set:
///   - flags in `SECRET_FLAG_NAMES`
///   - any flag name declared `secret: true` ANYWHERE across the 4 CLI
///     schemas (`secrets::schema_secret_flag_names()` — v0.31.1 SPEC §3
///     belt-and-suspenders; a NAME-level net, not a guarantee that
///     secrets never enter `values`)
///   - `NodeValueComposite` whose `node` is in `SECRET_NODE_TYPES`
///   - slot rows whose subkey is in `SECRET_SLOT_SUBKEYS`
pub fn redact_for_persistence(state: &FormState) -> FormState {
    let schema_secret_names = crate::secrets::schema_secret_flag_names();
    let values: Vec<(String, FlagValue)> = state
        .values
        .iter()
        .filter(|(k, v)| {
            // Drop secret-class flags.
            if SECRET_FLAG_NAMES.contains(&k.as_str()) {
                return false;
            }
            // v0.31.1 (SPEC §3): drop any entry whose flag name is
            // schema-secret anywhere. Closes the FUTURE-drift class (any
            // later code path writing a secret-NAMED value is redacted at
            // persist) and incidentally closes two live plaintext leaks
            // via cross-CLI name collisions (`--phrase` on xpub-search,
            // `--ms1` on `ms repair` — see schema_secret_flag_names docs).
            if schema_secret_names.contains(&k.as_str()) {
                return false;
            }
            // Drop secret-class NodeValueComposite entries.
            if let FlagValue::NodeValueComposite { node, .. } = v {
                if SECRET_NODE_TYPES.contains(&node.as_str()) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    let slot_rows = state
        .slots
        .rows
        .iter()
        .filter(|r| !SECRET_SLOT_SUBKEYS.contains(&r.subkey.as_str()))
        .cloned()
        .collect();

    FormState {
        values,
        slots: SlotState { rows: slot_rows },
        // v0.34.0 (audit I5): drop ALL positionals at persist. This layer
        // has no subcommand context, so name-aware positional redaction is
        // impossible here -- the belt is unconditional / fail-closed. Secret
        // positionals never reach state.positionals anyway (they live in
        // secret_widgets under "positional:<name>" -- type-level
        // never-persist); non-secret positionals are cheap re-pastes.
        // Serde-shape compatible: no SCHEMA_VERSION bump (no state.json
        // exists in the wild -- save/load have never had callers).
        positionals: Vec::new(),
        // SPEC §3 / v0.2 Phase B.1: secret_widgets is never persisted
        // (#[serde(skip)]) and is freshly default-constructed here so
        // the redacted FormState has the never-persist invariant
        // satisfied by type.
        secret_widgets: BTreeMap::new(),
        // v0.32.0 (node-tree builder SPEC §1.3): tree persists with every
        // node's `key`/`keys` entries BLANKED when xprv-matching (the
        // toolkit gate.rs heuristic — strip `[origin]` via rsplit(']'),
        // bytes 1..4 == b"prv"; recursive walk incl. surplus children).
        // Hashlock `hex` digests deliberately survive (public
        // commitments); diagnostics/validate_ok are absent by type.
        tree: state
            .tree
            .as_ref()
            .map(crate::form::tree_model::TreeState::redacted_for_persistence),
        // v0.32.0 P3: the "Edit as tree…" error is transient by type
        // (#[serde(skip)]) — freshly None here, same posture as
        // secret_widgets.
        edit_as_tree_error: None,
    }
}

/// Apply `redact_for_persistence` to every form state in
/// `PersistedState`. Returns a new `PersistedState` with the redacted
/// entries. The original (in-memory) `PersistedState` is not mutated.
pub fn redact_persisted_state(state: &PersistedState) -> PersistedState {
    let form_state_per_subcommand: BTreeMap<String, FormState> = state
        .form_state_per_subcommand
        .iter()
        .map(|(k, v)| (k.clone(), redact_for_persistence(v)))
        .collect();
    PersistedState {
        schema_version: state.schema_version,
        last_cli_tab: state.last_cli_tab.clone(),
        last_subcommand_per_tab: state.last_subcommand_per_tab.clone(),
        window_size: state.window_size,
        window_position: state.window_position,
        show_cmdline: state.show_cmdline,
        show_stdout: state.show_stdout,
        show_stderr: state.show_stderr,
        form_state_per_subcommand,
    }
}

/// Schema lookup by tab. Replicates the bin-private
/// `MnemonicGuiApp::schema_for` (main.rs) so the restore-validation
/// helper below is lib-side and reachable from `tests/` (v0.35.0 SPEC
/// T2 / R0-r1 M3).
fn schema_for(tab: CliTab) -> &'static crate::schema::Schema {
    match tab {
        CliTab::Mnemonic => &crate::schema::mnemonic::SCHEMA,
        CliTab::Md => &crate::schema::md::SCHEMA,
        CliTab::Ms => &crate::schema::ms::SCHEMA,
        CliTab::Mk => &crate::schema::mk::SCHEMA,
    }
}

/// Per-tab default subcommand — mirrors the hardcoded seed in
/// `MnemonicGuiApp::new` (`bundle` for mnemonic, `inspect` for the three
/// codec tabs).
fn default_subcommand(tab: CliTab) -> &'static str {
    match tab {
        CliTab::Mnemonic => "bundle",
        CliTab::Md | CliTab::Ms | CliTab::Mk => "inspect",
    }
}

/// Validate + map the persisted tab/subcommand selections onto live app
/// selections (v0.35.0 Phase-8 wiring, SPEC Decision 2).
///
/// - `last_cli_tab` parses via [`CliTab::from_bin_name`]; unknown names
///   OR tabs whose binary is unavailable (`avail` returns `false` — the
///   caller passes `AppState::tab_available`) fall back to
///   [`CliTab::Mnemonic`].
/// - Each `last_subcommand_per_tab` entry is validated against that
///   tab's schema `subcommands`; invalid/missing entries fall back to the
///   per-tab default (`bundle`/`inspect`). Every tab gets an entry, so a
///   `PersistedState::default()` input reproduces exactly the pre-wiring
///   hardcoded selections.
pub fn restore_selections(
    state: &PersistedState,
    avail: impl Fn(CliTab) -> bool,
) -> (CliTab, BTreeMap<CliTab, String>) {
    let tab = CliTab::from_bin_name(&state.last_cli_tab)
        .filter(|t| avail(*t))
        .unwrap_or(CliTab::Mnemonic);
    let subcommands = CliTab::ALL
        .iter()
        .map(|t| {
            let restored = state
                .last_subcommand_per_tab
                .get(t.bin_name())
                .filter(|s| {
                    schema_for(*t)
                        .subcommands
                        .iter()
                        .any(|sub| sub.name == s.as_str())
                })
                .cloned()
                .unwrap_or_else(|| default_subcommand(*t).to_string());
            (*t, restored)
        })
        .collect();
    (tab, subcommands)
}

/// Serialize + write the redacted state to `path`. The on-disk JSON
/// NEVER contains secret-class entries.
///
/// R1 I-1 fold: `schema_version` is stamped to `SCHEMA_VERSION`
/// unconditionally regardless of the caller-supplied value. This makes
/// `save()` self-contained — callers cannot accidentally write a stale
/// or zero version (which would cause the next `load()` to rename the
/// file to `.bak` and silently discard state on cold start).
pub fn save(state: &PersistedState, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut redacted = redact_persisted_state(state);
    redacted.schema_version = SCHEMA_VERSION;
    let body = serde_json::to_string_pretty(&redacted)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, body)
}

/// Read + parse `state.json` from `path`. Returns `Some(state)` on
/// success. On schema-version mismatch OR malformed JSON, renames
/// `<path>` to `<path with .json.bak extension>` and returns `None`
/// (caller writes a fresh default; the bad file is preserved for
/// diagnosis instead of being silently overwritten at the next save —
/// v0.35.0 SPEC Decision 5). A missing file returns plain `None`.
pub fn load(path: &Path) -> Option<PersistedState> {
    let raw = fs::read_to_string(path).ok()?;
    let parsed: PersistedState = match serde_json::from_str(&raw) {
        Ok(p) => p,
        Err(_) => {
            // v0.35.0: symmetric with the version-mismatch leg below —
            // preserve the corrupt file (e.g. a save torn by the signal
            // handler's 3 s process::exit grace) as `.json.bak`.
            let bak = path.with_extension("json.bak");
            let _ = fs::rename(path, &bak);
            return None;
        }
    };
    if parsed.schema_version != SCHEMA_VERSION {
        let bak = path.with_extension("json.bak");
        let _ = fs::rename(path, &bak);
        return None;
    }
    Some(parsed)
}

/// Convenience for the GUI's `directories::ProjectDirs`-based config-dir
/// lookup. SPEC §10: `<config_dir>/state.json`. Returns `None` if no
/// platform config dir is available (rare; treated as "don't persist").
///
/// v0.35.0 (Phase-8 wiring, SPEC Decision 6): when the
/// `MNEMONIC_GUI_STATE_PATH` env var is set to a non-empty value, it
/// overrides the platform config-dir resolution and is used verbatim as
/// the state-file path. This is user-visible production behavior (point
/// the GUI at an alternate state file / a throwaway path) and the test
/// seam (integration tests never touch the real config dir).
pub fn default_state_path() -> Option<std::path::PathBuf> {
    if let Ok(overridden) = std::env::var("MNEMONIC_GUI_STATE_PATH") {
        if !overridden.is_empty() {
            return Some(std::path::PathBuf::from(overridden));
        }
    }
    let dirs = directories::ProjectDirs::from("org", "mnemonic-gui", "mnemonic-gui")?;
    Some(dirs.config_dir().join("state.json"))
}
