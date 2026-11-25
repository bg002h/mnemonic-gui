//! Secret M/N batch gui-2 (architect-secret-mn-decisions-1124 G1, G3).
//! Fake signet-shaped vectors only (counting bytes, no real key).

use mnemonic_gui::form::tree_form::{self, PRIVATE_COPY_REASON};
use mnemonic_gui::form::tree_model::{TreeNode, TreeState};
use mnemonic_gui::schema::FormState;

const XPUB: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
const FWIF: &str = "cMecTxSiV5a44c4jqgSqYmV9bFAWNV43oPhkqu1Ty3GapoXoCSdq";
const DIGEST: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

fn persisted(node: TreeNode) -> TreeNode {
    let state = TreeState {
        root: node,
        ..Default::default()
    };
    state.redacted_for_persistence().root
}

#[test]
fn gui_tree_hexw_nonxprv_private_persists_wif_blanks_in_hex_and_w() {
    let r = persisted(TreeNode {
        hex: FWIF.into(),
        w: FWIF.into(),
        ..Default::default()
    });
    assert_eq!(
        r.hex, "",
        "a WIF in the hex field must be blanked at persist"
    );
    assert_eq!(r.w, "", "a WIF in the w field must be blanked at persist");
}

#[test]
fn gui_tree_hexw_nonxprv_private_persists_64_hex_digest_and_wrapper_persist() {
    let r = persisted(TreeNode {
        hex: DIGEST.into(),
        w: "sv".into(),
        ..Default::default()
    });
    assert_eq!(r.hex, DIGEST);
    assert_eq!(r.w, "sv");
}

fn complete_tree(hex_field_wif: bool) -> FormState {
    // sha256(pk, ...) is not needed: a single pk root is structurally complete.
    let mut root = TreeNode {
        kind: "pk".into(),
        key: XPUB.into(),
        ..Default::default()
    };
    if hex_field_wif {
        root.w = FWIF.into();
    }
    let mut tree = TreeState::fresh();
    tree.enabled = true;
    tree.root = root;
    tree.recompute_next_id();
    FormState {
        tree: Some(tree),
        ..Default::default()
    }
}

#[test]
fn gui_tree_spec_json_unmasked_copy_refused_when_tree_holds_private_shape() {
    let argv = vec!["mnemonic".to_string(), "build-descriptor".to_string()];
    let state = complete_tree(true);
    assert_eq!(
        tree_form::tree_copy_blocker(&state),
        Some(PRIVATE_COPY_REASON)
    );
    assert_eq!(
        PRIVATE_COPY_REASON,
        "the tree holds a private key; Run sends it privately, Copy would not"
    );
    let (posix, windows, spec) = tree_form::tree_copy_texts(&state, &argv);
    assert!(
        posix.is_none() && windows.is_none() && spec.is_none(),
        "all three copies disabled"
    );
    // Stdin (Run's private channel) is unchanged.
    assert!(tree_form::spec_stdin_bytes(&state).is_some());
}

#[test]
fn gui_tree_spec_json_unmasked_public_tree_copies_stay_enabled() {
    let argv = vec!["mnemonic".to_string(), "build-descriptor".to_string()];
    let state = complete_tree(false);
    assert_eq!(tree_form::tree_copy_blocker(&state), None);
    let (posix, windows, spec) = tree_form::tree_copy_texts(&state, &argv);
    assert!(posix.is_some() && windows.is_some() && spec.is_some());
}
