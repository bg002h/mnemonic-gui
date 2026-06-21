//! Secret-handling modals + zeroize sweep (SPEC §9 + §10 + Phase 7).
//!
//! `SECRET_NODE_TYPES` and `SECRET_SLOT_SUBKEYS` are re-exported from
//! `mnemonic_toolkit::secret_taxonomy` (v0.14.0+). The toolkit owns
//! the single source of truth via per-variant parity tests on its
//! private `NodeType::is_secret_bearing()` /
//! `SlotSubkey::is_secret_bearing()` predicates; the GUI consumes the
//! canonical taxonomy as a library import. See toolkit FOLLOWUPS entry
//! `secret-taxonomy-public-api-promotion` for the full architectural
//! rationale.
//!
//! Prior to v0.4.0 the GUI's `build.rs` (now deleted) scraped the
//! toolkit's private source via `syn::parse_file`. That codegen path
//! emitted empty `&[]` arrays under `cargo install --git` (cargo's
//! sandbox has no adjacent toolkit checkout), silently disabling
//! `persistence::redact_for_persistence` and leaking BIP-39 phrases to
//! `~/.config/mnemonic-gui/state.json` in plaintext (HIGH-severity bug
//! in v0.3.0..v0.3.2; tactically patched in v0.3.3 with a committed
//! fallback in build.rs; structurally retired here).
//!
//! ### Belt-and-suspenders (one-cycle overlap, v0.4.0 only)
//!
//! Per the architect's recommendation in the toolkit FOLLOWUP, this
//! release retains the v0.3.3 `CANONICAL_FALLBACK_*` snapshot AND
//! adds a compile-time assertion that the snapshot equals the
//! toolkit-imported constants. This catches a supply-chain class of
//! regression where the GUI's pinned toolkit tag could (in principle)
//! resolve to a build with empty / unexpected `SECRET_*` arrays. The
//! snapshot will be removed in v0.5.0 once the new contract has been
//! exercised through one release cycle.
//!
//! Modal copy texts are pinned byte-exact per SPEC §9.

pub use mnemonic_toolkit::secret_taxonomy::{
    SECRET_NODE_TYPES, SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS,
};

/// One-cycle belt-and-suspenders snapshot. Removed in v0.5.0.
mod v0_3_canonical_fallback {
    /// Snapshot of `SECRET_NODE_TYPES` as it shipped in mnemonic-gui
    /// v0.3.3 (committed fallback in the now-deleted `build.rs`).
    /// Asserted byte-equal to the toolkit-imported `SECRET_NODE_TYPES`
    /// at compile time below.
    pub const SECRET_NODE_TYPES: &[&str] = &[
        "phrase",
        "entropy",
        "xprv",
        "wif",
        "ms1",
        "bip38",
        "electrum-phrase",
        // v0.16.2 (toolkit v0.31.6): `seedqr` added as a NodeType (decodes
        // to a BIP-39 phrase; secret-bearing). Closes the supply-chain
        // drift fired by the toolkit pin bump v0.31.3 → v0.31.6.
        "seedqr",
    ];

    /// Snapshot of `SECRET_SLOT_SUBKEYS`. Maintained in lockstep with
    /// toolkit pin bumps that change the secret-class set.
    /// History:
    /// - v0.3.3 baseline: `["phrase", "entropy", "xprv", "wif"]`
    /// - v0.16.1 (toolkit v0.31.3): `seedqr` added (new SlotSubkey
    ///   variant; secret-bearing because it decodes to a BIP-39 phrase).
    /// - v0.22.0 (toolkit v0.41.0): `ms1` added (new SlotSubkey variant;
    ///   secret-bearing — a raw BIP-93 codex32 secret decoded inline to
    ///   entropy). Lockstep with toolkit `--slot @N.ms1=`. The
    ///   compile-time drift guard below stays consistent only once the
    ///   `mnemonic-toolkit` Cargo.toml pin is bumped to >= v0.41.0.
    pub const SECRET_SLOT_SUBKEYS: &[&str] =
        &["phrase", "seedqr", "entropy", "ms1", "xprv", "wif"];
}

// Compile-time supply-chain guard: the v0.3.3 snapshot must equal the
// toolkit-imported constants. If a future GUI Cargo.toml tag bump
// resolves to a toolkit release with different SECRET_* arrays, this
// `const _: () = assert!(...)` block fails to compile and the maintainer
// must explicitly acknowledge the change (drift gate). To drop the
// belt-and-suspenders in v0.5.0, delete `v0_3_canonical_fallback` mod
// and these two assertions.
const _: () = assert!(
    secret_slice_eq(
        SECRET_NODE_TYPES,
        v0_3_canonical_fallback::SECRET_NODE_TYPES,
    ),
    "supply-chain drift: mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES \
     diverged from the v0.3.3 committed snapshot. Either update \
     v0_3_canonical_fallback::SECRET_NODE_TYPES to match the new toolkit \
     pin (and document the change in CHANGELOG), or revert the toolkit \
     dep tag bump.",
);
const _: () = assert!(
    secret_slice_eq(
        SECRET_SLOT_SUBKEYS,
        v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS,
    ),
    "supply-chain drift: mnemonic_toolkit::secret_taxonomy::SECRET_SLOT_SUBKEYS \
     diverged from the v0.3.3 committed snapshot. Either update \
     v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS to match the new toolkit \
     pin (and document the change in CHANGELOG), or revert the toolkit \
     dep tag bump.",
);

/// Const-compatible `&[&str]` equality. Stable Rust does not implement
/// `==` for slices in `const` context; this is the standard workaround.
const fn secret_slice_eq(a: &[&str], b: &[&str]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !const_str_eq(a[i], b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

const fn const_str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

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

/// True iff `subkey` is in the toolkit-imported secret-class set
/// (`mnemonic_toolkit::secret_taxonomy::SECRET_SLOT_SUBKEYS`).
pub fn slot_subkey_is_secret(subkey: SlotSubkey) -> bool {
    SECRET_SLOT_SUBKEYS.contains(&subkey.as_str())
}

/// True iff `node` (e.g. `"phrase"`, `"xpub"`) is in the NARROW secret-class
/// set (`SECRET_NODE_TYPES`). Kept for any persistence-narrow semantics; the
/// argv/redaction surfaces use [`node_type_is_argv_secret`] (the wider set).
pub fn node_type_is_secret(node: &str) -> bool {
    SECRET_NODE_TYPES.contains(&node)
}

/// True iff `node` is in the WIDER argv/redaction secret set
/// (`SECRET_NODE_TYPES_ARGV` = narrow + `minikey`). cycle-3 H3: use for
/// argv-mask / run-confirm / persistence-redact / paste-warn. The toolkit ships
/// the two sets as INTENTIONALLY distinct (`minikey`, a Casascius mini private
/// key, is argv-secret but is omitted from the narrow persistence-historical
/// set); two named predicates keep each call site self-documenting about which
/// semantic it wants and prevent a "simplify back to one set" re-narrowing.
pub fn node_type_is_argv_secret(node: &str) -> bool {
    SECRET_NODE_TYPES_ARGV.contains(&node)
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
    // v0.34.0 (audit I5): check secret POSITIONALS — their rows live in
    // secret_widgets under the "positional:<name>" reserved key.
    for pos in subcommand.positional_args {
        if pos.secret {
            if let Some(rows) = state.secret_widgets.get(&format!("positional:{}", pos.name)) {
                if rows.iter().any(|w| !w.is_empty()) {
                    return true;
                }
            }
        }
    }
    // Check NodeValueComposite values — secrecy is node-dependent. cycle-3 H3:
    // route through the WIDE set so a `--from minikey=…` confirms.
    for (_, v) in &state.values {
        if let crate::schema::FlagValue::NodeValueComposite { node, value } = v {
            if !value.is_empty() && node_type_is_argv_secret(node) {
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
    //
    // v0.31.1 (SPEC §4, R0-r1 I1): per-row sweep — the map value is now
    // `Vec<SecretLineEdit>` (one widget per repeating-secret row), so the
    // sweep flattens over every row of every flag.
    for widget in state.secret_widgets.values_mut().flatten() {
        widget.zeroize();
    }
}

/// v0.31.1 (SPEC §3): the union of every flag NAME declared `secret: true`
/// in ANY of the 4 CLI schemas, plus the hand-maintained
/// [`SECRET_FLAG_NAMES`]. Extracted from the schema STRUCTS' `secret`
/// FIELD (never a text grep — R0-r3 m-NEW2: comment lines in mnemonic.rs
/// contain the literal `secret: true`).
///
/// Consumed by `persistence::redact_for_persistence` as a belt-and-
/// suspenders NAME-level net: any `state.values` entry under a
/// secret-anywhere name is dropped at persist. This is NOT a guarantee
/// that secrets never enter `values` — it is a name net. Deliberate side
/// effects (SPEC §3):
/// - the 6 Boolean `secret: true` `*-stdin` toggles join the union, so
///   their persisted checkbox state resets across restarts (accepted —
///   the `--passphrase-stdin` precedent; a stale-persisted stdin toggle
///   is itself a foot-gun);
/// - the cross-CLI name collisions `--phrase` (3 mnemonic xpub-search
///   `secret: false` sites) and `--ms1` (`ms repair`, `secret: false`)
///   silently stop persisting — accepted and WELCOMED: both were live
///   plaintext-persistence leaks of master-secret material (FOLLOWUPs
///   `xpub-search-inline-phrase-not-secret-classified` +
///   `ms-repair-ms1-not-secret-classified` track the underlying
///   mis-classifications).
pub fn schema_secret_flag_names() -> &'static std::collections::BTreeSet<&'static str> {
    static NAMES: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names: std::collections::BTreeSet<&'static str> =
            SECRET_FLAG_NAMES.iter().copied().collect();
        for schema in [
            &crate::schema::mnemonic::SCHEMA,
            &crate::schema::md::SCHEMA,
            &crate::schema::ms::SCHEMA,
            &crate::schema::mk::SCHEMA,
        ] {
            for sub in schema.subcommands {
                for flag in sub.flags {
                    if flag.secret {
                        names.insert(flag.name);
                    }
                }
            }
        }
        names
    })
}
