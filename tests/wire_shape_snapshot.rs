//! Wave-3 W3-1 — per-consumer `--json` WIRE-SHAPE golden-snapshot regression
//! tests for the two `mnemonic` subcommands whose `--json` envelope the GUI
//! consumes/renders: `xpub-search` (path/account/passphrase modes) and
//! `import-wallet`.
//!
//! Closes the option-(b) residual of FOLLOWUP
//! `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`: the
//! `schema_mirror` gate enforces clap flag-NAME parity, NOT the runtime
//! `--json` wire-shape. These cells pin the wire SHAPE (top-level + nested
//! key SETS, the match-vs-no_match conditional-key drift class, and the stable
//! scalar values) keyed to the `mnemonic-toolkit-v0.70.0` toolkit pin
//! (`Cargo.toml` `[dependencies] mnemonic-toolkit`).
//!
//! ## Capture-from-binary contract (load-bearing — see SOURCE.md §wire-shape)
//! The expected key-sets below were captured FROM the v0.70.0 binary, NOT
//! hand-written from prose and NOT copied from the retired stale `v0.27.0`
//! vendored envelope fixtures (which encoded the OLD shape — e.g.
//! `path/template/account: null` on a `no_match`, since corrected to key
//! OMISSION). The CI `schema-mirror.yml` `cargo-test-full-suite` job installs
//! the toolkit via `cargo install --locked --git … --tag
//! mnemonic-toolkit-v0.70.0` and runs `cargo test --workspace` with
//! `MNEMONIC_BIN=mnemonic`, so these cells RUN against that exact binary in CI.
//!
//! ## Maintenance: every toolkit-pin bump MUST re-verify these goldens
//! The goldens are valid ONLY at the v0.70.0 pin. A future `Cargo.toml` /
//! `pinned-upstream.toml` toolkit-pin bump MUST re-run these 8 cells against
//! the new binary in the SAME cycle; a changed key-set is an intentional
//! wire-shape evolution → update the assertion + bump the GUI in lockstep.
//! This is the LEADING drift gate the slug wants.
//!
//! ## Skip-when-absent
//! `MNEMONIC_BIN` wins; else a bare `mnemonic` is probed on `$PATH`. With no
//! binary (a dev laptop), every cell early-returns (skips) so an offline
//! `cargo test --workspace` still PASSES. This mirrors `schema_mirror.rs`'s
//! `resolve_bin()` / skip discipline.

use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

// ── Canonical no-funds inputs (abandon×11+about test seed) ──────────────────
//
// All confirmed reproducible against the v0.70.0 binary (2026-06-22). The seed
// holds no real funds; it is the standard BIP-39 all-zero-entropy + valid
// checksum vector.

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// abandon×11+about, bip84 account-0 acct xpub at `m/84'/0'/0'` (NO passphrase).
/// Live MATCH (path=`m/84'/0'/0'`, template=`bip84`, account=0).
const ABANDON_BIP84_ACCT0_XPUB: &str =
    "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// An unrelated mainnet xpub. Live NO_MATCH under the abandon×11+about seed
/// across the searched set (the same xpub the retired v0.27.0 no_match fixture
/// used).
const UNRELATED_XPUB: &str =
    "xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV";

/// A literal-xpub single-cosigner descriptor whose key == `ABANDON_BIP84_ACCT0_XPUB`.
/// Live MATCH: `matched_cosigners[0] = {cosigner_index:0, path:"m/84'/0'/0'",
/// template:"bip84", account:0}`, `descriptor_shape == "literal_xpub"`.
const LITERAL_XPUB_DESC_MATCH: &str =
    "wpkh([5436d724/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*)";

/// Same descriptor SHAPE but an unrelated xpub → live NO_MATCH
/// (`matched_cosigners` OMITTED).
const LITERAL_XPUB_DESC_NO_MATCH: &str =
    "wpkh([5436d724/84'/0'/0']xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV/<0;1>/*)";

/// A FORWARD-DERIVED (passphrase, xpub) pair for the abandon×11+about seed:
/// passphrase `wave3test` → bip84 account-0 acct xpub. Captured at impl time
/// via `bundle … --passphrase wave3test --template bip84 --account 0 --json`
/// then `convert --from mk1=<chunks> --to xpub` (the bundle's mk1 card decodes
/// to this xpub). Live MATCH under `passphrase-of-xpub`.
const PASSPHRASE: &str = "wave3test";
const PASSPHRASE_BIP84_ACCT0_XPUB: &str =
    "xpub6CKcCdneZ4pbhRKXPQoFJUQzufniKtnqJkDTRAtqGi8SybUgrVTkZUxTMKX4Em9HvJnHxFRNqFv1VU6uyrmTKmX5mvSm5WnXh7NW6CRePvj";

// ── Binary resolution + skip helper ─────────────────────────────────────────

/// `MNEMONIC_BIN` wins; else probe bare `mnemonic` on `$PATH`. `None` => skip
/// (no binary present). Mirrors `schema_mirror.rs:608-620`'s skip idiom.
fn mnemonic_bin() -> Option<String> {
    if let Ok(b) = std::env::var("MNEMONIC_BIN") {
        return Some(b);
    }
    match Command::new("mnemonic").arg("--version").output() {
        Ok(_) => Some("mnemonic".into()),
        Err(_) => None,
    }
}

// ── Recursive key-set extractor (a key ADD or REMOVE anywhere fails) ─────────

/// Flatten EVERY nested-object key path in `v` to a `/`-joined set. Array
/// elements collapse onto their parent path (index-agnostic) so a 1-element vs
/// N-element array share one key-path; this pins object SHAPE without coupling
/// to array length. Scalar/null leaves contribute their own (already-recorded)
/// parent key — they do not extend the path.
fn key_paths(v: &Value, prefix: &str, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}/{k}")
                };
                out.insert(path.clone());
                key_paths(child, &path, out);
            }
        }
        Value::Array(items) => {
            // Collapse all elements onto the same parent path (index-agnostic).
            for item in items {
                key_paths(item, prefix, out);
            }
        }
        // Scalars / null: leaf — the parent key was already inserted.
        _ => {}
    }
}

/// Top-level key set of a JSON object (direct children only).
fn top_keys(v: &Value) -> BTreeSet<String> {
    v.as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

/// Key set of the object at object-path `obj.get(key)` (direct children only).
fn nested_keys(v: &Value, key: &str) -> BTreeSet<String> {
    v.get(key).map(top_keys).unwrap_or_default()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

// ── Invocation helper ────────────────────────────────────────────────────────

/// Run `bin <args…>` feeding `stdin_data` on stdin (when `Some`). Returns the
/// parsed stdout JSON. NOT exit-code-gated: the `--json` envelope is emitted on
/// stdout even when the process exits non-zero (no_match → exit 4); stderr
/// (argv-leakage advisory + "no match in searched set" line) is ignored.
fn run_json(bin: &str, args: &[&str], stdin_data: Option<&str>) -> Value {
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .stdin(if stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("spawn `{bin} {args:?}` failed: {e}"));
    if let Some(data) = stdin_data {
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(data.as_bytes())
            .expect("write stdin");
    }
    let out = child.wait_with_output().expect("wait_with_output");
    serde_json::from_slice::<Value>(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout of `{bin} {args:?}` is not JSON: {e}\nstdout was:\n{}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

/// Resolve the BSMS 2-of-3 blob path: prefer the in-repo vendored copy; the
/// cell SKIPS if it is somehow absent (defensive — it is vendored alongside).
fn bsms_blob_path() -> Option<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wallet_import/bsms-2line-multi-2of3.txt");
    p.exists().then(|| p.to_string_lossy().into_owned())
}

fn coldcard_blob_path() -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/coldcard_generic_bip84_mainnet.json")
        .to_string_lossy()
        .into_owned()
}

// ── xpub-search cells (seed via --phrase-stdin; NEVER on argv — §2.5) ────────

#[test]
fn wireshape_path_of_xpub_match() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let v = run_json(
        &bin,
        &[
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            ABANDON_BIP84_ACCT0_XPUB,
            "--json",
        ],
        Some(SEED),
    );
    assert_eq!(
        top_keys(&v),
        set(&[
            "schema_version",
            "mode",
            "result",
            "path",
            "template",
            "account",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ]),
        "path-of-xpub MATCH top-level key set drift"
    );
    assert_eq!(v["result"], "match");
    assert_eq!(v["mode"], "path-of-xpub");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["path"], "m/84'/0'/0'");
    assert_eq!(v["template"], "bip84");
    assert_eq!(v["account"], 0);
    assert_eq!(v["searched_count"], 140);
}

#[test]
fn wireshape_path_of_xpub_no_match() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let v = run_json(
        &bin,
        &[
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            UNRELATED_XPUB,
            "--json",
        ],
        Some(SEED),
    );
    assert_eq!(
        top_keys(&v),
        set(&[
            "schema_version",
            "mode",
            "result",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ]),
        "path-of-xpub NO_MATCH top-level key set drift"
    );
    // The drift class: path/template/account are OMITTED (not null) on no_match.
    assert!(v.get("path").is_none(), "`path` must be ABSENT on no_match");
    assert!(
        v.get("template").is_none(),
        "`template` must be ABSENT on no_match"
    );
    assert!(
        v.get("account").is_none(),
        "`account` must be ABSENT on no_match"
    );
    assert_eq!(v["result"], "no_match");
    assert_eq!(v["mode"], "path-of-xpub");
}

#[test]
fn wireshape_account_of_descriptor_match() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let v = run_json(
        &bin,
        &[
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            LITERAL_XPUB_DESC_MATCH,
            "--json",
        ],
        Some(SEED),
    );
    assert_eq!(
        top_keys(&v),
        set(&[
            "schema_version",
            "mode",
            "result",
            "matched_cosigners",
            "cosigners_total",
            "searched_count_per_cosigner",
            "descriptor_shape",
            "unspendable_internal_keys",
        ]),
        "account-of-descriptor MATCH top-level key set drift"
    );
    let cosigner0 = &v["matched_cosigners"][0];
    assert_eq!(
        top_keys(cosigner0),
        set(&["cosigner_index", "path", "template", "account"]),
        "matched_cosigners[0] key set drift"
    );
    assert_eq!(v["result"], "match");
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["descriptor_shape"], "literal_xpub");
    assert_eq!(cosigner0["cosigner_index"], 0);
    assert_eq!(cosigner0["path"], "m/84'/0'/0'");
    assert_eq!(cosigner0["template"], "bip84");
    assert_eq!(cosigner0["account"], 0);
}

#[test]
fn wireshape_account_of_descriptor_no_match() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let v = run_json(
        &bin,
        &[
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            LITERAL_XPUB_DESC_NO_MATCH,
            "--json",
        ],
        Some(SEED),
    );
    assert_eq!(
        top_keys(&v),
        set(&[
            "schema_version",
            "mode",
            "result",
            "cosigners_total",
            "searched_count_per_cosigner",
            "descriptor_shape",
            "unspendable_internal_keys",
        ]),
        "account-of-descriptor NO_MATCH top-level key set drift"
    );
    // The drift class: matched_cosigners is OMITTED (present-on-match only).
    assert!(
        v.get("matched_cosigners").is_none(),
        "`matched_cosigners` must be ABSENT on no_match"
    );
    assert_eq!(v["result"], "no_match");
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["descriptor_shape"], "literal_xpub");
}

#[test]
fn wireshape_passphrase_of_xpub_match() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let v = run_json(
        &bin,
        &[
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            PASSPHRASE,
            "--target-xpub",
            PASSPHRASE_BIP84_ACCT0_XPUB,
            "--json",
        ],
        Some(SEED),
    );
    assert_eq!(
        top_keys(&v),
        set(&[
            "schema_version",
            "mode",
            "result",
            "path",
            "template",
            "account",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ]),
        "passphrase-of-xpub MATCH top-level key set drift"
    );
    assert_eq!(v["result"], "match");
    assert_eq!(v["mode"], "passphrase-of-xpub");
    assert_eq!(v["path"], "m/84'/0'/0'");
    assert_eq!(v["template"], "bip84");
    assert_eq!(v["account"], 0);
}

#[test]
fn wireshape_passphrase_of_xpub_no_match() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let v = run_json(
        &bin,
        &[
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            PASSPHRASE,
            "--target-xpub",
            UNRELATED_XPUB,
            "--json",
        ],
        Some(SEED),
    );
    assert_eq!(
        top_keys(&v),
        set(&[
            "schema_version",
            "mode",
            "result",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ]),
        "passphrase-of-xpub NO_MATCH top-level key set drift"
    );
    assert!(
        v.get("path").is_none(),
        "`path` must be ABSENT on no_match"
    );
    assert!(
        v.get("template").is_none(),
        "`template` must be ABSENT on no_match"
    );
    assert!(
        v.get("account").is_none(),
        "`account` must be ABSENT on no_match"
    );
    assert_eq!(v["result"], "no_match");
    assert_eq!(v["mode"], "passphrase-of-xpub");
}

// ── import-wallet cells (watch-only blob import; NO seed/--phrase-stdin) ──────

#[test]
fn wireshape_import_wallet_bsms_multisig() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let Some(blob) = bsms_blob_path() else {
        eprintln!("bsms-2line-multi-2of3.txt vendored fixture absent; skipping");
        return;
    };
    let v = run_json(
        &bin,
        &["import-wallet", "--blob", &blob, "--format", "bsms", "--json"],
        None,
    );
    let entries = v.as_array().expect("top-level is a JSON array");
    assert!(!entries.is_empty(), "envelope has at least one bundle entry");
    let entry = &entries[0];
    assert_eq!(
        top_keys(entry),
        set(&["bundle", "roundtrip", "schema_version", "source_format"]),
        "import-wallet bsms entry key set drift (note: NO source-metadata key for bsms)"
    );
    assert_eq!(entry["source_format"], "bsms");
    // FULL-RECURSIVE whole-entry key-path set: a key ADD or REMOVE ANYWHERE
    // (incl. a nested object not hand-enumerated below) fails. Array elements
    // collapse onto their parent path (index-agnostic) so the per-cosigner
    // sub-keyset is pinned once regardless of cosigner count.
    let mut recursive = BTreeSet::new();
    key_paths(entry, "", &mut recursive);
    assert_eq!(
        recursive,
        set(&[
            "bundle",
            "bundle/account",
            "bundle/descriptor",
            "bundle/master_fingerprint",
            "bundle/md1",
            "bundle/mk1",
            "bundle/mode",
            "bundle/ms1",
            "bundle/multisig",
            "bundle/multisig/cosigner_count",
            "bundle/multisig/cosigners",
            "bundle/multisig/cosigners/index",
            "bundle/multisig/cosigners/master_fingerprint",
            "bundle/multisig/cosigners/origin_path",
            "bundle/multisig/cosigners/xpub",
            "bundle/multisig/path_family",
            "bundle/multisig/template",
            "bundle/multisig/threshold",
            "bundle/network",
            "bundle/origin_path",
            "bundle/origin_paths",
            "bundle/privacy_preserving",
            "bundle/schema_version",
            "bundle/template",
            "roundtrip",
            "roundtrip/byte_exact",
            "roundtrip/diff",
            "roundtrip/semantic_match",
            "roundtrip/status",
            "schema_version",
            "source_format",
        ]),
        "import-wallet bsms full-recursive key-path set drift"
    );
    // roundtrip key set — `status` MUST be present (the under-asserted key the
    // retired smoke test never pinned). Do NOT assert `diff` byte content.
    assert_eq!(
        nested_keys(entry, "roundtrip"),
        set(&["byte_exact", "diff", "semantic_match", "status"]),
        "import-wallet roundtrip key set drift"
    );
    assert!(
        entry["roundtrip"]["status"].is_string(),
        "roundtrip.status is a string"
    );
    assert!(entry["roundtrip"]["byte_exact"].is_boolean());
    assert!(entry["roundtrip"]["semantic_match"].is_boolean());
    // bundle key set.
    assert_eq!(
        nested_keys(entry, "bundle"),
        set(&[
            "account",
            "descriptor",
            "master_fingerprint",
            "md1",
            "mk1",
            "mode",
            "ms1",
            "multisig",
            "network",
            "origin_path",
            "origin_paths",
            "privacy_preserving",
            "schema_version",
            "template",
        ]),
        "import-wallet bundle key set drift"
    );
    // bundle.multisig sub-object + per-cosigner sub-keyset.
    assert_eq!(
        top_keys(&entry["bundle"]["multisig"]),
        set(&[
            "cosigner_count",
            "cosigners",
            "path_family",
            "template",
            "threshold",
        ]),
        "bundle.multisig key set drift"
    );
    assert_eq!(
        top_keys(&entry["bundle"]["multisig"]["cosigners"][0]),
        set(&["index", "master_fingerprint", "origin_path", "xpub"]),
        "bundle.multisig.cosigners[0] key set drift"
    );
}

#[test]
fn wireshape_import_wallet_coldcard_singlesig() {
    let Some(bin) = mnemonic_bin() else {
        return;
    };
    let blob = coldcard_blob_path();
    let v = run_json(
        &bin,
        &[
            "import-wallet",
            "--blob",
            &blob,
            "--format",
            "coldcard",
            "--json",
        ],
        None,
    );
    let entries = v.as_array().expect("top-level is a JSON array");
    assert!(!entries.is_empty(), "envelope has at least one bundle entry");
    let entry = &entries[0];
    // Note the source-format-specific key `coldcard_source_metadata` (present
    // for coldcard; ABSENT for bsms; bitcoin-core uses `source_metadata`).
    assert_eq!(
        top_keys(entry),
        set(&[
            "bundle",
            "coldcard_source_metadata",
            "roundtrip",
            "schema_version",
            "source_format",
        ]),
        "import-wallet coldcard entry key set drift"
    );
    assert_eq!(entry["source_format"], "coldcard");
    // FULL-RECURSIVE whole-entry key-path set (a key add/remove ANYWHERE fails).
    // `bundle/multisig` is a null leaf for single-sig coldcard (no children).
    let mut recursive = BTreeSet::new();
    key_paths(entry, "", &mut recursive);
    assert_eq!(
        recursive,
        set(&[
            "bundle",
            "bundle/account",
            "bundle/descriptor",
            "bundle/master_fingerprint",
            "bundle/md1",
            "bundle/mk1",
            "bundle/mode",
            "bundle/ms1",
            "bundle/multisig",
            "bundle/network",
            "bundle/origin_path",
            "bundle/origin_paths",
            "bundle/privacy_preserving",
            "bundle/schema_version",
            "bundle/template",
            "coldcard_source_metadata",
            "coldcard_source_metadata/bip_derivation",
            "coldcard_source_metadata/chain",
            "coldcard_source_metadata/dropped_fields",
            "coldcard_source_metadata/raw_account",
            "coldcard_source_metadata/xfp",
            "roundtrip",
            "roundtrip/byte_exact",
            "roundtrip/diff",
            "roundtrip/semantic_match",
            "roundtrip/status",
            "schema_version",
            "source_format",
        ]),
        "import-wallet coldcard full-recursive key-path set drift"
    );
    assert_eq!(
        nested_keys(entry, "roundtrip"),
        set(&["byte_exact", "diff", "semantic_match", "status"]),
        "import-wallet coldcard roundtrip key set drift"
    );
    assert!(
        entry["roundtrip"]["status"].is_string(),
        "roundtrip.status is a string"
    );
    assert_eq!(
        nested_keys(entry, "coldcard_source_metadata"),
        set(&[
            "bip_derivation",
            "chain",
            "dropped_fields",
            "raw_account",
            "xfp",
        ]),
        "coldcard_source_metadata key set drift"
    );
}
