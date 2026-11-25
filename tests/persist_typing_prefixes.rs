//! BRIAN-Q-1 (2026-11-22): autosave never blanks a value a holder may be
//! mid-typing. Every prefix of an honest tree key persists; secret rows still
//! blank. Fake signet/testnet-shaped vectors only (version bytes + counting
//! bytes, no real key).

use mnemonic_gui::form::tree_model::{TreeNode, TreeState};

const TPUB: &str = "tpubD6NzVbkrYhZ4XgiXtGrdW5XDAPFCL9h7we1vwNCpn8tGbBcgfVYjXyhWo4E1xkh56hjod1RhGjxbaTLV3X4FyWuejifB9jusQ46QzG87VKp";
const XPUB: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
const ZPUB: &str = "zpub6jftahH18ngZxLmXaKw3GSZzZsszmt9WqedkyZdezFtWRFBZqsQH5hyUmb4pCEeZGmVfQuP5bedXTB8is6fTv19U1GQRyQUKQGUTzyHACMF";
// base58-valid fakes (testnet version bytes; payload = counting bytes)
const FTPRV: &str = "tprv8ZgxMBicQKsPcsbCVeqqF1KVdH7gwDJbxbzpCxDUsoXHdb6SnTPYxdwSAKDdqVTLYjdWmBQE9ZaguUmxmWABGxYYf9HnHQdxJCeWLx5enwV";
const FWIF: &str = "cMecTxSiV5a44c4jqgSqYmV9bFAWNV43oPhkqu1Ty3GapoXoCSdq";
// a 77-digit decimal number (the size of a decimal secp256k1 private key); fake
const DEC77: &str = "12345678901234567890123456789012345678901234567890123456789012345678901234567";

fn kept(v: &str) -> bool {
    let state = TreeState {
        root: TreeNode { key: v.to_string(), keys: vec![v.to_string()], ..Default::default() },
        ..Default::default()
    };
    let r = state.redacted_for_persistence();
    r.root.key == v && r.root.keys == vec![v.to_string()]
}

fn honest_shapes() -> Vec<(&'static str, String)> {
    vec![
        ("bare tpub", TPUB.to_string()),
        ("bare xpub", XPUB.to_string()),
        ("origin h", format!("[73c5da0a/84h/1h/0h]{TPUB}")),
        ("origin ' + multipath", format!("[73c5da0a/48'/1'/0'/2']{TPUB}/<0;1>/*")),
        ("origin H", format!("[73c5da0a/84H/1H/0H]{TPUB}")),
        ("empty origin", format!("[]{TPUB}")),
        ("suffix /0/*", format!("{TPUB}/0/*")),
        ("suffix /0h/*h", format!("{TPUB}/0h/*h")),
        ("origin + zpub", format!("[deadbeef/84'/0'/0']{ZPUB}")),
        ("leading space", format!(" [73c5da0a/84h/1h/0h]{TPUB}")),
        ("leading tab", format!("\t{TPUB}")),
        ("trailing space", format!("{TPUB} ")),
    ]
}

#[test]
fn q1_every_prefix_of_an_honest_key_persists() {
    let mut blanked = Vec::new();
    let mut total = 0usize;
    for (label, v) in honest_shapes() {
        for i in 1..=v.len() {
            total += 1;
            if !kept(&v[..i]) {
                blanked.push(format!("{label}: {:?}", &v[..i.min(40)]));
            }
        }
    }
    eprintln!("q1 prefixes checked: {total}; blanked: {}", blanked.len());
    assert!(blanked.is_empty(), "{} of {total} honest prefixes blanked: {blanked:#?}", blanked.len());
}

#[test]
fn q1_secret_rows_still_blank_with_prefix_closure() {
    let hex64 = "a1".repeat(32);
    let cases: Vec<(&str, String, String)> = vec![
        // prefix-closure surface: unclosed origin, leading whitespace
        ("[wif (unclosed)", format!("[{FWIF}"), FWIF.into()),
        ("[fp/wif (unclosed)", format!("[73c5da0a/{FWIF}"), FWIF.into()),
        ("[hex64 (unclosed)", format!("[{hex64}"), hex64.clone()),
        (" wif", format!(" {FWIF}"), FWIF.into()),
        (" tprv", format!(" {FTPRV}"), FTPRV.into()),
        (" [wif]tpub", format!(" [{FWIF}]{TPUB}"), FWIF.into()),
        ("[origin]wif", format!("[73c5da0a/84h/1h/0h]{FWIF}"), FWIF.into()),
        ("hex64", hex64.clone(), hex64.clone()),
        // FIX_sec-gui-allowlist-shape Rule S: a suffix digit run of 11+ digits
        // (a decimal private key after a path separator).
        ("tpub/77 digits", format!("{TPUB}/{DEC77}"), DEC77.into()),
    ];
    let mut leaked = Vec::new();
    for (label, v, secret) in &cases {
        let state = TreeState {
            root: TreeNode { key: v.clone(), keys: vec![v.clone()], ..Default::default() },
            ..Default::default()
        };
        let json = serde_json::to_string(&state.redacted_for_persistence()).unwrap();
        if json.contains(secret.as_str()) {
            leaked.push(*label);
        }
    }
    assert!(leaked.is_empty(), "secret survived persist for: {leaked:?}");
}

// ── FIX_sec-gui-allowlist-shape (Rules S and E) ─────────────────────────────
//
// Rows gui-persist-allowlist-concat-residue / fixdoc-secguibracket-r1-concat-residue:
// a public-prefixed key value carrying a secret after a separator, or
// concatenated, persisted. Rule S checks the body's shape; Rule E blanks a
// whole WIF or 64 hex digits inside a value Rule S keeps. Only the persisted
// copy is blanked; the live field is never changed.

fn leaked(cases: &[(&str, String, String)]) -> Vec<String> {
    let mut leaked = Vec::new();
    for (label, v, secret) in cases {
        let state = TreeState {
            root: TreeNode { key: v.clone(), keys: vec![v.clone()], ..Default::default() },
            ..Default::default()
        };
        let json = serde_json::to_string(&state.redacted_for_persistence()).unwrap();
        if json.contains(secret.as_str()) {
            leaked.push(label.to_string());
        }
    }
    leaked
}

#[test]
fn s1_rule_s_blanks_separator_and_overlength_secrets() {
    let hex64 = "a1".repeat(32);
    let cases: Vec<(&str, String, String)> = vec![
        ("xpub|wif", format!("{XPUB}|{FWIF}"), FWIF.into()),
        ("xpub/hex64", format!("{XPUB}/{hex64}"), hex64.clone()),
        ("xpubwif", format!("{XPUB}{FWIF}"), FWIF.into()),
        ("[origin]tpub/wif", format!("[73c5da0a/84h/1h/0h]{TPUB}/{FWIF}"), FWIF.into()),
        ("tpub wif", format!("{TPUB} {FWIF}"), FWIF.into()),
        ("tpub,wif", format!("{TPUB},{FWIF}"), FWIF.into()),
        ("tpubtprv", format!("{TPUB}{FTPRV}"), FTPRV.into()),
        ("tpub+tprv", format!("tpub{FTPRV}"), FTPRV.into()),
    ];
    let leaked = leaked(&cases);
    assert!(leaked.is_empty(), "secret survived persist for: {leaked:?}");
}

#[test]
fn s2_rule_s_blanks_an_eleven_digit_suffix_run() {
    let cases: Vec<(&str, String, String)> = vec![
        ("tpub/11 digits", format!("{TPUB}/12345678901"), "12345678901".into()),
        ("tpub/0/11 digits h", format!("{TPUB}/0/12345678901h"), "12345678901".into()),
        ("[origin]tpub/<0;11 digits>", format!("[73c5da0a/84h/1h/0h]{TPUB}/<0;12345678901>/*"), "12345678901".into()),
        ("tpub/77 digits", format!("{TPUB}/{DEC77}"), DEC77.into()),
    ];
    let leaked = leaked(&cases);
    assert!(leaked.is_empty(), "secret survived persist for: {leaked:?}");
}

#[test]
fn e1_rule_e_blanks_a_whole_wif_inside_an_incomplete_key() {
    let cases: Vec<(&str, String, String)> = vec![
        ("tpub[..40]+wif", format!("{}{FWIF}", &TPUB[..40]), FWIF.into()),
        ("tpub+wif", format!("tpub{FWIF}"), FWIF.into()),
        ("[origin]tpub[..40]+wif", format!("[73c5da0a/84h/1h/0h]{}{FWIF}", &TPUB[..40]), FWIF.into()),
        ("tpub[..40]+wif+space", format!("{}{FWIF} ", &TPUB[..40]), FWIF.into()),
    ];
    let leaked = leaked(&cases);
    assert!(leaked.is_empty(), "secret survived persist for: {leaked:?}");
}

#[test]
fn e2_rule_e_blanks_64_hex_inside_an_incomplete_key() {
    let lower = "a1".repeat(32);
    let upper = "A1".repeat(32);
    let cases: Vec<(&str, String, String)> = vec![
        ("tpub[..20]+hex64", format!("{}{lower}", &TPUB[..20]), lower.clone()),
        ("tpub+HEX64", format!("tpub{upper}"), upper.clone()),
        ("[origin]tpub[..20]+hex64", format!("[73c5da0a/84h/1h/0h]{}{lower}", &TPUB[..20]), lower.clone()),
    ];
    let leaked = leaked(&cases);
    assert!(leaked.is_empty(), "secret survived persist for: {leaked:?}");
}

#[test]
fn k1_typing_in_progress_public_keys_persist() {
    let rows: Vec<String> = vec![
        "t".into(),
        TPUB[..40].into(),
        "[73c5da0a/84h".into(),
        format!("[73c5/84h]{TPUB}"),
        format!("[]{TPUB}"),
        format!("{TPUB}/2147483647"),
        format!("{TPUB}/<0;1>/*"),
        format!("{TPUB} "),
        format!("{TPUB}/0/* \t"),
        format!("{} ", &TPUB[..40]),
        "tp ".into(),
        "[73c5da0a/84h ".into(),
    ];
    let blanked: Vec<&String> = rows.iter().filter(|v| !kept(v)).collect();
    assert!(blanked.is_empty(), "honest typing blanked: {blanked:?}");
}

#[test]
fn k2_live_field_is_never_changed() {
    // Guard (passes before and after the fix): saving blanks only the
    // persisted copy; the on-screen field keeps every character typed.
    let incomplete = format!("{}{FWIF}", &TPUB[..40]);
    let state = TreeState {
        root: TreeNode {
            key: FWIF.to_string(),
            keys: vec![incomplete.clone(), format!("{TPUB}/{DEC77}")],
            ..Default::default()
        },
        ..Default::default()
    };
    let before = state.clone();
    let persisted = state.redacted_for_persistence();
    assert_eq!(state, before, "redacted_for_persistence changed the live tree");
    assert_eq!(state.root.key, FWIF, "the live key field changed");
    assert!(persisted.root.key.is_empty(), "the persisted copy kept the WIF");
}

#[test]
fn e3_rule_e_blanks_every_wif_form() {
    let mc = "KwFfpDsaF7yxCELuyrH9gP5XL7TAt5b9HPWC1xCQbmrxvhJgMQHb";
    let mu = "5HpjKrb7dH5kKQQzmbjB87Mxova7mek5bXUTWfndcX6tBoqUwzm";
    let tu = "91bMubQfDW9tHTvHPwd5zhuvTavpvpHGwULQbJ98xFqvxsUsWbZ";
    let mut cases: Vec<(&str, String, String)> = vec![];
    for (l, w) in [("main-compressed", mc), ("main-uncompressed", mu), ("test-uncompressed", tu)] {
        cases.push((l, format!("{}{w}", &TPUB[..40]), w.into()));
    }
    let leaked = leaked(&cases);
    assert!(leaked.is_empty(), "secret survived persist for: {leaked:?}");
}
