//! Per-subcommand conditional-visibility engine (SPEC §5).
//!
//! Each function pointer takes `&FormState` and returns a `FlagVisibility`
//! map listing the per-flag visibility OVERRIDES (Required / Disabled /
//! Hidden); flags absent from the map default to `Visible`.
//!
//! The constraints encoded here mirror the upstream clap-derive
//! `conflicts_with` / `required_unless_present_any` attributes in
//! `crates/mnemonic-toolkit/src/cmd/*.rs`. Phase 5's
//! `tests/conditional_visibility.rs` enumerates EVERY active constraint
//! as a discrete test cell — drift surfaces as a test failure.

use crate::schema::{FlagVisibility, FormState, Visibility};

/// `bundle` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/bundle.rs`):
///   :25 `--template` required_unless_present_any = ["descriptor", "descriptor_file"]
///   :30 `--descriptor` conflicts_with = "descriptor_file"
pub fn bundle(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_descriptor = state.has_value("--descriptor");
    let has_descriptor_file = state.has_value("--descriptor-file");

    if !has_descriptor && !has_descriptor_file {
        vis.push(("--template", Visibility::Required));
    }
    if has_descriptor_file {
        vis.push(("--descriptor", Visibility::Disabled));
    }
    if has_descriptor {
        vis.push(("--descriptor-file", Visibility::Disabled));
    }
    vis
}

/// `verify-bundle` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`):
///   :28 `--template`     required_unless_present_any = ["descriptor", "descriptor_file"]
///   :32 `--descriptor`   conflicts_with = "descriptor_file"
///   :54 `--ms1`          conflicts_with = "bundle_json"
///   :57 `--mk1`          required_unless_present = "bundle_json", conflicts_with = "bundle_json"
///   :60 `--md1`          required_unless_present = "bundle_json", conflicts_with = "bundle_json"
///   :67 `--bundle-json`  conflicts_with_all = ["ms1", "mk1", "md1"]
pub fn verify_bundle(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_descriptor = state.has_value("--descriptor");
    let has_descriptor_file = state.has_value("--descriptor-file");
    let has_bundle_json = state.has_value("--bundle-json");
    let has_ms1 = state.has_value("--ms1");
    let has_mk1 = state.has_value("--mk1");
    let has_md1 = state.has_value("--md1");
    let any_card = has_ms1 || has_mk1 || has_md1;

    // Descriptor-side mutual-required-one-of + XOR.
    if !has_descriptor && !has_descriptor_file {
        vis.push(("--template", Visibility::Required));
    }
    if has_descriptor_file {
        vis.push(("--descriptor", Visibility::Disabled));
    }
    if has_descriptor {
        vis.push(("--descriptor-file", Visibility::Disabled));
    }

    // Card-side mutual-exclusion: --bundle-json XOR (--ms1, --mk1, --md1).
    if has_bundle_json {
        vis.push(("--ms1", Visibility::Disabled));
        vis.push(("--mk1", Visibility::Disabled));
        vis.push(("--md1", Visibility::Disabled));
    } else {
        // mk1 and md1 are required_unless bundle_json; ms1 is not required
        // unconditionally upstream — it's only `conflicts_with bundle_json`.
        vis.push(("--mk1", Visibility::Required));
        vis.push(("--md1", Visibility::Required));
    }
    if any_card {
        vis.push(("--bundle-json", Visibility::Disabled));
    }

    vis
}

/// `convert` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/convert.rs`):
///   :181 `--passphrase-stdin` conflicts_with = "passphrase"
pub fn convert(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");

    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }
    vis
}

/// `export-wallet` subcommand conditionals.
///
/// Upstream:
///   `cmd/export_wallet.rs:43`      — `--template conflicts_with = "descriptor"`
///   `cmd/export_wallet.rs:215-219` — runtime pre-check: neither flag set → BadInput
///                                    "export-wallet requires either --template or --descriptor"
///
/// Phase 5 R1 I-1 fold: model BOTH the clap conflicts_with AND the runtime
/// required-one-of pre-check, since the upstream help text already labels
/// the pair "Mutually-required-one-of." (Same posture would apply to any
/// similar runtime pre-check in a future subcommand.)
pub fn export_wallet(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_descriptor = state.has_value("--descriptor");
    let has_template = state.has_value("--template");

    if has_descriptor {
        vis.push(("--template", Visibility::Disabled));
    }
    if has_template {
        vis.push(("--descriptor", Visibility::Disabled));
    }
    // Runtime pre-check (export_wallet.rs:215-219): if neither is set,
    // upstream refuses with BadInput. Mark both as Required so the form
    // signals the constraint pre-Run.
    if !has_descriptor && !has_template {
        vis.push(("--template", Visibility::Required));
        vis.push(("--descriptor", Visibility::Required));
    }
    vis
}

// `derive-child` has no clap conflicts_with / required_unless_present
// constraints in v0.8.1 — all required flags are at clap-level (`--from`,
// `--application`, `--length`, `--index`). No conditional fn needed.

/// v0.2 D.2: `ms encode` conditionals.
///
/// Upstream: `--phrase` XOR `--hex` (required_one_of with mutual
/// exclusion); `--language` is ignored when `--hex` is supplied.
pub fn ms_encode(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_phrase = state.has_value("--phrase");
    let has_hex = state.has_value("--hex");
    if has_phrase {
        vis.push(("--hex", Visibility::Disabled));
    }
    if has_hex {
        vis.push(("--phrase", Visibility::Disabled));
        // --language is ignored when --hex is supplied (upstream help).
        vis.push(("--language", Visibility::Hidden));
    }
    if !has_phrase && !has_hex {
        vis.push(("--phrase", Visibility::Required));
        vis.push(("--hex", Visibility::Required));
    }
    vis
}

/// v0.2 D.2: `mk encode` conditionals.
///
/// Upstream: `--origin-fingerprint` conflicts_with `--privacy-preserving`
/// (bidirectional, explicit in help).
pub fn mk_encode(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_fp = state.has_value("--origin-fingerprint");
    let has_priv = state.has_value("--privacy-preserving");
    if has_fp {
        vis.push(("--privacy-preserving", Visibility::Disabled));
    }
    if has_priv {
        vis.push(("--origin-fingerprint", Visibility::Disabled));
    }
    vis
}
