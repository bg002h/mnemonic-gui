//! Secret-handling modals + zeroize sweep (SPEC §9 + §10 + Phase 7).
//!
//! `SECRET_NODE_TYPES` and `SECRET_SLOT_SUBKEYS` are generated at build
//! time by `build.rs` from the upstream `NodeType::is_secret_bearing()`
//! and `SlotSubkey::is_secret_bearing()` match-arm sets (SPEC §10 R1 I-4 +
//! Phase 7 R2 C-1 codegen). The runtime `tests/schema_mirror.rs` audit
//! re-parses the upstream files and asserts set-equality against these
//! constants — drift surfaces as a test failure.
//!
//! Modal copy texts are pinned byte-exact per SPEC §9.

include!(concat!(env!("OUT_DIR"), "/secrets_generated.rs"));

use zeroize::Zeroize;

use crate::form::slot_editor::SlotSubkey;
use crate::schema::{FlagSchema, SubcommandSchema};

/// Hard-coded flag names with no upstream is-secret accessor (SPEC §10
/// R1 I-4 fold). The GUI hand-maintains these; the schema_mirror gate
/// keeps the `NodeType` / `SlotSubkey` sets aligned with upstream.
pub const SECRET_FLAG_NAMES: &[&str] = &[
    "--passphrase",
    "--bip38-passphrase",
    "--passphrase-stdin",
];

/// True iff `flag` is secret-bearing in v0.1. Checks both
/// `FlagSchema.secret` and the hand-maintained `SECRET_FLAG_NAMES` set.
pub fn flag_is_secret(flag: &FlagSchema) -> bool {
    flag.secret || SECRET_FLAG_NAMES.contains(&flag.name)
}

/// True iff `subkey` is in the build.rs-generated secret-class set.
pub fn slot_subkey_is_secret(subkey: SlotSubkey) -> bool {
    SECRET_SLOT_SUBKEYS.contains(&subkey.as_str())
}

/// True iff `node` (e.g. `"phrase"`, `"xpub"`) is in the secret-class set.
pub fn node_type_is_secret(node: &str) -> bool {
    SECRET_NODE_TYPES.contains(&node)
}

/// Paste-warn modal copy text (SPEC §9 — one-shot per session, per
/// secret-class flag). Byte-exact per SPEC §9; the `(v0.2 deferred per
/// FOLLOWUPS ...)` lines name the explicit non-mitigations.
pub const PASTE_WARN_MODAL_TEXT: &str = "\
You are pasting potentially secret material (BIP-39 phrase, entropy hex, WIF, xprv,
ms1 share, BIP-38 ciphertext, or Electrum native seed).

This GUI runs the binary with the value you pasted as a command-line argument.
The OS may log this argument in process tables and shell history files. macOS
and Windows now suppress OS screenshot APIs (App Switcher / Task View) for this
window. Linux remains unmitigated; see FOLLOWUPS
`gui-os-snapshot-secret-occlusion`.

The GUI holds the primary secret buffer in `Zeroizing<Vec<u8>>`, zeroed on drop.
Transient `String` copies used for argv assembly are also wrapped in `Zeroizing`
per call. egui's internal undo ring retains `String` snapshots that this scheme
does not cover — a second-tier residue documented in FOLLOWUPS
`gui-secret-buffer-allocator-residue`.";

/// Run-confirm modal prefix (the full argv preview follows in body
/// rendering at the call site).
pub const RUN_CONFIRM_MODAL_PREFIX: &str = "\
This invocation passes secret-bearing arguments to ";

/// Minimum paste length that triggers the paste-warn modal (SPEC §9).
pub const PASTE_WARN_THRESHOLD: usize = 8;

/// Decide whether a paste-event into the given flag's widget should
/// trigger the paste-warn modal: paste length ≥ threshold AND the flag
/// is in the secret class.
pub fn should_warn_on_paste(flag: &FlagSchema, paste_len: usize) -> bool {
    flag_is_secret(flag) && paste_len >= PASTE_WARN_THRESHOLD
}

/// Decide whether a Run-button click should trigger the run-confirm
/// modal: any secret-class flag in the subcommand has a non-empty value.
pub fn should_confirm_run(
    subcommand: &SubcommandSchema,
    state: &crate::schema::FormState,
) -> bool {
    // Check flag-level secrets via the FlagSchema flag (or hand-coded set).
    for flag in subcommand.flags {
        if flag_is_secret(flag) && state.has_value(flag.name) {
            return true;
        }
    }
    // Check slot-level secrets via SECRET_SLOT_SUBKEYS.
    for row in &state.slots.rows {
        if !row.value.is_empty() && slot_subkey_is_secret(row.subkey) {
            return true;
        }
    }
    // Check NodeValueComposite values — secrecy is value-dependent.
    for (_, v) in &state.values {
        if let crate::schema::FlagValue::NodeValueComposite { node, value } = v {
            if !value.is_empty() && node_type_is_secret(node) {
                return true;
            }
        }
    }
    false
}

/// Best-effort zero-on-drop wrapper for secret-bearing String buffers.
/// `Zeroize` overwrites the bytes with 0; the heap allocator may still
/// hold residue (v0.2 deferred per FOLLOWUPS
/// `gui-secret-buffer-allocator-residue`).
#[derive(Default, Debug)]
pub struct SecretBuffer(pub String);

impl SecretBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_text(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Zeroize for SecretBuffer {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for SecretBuffer {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// App-exit sweep — call from main.rs / app shutdown. Walks the form
/// state and zeroes every secret-class buffer. Best-effort caveat per
/// SPEC §9 (allocator residue + OS snapshots unaffected; deferred to v0.2).
pub fn zeroize_form_state(state: &mut crate::schema::FormState) {
    for (_, v) in &mut state.values {
        match v {
            crate::schema::FlagValue::Text(s)
            | crate::schema::FlagValue::Dropdown(s)
            | crate::schema::FlagValue::Path(s) => {
                s.zeroize();
            }
            crate::schema::FlagValue::NodeValueComposite { value, .. } => {
                value.zeroize();
            }
            _ => {}
        }
    }
    for row in &mut state.slots.rows {
        row.value.zeroize();
    }
    for pos in &mut state.positionals {
        pos.zeroize();
    }
    // v0.2 Phase B.1: sweep SecretLineEdit buffers. Each widget owns a
    // `Zeroizing<Vec<u8>>` (zeroed on Drop), but the explicit zeroize()
    // call here drops the bytes ahead of the BTreeMap's own Drop pass,
    // ensuring the secret memory is overwritten as early as possible on
    // app shutdown.
    for widget in state.secret_widgets.values_mut() {
        widget.zeroize();
    }
}
