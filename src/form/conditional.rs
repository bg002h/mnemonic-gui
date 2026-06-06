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

use crate::schema::{FlagValue, FlagVisibility, FormState, Visibility};

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

/// SPEC §6.10.8 multisig template set (mirror partition of
/// `SINGLE_SIG_TEMPLATES`). Source-of-truth: toolkit
/// `crates/mnemonic-toolkit/src/template.rs::CliTemplate::is_multisig()`
/// returning `true`. v0.7.0 SPEC §6.10.7 v3-cycle uses this list for
/// the bundle row-11 `disable_options` visibility projection
/// (`slot_count_eq: 1` → `--template` multisig options greyed out).
/// Parity with the toolkit's `meta.template_groups.multisig` is gated
/// by `tests/schema_mirror.rs` (sibling of the SINGLE_SIG parity test).
pub const MULTISIG_TEMPLATES: &[&str] = &[
    // Order matches toolkit `CliTemplate::value_variants()` iteration
    // order (template.rs:25-40). The drift gate's per-rule check
    // compares the GUI's emitted Vec<String> bit-exactly with the
    // toolkit's emitted JSON array, so order must be preserved.
    "wsh-multi",
    "wsh-sortedmulti",
    "sh-wsh-multi",
    "sh-wsh-sortedmulti",
    "tr-multi-a",
    "tr-sortedmulti-a",
];

/// SPEC §6.10.7 taproot-multi-leaf template set (templates that require an
/// explicit internal key). Same source-of-truth + drift-gate posture as
/// `SINGLE_SIG_TEMPLATES`.
pub const TAPROOT_INTERNAL_KEY_TEMPLATES: &[&str] = &["tr-multi-a", "tr-sortedmulti-a"];

/// v0.8.0 Phase 6 — descriptor canonicity classification (SPEC §4.12.a).
///
/// `Canonical` means the descriptor matches one of the 5 wrapper shapes in
/// md-codec's `canonical_origin` table (`canonical_origin.rs:45-79`). For
/// canonical descriptors, the toolkit's `gui-schema`-emitted
/// `DESCRIPTOR_WITH_NONZERO_ACCOUNT` rule (pin `--account` to 0) applies
/// unchanged. For `NonCanonical`, the GUI's `bundle()` Option-A override
/// LIFTS the pin so the user-typed `--account N` flows through to the
/// toolkit (which consumes it per SPEC §4.12.b default-path inference).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Canonicity {
    /// Descriptor matches a canonical wrapper shape (pkh/wpkh/tr-keypath/
    /// wsh-multi-sortedmulti/sh-wsh-multi-sortedmulti); canonical_origin
    /// table supplies the per-shape default origin path.
    Canonical,
    /// Descriptor is non-canonical (bare wsh(@N), wsh(miniscript-body),
    /// tr with TapTree, legacy sh(sortedmulti), etc.); default-path
    /// inference per SPEC §4.12.b applies (toolkit-side).
    NonCanonical,
}

/// v0.8.0 Phase 6 — classify a descriptor string by wrapper shape.
///
/// **Implementation: wrapper-form regex match** (R2 #5 fold from V4 plan-doc).
/// 5 patterns mirror md-codec's `canonical_origin` table verbatim; anything
/// else is `NonCanonical`. Regexes are wrapper-form prefix matches — they
/// don't care what's nested inside the canonical wrappers; the inner
/// miniscript body (if any) doesn't affect classification. The strip of
/// optional `[fp/path]` annotations is implicit because the wrapper-form
/// matches only look at the wrapper prefix.
///
/// **Empty descriptor** classifies as `NonCanonical` — the GUI may call this
/// before the user has typed anything; the banner-emission logic
/// (`descriptor_non_canonical_default_path_notice`) gates on whether the
/// user has supplied a descriptor at all, suppressing the banner when the
/// field is empty.
///
/// **Drift gate:** `tests/canonicity_drift.rs` exercises a corpus of
/// canonical + non-canonical fixtures + shells out to
/// `mnemonic gui-schema --classify-descriptor` to confirm the GUI's view
/// matches the toolkit's (R2 #2 fold from V4 plan-doc).
pub fn classify_descriptor_canonicity(desc: &str) -> Canonicity {
    use std::sync::OnceLock;
    static RES: OnceLock<[regex::Regex; 5]> = OnceLock::new();
    let res = RES.get_or_init(|| {
        [
            // pkh(@N) single-key — pkh( + optional [fp/...] + @<digit>+ +
            // optional multipath `/<...>` + optional `/*` or `/**` wildcard
            // (BIP-388 shorthand) + close paren.
            regex::Regex::new(r"^pkh\((?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?@\d+(?:/<[0-9;]+>)?(?:/\*+'?h?)?\)$").expect("static regex"),
            // wpkh(@N)
            regex::Regex::new(r"^wpkh\((?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?@\d+(?:/<[0-9;]+>)?(?:/\*+'?h?)?\)$").expect("static regex"),
            // tr(@N) key-path-only — single arg, no comma, no TapTree
            regex::Regex::new(r"^tr\((?:\[[0-9a-fA-F]{8}(?:/\d+'?h?)*\])?@\d+(?:/<[0-9;]+>)?(?:/\*+'?h?)?\)$").expect("static regex"),
            // wsh(multi(...)) or wsh(sortedmulti(...))
            regex::Regex::new(r"^wsh\((?:multi|sortedmulti)\(").expect("static regex"),
            // sh(wsh(multi(...))) or sh(wsh(sortedmulti(...)))
            regex::Regex::new(r"^sh\(wsh\((?:multi|sortedmulti)\(").expect("static regex"),
        ]
    });
    if desc.is_empty() {
        return Canonicity::NonCanonical;
    }
    if res.iter().any(|re| re.is_match(desc)) {
        Canonicity::Canonical
    } else {
        Canonicity::NonCanonical
    }
}

/// v0.8.0 Phase 6 — `is_descriptor_non_canonical(state)` is the
/// canonicity-gating predicate used by `bundle()` to LIFT the
/// `--account → PinValue(0)` push when descriptor is non-canonical.
///
/// Returns true ONLY when `--descriptor` is supplied AND its value
/// classifies as `NonCanonical`. Returns false when `--descriptor` is
/// absent (so the existing pin still fires under template mode +
/// `--descriptor-file` — those modes preserve the v0.17.0 behavior).
fn is_descriptor_non_canonical(state: &FormState) -> bool {
    state
        .text_value("--descriptor")
        .map(|s| classify_descriptor_canonicity(s) == Canonicity::NonCanonical)
        .unwrap_or(false)
}

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
        // --account is pinned to 0 when --descriptor is present AND the
        // descriptor is canonical. PinValue REPLACES user-typed value in
        // argv per §6.10.4 emission table — distinct from Disabled which
        // would suppress.
        //
        // v0.8.0 Phase 6 R4 C1 fold (SPEC §6.10.7 row 12 amendment) — the
        // pin is canonicity-gated. Non-canonical descriptors (per SPEC
        // §4.12.a; classified GUI-side via `classify_descriptor_canonicity`)
        // CONSUME `--account N` for §4.12.b default-path inference; the
        // pin would silently coerce N to 0, defeating that feature. So the
        // pin lifts in non-canonical mode. The toolkit's gui-schema-emitted
        // rule remains unchanged (still encoded; drift-gate continues to
        // assert its presence); this Option-A override is GUI-internal.
        if !is_descriptor_non_canonical(state) {
            vis.push((
                "--account",
                Visibility::PinValue {
                    value: serde_json::json!(0),
                },
            ));
        }
    }
    // v0.16.0 SPEC §6.10.7: single-sig template disables --threshold /
    // --multisig-path-family. SPEC §6.6 rows T1 + T2 +
    // bundle.rs::mode_text::{THRESHOLD_WITHOUT_MULTISIG, PATH_FAMILY_WITHOUT_MULTISIG}.
    if template_is_single_sig {
        vis.push(("--threshold", Visibility::Disabled));
        vis.push(("--multisig-path-family", Visibility::Disabled));
    }
    // v0.7.1 introduced row 10/11 DisableOptions pushes here; v0.7.2
    // reverted them. Row 11 was a design flaw: slot_count == 1 is the
    // natural TRANSIENT state during multisig setup (slots added one
    // at a time, passing through 1 on the way to 2+). Disabling
    // multisig templates at that transient state prevented users from
    // selecting their intended template before completing slot setup.
    // Row 10 had the symmetric flaw on multisig→single-sig switches.
    // Template/slot_count mismatch detection migrated to
    // `template_slot_count_warning` + an inline warning banner
    // rendered in `main.rs` adjacent to the slot grid (Option A
    // pattern matching v0.7.1 row-8 slot-contiguity check).
    vis
}

/// v0.7.2 SPEC §6.6 rows 10/11 — GUI-internal template/slot_count
/// mismatch detector. Returns `Some(warning_text)` when the chosen
/// `--template` value combined with the current `slot_count` would
/// fail CLI row 10 or row 11 at runtime; `None` otherwise.
///
/// The returned text is rendered as an inline warning banner adjacent
/// to the slot grid (see `main.rs` consumer). Suggests both directions
/// of fix (change template OR adjust slot count) so the user can pick
/// whichever matches their intent.
///
/// **Option A pattern** (mirrors v0.7.1 row-8 `detect_slot_index_gaps`):
/// pure GUI-internal pre-check; no toolkit wire-format involvement.
/// The CLI's mode-violation ladder (§6.6 rows 10/11) remains the
/// authoritative gate.
///
/// `template` is `None` when descriptor mode is active (`--template`
/// disabled by the v0.16.0 mutex rule); the warning suppresses there.
/// v0.8.1 F3 — `coin_type` for the `--network` dropdown value. Mirrors
/// `mnemonic-toolkit::network::CliNetwork::coin_type` semantics inline
/// (BIP-44 coin index: mainnet=0; all test networks=1). Inline mirror per
/// the v0.20.0 plan §2 GUI row 2 — no full CliNetwork enum port needed
/// since the GUI's dropdown already carries the network as a string.
pub fn coin_type_for_network(network_str: &str) -> u32 {
    match network_str {
        "mainnet" => 0,
        _ => 1, // testnet / signet / regtest
    }
}

/// v0.8.1 F3 — non-canonical descriptor default-path-inference banner.
/// Closes the v0.19.0 cycle's I2 perceptibility gap: CLI users see the
/// `info: non-canonical descriptor; ...` stderr notice on default-path
/// emission (`bundle.rs:1381-1402`); GUI users get the correct behavior
/// but no visual cue. This helper produces a banner string for the GUI
/// to render adjacent to the slot grid in `main.rs`.
///
/// **Banner text shape (paraphrase-accepted per v0.20.0 plan R1 I1 fold):**
/// the toolkit emits `info: ... defaulting origin path for @0,@1,@2 to
/// m/48'/0'/0'/2' (BIP-48 cosigner path). Override per-placeholder ...`
/// with `idx_list` enumerating the actual defaulted `@N` indices. The GUI
/// does NOT have descriptor-parse infrastructure to compute
/// `defaulted_indices`, so we paraphrase using a literal "@N" reference;
/// the substantive content (default path, BIP-48 convention, override
/// mechanism) is preserved.
///
/// **Empty-descriptor guard (R3 mitigation):**
/// `classify_descriptor_canonicity` returns `NonCanonical` for empty
/// descriptor (v0.8.0 semantics; treats absence as the "more permissive"
/// branch). This helper additionally gates on `!descriptor.is_empty()`
/// so the banner doesn't fire on a fresh form before the user types.
///
/// **Strict-parity deferred:** a future FOLLOWUP would port
/// `bundle.rs::expand_descriptor_with_default_path`'s `defaulted_indices`
/// computation into the GUI (or surface it via a new
/// `--classify-descriptor-verbose` toolkit flag returning the indices),
/// then render the banner byte-for-byte against the stderr notice.
pub fn descriptor_non_canonical_default_path_notice(
    descriptor: &str,
    network: &str,
    account: u32,
) -> Option<String> {
    if descriptor.is_empty() {
        return None;
    }
    if classify_descriptor_canonicity(descriptor) != Canonicity::NonCanonical {
        return None;
    }
    let coin = coin_type_for_network(network);
    Some(format!(
        "ℹ non-canonical descriptor; missing-origin @N placeholders default to \
         m/48'/{coin}'/{account}'/2' (BIP-48 cosigner path). Override per-placeholder \
         with [fp/path]@N or --slot @N.path=m/..."
    ))
}

pub fn template_slot_count_warning(
    template: Option<&str>,
    slot_count: usize,
) -> Option<String> {
    let template = template?;
    if SINGLE_SIG_TEMPLATES.contains(&template) {
        // Row 10: single-sig requires exactly 1 slot.
        if slot_count >= 2 {
            return Some(format!(
                "⚠ template '{template}' is single-sig (requires 1 slot); \
                 you have {slot_count} slots. Use a multisig template or \
                 remove slots."
            ));
        }
    } else if MULTISIG_TEMPLATES.contains(&template) {
        // Row 11: multisig requires 2+ slots.
        if slot_count < 2 {
            return Some(format!(
                "⚠ template '{template}' is multisig (requires 2+ slots); \
                 you have {slot_count} slot{}. Add slots or use a \
                 single-sig template.",
                if slot_count == 1 { "" } else { "s" }
            ));
        }
    }
    None
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

/// v0.10.0 B.4 helper — return true iff `--to` (a repeating Dropdown in the
/// convert subcommand schema) contains an entry whose Dropdown value equals
/// `target`. `dropdown_value` only inspects the first match in `state.values`,
/// so for repeating dropdowns we iterate manually.
///
/// SPEC §6.7 + `schema/mnemonic.rs:415-422` define `--to` as
/// `FlagKind::Dropdown(NODE_TYPES)` with `repeating: true`; the argv assembler
/// at `form/invocation.rs:175-178` consumes all matching rows in order. This
/// helper mirrors the same iteration shape so conditional predicates see
/// every populated `--to` value.
fn to_contains(state: &FormState, target: &str) -> bool {
    state.values.iter().any(|(k, v)| {
        k == "--to" && matches!(v, FlagValue::Dropdown(s) if s == target)
    })
}

/// `convert` subcommand conditionals.
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/convert.rs`):
///   :181 `--passphrase-stdin` conflicts_with = "passphrase"
///   :203 (v0.13.0 drift fix) `--bip38-passphrase-stdin` conflicts_with = "bip38_passphrase"
///
/// v0.10.0 B.4 — add 6 gap rules (electrum, script-type, template, path,
/// xpub-prefix) gated on the `--from` node and the contents of the repeating
/// `--to` flag. Toolkit-side these are runtime refusals (not clap-derive
/// conflicts), so they're not emitted by the toolkit's `gui-schema`
/// `conditional_rules` projection — they live exclusively in the GUI fn.
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

    // ─── v0.10.0 B.4: 6 gap rules ────────────────────────────────────────
    //
    // Toolkit semantics ground each rule. None of these are clap-derive
    // conflicts; they're runtime refusals in `cmd/convert.rs` (or
    // value-dependent flag consumption). The GUI surfaces them pre-Run so
    // the user sees inapplicable flags grey out as they pick `--from` /
    // `--to` combinations.
    let from_is_electrum = state.composite_node("--from") == Some("electrum-phrase");
    let to_has_electrum = to_contains(state, "electrum-phrase");
    let to_has_address = to_contains(state, "address");
    let to_has_wif = to_contains(state, "wif");
    let to_has_bip38 = to_contains(state, "bip38");
    let to_has_xpub = to_contains(state, "xpub");
    let to_has_xprv = to_contains(state, "xprv");
    let to_has_fingerprint = to_contains(state, "fingerprint");
    let has_template = state.has_value("--template");

    // Gap 1 + 2: --electrum-version / --electrum-language are used ONLY by
    // the (entropy, electrum-phrase) encode arm (convert.rs:1138-1151) and
    // the (electrum-phrase, entropy) decode arm (convert.rs:1429-1436).
    // Disable when neither side touches electrum-phrase.
    if !from_is_electrum && !to_has_electrum {
        vis.push(("--electrum-version", Visibility::Disabled));
        vis.push(("--electrum-language", Visibility::Disabled));
    }

    // Gap 3: --script-type is used ONLY by (*, address) edges via
    // `resolve_script_type` (convert.rs:1461-1469).
    //   - Required when --to contains "address" AND no --template (else
    //     the toolkit refuses with `refusal_address_no_script_type` when
    //     both are absent).
    //   - Disabled when --to does NOT contain "address".
    // Required emits before Disabled so first-rule-wins picks Required
    // when both could fire (none can; the conditions are disjoint, but
    // the order matches export_wallet's --taproot-internal-key precedent).
    if to_has_address && !has_template {
        vis.push(("--script-type", Visibility::Required));
    }
    if !to_has_address {
        vis.push(("--script-type", Visibility::Disabled));
    }

    // Gap 4: --template is consumed by TWO paths in convert.rs:
    //   - Phrase/Entropy → {Xpub, Xprv, Fingerprint} derivation
    //     (convert.rs:1049-1063 — `needs_derive` gate).
    //   - resolve_script_type for (*, Address) script-type inference
    //     (convert.rs:1465 — used only when --script-type is absent).
    // Disable when --to has none of {address, xpub, xprv, fingerprint}.
    // (The recon's narrower "address-only" predicate would block the
    // legitimate phrase→xpub template-derivation case; we widen to match
    // toolkit ground truth.)
    if !to_has_address && !to_has_xpub && !to_has_xprv && !to_has_fingerprint {
        vis.push(("--template", Visibility::Disabled));
    }

    // Gap 5: --path is consumed when --to contains any of:
    //   - wif    (convert.rs:1094 phrase|entropy→wif; toolkit refuses with
    //              refusal_phrase_entropy_to_wif_no_path)
    //   - bip38  (convert.rs:1120 phrase|entropy→bip38 composite; same
    //              refusal helper as wif)
    //   - address (convert.rs:1164 / 1216 phrase|entropy→address and
    //              xpub→address; toolkit refuses with refusal_address_no_path)
    // --path is mandatory for these edges regardless of --template
    // (template feeds script-type, NOT path inference — line 1465 only
    // routes to script_type_from_template). Disable when --to has none
    // of {wif, bip38, address}.
    if to_has_wif || to_has_bip38 || to_has_address {
        vis.push(("--path", Visibility::Required));
    }
    if !to_has_wif && !to_has_bip38 && !to_has_address {
        vis.push(("--path", Visibility::Disabled));
    }

    // Gap 6: --xpub-prefix is the SLIP-0132 prefix swap on --to xpub
    // emission (convert.rs:277-289 doc cites SPEC v0.6.1 §11.a). The flag
    // is consumed only on (*, Xpub) targets where xpub_prefix re-encodes
    // the canonical xpub/tpub bytes. Disable when --to does NOT contain
    // "xpub".
    if !to_has_xpub {
        vis.push(("--xpub-prefix", Visibility::Disabled));
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

/// v0.10.0 Tranche C.2 (D36) — 3-way card at-least-one helper shared
/// between `repair` and `inspect`. Replaces the prior v0.9.0 A.2
/// 3-way-card-mutex helper in lockstep with toolkit's D35 drop of the
/// `<--ms1|--mk1|--md1>` clap required-group: the toolkit now accepts
/// mixed-HRP invocations (2 or 3 cards in a single run), so the GUI
/// must stop disabling the "other" cards once one is filled.
///
/// **NET-NEW pattern** — `verify_bundle::*` at L415-428 above is a
/// `--bundle-json XOR (cards-group)` mutex (a 2-way between one flag and
/// a group), NOT a 3-way among 3 equal-status cards. Phase A.0 recon
/// (v0.9.0) confirmed no prior `conditional.rs` rule had this shape;
/// C.2 keeps the helper shape and only shifts the semantic from mutex
/// to at-least-one.
///
/// Semantics (count of populated card flags in `state.has_value`):
/// - 0 set → all 3 marked Required (visual prompt: "fill at least one";
///   toolkit-CLI will fail with its own at-least-one diagnostic if all
///   three are still empty at Run time).
/// - ≥1 set → emit NOTHING for the three card flags; they all fall
///   through to `Visibility::Visible` via `vis_of`'s default. 2 or 3
///   cards filled is allowed (matches D35's mixed-HRP capability) and
///   carries no Disabled/Required override.
fn three_way_card_at_least_one(state: &FormState) -> FlagVisibility {
    let any_set = ["--ms1", "--mk1", "--md1"]
        .iter()
        .any(|name| state.has_value(name));
    let mut vis = Vec::new();
    if !any_set {
        vis.push(("--ms1", Visibility::Required));
        vis.push(("--mk1", Visibility::Required));
        vis.push(("--md1", Visibility::Required));
    }
    // ≥1 set: no override emitted; all 3 stay Visible per default.
    vis
}

/// `repair` subcommand conditionals (v0.9.0 Phase A.2 → v0.10.0 C.2).
///
/// 3-way at-least-one rule among `--ms1` / `--mk1` / `--md1`. `--json`
/// is orthogonal (always Visible; never Required; never Disabled).
///
/// Upstream (`crates/mnemonic-toolkit/src/cmd/repair.rs`): D35 dropped
/// the prior `<--ms1|--mk1|--md1>` clap required-group; the toolkit now
/// accepts any non-empty subset (1, 2, or 3 cards) in a single
/// invocation and emits its own diagnostic if all three are empty.
pub fn repair(state: &FormState) -> FlagVisibility {
    three_way_card_at_least_one(state)
}

/// `inspect` subcommand conditionals (v0.9.0 Phase A.2 → v0.10.0 C.2).
///
/// Same 3-way at-least-one rule as `repair`. `--reveal-secret` and
/// `--json` are orthogonal (always Visible; never Required; never
/// Disabled). Same upstream D35 while history as `repair`.
pub fn inspect(state: &FormState) -> FlagVisibility {
    three_way_card_at_least_one(state)
}

/// `compare-cost` subcommand conditionals (v0.11.0 / toolkit v0.26.0).
///
/// `--miniscript` and `--descriptor` are mutually exclusive (exactly-one-of):
/// when one is filled, the other is Disabled. When neither is filled, both
/// are Visible and the user can choose. Clap-side `conflicts_with` is the
/// authoritative gate; the GUI surface mirrors the same UX for the user.
pub fn compare_cost(state: &FormState) -> FlagVisibility {
    let mut vis: FlagVisibility = Vec::new();
    let has_miniscript = state
        .text_value("--miniscript")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let has_descriptor = state
        .text_value("--descriptor")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    if has_miniscript {
        vis.push(("--descriptor", Visibility::Disabled));
    }
    if has_descriptor {
        vis.push(("--miniscript", Visibility::Disabled));
    }
    vis
}

/// `restore` subcommand conditionals (v0.25.0 / toolkit v0.44.0).
///
/// Mirrors `RestoreArgs.from` `required_unless_present = "md1"`
/// (`crates/mnemonic-toolkit/src/cmd/restore.rs`): `--from` is required for
/// single-sig restore, but OPTIONAL (an own-cosigner cross-check) once
/// `--md1` is supplied (multisig-cosigner restore reconstructs the
/// descriptor from the md1 alone). When neither is present the toolkit
/// errors at run time; the GUI surfaces the requirement up-front by marking
/// `--from` Required while `--md1` is empty.
///
/// Toolkit-projected + drift-gated rule (since `mnemonic-toolkit-v0.46.2`):
/// the `gui-schema` `conditional_rules` projection
/// (`build_subcommand_conditional_rules`, `cmd/gui_schema.rs`) now carries a
/// `restore` arm (`restore_conditional_rules()`) emitting
/// `not(flag_present "--md1") → {--from, required}` — exactly the shape this
/// fn produces. So this rule IS covered by `gui_schema_conditional_drift`
/// (with `("restore", 1)` in `SUBCOMMAND_FLOORS`), and this fn MUST keep
/// matching the toolkit projection. The mirrored-shape precedent is
/// `verify_bundle`'s `required_unless_present` modeling above.
pub fn restore(state: &FormState) -> FlagVisibility {
    let mut vis = Vec::new();
    if !state.has_value("--md1") {
        vis.push(("--from", Visibility::Required));
    }
    vis
}
