//! Rule A bounds: a value that is not a prefix of an honest key persists nothing.
use mnemonic_gui::form::tree_model::{TreeNode, TreeState};

fn kept(v: &str) -> bool {
    let state = TreeState {
        root: TreeNode {
            key: v.to_string(),
            keys: vec![v.to_string()],
            ..Default::default()
        },
        ..Default::default()
    };
    let r = state.redacted_for_persistence();
    r.root.key == v && r.root.keys == vec![v.to_string()]
}

#[test]
fn a3_short_non_prefix_bodies_blank_and_origin_bounds_hold() {
    let must_blank = [
        "5",
        "c",
        "K",
        "5Kab",
        "cMecT",
        "tpuc",
        "[73c5da0a]5Kab",
        "[73c5da0a]cMecT",
        "[123456789]",
        "[123456789",
        "[73c5da0a/12345678901]",
        "[73c5da0a/12345678901",
        "[73c5da0a/84h/1h/0h]cMecT",
    ];
    let must_keep = [
        "xpu",
        "[73c5da0a",
        "[12345678",
        "[73c5da0a/1234567890",
        "[73c5da0a/1234567890h]tp",
    ];
    let bad_b: Vec<_> = must_blank.iter().filter(|v| kept(v)).collect();
    let bad_k: Vec<_> = must_keep.iter().filter(|v| !kept(v)).collect();
    assert!(bad_b.is_empty(), "kept: {bad_b:?}");
    assert!(bad_k.is_empty(), "blanked: {bad_k:?}");
}
