//! cycle-15 Lane G — on-screen masking of secret-input widgets (slugs 3 & 4).
//!
//! Two secret-input `text_edit_singleline` widgets render cleartext today:
//! slug 3 is the composite VALUE field (`form/widget.rs` NodeValueComposite
//! arm) for an argv-secret node (e.g. `--from minikey=…`, `--from phrase=…`);
//! slug 4 is the tree-builder Key / KeyQuorum fields (`form/tree_form.rs`) when
//! the entered key is `is_xprv_like` (a mis-pasted private key). Both must
//! render `.password`-masked (`Role::PasswordInput`) while leaving the
//! WATCH-ONLY case (xpub) readable.
//!
//! - T5 (composite, kittest): a secret node's value field is PasswordInput; a
//!   non-secret node's is NOT.
//! - T5b (M3 regression): the `.password` swap preserves the paste-warn co-fire
//!   for a secret composite (the `ui.add(...)` form still returns the Response).
//! - T6 (split-brain pin): the masking gate == the argv-secret gate for ALL
//!   composite node types.
//! - T7 (tree key, kittest): a Key/KeyQuorum field holding an `is_xprv_like`
//!   string is PasswordInput; an xpub-shaped (watch-only) one is NOT. Driven
//!   through the PUBLIC `tree_form::render` with a constructed Key-node tree
//!   (the proven `tree_form_harness` seam — no API widening).

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::secret_widget::paste_warn_id;
use mnemonic_gui::form::tree_form;
use mnemonic_gui::form::tree_model::{TreeNode, TreeState};
use mnemonic_gui::form::widget;
use mnemonic_gui::schema::{self, FlagSchema, FlagValue, FormState};
use mnemonic_gui::secrets::{self, PASTE_WARN_THRESHOLD};

// ── composite harness (mirrors h3_minikey_paste_warn.rs) ─────────────────────

/// The `convert --from` FlagSchema (NodeValueComposite over CONVERT_FROM_NODES,
/// which includes the argv-secret `minikey`/`phrase` and the public `xpub`).
fn from_flag() -> &'static FlagSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "convert")
        .expect("convert subcommand")
        .flags
        .iter()
        .find(|f| f.name == "--from")
        .expect("convert --from flag")
}

fn composite_harness(node: &'static str) -> Harness<'static, FlagValue> {
    let initial = FlagValue::NodeValueComposite {
        node: node.into(),
        value: String::new(),
    };
    Harness::new_ui_state(
        move |ui, value: &mut FlagValue| {
            let state = FormState::default();
            widget::render(
                ui,
                CliTab::Mnemonic,
                "convert",
                from_flag(),
                value,
                &state,
                &[],
            );
        },
        initial,
    )
}

// ── T5 — composite secret masking ────────────────────────────────────────────

#[test]
fn t5_secret_composite_value_renders_as_password_field() {
    let mut h = composite_harness("phrase");
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_some(),
        "a secret-class composite node's value field must render masked (PasswordInput)"
    );
}

#[test]
fn t5_secret_composite_minikey_renders_as_password_field() {
    let mut h = composite_harness("minikey");
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_some(),
        "the argv-secret minikey composite value must render masked (PasswordInput)"
    );
}

#[test]
fn t5b_non_secret_composite_value_is_not_masked() {
    let mut h = composite_harness("xpub");
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_none(),
        "a watch-only (xpub) composite value must NOT render as a password field"
    );
}

// ── T5b — M3 regression: paste-warn co-fires under the .password swap ─────────

#[test]
fn t5b_secret_composite_paste_warn_still_co_fires_after_mask_swap() {
    let mut h = composite_harness("minikey");
    h.run();
    // The masked value field is now PasswordInput (slug-3 swap). Focus it and
    // inject an over-threshold paste — the `ui.add(...)` form must STILL return
    // the Response so `response.changed()` drives the paste-warn bus flag.
    h.get_by_role(egui::accesskit::Role::PasswordInput).focus();
    h.run();
    h.input_mut()
        .events
        .push(egui::Event::Paste("x".repeat(PASTE_WARN_THRESHOLD + 4)));
    h.run();
    let flag = h.ctx.data_mut(|d| d.get_temp::<bool>(paste_warn_id()));
    assert_eq!(
        flag,
        Some(true),
        "the .password swap must NOT regress the composite paste-warn (M3): an \
         over-threshold paste into the masked secret value must still raise the bus flag"
    );
}

// ── T6 — split-brain pin (masking gate == argv-secret gate) ──────────────────

#[test]
fn t6_composite_masking_gate_is_a_single_hoisted_source_with_paste_warn() {
    // Structural split-brain pin (mirrors paste_warn_wiring_v0_40_0.rs's
    // source-introspection tripwire). The slug-3 anti-split-brain guarantee is
    // that ONE hoisted `is_secret_node` boolean physically drives BOTH the
    // `.password(..)` mask AND the paste-warn `&& is_secret_node` condition —
    // so render-mask and paste-warn classification cannot diverge. A behavioral
    // assert (`f(x) == f(x)`) is tautological; pin the structure instead:
    //   1. the composite arm computes `node_type_is_argv_secret(node)` exactly
    //      ONCE (a future refactor re-inlining a second call would re-introduce
    //      the double-eval drift this hoist removed), and
    //   2. that single gate feeds `.password(is_secret_node)`.
    // Whitespace-flattened so a future line-wrap can't evade the match.
    let widget = include_str!("../src/form/widget.rs");
    let flat: String = widget.split_whitespace().collect();
    assert_eq!(
        flat.matches("node_type_is_argv_secret(node.as_str())").count(),
        1,
        "the composite arm must compute node_type_is_argv_secret(node) exactly \
         ONCE (hoisted) — a second inline call would re-split the render-mask \
         gate from the paste-warn gate"
    );
    assert!(
        flat.contains(".password(is_secret_node&&!reveal)"),
        "the hoisted is_secret_node gate must drive the composite value \
         `.password(..)` mask (v0.57.0: ANDed with the reveal predicate — the \
         eye only ever un-masks a field that IS secret-masked; the hoisted \
         `is_secret_node` gate still solely decides maskedness)"
    );
    // Value-level floor: the gate the impl masks on IS the argv-secret
    // predicate, with BOTH branches exercised across the composite's nodes (a
    // watch-only xpub stays readable, a secret phrase/minikey masks).
    let nodes: &[&str] = match from_flag().kind {
        schema::FlagKind::NodeValueComposite(opts) => opts,
        _ => panic!("--from must be a NodeValueComposite flag"),
    };
    assert!(!nodes.is_empty(), "the composite must offer at least one node");
    assert!(
        nodes.iter().any(|n| secrets::node_type_is_argv_secret(n)),
        "at least one composite node must be argv-secret (so masking is exercised)"
    );
    assert!(
        nodes.iter().any(|n| !secrets::node_type_is_argv_secret(n)),
        "at least one composite node must be non-secret (readable branch exercised)"
    );
}

// ── T7 — tree key masking (driven through the PUBLIC tree_form::render) ───────

fn assign_ids(node: &mut TreeNode, next: &mut u64) {
    node.id = *next;
    *next += 1;
    for c in &mut node.children {
        assign_ids(c, next);
    }
}

fn enabled_tree(mut root: TreeNode) -> TreeState {
    let mut next = 0u64;
    assign_ids(&mut root, &mut next);
    let mut state = TreeState::fresh();
    state.enabled = true;
    state.root = root;
    state.recompute_next_id();
    state
}

fn form_with_tree(tree: TreeState) -> FormState {
    FormState { tree: Some(tree), ..Default::default() }
}

fn tree_form_harness(initial: FormState) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            tree_form::render(ui, state, "mnemonic");
        },
        initial,
    )
}

/// A `pk` node (PayloadShape::Key) carrying `key`.
fn pk_node(key: &str) -> TreeNode {
    TreeNode { kind: "pk".into(), key: key.into(), ..Default::default() }
}

/// A `multi` node (PayloadShape::KeyQuorum) with k + one key in `keys[0]`.
fn multi_node(key: &str) -> TreeNode {
    TreeNode { kind: "multi".into(), k: 1, keys: vec![key.into()], ..Default::default() }
}

const XPRV_SHAPED: &str = "xprv9s21ZrQH143K3QTDL4LXw2F";
const XPUB_SHAPED: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d";

#[test]
fn t7_tree_key_field_masks_when_xprv_shaped() {
    let mut h = tree_form_harness(form_with_tree(enabled_tree(pk_node(XPRV_SHAPED))));
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_some(),
        "a Key field holding an xprv-shaped (private) string must render masked"
    );
}

#[test]
fn t7_tree_key_field_not_masked_when_xpub_shaped() {
    let mut h = tree_form_harness(form_with_tree(enabled_tree(pk_node(XPUB_SHAPED))));
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_none(),
        "a Key field holding a watch-only xpub must NOT render masked"
    );
}

#[test]
fn t7_tree_keyquorum_field_masks_when_xprv_shaped() {
    let mut h = tree_form_harness(form_with_tree(enabled_tree(multi_node(XPRV_SHAPED))));
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_some(),
        "a KeyQuorum keys[i] field holding an xprv-shaped string must render masked"
    );
}

#[test]
fn t7_tree_keyquorum_field_not_masked_when_xpub_shaped() {
    let mut h = tree_form_harness(form_with_tree(enabled_tree(multi_node(XPUB_SHAPED))));
    h.run();
    assert!(
        h.query_by_role(egui::accesskit::Role::PasswordInput).is_none(),
        "a KeyQuorum keys[i] field holding a watch-only xpub must NOT render masked"
    );
}
