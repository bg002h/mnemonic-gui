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

#[test]
fn q1_known_residual_concat_inside_incomplete_key_still_persists() {
    // gui-persist-allowlist-concat-residue (I): documented, stays open.
    let v = format!("{}{}", &TPUB[..40], FWIF);
    assert!(kept(&v), "residual row changed: re-grade gui-persist-allowlist-concat-residue");
}
