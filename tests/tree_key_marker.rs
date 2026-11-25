//! FIX_sec-gui-allowlist-shape (architect-gui-q1-typing-prefixes Q2): the one
//! line shown under a tree `key`/`keys[i]` field. Fake signet/testnet-shaped
//! vectors only.

use mnemonic_gui::form::tree_model::key_marker;

const TPUB: &str = "tpubD6NzVbkrYhZ4XgiXtGrdW5XDAPFCL9h7we1vwNCpn8tGbBcgfVYjXyhWo4E1xkh56hjod1RhGjxbaTLV3X4FyWuejifB9jusQ46QzG87VKp";
const FWIF: &str = "cMecTxSiV5a44c4jqgSqYmV9bFAWNV43oPhkqu1Ty3GapoXoCSdq";

#[test]
fn m1_key_marker_texts() {
    let swapped = format!("{}{}{}{}", &TPUB[..50], &TPUB[51..52], &TPUB[50..51], &TPUB[52..]);
    let m = |s: &str| key_marker(s).map(|(t, a)| format!("{}{t}", if a { "A:" } else { "w:" }));
    let rows: Vec<(String, Option<&str>)> = vec![
        (String::new(), None),
        ("  ".into(), None),
        (TPUB.into(), None),
        (format!("[73c5da0a/84h/1h/0h]{TPUB}"), None),
        (format!("{TPUB}/<0;1>/*"), None),
        (format!("{TPUB} "), None),
        ("t".into(), Some("w:incomplete: 1 of 111 key characters")),
        (TPUB[..40].into(), Some("w:incomplete: 40 of 111 key characters")),
        ("[73c5da0a/84h".into(), Some("w:incomplete: the origin needs a closing ]")),
        (format!("[73c5/84h]{TPUB}"), Some("w:incomplete: the fingerprint needs 8 hex digits")),
        (format!("[]{TPUB}"), Some("w:incomplete: the fingerprint needs 8 hex digits")),
        (swapped, Some("A:⚠ checksum fails: a character is wrong or swapped")),
        (FWIF.into(), Some("A:⚠ not an extended public key: this value will not be saved")),
        (format!("{}{FWIF}", &TPUB[..40]), Some("A:⚠ not an extended public key: this value will not be saved")),
        (format!("{TPUB}/12345678901"), Some("A:⚠ not an extended public key: this value will not be saved")),
        ("tprv8ZgxMBicQKsP".into(), None),
    ];
    let mut wrong = Vec::new();
    for (v, want) in rows {
        let got = m(&v);
        if got.as_deref() != want {
            wrong.push(format!("{v:?}: got {got:?}, want {want:?}"));
        }
    }
    assert!(wrong.is_empty(), "key_marker rows wrong: {wrong:#?}");
}
