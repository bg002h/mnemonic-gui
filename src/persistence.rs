//! Window-state + form-state round-trip via `state.json` (SPEC §B.10).
//!
//! Persisted (SPEC §B.10):
//!   - `last_cli_tab` + `last_subcommand_per_tab`
//!   - `window_size`, `window_position`
//!   - `show_cmdline` / `show_stdout` / `show_stderr` toggles
//!   - Per-subcommand non-secret form values
//!   - Watch-only slot rows (subkey NOT in `SECRET_SLOT_SUBKEYS`)
//!
//! NEVER persisted (SPEC §10 R1 I-4 + Phase 7 build.rs codegen):
//!   - Any flag whose name is in `SECRET_FLAG_NAMES`
//!   - `NodeValueComposite` entries where `node` is in `SECRET_NODE_TYPES`
//!   - Slot rows whose subkey is in `SECRET_SLOT_SUBKEYS`
//!
//! Schema version mismatch → rename `state.json` to `state.json.bak` and
//! a fresh default is loaded (non-fatal).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::form::slot_editor::SlotState;
use crate::schema::{FlagValue, FormState};
use crate::secrets::{SECRET_FLAG_NAMES, SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};

/// Current persistence schema version. Bump when the on-disk shape
/// changes incompatibly; loader writes `.bak` on mismatch.
pub const SCHEMA_VERSION: u32 = 1;

/// On-disk shape — what's actually written to `state.json`.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
///   - `NodeValueComposite` whose `node` is in `SECRET_NODE_TYPES`
///   - slot rows whose subkey is in `SECRET_SLOT_SUBKEYS`
pub fn redact_for_persistence(state: &FormState) -> FormState {
    let values: Vec<(String, FlagValue)> = state
        .values
        .iter()
        .filter(|(k, v)| {
            // Drop secret-class flags.
            if SECRET_FLAG_NAMES.contains(&k.as_str()) {
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
        positionals: state.positionals.clone(),
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
/// success. On schema-version mismatch, renames `<path>` to
/// `<path>.bak` and returns `None` (caller writes a fresh default).
/// On any other error (missing file, malformed JSON), returns `None`.
pub fn load(path: &Path) -> Option<PersistedState> {
    let raw = fs::read_to_string(path).ok()?;
    let parsed: PersistedState = match serde_json::from_str(&raw) {
        Ok(p) => p,
        Err(_) => return None,
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
pub fn default_state_path() -> Option<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("org", "mnemonic-gui", "mnemonic-gui")?;
    Some(dirs.config_dir().join("state.json"))
}
