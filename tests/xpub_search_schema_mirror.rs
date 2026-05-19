//! v0.11.0 — xpub-search umbrella schema-mirror gate.
//!
//! Dedicated coverage for the four `xpub-search-*` subcommands added in
//! the toolkit v0.26.0 cycle. Complements the generic
//! `tests/schema_mirror.rs` (which iterates every subcommand in the
//! GUI's schema and check-set-equals against `gui-schema` JSON) with
//! per-subcommand invariants:
//!
//! 1. **Presence:** all four umbrella entries appear in
//!    `SCHEMA.subcommands`. This is pure-logic (no toolkit binary
//!    required) and guards against accidental removal of a
//!    SubcommandSchema entry during a future refactor.
//!
//! 2. **Required-flag invariants:** the `--target-xpub` (P1/P4),
//!    `--xpub` (P3 optional but routes target identification), and
//!    `--target-address` (P3 required) flags carry the expected
//!    `required` bool per the toolkit's clap-derive `required = true`
//!    declarations. These are the load-bearing per-mode identifying
//!    inputs; if `required: true` regresses to `false` the GUI would
//!    silently allow empty-form invocations and shift error surface
//!    from the form's required-marker to the run-confirm modal's
//!    toolkit-stderr pane.
//!
//! 3. **Secret-flag invariants:** every umbrella subcommand carries
//!    `--passphrase` + `--passphrase-stdin` as `secret: true` on the
//!    GUI side (mirrors the toolkit v5 schema's per-flag `secret`
//!    field). The `schema_mirror_secret_drift.rs` gate enforces this
//!    union via gui-schema JSON; this cell is the pure-logic backstop
//!    that doesn't depend on the toolkit binary being installed.
//!
//! 4. **Flag-name set parity (when toolkit binary available):**
//!    delegates to `schema_check::json_flag_names` to assert
//!    set-equality against the toolkit's `gui-schema` JSON output.
//!    SKIP path mirrors the generic `schema_mirror.rs` discipline
//!    (toolkit binary lookup via `MNEMONIC_BIN` env var, with `$PATH`
//!    fallback). When the toolkit binary is missing OR pre-v0.26.0
//!    (does not expose the four xpub-search subcommands), the cell
//!    skips with an eprintln rather than failing.

use std::collections::BTreeSet;

use mnemonic_gui::schema;

const XPUB_SEARCH_SUBCOMMANDS: &[&str] = &[
    "xpub-search-path-of-xpub",
    "xpub-search-account-of-descriptor",
    "xpub-search-address-of-xpub",
    "xpub-search-passphrase-of-xpub",
];

fn find_subcommand(name: &str) -> Option<&'static schema::SubcommandSchema> {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
}

#[test]
fn umbrella_subcommands_all_present_in_schema() {
    // Pure-logic presence check. Guards against accidental removal of
    // a SubcommandSchema entry during a future refactor; no toolkit
    // binary required.
    for name in XPUB_SEARCH_SUBCOMMANDS {
        assert!(
            find_subcommand(name).is_some(),
            "subcommand `{}` must be present in SCHEMA.subcommands \
             (v0.11.0 xpub-search umbrella)",
            name
        );
    }
}

#[test]
fn path_of_xpub_target_xpub_is_required() {
    let sub =
        find_subcommand("xpub-search-path-of-xpub").expect("subcommand present");
    let flag = sub
        .flags
        .iter()
        .find(|f| f.name == "--target-xpub")
        .expect("--target-xpub flag present");
    assert!(
        flag.required,
        "xpub-search-path-of-xpub: --target-xpub must be required=true \
         (toolkit clap-derive `required = true`; load-bearing per-mode \
         identifying input)"
    );
}

#[test]
fn account_of_descriptor_descriptor_flags_optional_either_or() {
    // Either --descriptor or --descriptor-from must be provided, but
    // neither flag is individually clap-required (the toolkit's
    // descriptor_intake validates at run-time). The GUI's required
    // bool tracks the per-flag clap-required field, not the
    // either-or rule. This cell documents that invariant so a
    // future "fix" doesn't silently flip the required field.
    let sub = find_subcommand("xpub-search-account-of-descriptor")
        .expect("subcommand present");
    let descriptor = sub
        .flags
        .iter()
        .find(|f| f.name == "--descriptor")
        .expect("--descriptor flag present");
    let descriptor_from = sub
        .flags
        .iter()
        .find(|f| f.name == "--descriptor-from")
        .expect("--descriptor-from flag present");
    assert!(
        !descriptor.required,
        "xpub-search-account-of-descriptor: --descriptor must be \
         required=false (the either-or rule is enforced run-time, not \
         clap-required)"
    );
    assert!(
        !descriptor_from.required,
        "xpub-search-account-of-descriptor: --descriptor-from must be \
         required=false (the either-or rule is enforced run-time, not \
         clap-required)"
    );
}

#[test]
fn address_of_xpub_target_address_is_required_and_repeating() {
    let sub = find_subcommand("xpub-search-address-of-xpub")
        .expect("subcommand present");
    let flag = sub
        .flags
        .iter()
        .find(|f| f.name == "--target-address")
        .expect("--target-address flag present");
    assert!(
        flag.required,
        "xpub-search-address-of-xpub: --target-address must be \
         required=true (toolkit clap-derive `required = true`)"
    );
    assert!(
        flag.repeating,
        "xpub-search-address-of-xpub: --target-address must be \
         repeating=true (Vec<String>; user supplies one or more \
         addresses to search for)"
    );
}

#[test]
fn passphrase_of_xpub_target_xpub_is_required() {
    let sub = find_subcommand("xpub-search-passphrase-of-xpub")
        .expect("subcommand present");
    let flag = sub
        .flags
        .iter()
        .find(|f| f.name == "--target-xpub")
        .expect("--target-xpub flag present");
    assert!(
        flag.required,
        "xpub-search-passphrase-of-xpub: --target-xpub must be \
         required=true (toolkit clap-derive `required = true`)"
    );
}

#[test]
fn every_umbrella_subcommand_marks_passphrase_flags_secret() {
    // Toolkit v5 schema marks --passphrase + --passphrase-stdin as
    // secret=true on every umbrella subcommand. The GUI's hand-coded
    // FlagSchema.secret must mirror this for the persistence + paste-
    // warn machinery to function. The schema_mirror_secret_drift
    // gate enforces this union via gui-schema JSON; this cell is a
    // pure-logic backstop without the toolkit binary.
    //
    // P1/P2/P4 all expose both flags; P3 (address-of-xpub) is
    // ms1/phrase-free (single-input parent xpub) and does NOT
    // expose the passphrase flags. This cell tolerates that
    // partition by gating on per-flag presence rather than
    // hardcoding which subcommand carries which flag.
    for name in XPUB_SEARCH_SUBCOMMANDS {
        let sub = find_subcommand(name).expect("subcommand present");
        for flag_name in ["--passphrase", "--passphrase-stdin"] {
            if let Some(flag) = sub.flags.iter().find(|f| f.name == flag_name) {
                assert!(
                    flag.secret,
                    "{name}: {flag_name} must be secret=true (toolkit v5 \
                     ground truth + GUI persistence/paste-warn invariant)"
                );
            }
        }
    }
}

#[test]
fn every_umbrella_subcommand_marks_ms1_secret_when_present() {
    // P1/P2/P4 expose --ms1 (HRP-autodetect single-ms1 input); P3
    // does not. When present, --ms1 must be secret=true (toolkit v5
    // ground truth — ms1 carries BIP-39 entropy).
    for name in XPUB_SEARCH_SUBCOMMANDS {
        let sub = find_subcommand(name).expect("subcommand present");
        if let Some(flag) = sub.flags.iter().find(|f| f.name == "--ms1") {
            assert!(
                flag.secret,
                "{name}: --ms1 must be secret=true (toolkit v5 ground \
                 truth — ms1 carries BIP-39 entropy)"
            );
        }
    }
}

#[test]
fn every_umbrella_subcommand_carries_no_auto_repair_global() {
    // toolkit v5 schema emits --no-auto-repair on every subcommand with
    // global=true (clap-derive `global = true` propagation). The GUI
    // mirrors via NO_AUTO_REPAIR_FLAG appended to each flag list.
    for name in XPUB_SEARCH_SUBCOMMANDS {
        let sub = find_subcommand(name).expect("subcommand present");
        let flag = sub
            .flags
            .iter()
            .find(|f| f.name == "--no-auto-repair")
            .unwrap_or_else(|| {
                panic!("{name}: --no-auto-repair must be present")
            });
        assert!(
            flag.global,
            "{name}: --no-auto-repair must be global=true (toolkit v5 \
             clap-derive `global = true`)"
        );
    }
}

/// Schema-mirror flag-name parity against the toolkit's `gui-schema`
/// JSON output. SKIP discipline mirrors `schema_mirror.rs`: locate
/// `MNEMONIC_BIN` via env-var or `$PATH`; if the binary doesn't yet
/// expose the four xpub-search subcommands (pre-v0.26.0 install) the
/// `json_flag_names` helper returns `None` and we skip rather than
/// failing.
#[test]
fn umbrella_flag_names_match_toolkit_gui_schema_json() {
    let mut checked = 0_usize;
    let mut skipped = 0_usize;
    for name in XPUB_SEARCH_SUBCOMMANDS {
        let sub = find_subcommand(name).expect("subcommand present");
        let from_schema: BTreeSet<String> =
            sub.flags.iter().map(|f| f.name.to_string()).collect();
        let from_toolkit =
            match mnemonic_gui::schema_check::json_flag_names("mnemonic", name)
            {
                Some(names) => names,
                None => {
                    eprintln!(
                        "skipping {name}: toolkit gui-schema JSON did not \
                         expose this subcommand (pre-v0.26.0 install?)"
                    );
                    skipped += 1;
                    continue;
                }
            };
        let only_in_schema: Vec<_> =
            from_schema.difference(&from_toolkit).collect();
        let only_in_toolkit: Vec<_> =
            from_toolkit.difference(&from_schema).collect();
        assert!(
            only_in_schema.is_empty() && only_in_toolkit.is_empty(),
            "xpub-search schema-mirror drift for `mnemonic {name}`:\n  \
             only in GUI schema: {only_in_schema:?}\n  \
             only in toolkit gui-schema JSON: {only_in_toolkit:?}",
        );
        checked += 1;
    }
    eprintln!(
        "xpub_search_schema_mirror: {checked}/4 subcommands checked, \
         {skipped} skipped (toolkit binary not at v0.26.0)"
    );
}
