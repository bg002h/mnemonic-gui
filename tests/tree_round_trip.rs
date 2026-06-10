//! v0.32.0 (node-tree builder SPEC §4 gate 3 + the §5 P1-able cells) —
//! spec round-trip goldens + model-law cells.
//!
//! Goldens: the 5 toolkit archetype fixtures vendored at
//! `tests/fixtures/descriptor_builder/` (immutability is CYCLE-scoped —
//! the live exit-0 leg through the binary is the staleness tether).
//!
//! Round-trip laws (SPEC §1.2, R0-r1 I1):
//!   (a) `to(from(j)) == j` value-equal for any valid spec (the goldens);
//!   (b) `from(to(t)) == projection(t)` for wide nodes (stale inactive
//!       fields + surplus children deliberately do NOT serialize) —
//!       equivalently `to(from(to(t))) == to(t)` (to-idempotence).
//!
//! Plus: TreeState serde persistence round-trip; the xprv redaction cells
//! (SPEC §1.3); the next_id import invariant (R0-r1 I3); the depth cap;
//! completeness per class; kind-switch preservation; the pre-v0.32.0
//! state.json migration cell (R0-r1 M6a).

use std::io::Write;
use std::process::{Command, Stdio};

use mnemonic_gui::form::tree_model::{
    completeness, from_spec_json, key_path, max_id, node_paths, to_spec_json, TreeNode,
    TreeState, MAX_TREE_DEPTH,
};
use mnemonic_gui::persistence::redact_for_persistence;
use mnemonic_gui::schema::FormState;

const FIXTURES: &[&str] = &[
    "decaying-multisig",
    "hashlock-gated",
    "kofn-recovery",
    "simple-timelocked-inheritance",
    "tiered-recovery",
];

fn fixture(name: &str) -> serde_json::Value {
    let path = format!(
        "{}/tests/fixtures/descriptor_builder/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse fixture {path}: {e}"))
}

// ================================================================ law (a)

/// Round-trip law (a): `to(from(j)) == j` value-identical for each of the
/// 5 vendored archetype fixtures.
#[test]
fn golden_fixtures_round_trip_value_identical() {
    for name in FIXTURES {
        let doc = fixture(name);
        let root = from_spec_json(&doc)
            .unwrap_or_else(|e| panic!("{name}: from_spec_json failed: {e}"));
        // Imported trees are structurally complete.
        assert!(
            completeness(&root).is_empty(),
            "{name}: imported fixture must be structurally complete; got {:?}",
            completeness(&root)
        );
        let back = to_spec_json(&root);
        assert_eq!(back, doc, "{name}: to(from(j)) != j — round-trip law (a) broken");
    }
}

/// Imported ids are assigned 0..n in walk order (unique, dense).
#[test]
fn golden_fixture_import_ids_dense_preorder() {
    let root = from_spec_json(&fixture("tiered-recovery")).expect("import");
    let n = root.node_count() as u64;
    assert_eq!(max_id(&root), n - 1, "ids must be 0..n (dense)");
    let mut ids: Vec<u64> = node_paths(&root).iter().map(|(_, id)| *id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len() as u64, n, "ids must be unique");
    assert_eq!(root.id, 0, "root id is 0 (preorder)");
}

// ================================================== live exit-0 tether leg

/// The staleness tether: each vendored fixture must still run exit-0
/// through the live binary `--spec - --json` (skip-if-absent, the
/// archetype_schema_mirror posture).
#[test]
fn golden_fixtures_exit_zero_through_binary() {
    let bin: String = match std::env::var("MNEMONIC_BIN").ok() {
        Some(b) => b,
        None => match Command::new("mnemonic").arg("--help").output() {
            Ok(_) => "mnemonic".into(),
            Err(_) => {
                eprintln!(
                    "MNEMONIC_BIN unset + PATH lookup failed; \
                     skipping golden-fixture exit-0 tether"
                );
                return;
            }
        },
    };
    for name in FIXTURES {
        let doc = fixture(name);
        // Round-trip THROUGH the model first — what the GUI would send.
        let root = from_spec_json(&doc).expect("import");
        let spec = to_spec_json(&root);
        let mut child = Command::new(&bin)
            .args(["build-descriptor", "--spec", "-", "--json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn build-descriptor");
        {
            let mut stdin = child.stdin.take().expect("stdin piped");
            let _ = stdin.write_all(spec.to_string().as_bytes());
        }
        let out = child.wait_with_output().expect("wait_with_output");
        assert!(
            out.status.success(),
            "{name}: vendored fixture no longer validates exit-0 (stale \
             golden — re-vendor from the toolkit):\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}

// ================================================================ law (b)

/// A wide node with stale inactive fields + surplus children: or_d whose
/// payload fields are all stale-populated, a pk child carrying stale hex
/// + quorum data, and a SURPLUS third child.
fn wide_tree() -> TreeNode {
    TreeNode {
        id: 0,
        kind: "or_d".into(),
        // Stale inactive payload fields (a previous kind's data — preserved
        // by the wide-node model, never serialized).
        key: "stale-key".into(),
        k: 7,
        keys: vec!["stale-quorum-key".into()],
        n: 999,
        hex: "deadbeef".into(),
        w: "sv".into(),
        children: vec![
            TreeNode {
                id: 1,
                kind: "pk".into(),
                key: "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13".into(),
                hex: "stale".into(),
                k: 3,
                ..Default::default()
            },
            TreeNode { id: 2, kind: "older".into(), n: 144, key: "stale".into(), ..Default::default() },
            // Surplus child beyond or_d's arity (preserve-and-flag).
            TreeNode { id: 3, kind: "after".into(), n: 1, ..Default::default() },
        ],
    }
}

/// Round-trip law (b): `from(to(t)) == projection(t)` — the projection is
/// active-kind fields + the in-arity child prefix (ids renumbered 0..n by
/// the import walk). Equivalently `to(from(to(t))) == to(t)`.
#[test]
fn wide_node_projection_law() {
    let t = wide_tree();
    let j = to_spec_json(&t);
    let projected = from_spec_json(&j).expect("from(to(t))");

    // to-idempotence: to(from(to(t))) == to(t).
    assert_eq!(to_spec_json(&projected), j, "to-idempotence broken");

    // from(to(t)) == projection(t): stale fields cleared, surplus dropped.
    assert_eq!(projected.kind, "or_d");
    assert_eq!(projected.key, "", "inactive key must not survive projection");
    assert_eq!(projected.k, 0);
    assert!(projected.keys.is_empty());
    assert_eq!(projected.n, 0);
    assert_eq!(projected.hex, "");
    assert_eq!(projected.w, "");
    assert_eq!(projected.children.len(), 2, "surplus child must not serialize");
    assert_eq!(projected.children[0].kind, "pk");
    assert_eq!(projected.children[0].hex, "", "stale child hex cleared");
    assert_eq!(projected.children[0].k, 0, "stale child k cleared");
    assert_eq!(projected.children[1].kind, "older");
    assert_eq!(projected.children[1].n, 144);
    assert_eq!(projected.children[1].key, "");
}

// ============================================== TreeState serde round-trip

#[test]
fn tree_state_serde_persistence_round_trip() {
    let mut state = TreeState::fresh();
    state.enabled = true;
    state.root = wide_tree();
    state.recompute_next_id();
    // Transients populated — must NOT survive (or even appear in) the
    // serialized form.
    state.diagnostics.push(mnemonic_gui::form::tree_model::TreeDiag {
        node_path: "root".into(),
        kind: "sigless_branch".into(),
        message: "m".into(),
    });
    state.validate_ok = Some(mnemonic_gui::form::tree_model::ValidateOk {
        descriptor: "wsh(...)".into(),
        cost: serde_json::json!({}),
    });
    state.strip.push("error: transient strip notice".into());

    let raw = serde_json::to_string(&state).expect("serialize TreeState");
    assert!(
        !raw.contains("diagnostics") && !raw.contains("validate_ok") && !raw.contains("strip"),
        "transient fields must be #[serde(skip)] — absent from the wire"
    );
    let back: TreeState = serde_json::from_str(&raw).expect("deserialize TreeState");
    assert_eq!(back.enabled, state.enabled);
    assert_eq!(back.next_id, state.next_id);
    assert_eq!(back.root, state.root, "the full wide tree (incl. surplus) persists");
    assert!(back.diagnostics.is_empty(), "diagnostics never persist");
    assert!(back.validate_ok.is_none(), "validate_ok never persists");
    assert!(back.strip.is_empty(), "the error strip never persists");
}

// ===================================================== xprv redaction cells

const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
const SHA_DIGEST: &str = "926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc";

/// SPEC §1.3: an xprv in a tree key field does not survive a redact
/// round-trip; xpubs + hashlock hex digests do. The walk covers `key`,
/// `keys[i]`, origin-prefixed xprvs, and SURPLUS children.
#[test]
fn redaction_blanks_xprv_keys_keeps_xpub_and_hex() {
    let xpub = "xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
    let mut tree = TreeState::fresh();
    tree.enabled = true;
    tree.root = TreeNode {
        id: 0,
        kind: "or_d".into(),
        children: vec![
            TreeNode {
                id: 1,
                kind: "multi".into(),
                k: 2,
                keys: vec![
                    xpub.to_string(),
                    XPRV.to_string(),
                    format!("[11111111/48h/0h/0h/2h]{XPRV}"), // origin-prefixed leg
                ],
                ..Default::default()
            },
            TreeNode { id: 2, kind: "sha256".into(), hex: SHA_DIGEST.into(), ..Default::default() },
            // Surplus child holding a secret in its scalar key field — the
            // surplus subtree PERSISTS, so the walk must cover it.
            TreeNode { id: 3, kind: "pk".into(), key: XPRV.into(), ..Default::default() },
        ],
        ..Default::default()
    };
    tree.recompute_next_id();

    let mut form = FormState::from_pairs(Vec::<(String, mnemonic_gui::schema::FlagValue)>::new());
    form.tree = Some(tree);

    let redacted = redact_for_persistence(&form);
    let rtree = redacted.tree.as_ref().expect("tree field survives redaction");
    assert!(rtree.enabled, "non-secret tree state survives");
    let quorum = &rtree.root.children[0];
    assert_eq!(quorum.keys[0], xpub, "xpub survives");
    assert_eq!(quorum.keys[1], "", "bare xprv blanked");
    assert_eq!(quorum.keys[2], "", "origin-prefixed xprv blanked");
    assert_eq!(quorum.k, 2, "non-key payload survives");
    assert_eq!(
        rtree.root.children[1].hex, SHA_DIGEST,
        "hashlock hex deliberately NOT redacted (public commitment)"
    );
    assert_eq!(rtree.root.children[2].key, "", "xprv in a SURPLUS child blanked");
    assert!(rtree.diagnostics.is_empty() && rtree.validate_ok.is_none());

    // And the on-disk JSON never carries the secret bytes.
    let raw = serde_json::to_string(&redacted).expect("serialize");
    assert!(!raw.contains(XPRV), "no xprv bytes on the wire");
    assert!(raw.contains(SHA_DIGEST) && raw.contains(xpub));
}

// ================================================== next_id import invariant

/// R0-r1 I3: every populate-from-import site recomputes
/// `next_id = max_id(root) + 1` — post-import adds must not collide with
/// imported ids (egui identity).
#[test]
fn next_id_recomputed_after_import_no_collision() {
    let mut state = TreeState::fresh();
    // Stale-low next_id scenario: fresh state has next_id == 1, the
    // imported fixture carries many more nodes.
    let root = from_spec_json(&fixture("decaying-multisig")).expect("import");
    let imported_max = max_id(&root);
    assert!(imported_max + 1 > state.next_id, "scenario premise: stale-low next_id");
    state.import_root(root);
    assert_eq!(state.next_id, imported_max + 1, "the named recompute helper invariant");

    // A post-import add yields a fresh, non-colliding id.
    let existing: Vec<u64> = node_paths(&state.root).iter().map(|(_, id)| *id).collect();
    let new_id = state.alloc_id();
    assert!(!existing.contains(&new_id), "post-import id collides with an imported id");
}

// ============================================================== depth cap

fn nested_wrap_spec(wrap_count: usize) -> serde_json::Value {
    let mut node = serde_json::json!({ "older": 1 });
    for _ in 0..wrap_count {
        node = serde_json::json!({ "wrap": { "w": "v", "sub": node } });
    }
    serde_json::json!({ "schema_version": 1, "wrapper": "wsh", "root": node })
}

#[test]
fn depth_cap_enforced_on_import_and_add() {
    // 63 wraps + leaf = depth 64 — at the cap, accepted.
    assert!(from_spec_json(&nested_wrap_spec(MAX_TREE_DEPTH - 1)).is_ok());
    // 64 wraps + leaf = depth 65 — beyond the cap, path-addressed refusal.
    let err = from_spec_json(&nested_wrap_spec(MAX_TREE_DEPTH)).expect_err("must refuse");
    assert!(
        err.contains("maximum tree depth") && err.contains("64"),
        "depth refusal must carry the cap: {err}"
    );

    // Add-child refusal at the cap.
    let mut node = TreeNode::new_unset(0);
    let mut next_id = 1u64;
    assert!(node.try_add_child(MAX_TREE_DEPTH, &mut next_id).is_err());
    assert!(node.children.is_empty(), "refused add must not mutate");
    // set_kind needing placeholder padding refuses too; the kind is
    // left unchanged (amber message, no partial switch).
    assert!(node.set_kind("and_v", MAX_TREE_DEPTH, &mut next_id).is_err());
    assert_eq!(node.kind, "", "refused kind switch must not mutate");
    // ... but a LEAF kind at max depth is fine.
    assert!(node.set_kind("older", MAX_TREE_DEPTH, &mut next_id).is_ok());
}

// ================================================ import refusal cells (M5)

#[test]
fn import_rejects_unknown_kind_and_fields_path_addressed() {
    let bad_kind = serde_json::json!({
        "schema_version": 1, "wrapper": "wsh",
        "root": { "or_d": [ { "pk": "x" }, { "bogus": 1 } ] }
    });
    let err = from_spec_json(&bad_kind).expect_err("unknown kind");
    assert!(
        err.contains("root.or_d[1]") && err.contains("bogus"),
        "unknown-kind refusal must be path-addressed: {err}"
    );

    let bad_field = serde_json::json!({
        "schema_version": 1, "wrapper": "wsh",
        "root": { "multi": { "k": 1, "keys": ["x"], "extra": true } }
    });
    let err = from_spec_json(&bad_field).expect_err("unknown field");
    assert!(
        err.contains("root") && err.contains("extra"),
        "unknown-field refusal must be path-addressed: {err}"
    );

    let bad_top = serde_json::json!({
        "schema_version": 1, "wrapper": "wsh", "root": { "older": 1 }, "surprise": 1
    });
    assert!(from_spec_json(&bad_top).expect_err("top-level field").contains("surprise"));
}

/// R0-r1 M5: runtime parity with the test-time refuse-loud version pins —
/// `schema_version == 1` and `wrapper == "wsh"` checked on import.
#[test]
fn import_rejects_wrong_version_and_wrapper() {
    let v2 = serde_json::json!({ "schema_version": 2, "wrapper": "wsh", "root": { "older": 1 } });
    assert!(from_spec_json(&v2).expect_err("v2").contains("schema_version"));
    let tr = serde_json::json!({ "schema_version": 1, "wrapper": "tr", "root": { "older": 1 } });
    assert!(from_spec_json(&tr).expect_err("tr").contains("wrapper"));
}

// ======================================================= completeness cells

fn complete_tree() -> TreeNode {
    // or_d( multi(1, [xpub]), older(144) ) — minimal complete tree.
    TreeNode {
        id: 0,
        kind: "or_d".into(),
        children: vec![
            TreeNode {
                id: 1,
                kind: "multi".into(),
                k: 1,
                keys: vec!["xpub-ish".into()],
                ..Default::default()
            },
            TreeNode { id: 2, kind: "older".into(), n: 144, ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Each structural-incompleteness class blocks (R0-r1 M3: STRUCTURAL
/// emptiness only — k>n / hex validity / key format are the toolkit
/// gate's, returned node-addressed); a complete tree unblocks.
#[test]
fn completeness_classes() {
    // Complete tree → empty.
    assert!(completeness(&complete_tree()).is_empty());

    // Unset kind (the fresh placeholder).
    assert_eq!(completeness(&TreeNode::new_unset(0)), vec!["root".to_string()]);

    // Empty key on a key kind.
    let t = TreeNode { kind: "pk".into(), ..Default::default() };
    assert_eq!(completeness(&t), vec!["root".to_string()]);

    // k<1 on a quorum (the UNSET-sentinel reading of the wide-node k:i64
    // default 0) — and an empty key ROW is key-path-addressed.
    let t = TreeNode { kind: "multi".into(), k: 0, keys: vec!["x".into(), String::new()], ..Default::default() };
    assert_eq!(
        completeness(&t),
        vec!["root".to_string(), key_path("root", "multi", 1)]
    );

    // Quorum with no key rows at all.
    let t = TreeNode { kind: "sortedmulti".into(), k: 1, ..Default::default() };
    assert_eq!(completeness(&t), vec!["root".to_string()]);

    // Empty hex.
    let t = TreeNode { kind: "sha256".into(), ..Default::default() };
    assert_eq!(completeness(&t), vec!["root".to_string()]);

    // Empty w on wrap (child present + complete).
    let t = TreeNode {
        kind: "wrap".into(),
        children: vec![TreeNode { kind: "older".into(), n: 1, id: 1, ..Default::default() }],
        ..Default::default()
    };
    assert_eq!(completeness(&t), vec!["root".to_string()]);

    // thresh with k<1 / no branches.
    let t = TreeNode { kind: "thresh".into(), ..Default::default() };
    assert_eq!(completeness(&t), vec!["root".to_string()]);

    // In-arity UNSET child is path-addressed; missing in-arity children
    // flag the parent.
    let mut t = complete_tree();
    t.children[1] = TreeNode::new_unset(9);
    assert_eq!(completeness(&t), vec!["root.or_d[1]".to_string()]);
    t.children.pop();
    assert_eq!(completeness(&t), vec!["root".to_string()]);

    // Incomplete SURPLUS children do NOT block (they don't emit).
    let mut t = complete_tree();
    t.children.push(TreeNode::new_unset(7));
    assert!(completeness(&t).is_empty(), "surplus never blocks");
}

// ================================================ kind-switch preservation

/// SPEC §5: kind switches never destroy payload data — multi→thresh keeps
/// k; pk→pkh keeps key; children are kept verbatim (surplus does not emit
/// — covered by the projection-law cell).
#[test]
fn kind_switch_preservation() {
    let mut next_id = 100u64;

    // multi → thresh keeps k (and the keys data stays for a switch back).
    let mut n = TreeNode {
        id: 0,
        kind: "multi".into(),
        k: 2,
        keys: vec!["a".into(), "b".into()],
        ..Default::default()
    };
    n.set_kind("thresh", 1, &mut next_id).unwrap();
    assert_eq!(n.kind, "thresh");
    assert_eq!(n.k, 2, "multi→thresh keeps k");
    assert_eq!(n.keys, vec!["a".to_string(), "b".to_string()], "keys data preserved");

    // pk → pkh keeps key.
    let mut n = TreeNode { id: 1, kind: "pk".into(), key: "xpub-ish".into(), ..Default::default() };
    n.set_kind("pkh", 1, &mut next_id).unwrap();
    assert_eq!(n.kind, "pkh");
    assert_eq!(n.key, "xpub-ish", "pk→pkh keeps key");

    // Parent → leaf keeps children verbatim (now ALL surplus; nothing
    // destroyed, nothing emitted).
    let mut n = complete_tree();
    n.set_kind("pk", 1, &mut next_id).unwrap();
    assert_eq!(n.children.len(), 2, "children preserved on switch to a leaf kind");
    let j = to_spec_json(&n);
    assert!(j["root"]["pk"].is_string(), "leaf projection emits no children");

    // Leaf → fixed-arity pads with fresh unset placeholders.
    let mut n = TreeNode { id: 5, kind: "older".into(), n: 1, ..Default::default() };
    let before = next_id;
    n.set_kind("andor", 1, &mut next_id).unwrap();
    assert_eq!(n.children.len(), 3);
    assert_eq!(next_id, before + 3);
    assert_eq!(n.n, 1, "uint payload preserved for a switch back");
}

// ===================================================== migration cell (M6a)

/// A pre-v0.32.0 `state.json` FormState (no `tree` field) loads to
/// `tree == None` → non-tree mode. Built by serializing a CURRENT
/// FormState and physically REMOVING the `tree` key (absent ≠ null).
#[test]
fn pre_v0_32_form_state_without_tree_field_loads_none() {
    let form = FormState::from_pairs(vec![(
        "--network".to_string(),
        mnemonic_gui::schema::FlagValue::Dropdown("mainnet".into()),
    )]);
    let mut as_value = serde_json::to_value(&form).expect("serialize FormState");
    let obj = as_value.as_object_mut().expect("FormState serializes as an object");
    assert!(obj.remove("tree").is_some(), "premise: current shape carries `tree`");

    let migrated: FormState =
        serde_json::from_value(as_value).expect("pre-v0.32.0 state.json must load clean");
    assert!(migrated.tree.is_none(), "missing tree field → None (non-tree mode)");
    assert_eq!(migrated.values.len(), 1, "the rest of the form survives");
}
