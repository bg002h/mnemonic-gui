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
///   :51 (v0.13.0 drift fix) `--passphrase-stdin` conflicts_with = "passphrase"
pub fn bundle(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_descriptor = state.has_value("--descriptor");
    let has_descriptor_file = state.has_value("--descriptor-file");
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");

    if !has_descriptor && !has_descriptor_file {
        vis.push(("--template", Visibility::Required));
    }
    if has_descriptor_file {
        vis.push(("--descriptor", Visibility::Disabled));
    }
    if has_descriptor {
        vis.push(("--descriptor-file", Visibility::Disabled));
    }
    // v0.3 drift fix: passphrase XOR passphrase-stdin.
    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }
    vis
}

/// `verify-bundle` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`):
///   :28 `--template`     required_unless_present_any = ["descriptor", "descriptor_file"]
///   :32 `--descriptor`   conflicts_with = "descriptor_file"
///   :51 (v0.13.0 drift fix) `--passphrase-stdin` conflicts_with = "passphrase"
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
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");
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

    // v0.3 drift fix: passphrase XOR passphrase-stdin.
    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }

    vis
}

/// `convert` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/convert.rs`):
///   :181 `--passphrase-stdin` conflicts_with = "passphrase"
///   :203 (v0.13.0 drift fix) `--bip38-passphrase-stdin` conflicts_with = "bip38_passphrase"
pub fn convert(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");
    let has_bip38_passphrase = state.has_value("--bip38-passphrase");
    let has_bip38_passphrase_stdin = state.has_value("--bip38-passphrase-stdin");

    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }
    // v0.3 drift fix: bip38-passphrase XOR bip38-passphrase-stdin
    // (additive; the bip38 pair is independent of the non-bip38 pair).
    if has_bip38_passphrase {
        vis.push(("--bip38-passphrase-stdin", Visibility::Disabled));
    }
    if has_bip38_passphrase_stdin {
        vis.push(("--bip38-passphrase", Visibility::Disabled));
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

/// `derive-child` subcommand conditionals (v0.3 drift fix).
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/derive_child.rs`):
///   :68 `--passphrase-stdin` conflicts_with = "passphrase"
///
/// (v0.8.1 had no clap conflicts; v0.13.0 adds `--passphrase-stdin`.
/// SubcommandSchema entry flips `conditional: None` → `Some(...)` in
/// schema/mnemonic.rs.)
pub fn derive_child(state: &FormState) -> FlagVisibility {
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

/// v0.2 D.3: `md encode` conditionals.
///
/// Upstream constraints (md encode --help + md-cli source):
/// - `[TEMPLATE]` positional XOR `--from-policy` (runtime pre-check;
///   neither clap-required individually).
/// - `--context` is conditionally required when `--from-policy` is set.
/// - `--unspendable-key` is rejected when `--context` value == "segwitv0"
///   (value-inspect, not presence-check).
/// - `--key` and `--fingerprint` are template-placeholder substitutions;
///   irrelevant when the positional template is filled (which already
///   includes resolved keys).
pub fn md_encode(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_template_pos = state.has_positional(0);
    let has_from_policy = state.has_value("--from-policy");

    if has_template_pos {
        vis.push(("--from-policy", Visibility::Disabled));
        vis.push(("--context", Visibility::Hidden));
        vis.push(("--unspendable-key", Visibility::Hidden));
    }
    if has_from_policy {
        // The positional template input slot would conflict if filled.
        vis.push(("--context", Visibility::Required));
    }
    if !has_template_pos && !has_from_policy {
        // Neither input mode chosen — both Required for the runtime
        // pre-check to pass.
        vis.push(("--from-policy", Visibility::Required));
        // (positional Required marker is handled by PositionalArgSchema,
        // not FlagVisibility — leave to widget layer.)
    }
    // --unspendable-key value-disabled by --context (D.1 finding #2,
    // first dropdown-value-inspect conditional in the codebase).
    if state.dropdown_value("--context") == Some("segwitv0") {
        vis.push(("--unspendable-key", Visibility::Disabled));
    }
    vis
}

/// v0.2 D.3: `md compile` conditionals.
///
/// Upstream: `--unspendable-key` rejected when `--context` is "segwitv0".
/// `--context` is clap-required, so no `Required` marker needed here.
pub fn md_compile(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    if state.dropdown_value("--context") == Some("segwitv0") {
        vis.push(("--unspendable-key", Visibility::Disabled));
    }
    vis
}

/// v0.2 D.3: `md address` conditionals.
///
/// Upstream constraints (md address --help):
/// - `[PHRASES]` positional XOR `--template`.
/// - `--key` and `--fingerprint` require `--template` (they substitute
///   into the template's `@i` placeholders); disabled when the positional
///   is filled.
/// - `--change` / `--chain` relationship: help describes `--change` as
///   "Sugar for --chain 1". Upstream clap `conflicts_with` not confirmed
///   from help; left out of conditional fn pending md-cli source audit.
pub fn md_address(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_phrases_pos = state.has_positional(0);
    let has_template = state.has_value("--template");

    if has_phrases_pos {
        vis.push(("--template", Visibility::Disabled));
        vis.push(("--key", Visibility::Disabled));
        vis.push(("--fingerprint", Visibility::Disabled));
    }
    if !has_phrases_pos && !has_template {
        vis.push(("--template", Visibility::Required));
        // positional Required handled at widget layer.
    }
    vis
}

/// v0.3: `slip39-split` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/slip39.rs`):
///   :101 `--passphrase` conflicts_with = "passphrase_stdin"
///   :131 `--language`   doc: "BIP-39 language of input phrase; ignored
///                              for `entropy=` inputs" — Hidden when
///                              `--from` node == entropy.
pub fn slip39_split(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");

    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }

    // --language is input-side only (BIP-39 parsing of `--from phrase=…`);
    // ignored when `--from entropy=…`. Hide when entropy mode.
    if state.composite_node("--from") == Some("entropy") {
        vis.push(("--language", Visibility::Hidden));
    }
    vis
}

/// v0.3: `slip39-combine` subcommand conditionals.
///
/// Upstream (`cmd/slip39.rs`):
///   :157 `--passphrase` conflicts_with = "passphrase_stdin"
///   :170 `--language`   doc: "BIP-39 language for `--to phrase`; ignored
///                              for `--to entropy`" — Hidden when
///                              `--to` == entropy (the default).
pub fn slip39_combine(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    let has_passphrase = state.has_value("--passphrase");
    let has_passphrase_stdin = state.has_value("--passphrase-stdin");

    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }

    // --language used only when --to == "phrase"; ignored for entropy
    // (the default). Hidden when entropy mode (matches `md_encode` precedent
    // for --language Hidden-when-form-irrelevant).
    let to = state.dropdown_value("--to");
    if to == Some("entropy") || to.is_none() {
        vis.push(("--language", Visibility::Hidden));
    }
    vis
}
