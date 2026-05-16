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

/// SPEC §6.10.7 single-sig template set, mirrored from
/// `mnemonic-toolkit::CliTemplate::is_multisig()` source-of-truth at
/// `crates/mnemonic-toolkit/src/template.rs:46-56`. Parity with the
/// toolkit's emitted `dropdown_value_in.values` set is enforced by the
/// drift gate test at `tests/gui_schema_conditional_drift.rs`.
///
/// v0.6.0 (SPEC §6.10.8 v3): the toolkit now also emits
/// `meta.template_groups.single_sig` per template-consuming subcommand;
/// `tests/schema_mirror.rs::single_sig_templates_const_matches_meta_template_groups`
/// asserts this const matches the toolkit's meta block. This pair-of-checks
/// posture (drift gate for the per-rule projection + const-vs-meta parity
/// for the bulk list) closes FOLLOWUP `gui-schema-template-groups-meta-field`
/// without coupling conditional-fn purity to a runtime subprocess fetch.
/// The const remains the runtime source-of-truth; the meta block is the
/// SPEC source-of-truth; the parity test gates them.
pub const SINGLE_SIG_TEMPLATES: &[&str] = &["bip44", "bip49", "bip84", "bip86"];

/// SPEC §6.10.7 taproot-multi-leaf template set (templates that require an
/// explicit internal key). Same source-of-truth + drift-gate posture as
/// `SINGLE_SIG_TEMPLATES`.
pub const TAPROOT_INTERNAL_KEY_TEMPLATES: &[&str] = &["tr-multi-a", "tr-sortedmulti-a"];

/// v0.6.0 P4 — return the template-aware default flag values for a given
/// `--template` selection. Single-sig templates have no template-specific
/// defaults (the universal defaults at the form-state seed already cover
/// `--network` / `--account`). Multisig templates default `--threshold = 2`
/// AND `--multisig-path-family = bip48` so the form is one-click-runnable
/// after the user picks a multisig template.
///
/// The egui-frame hook in `main.rs` consumes this on `--template` change,
/// applying defaults ONLY to flags that aren't already set (seed-on-empty
/// pattern — preserves any value the user explicitly typed across template
/// switches). The visibility gate handles the inverse direction (single-sig
/// template → Disabled threshold/path-family); this helper handles the
/// "fresh form ergonomics" direction.
///
/// SPEC reference: §6.10.7 row T1 (THRESHOLD_WITHOUT_MULTISIG),
/// row T2 (PATH_FAMILY_WITHOUT_MULTISIG). The defaults match
/// `mnemonic-toolkit::cmd::bundle`'s implicit "what to set for multisig
/// templates" — `bip48` is the canonical multisig path family;
/// threshold-of-2 is the smallest non-degenerate threshold.
pub fn template_defaults_for(template: &str) -> Vec<(&'static str, crate::schema::FlagValue)> {
    use crate::schema::FlagValue;
    if SINGLE_SIG_TEMPLATES.contains(&template) {
        // Single-sig: no template-specific defaults.
        Vec::new()
    } else {
        // All other templates are multisig (or external like --descriptor
        // mode, which doesn't use --template). Seed canonical defaults.
        vec![
            ("--threshold", FlagValue::Number(2)),
            ("--multisig-path-family", FlagValue::Dropdown("bip48".into())),
        ]
    }
}

fn template_is_in(state: &FormState, names: &[&str]) -> bool {
    state
        .dropdown_value("--template")
        .map(|v| names.contains(&v))
        .unwrap_or(false)
}

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
    let template_is_single_sig = template_is_in(state, SINGLE_SIG_TEMPLATES);

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
    // v0.16.0 SPEC §6.10.7: descriptor-mode disables --template /
    // --threshold / --multisig-path-family. Emit BEFORE the single-sig
    // template rules so first-rule-wins per `main.rs:391-394` picks the
    // more-specific predicate. SPEC §6.6 row 2 +
    // bundle.rs::mode_text::{DESCRIPTOR_AND_TEMPLATE, DESCRIPTOR_WITH_THRESHOLD,
    // DESCRIPTOR_WITH_PATH_FAMILY}.
    if has_descriptor {
        vis.push(("--template", Visibility::Disabled));
        vis.push(("--threshold", Visibility::Disabled));
        vis.push(("--multisig-path-family", Visibility::Disabled));
        // v0.17.0 SPEC §6.10.7 row 12 (DESCRIPTOR_WITH_NONZERO_ACCOUNT):
        // --account is pinned to 0 when --descriptor is present (the
        // descriptor encodes the account in @i origin paths). PinValue
        // REPLACES user-typed value in argv per §6.10.4 emission table —
        // distinct from Disabled which would suppress.
        vis.push((
            "--account",
            Visibility::PinValue {
                value: serde_json::json!(0),
            },
        ));
    }
    // v0.16.0 SPEC §6.10.7: single-sig template disables --threshold /
    // --multisig-path-family. SPEC §6.6 rows T1 + T2 +
    // bundle.rs::mode_text::{THRESHOLD_WITHOUT_MULTISIG, PATH_FAMILY_WITHOUT_MULTISIG}.
    if template_is_single_sig {
        vis.push(("--threshold", Visibility::Disabled));
        vis.push(("--multisig-path-family", Visibility::Disabled));
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
    let template_is_single_sig = template_is_in(state, SINGLE_SIG_TEMPLATES);

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
    // v0.16.0 SPEC §6.10.7: --template disabled when --descriptor present
    // (mirrors bundle rule). SPEC §6.6 row 2 (verify-bundle mirror).
    if has_descriptor {
        vis.push(("--template", Visibility::Disabled));
    }
    // v0.16.0 SPEC §6.10.7: single-sig template disables --threshold
    // (mirrors bundle rule T1).
    if template_is_single_sig {
        vis.push(("--threshold", Visibility::Disabled));
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
    let template_is_single_sig = template_is_in(state, SINGLE_SIG_TEMPLATES);
    let template_needs_tr_internal_key = template_is_in(state, TAPROOT_INTERNAL_KEY_TEMPLATES);

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
    // v0.16.0 SPEC §6.10.7: --taproot-internal-key is Required when the
    // chosen template is a taproot multi-leaf (tr-multi-a / tr-sortedmulti-a)
    // and Disabled otherwise. Required-rule emit BEFORE Disabled so
    // first-rule-wins honours the more-specific predicate.
    if template_needs_tr_internal_key {
        vis.push(("--taproot-internal-key", Visibility::Required));
    } else {
        vis.push(("--taproot-internal-key", Visibility::Disabled));
    }
    // v0.16.0 SPEC §6.10.7: single-sig template disables --threshold +
    // --multisig-path-family (mirrors bundle rule T1 + T2).
    if template_is_single_sig {
        vis.push(("--threshold", Visibility::Disabled));
        vis.push(("--multisig-path-family", Visibility::Disabled));
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
    let application_is_dice = state
        .dropdown_value("--application")
        .map(|v| v == "dice")
        .unwrap_or(false);

    if has_passphrase {
        vis.push(("--passphrase-stdin", Visibility::Disabled));
    }
    if has_passphrase_stdin {
        vis.push(("--passphrase", Visibility::Disabled));
    }
    // v0.16.0 SPEC §6.10.7: --dice-sides is Required when --application is
    // set to "dice" (mirrors cmd/derive_child.rs clap-derive required_if_eq).
    if application_is_dice {
        vis.push(("--dice-sides", Visibility::Required));
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
/// Upstream mutual-exclusion: `--origin-fingerprint` ↔ `--privacy-preserving`.
/// The mk-cli source does NOT carry a clap `conflicts_with`; the constraint
/// is enforced by a runtime guard at `mk-cli/src/cmd/encode.rs:58-62`. The
/// help-text + doc-comments phrase it as "Mutually exclusive", which renders
/// in `--help` but does not add a clap conflict (manual-gui batch-8 R0 catch).
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
