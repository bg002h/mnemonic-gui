//! T5-S3 (#15) — funds-core `bundle`→`restore` with a spec-independent
//! oracle.
//!
//! ## The gap
//! The only end-to-end GUI bundle coverage before this cell was a
//! **snapshot-of-own-pipeline** pin: `wire_shape_snapshot.rs` never invokes
//! `bundle`/`restore` at all; `ui_harness_i4_realcli.rs`'s 4 cells are
//! decode-only; `tests/tutorial/*` drives bundle→restore end-to-end but
//! byte-gates against a COMMITTED golden it re-captures — a symmetric bug
//! shifting both the encode path and the expected golden in lockstep would
//! stay green forever.
//!
//! ## This cell
//! Builds a `bundle` `FormState` via the SLOT idiom (bundle at v0.75.0 has
//! no `--phrase`/`--phrase-stdin`; the seed source is `--slot
//! @N.phrase=<value>`), `Template = bip84` (scope-guarded to template mode,
//! NOT `--descriptor` mode — see the scope-guard note below), assembles argv
//! via the GUI's own `assemble_argv`, and runs it against the pinned
//! `MNEMONIC_BIN`. It then asserts the bundle's emitted `ms1`/
//! `master_fingerprint` against TWO external, code-under-test-independent
//! oracles (a published `ms vectors` constant + an R0-verified literal), and
//! feeds the emitted `ms1` + `md1` chunks straight back into `restore`,
//! asserting the restored wallet's first receive address against a
//! **triple-verified BIP-84 test vector** (authoritative bip-0084.mediawiki,
//! an independent stdlib-only derivation, and the pinned binary's own
//! bundle→restore all agree — see SPEC_test_hardening_T5_gui.md §T5-S3).
//!
//! ## Scope guard
//! Scoped to `--template bip84` only (matches the eval's literal text). A
//! `--descriptor`-mode variant would exercise Cycle-A's C1 use-site-collapse
//! fix end-to-end but needs the GUI's toolkit pin bumped to >= v0.76.0
//! (currently v0.75.0, pre-C1) — out of scope for this NO-BUMP-parallel
//! cell; tracked as a follow-on FOLLOWUP
//! (`gui-descriptor-mode-bundle-restore-independent-oracle`).
//!
//! ## Env-gating
//! Early-return-skip on `MNEMONIC_BIN` unset / binary absent from PATH —
//! mirrors `canonicity_drift.rs` / `ui_harness_i4_realcli.rs`'s convention
//! (never `#[ignore]`, so CI-with-pins actually exercises this cell while
//! CI-without-binaries just skips it).

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::runner;
use mnemonic_gui::schema::{self, FlagValue, FormState};

use std::process::Command;

/// The canonical published BIP-39 all-zero test vector ("abandon … about",
/// 12 words / 16 bytes of zero entropy), encoded as an `ms1` card. Same
/// constant as `ui_harness_i4_realcli.rs:63` (`MS1_ALL_ZERO`) — duplicated
/// here (rather than imported) because that file's const is private to its
/// own crate-test binary; provenance: the toolkit's published `ms vectors`
/// first entry.
const SEED_PHRASE_ALL_ZERO: &str = "abandon abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon about";

/// `ms vectors` first entry — the `ms1` encoding of the all-zero BIP-39
/// entropy above. External oracle: a published toolkit test vector,
/// independent of this GUI's `bundle` encode path.
const MS1_ALL_ZERO: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// The master fingerprint of the all-zero BIP-39 seed (no passphrase).
/// R0-verified literal (`SPEC_test_hardening_T5_gui.md` §T5-S3).
const MASTER_FINGERPRINT_ALL_ZERO: &str = "73c5da0a";

/// BIP-84 (`m/84'/0'/0'/0/0`) first mainnet receive address for the
/// all-zero seed. Triple-verified per the T5 spec: the authoritative
/// bip-0084.mediawiki test vector, an independent stdlib-only derivation,
/// and the pinned binary's own bundle→restore round-trip all agree.
const BIP84_FIRST_RECEIVE_ADDRESS_ALL_ZERO: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

/// BIP-84 (`m/84'/0'/0'/1/0`) FIRST CHANGE address for the all-zero seed.
///
/// Quoted from bip-0084.mediawiki's own test-vector section, fetched from
/// the bitcoin/bips repository on 2026-08-19 — not from memory, and not from
/// this toolchain (an oracle produced by the code under test is not an oracle).
///
/// WHY IT IS HERE (P2, constellation journey recon): **no journey anywhere in
/// the constellation asserted a chain-1 address.** The definition names this
/// exactly — "receive-only is the check that passes while a policy mismatch
/// quietly loses money on the change chain". The restored descriptor is
/// MULTIPATH (`<0;1>`), so chain 1 was being derived and never checked, and
/// `restore --count` is documented as "first-RECEIVE addresses per wallet
/// type" — the JSON has no change field at all.
const BIP84_FIRST_CHANGE_ADDRESS_ALL_ZERO: &str = "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el";

fn mnemonic_bin() -> Option<String> {
    if let Ok(p) = std::env::var("MNEMONIC_BIN") {
        if !p.trim().is_empty() {
            return Some(p);
        }
        return None;
    }
    let probe = Command::new("mnemonic").arg("--version").output().ok()?;
    if probe.status.success() {
        Some("mnemonic".to_string())
    } else {
        None
    }
}

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

/// Assemble argv for `sub` from `state`, swap in the pinned binary at
/// `argv[0]`, spawn it, assert exit 0, and return the parsed `--json`
/// stdout. Panics with COORDINATES-ONLY on a JSON-parse failure (this
/// pathway carries seed-derived material for the bundle leg, so the raw
/// payload is withheld from the panic message on failure).
fn run_and_parse_json(bin: &str, sub_name: &str, state: &FormState) -> serde_json::Value {
    let sub = subcommand(sub_name);
    let mut argv = assemble_argv(&schema::mnemonic::SCHEMA, sub, state);
    argv[0] = bin.to_string();
    let result = runner::run(argv).unwrap_or_else(|e| {
        panic!("bundle_restore_independent_oracle [{sub_name}]: runner spawn failed: {e}")
    });
    let stderr = String::from_utf8_lossy(&result.stderr).into_owned();
    assert_eq!(
        result.exit_code,
        Some(0),
        "bundle_restore_independent_oracle [{sub_name}]: expected exit 0; stderr:\n{stderr}"
    );
    // RunResult impls Drop (whole-holder zeroize) so stdout cannot be moved
    // out (E0509) — clone the bytes before `result` drops.
    let stdout = result.stdout.clone();
    serde_json::from_slice(&stdout).unwrap_or_else(|e| {
        panic!(
            "bundle_restore_independent_oracle [{sub_name}]: --json stdout did not parse as \
             JSON: {e} (payload withheld)"
        )
    })
}

#[test]
fn bundle_bip84_all_zero_then_restore_matches_external_oracles() {
    let Some(bin) = mnemonic_bin() else {
        eprintln!(
            "skip: MNEMONIC_BIN unset and `mnemonic` not on PATH \
             (bundle_restore_independent_oracle)"
        );
        return;
    };

    // ── leg 1: bundle --template bip84 (single-sig, no --descriptor) ────
    let bundle_state = FormState::from_pairs(vec![
        ("--network", FlagValue::Dropdown("mainnet".into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
        ("--json", FlagValue::Boolean(true)),
    ])
    .with_slots(SlotState {
        rows: vec![SlotRow {
            index: 0,
            subkey: SlotSubkey::Phrase,
            value: SEED_PHRASE_ALL_ZERO.to_string(),
        }],
    });
    let bundle_json = run_and_parse_json(&bin, "bundle", &bundle_state);

    let ms1 = bundle_json["ms1"][0]
        .as_str()
        .expect("bundle --json carries a non-empty ms1 array");
    assert_eq!(
        ms1, MS1_ALL_ZERO,
        "bundle's emitted ms1 must equal the published all-zero `ms vectors` constant \
         (external oracle, independent of the GUI's bundle argv path)"
    );
    let master_fingerprint = bundle_json["master_fingerprint"]
        .as_str()
        .expect("bundle --json carries master_fingerprint");
    assert_eq!(
        master_fingerprint, MASTER_FINGERPRINT_ALL_ZERO,
        "bundle's master_fingerprint must equal the R0-verified literal for the all-zero seed"
    );
    let md1_chunks: Vec<String> = bundle_json["md1"]
        .as_array()
        .expect("bundle --json carries an md1 chunk array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("each md1 array entry is a string")
                .to_string()
        })
        .collect();
    assert!(
        !md1_chunks.is_empty(),
        "bundle --template bip84 must emit at least one md1 chunk"
    );

    // ── leg 2: restore --from ms1=<emitted> --md1 <c1> --md1 <c2> … ─────
    let mut restore_values: Vec<(&str, FlagValue)> = vec![
        ("--from", FlagValue::Text(format!("ms1={ms1}"))),
        ("--json", FlagValue::Boolean(true)),
    ];
    for chunk in &md1_chunks {
        restore_values.push(("--md1", FlagValue::Text(chunk.clone())));
    }
    let restore_state = FormState::from_pairs(restore_values);
    let restore_json = run_and_parse_json(&bin, "restore", &restore_state);

    let first_address = restore_json["wallets"][0]["first_addresses"][0]
        .as_str()
        .expect("restore --json carries wallets[0].first_addresses[0]");
    assert_eq!(
        first_address, BIP84_FIRST_RECEIVE_ADDRESS_ALL_ZERO,
        "restore's first receive address must equal the triple-verified BIP-84 \
         (m/84'/0'/0'/0/0) test vector for the all-zero seed (external oracle, \
         independent of the GUI's bundle encode + restore decode paths)"
    );
    // ── THE CHANGE CHAIN ──────────────────────────────────────────────────
    //
    // Derived from the RESTORED descriptor's own xpub, not from the seed: the
    // point is that the wallet this restore produced controls the right change
    // addresses, and re-deriving from the phrase would prove only that the
    // toolkit agrees with itself.
    //
    // `restore --json` exposes no change address (its `--count` is
    // first-receive only), so the chain-1 half is reached through `addresses
    // --chain change` on the descriptor's xpub. Verified by hand before being
    // written: the same xpub through `addresses` and the same seed through
    // `addresses --from phrase=` agree, and both agree with the BIP.
    let restored_descriptor = restore_json["wallets"][0]["descriptor"]
        .as_str()
        .expect("restore --json carries wallets[0].descriptor");
    let restored_xpub = restored_descriptor
        .split(['(', ')', ']', '/'])
        .find(|t| t.starts_with("xpub"))
        .unwrap_or_else(|| {
            panic!("no xpub in the restored descriptor: {restored_descriptor}")
        });

    let change_state = FormState::from_pairs(vec![
        ("--from", FlagValue::Text(format!("xpub={restored_xpub}"))),
        ("--address-type", FlagValue::Text("p2wpkh".to_string())),
        // Dropdown and Number, NOT Text. assemble_argv matches FlagKind to
        // FlagValue and SILENTLY DROPS a mismatch — a Text here left --chain
        // unset, the CLI defaulted to receive, and the assertion compared the
        // receive address against the change vector. Caught because the failure
        // named both addresses; a weaker assertion would have passed.
        ("--chain", FlagValue::Dropdown("change".to_string())),
        ("--count", FlagValue::Number(1)),
        ("--json", FlagValue::Boolean(true)),
    ]);
    let change_json = run_and_parse_json(&bin, "addresses", &change_state);
    let change_address = change_json["addresses"][0]["address"]
        .as_str()
        .or_else(|| change_json["addresses"][0].as_str())
        .unwrap_or_else(|| {
            panic!("addresses --chain change --json shape unexpected: {change_json}")
        });
    assert_eq!(
        change_address, BIP84_FIRST_CHANGE_ADDRESS_ALL_ZERO,
        "the RESTORED wallet's first change address (m/84'/0'/0'/1/0) must equal \
         BIP-84's published vector. The descriptor is multipath <0;1>, so this \
         chain was always derived — it was simply never checked, which is the \
         failure that loses money quietly rather than loudly"
    );

    // v0.75.0 `restore --json` emits NO recovered-entropy field — do NOT
    // assert one here (R0 C1, empirically confirmed against the pinned
    // binary; see SPEC_test_hardening_T5_gui.md §T5-S3).
    let verification_status = restore_json["verification"]["status"]
        .as_str()
        .expect("restore --json carries verification.status");
    assert_eq!(
        verification_status, "verified",
        "restore's own cross-check must report verified"
    );
}
