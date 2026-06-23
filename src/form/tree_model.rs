//! v0.32.0 (node-tree builder SPEC §1.2) — the GUI-owned "wide node" tree
//! model behind the recursive build-descriptor tree builder, plus the
//! spec-JSON (de)serializers, the toolkit-gate-mirroring path grammar, the
//! structural completeness gate, and the persistence redaction walk.
//!
//! Wide node: all payload fields coexist on every node so kind switches
//! never destroy data (preserve-and-flag — brainstorm decision §3);
//! serialization projects ONLY the active kind's fields + the in-arity
//! child prefix.
//!
//! Round-trip laws (SPEC §1.2, R0-r1 I1 — the naive `from(to(t)) == t` is
//! FALSE for wide nodes):
//!   (a) `to(from(j)) == j` value-equal for any valid spec;
//!   (b) `from(to(t)) == projection(t)` (equivalently `to(from(to(t))) ==
//!       to(t)` — to-idempotence).
//!
//! Path grammar (toolkit `gate.rs::child_paths` + the `keys[i]` key paths;
//! 5 productions): `root`; `{path}.{kind}[i]` (binary + andor);
//! `{path}.thresh.subs[i]`; `{path}.wrap.sub`; `{path}.{kind}.keys[i]`.
//! The LIVE parity gate (`tests/tree_path_parity.rs`) is the tether.

use serde::{Deserialize, Serialize};

use crate::schema::nodes::{spec_for, PayloadShape};

/// Defensive recursion-depth cap (brainstorm I2 — the toolkit gate's
/// complexity envelope bounds keys+hashes, NOT depth; serde_json's
/// 128-level default is the toolkit's backstop, not the GUI's). Root is
/// depth 1. Add-child / paste-import refuse beyond it (amber message);
/// the serializers assert it defensively.
pub const MAX_TREE_DEPTH: usize = 64;

/// The build-descriptor tree-builder state. Persisted inside
/// `FormState.tree` (SPEC §1.3); diagnostics + validate_ok are transient
/// by type (`#[serde(skip)]` — never persisted, cleared on ANY tree
/// mutation via the `mark_dirty` chokepoint).
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeState {
    /// The tree-mode bit — the ONLY stored mode state (R0-r1 I4):
    /// Generic-vs-Archetype stays dropdown-derived exactly as v0.31.0.
    #[serde(default)]
    pub enabled: bool,
    /// Next fresh node id (egui identity). Every populate-from-import
    /// site MUST recompute via `recompute_next_id` (R0-r1 I3).
    #[serde(default)]
    pub next_id: u64,
    #[serde(default)]
    pub root: TreeNode,
    /// Parsed node-addressed diagnostics from the last Validate run.
    /// Transient: `#[serde(skip)]` (never persists); cleared on any tree
    /// mutation (`mark_dirty`). `TreeDiag` deliberately does NOT derive
    /// `Serialize` — never-persist by type.
    #[serde(skip)]
    pub diagnostics: Vec<TreeDiag>,
    /// Descriptor + cost summary from the last successful Validate.
    /// Transient, same discipline as `diagnostics`.
    #[serde(skip)]
    pub validate_ok: Option<ValidateOk>,
    /// v0.32.0 P2 — the GLOBAL error strip (SPEC §2.2 / R0-r1 I5
    /// fail-soft): unmatched-node_path diagnostics (drift, the `"params"`
    /// sentinel, stale paths), non-envelope stderr, the `--allow` override
    /// note on a green Validate, and depth-cap refusal notices. Transient,
    /// same `mark_dirty` lifetime as `diagnostics`.
    #[serde(skip)]
    pub strip: Vec<String>,
}

/// One GUI tree node — the "wide node": every payload field coexists so
/// kind switches never destroy data. `#[serde(default)]` per field keeps
/// persisted trees forward-compatible (additive fields load clean).
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Stable egui identity (survives sibling removal). Unique per tree.
    #[serde(default)]
    pub id: u64,
    /// Node kind (a `NODE_KIND_SPECS` kind) or `""` = unset placeholder.
    #[serde(default)]
    pub kind: String,
    /// `Key` shape (`pk`/`pkh`).
    #[serde(default)]
    pub key: String,
    /// Quorum `k` (`multi`/`sortedmulti`/`thresh`). Wide-node default `0`
    /// is the UNSET sentinel (`k < 1` = structurally incomplete —
    /// R0-r1 M3: NO semantic `k <= n` check GUI-side; the gate owns it).
    #[serde(default)]
    pub k: i64,
    /// Quorum key rows (`multi`/`sortedmulti`).
    #[serde(default)]
    pub keys: Vec<String>,
    /// `Uint` shape (`older`/`after`).
    #[serde(default)]
    pub n: i64,
    /// Hex digest (`sha256`/`hash256`/`hash160`/`ripemd160`). Deliberately
    /// NOT redacted at persist (digests are public commitments; preimages
    /// never enter the spec).
    #[serde(default)]
    pub hex: String,
    /// Wrap prefix string (`wrap`), e.g. `"v"` / `"sv"`.
    #[serde(default)]
    pub w: String,
    /// Children. May exceed the active kind's arity (preserve-and-flag:
    /// surplus children render flagged and do NOT emit).
    #[serde(default)]
    pub children: Vec<TreeNode>,
}

/// One node-addressed diagnostic parsed from the binary's failure
/// envelope. No `Serialize` derive — never persisted, by type.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeDiag {
    pub node_path: String,
    pub kind: String,
    pub message: String,
}

/// Successful-Validate summary (descriptor + the raw `cost` object the
/// P2 view renders). No `Serialize` derive — transient by type.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidateOk {
    pub descriptor: String,
    pub cost: serde_json::Value,
}

impl TreeState {
    /// A fresh tree: one unset placeholder root (id 0), `next_id` past it.
    pub fn fresh() -> Self {
        TreeState {
            enabled: false,
            next_id: 1,
            root: TreeNode::new_unset(0),
            diagnostics: Vec::new(),
            validate_ok: None,
            strip: Vec::new(),
        }
    }

    /// THE mutation chokepoint (SPEC §2.2 / R0-r1 I4 lifetime rule): every
    /// widget WRITE calls this — stale diagnostics tint the wrong node
    /// after sibling removal shifts `[i]` indices (a funds-tool
    /// correctness bug). Collapse/expand lives in egui memory, not
    /// TreeState, and must NOT route through here.
    pub fn mark_dirty(&mut self) {
        self.diagnostics.clear();
        self.validate_ok = None;
        self.strip.clear();
    }

    /// Allocate a fresh node id.
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// THE named next_id-recompute helper (R0-r1 I3 import invariant):
    /// `next_id = max_id(root) + 1`. Every populate-from-import site MUST
    /// call this — a stale-low `next_id` after import collides ids with
    /// imported nodes and breaks egui identity.
    pub fn recompute_next_id(&mut self) {
        self.next_id = max_id(&self.root) + 1;
    }

    /// Populate-from-import chokepoint: installs an imported root,
    /// recomputes `next_id`, and clears transient results.
    pub fn import_root(&mut self, root: TreeNode) {
        self.root = root;
        self.recompute_next_id();
        self.mark_dirty();
    }

    /// Persistence redaction (SPEC §1.3): clone with every node's
    /// `key`/`keys` entry BLANKED when NOT positively extended-public
    /// (recursive walk over ALL children incl. surplus — they persist too).
    /// Hashlock `hex` (and the `w` wrapper) are NOT redacted when they hold
    /// legitimate digest/wrap content; an xprv-SHAPED mis-paste into either
    /// is blanked via the narrow `is_xprv_like` deny-list (Wave-2 G2 —
    /// preserves valid digests/wrap strings; belt-and-suspenders against a
    /// private key fat-fingered into a non-key field). Transient fields come
    /// back empty (they are `#[serde(skip)]` anyway — belt and suspenders).
    pub fn redacted_for_persistence(&self) -> TreeState {
        let mut root = self.root.clone();
        blank_non_extended_public_keys(&mut root);
        TreeState {
            enabled: self.enabled,
            next_id: self.next_id,
            root,
            diagnostics: Vec::new(),
            validate_ok: None,
            strip: Vec::new(),
        }
    }
}

impl TreeNode {
    /// A fresh unset placeholder (`kind == ""`).
    pub fn new_unset(id: u64) -> Self {
        TreeNode { id, ..Default::default() }
    }

    /// Kind switch — preserve-and-flag (SPEC §1.2): payload fields and
    /// existing children are kept verbatim; fixed-arity kinds pad
    /// `children` with unset placeholders up to arity (2/3/1; ThreshSubs
    /// starts at 0 + "add branch"). `node_depth` is THIS node's depth
    /// (root = 1); padding that would create children beyond
    /// `MAX_TREE_DEPTH` refuses the switch (amber message — the kind is
    /// left unchanged).
    pub fn set_kind(
        &mut self,
        kind: &str,
        node_depth: usize,
        next_id: &mut u64,
    ) -> Result<(), String> {
        let arity = spec_for(kind).and_then(|s| fixed_child_arity(s.payload)).unwrap_or(0);
        if arity > self.children.len() && node_depth + 1 > MAX_TREE_DEPTH {
            return Err(format!(
                "cannot select `{kind}` here: children would exceed the \
                 maximum tree depth ({MAX_TREE_DEPTH})"
            ));
        }
        self.kind = kind.to_string();
        while self.children.len() < arity {
            let id = *next_id;
            *next_id += 1;
            self.children.push(TreeNode::new_unset(id));
        }
        Ok(())
    }

    /// Add one unset child (the `thresh` "add branch" gesture).
    /// `node_depth` is THIS node's depth (root = 1); refuses beyond
    /// `MAX_TREE_DEPTH` (amber message).
    pub fn try_add_child(
        &mut self,
        node_depth: usize,
        next_id: &mut u64,
    ) -> Result<&mut TreeNode, String> {
        if node_depth + 1 > MAX_TREE_DEPTH {
            return Err(format!(
                "cannot add a branch here: maximum tree depth ({MAX_TREE_DEPTH}) reached"
            ));
        }
        let id = *next_id;
        *next_id += 1;
        self.children.push(TreeNode::new_unset(id));
        Ok(self.children.last_mut().expect("just pushed"))
    }

    /// Total node count (header strip `N nodes`). Counts ALL children
    /// incl. surplus (what renders).
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(TreeNode::node_count).sum::<usize>()
    }

    /// M9 (cycle-11a): in-RAM secret scrub for the exit zeroize sweep
    /// (`secrets::zeroize_form_state`). Recursively zeroizes every
    /// secret-bearing `key`/`keys[i]` (xprv/WIF/raw-hex private keys the
    /// user can type) through ALL `children` (incl. surplus — they hold
    /// secrets too). EXCLUDES `hex` — a public digest commitment, never a
    /// private preimage (same exclusion as the on-disk redactor
    /// `blank_non_extended_public_keys`, which leaves `hex` untouched).
    /// `w`/`kind`/`n`/`k`/`id` are non-secret structural fields.
    pub fn zeroize_keys(&mut self) {
        use zeroize::Zeroize;
        self.key.zeroize();
        for k in &mut self.keys {
            k.zeroize();
        }
        for child in &mut self.children {
            child.zeroize_keys();
        }
    }
}

/// Fixed child arity per shape: `Some(2)` / `Some(3)` / `Some(1)` for
/// Children2 / Children3 / Wrap; `None` for ThreshSubs (variable, 0+);
/// `Some(0)` for the leaf shapes.
pub fn fixed_child_arity(shape: PayloadShape) -> Option<usize> {
    match shape {
        PayloadShape::Children2 => Some(2),
        PayloadShape::Children3 => Some(3),
        PayloadShape::Wrap => Some(1),
        PayloadShape::ThreshSubs => None,
        PayloadShape::Key
        | PayloadShape::KeyQuorum
        | PayloadShape::Uint
        | PayloadShape::Hex64
        | PayloadShape::Hex40 => Some(0),
    }
}

/// How many leading children EMIT (and are path-addressable) for a node:
/// the in-arity prefix for fixed-arity kinds, ALL children for `thresh`,
/// 0 for leaves / unset kind. Clamped to `children.len()`. `pub` since
/// v0.32.0 P2 — the tree form's renderer needs the same boundary to place
/// the surplus-children amber flags (SPEC §2.2).
pub fn emitted_child_count(node: &TreeNode) -> usize {
    let Some(spec) = spec_for(&node.kind) else { return 0 };
    match fixed_child_arity(spec.payload) {
        Some(arity) => arity.min(node.children.len()),
        None => node.children.len(),
    }
}

// ======================================================================
// Path grammar (mirrors toolkit gate.rs::child_paths + the keys[i] paths)
// ======================================================================

/// The path of child `i` of a node at `parent_path` with kind `kind` —
/// mirrors the toolkit gate's `child_paths` arms exactly:
/// binary kinds + `andor` → `{path}.{kind}[i]`; `thresh` →
/// `{path}.thresh.subs[i]`; `wrap` → `{path}.wrap.sub`.
pub fn child_path(parent_path: &str, kind: &str, i: usize) -> String {
    match kind {
        "thresh" => format!("{parent_path}.thresh.subs[{i}]"),
        "wrap" => format!("{parent_path}.wrap.sub"),
        // and_v / or_d / or_i / or_b / andor — the merged `{kind}[i]` form.
        _ => format!("{parent_path}.{kind}[{i}]"),
    }
}

/// The path of key row `i` of a quorum node (toolkit gate.rs key paths):
/// `{node_path}.{kind}.keys[i]`.
pub fn key_path(node_path: &str, kind: &str, i: usize) -> String {
    format!("{node_path}.{kind}.keys[{i}]")
}

/// Every live (path, node-id) pair from `root` down: per node its own
/// path; for quorum kinds additionally one entry per key row (mapped to
/// the OWNING node's id — keys are rows, not nodes); recursion covers the
/// EMITTED child prefix only (surplus children are not binary-addressable).
pub fn node_paths(root: &TreeNode) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    collect_paths(root, "root", &mut out);
    out
}

fn collect_paths(node: &TreeNode, path: &str, out: &mut Vec<(String, u64)>) {
    out.push((path.to_string(), node.id));
    if let Some(spec) = spec_for(&node.kind) {
        if spec.payload == PayloadShape::KeyQuorum {
            for i in 0..node.keys.len() {
                out.push((key_path(path, &node.kind, i), node.id));
            }
        }
    }
    for (i, child) in node.children.iter().take(emitted_child_count(node)).enumerate() {
        collect_paths(child, &child_path(path, &node.kind, i), out);
    }
}

/// Max node id in the tree (over ALL children incl. surplus — they hold
/// ids too). Feeds `recompute_next_id`.
pub fn max_id(node: &TreeNode) -> u64 {
    node.children.iter().map(max_id).fold(node.id, u64::max)
}

// ======================================================================
// Structural completeness (SPEC §1.2 — R0-r1 M3: STRUCTURAL emptiness
// ONLY; k<=n / hex validity / key format are the toolkit gate's,
// returned node-addressed — the no-second-validation-path ethos)
// ======================================================================

/// Paths of structurally unset/incomplete nodes. Non-empty ⇒ Validate /
/// Run / Copy-spec are disabled (P2). Classes: unset kind; empty `key` on
/// a key kind; `k < 1` on quorums (the UNSET-sentinel reading of the
/// wide-node `k: i64` default 0); empty/missing quorum key rows; empty
/// `hex`; empty `w`; missing or unset in-arity children. Surplus children
/// do NOT emit and are NOT checked.
pub fn completeness(root: &TreeNode) -> Vec<String> {
    let mut out = Vec::new();
    check_complete(root, "root", &mut out);
    out
}

fn check_complete(node: &TreeNode, path: &str, out: &mut Vec<String>) {
    let Some(spec) = spec_for(&node.kind) else {
        // Unset placeholder (or unknown kind) — incomplete; children are
        // not addressable without a kind, stop here.
        out.push(path.to_string());
        return;
    };
    match spec.payload {
        PayloadShape::Key => {
            if node.key.is_empty() {
                out.push(path.to_string());
            }
        }
        PayloadShape::KeyQuorum => {
            if node.k < 1 || node.keys.is_empty() {
                out.push(path.to_string());
            }
            for (i, key) in node.keys.iter().enumerate() {
                if key.is_empty() {
                    out.push(key_path(path, &node.kind, i));
                }
            }
        }
        PayloadShape::Uint => {} // `n` always carries a value structurally.
        PayloadShape::Hex64 | PayloadShape::Hex40 => {
            if node.hex.is_empty() {
                out.push(path.to_string());
            }
        }
        PayloadShape::ThreshSubs => {
            if node.k < 1 || node.children.is_empty() {
                out.push(path.to_string());
            }
        }
        PayloadShape::Wrap => {
            if node.w.is_empty() || node.children.is_empty() {
                out.push(path.to_string());
            }
        }
        PayloadShape::Children2 | PayloadShape::Children3 => {
            let arity = fixed_child_arity(spec.payload).unwrap_or(0);
            if node.children.len() < arity {
                out.push(path.to_string());
            }
        }
    }
    for (i, child) in node.children.iter().take(emitted_child_count(node)).enumerate() {
        check_complete(child, &child_path(path, &node.kind, i), out);
    }
}

// ======================================================================
// Serializers (SPEC §1.2)
// ======================================================================

/// Project the tree to the wrapped spec document
/// `{schema_version: 1, wrapper: "wsh", root: <node>}`. Projects ONLY the
/// active kind's fields + the in-arity child prefix (wide-node inactive
/// fields and surplus children deliberately do NOT serialize).
///
/// `to_spec_json` of an unset placeholder is undefined (callers gate by
/// `completeness` — R0-r1 M4); it degrades to JSON `null` (the binary's
/// spec-parse refuses → the global strip, never a GUI panic).
pub fn to_spec_json(root: &TreeNode) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "wrapper": "wsh",
        "root": node_to_json(root, 1),
    })
}

fn node_to_json(node: &TreeNode, depth: usize) -> serde_json::Value {
    // Defensive depth assert (SPEC §1.2): add-child/import already refuse
    // beyond the cap, so a violation here is a model bug.
    assert!(
        depth <= MAX_TREE_DEPTH,
        "to_spec_json: node at depth {depth} exceeds MAX_TREE_DEPTH ({MAX_TREE_DEPTH})"
    );
    let Some(spec) = spec_for(&node.kind) else {
        return serde_json::Value::Null;
    };
    let kind = spec.kind;
    let payload = match spec.payload {
        PayloadShape::Key => serde_json::json!(node.key),
        PayloadShape::KeyQuorum => serde_json::json!({ "k": node.k, "keys": node.keys }),
        PayloadShape::Uint => serde_json::json!(node.n),
        PayloadShape::Hex64 | PayloadShape::Hex40 => serde_json::json!(node.hex),
        PayloadShape::Children2 | PayloadShape::Children3 => {
            let arity = fixed_child_arity(spec.payload).unwrap_or(0);
            serde_json::Value::Array(
                node.children
                    .iter()
                    .take(arity)
                    .map(|c| node_to_json(c, depth + 1))
                    .collect(),
            )
        }
        PayloadShape::ThreshSubs => serde_json::json!({
            "k": node.k,
            "subs": node.children
                .iter()
                .map(|c| node_to_json(c, depth + 1))
                .collect::<Vec<_>>(),
        }),
        PayloadShape::Wrap => serde_json::json!({
            "w": node.w,
            "sub": node.children.first()
                .map(|c| node_to_json(c, depth + 1))
                .unwrap_or(serde_json::Value::Null),
        }),
    };
    serde_json::json!({ kind: payload })
}

/// Parse a wrapped spec document into a `TreeNode` (the full 17-kind
/// grammar). Ids are assigned `0..n` in walk (preorder) order — callers
/// MUST then recompute `next_id` (`TreeState::import_root` /
/// `recompute_next_id` — R0-r1 I3). Rejects unknown kinds/fields with a
/// path-addressed message; checks `schema_version == 1` and
/// `wrapper == "wsh"` on import (runtime parity with the test-time
/// refuse-loud pin — R0-r1 M5); refuses depth beyond `MAX_TREE_DEPTH`.
pub fn from_spec_json(doc: &serde_json::Value) -> Result<TreeNode, String> {
    let obj = doc.as_object().ok_or("spec: document must be a JSON object")?;
    for field in obj.keys() {
        if !matches!(field.as_str(), "schema_version" | "wrapper" | "root") {
            return Err(format!("spec: unknown top-level field \"{field}\""));
        }
    }
    let version = obj
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or("spec: missing/non-integer schema_version")?;
    if version != 1 {
        return Err(format!("spec: unsupported schema_version {version} (the GUI supports 1)"));
    }
    let wrapper = obj
        .get("wrapper")
        .and_then(|v| v.as_str())
        .ok_or("spec: missing/non-string wrapper")?;
    if wrapper != "wsh" {
        return Err(format!("spec: unsupported wrapper \"{wrapper}\" (the GUI supports \"wsh\")"));
    }
    let root = obj.get("root").ok_or("spec: missing root node")?;
    let mut next_id: u64 = 0;
    node_from_json(root, "root", 1, &mut next_id)
}

fn node_from_json(
    value: &serde_json::Value,
    path: &str,
    depth: usize,
    next_id: &mut u64,
) -> Result<TreeNode, String> {
    if depth > MAX_TREE_DEPTH {
        return Err(format!("{path}: exceeds the maximum tree depth ({MAX_TREE_DEPTH})"));
    }
    let obj = value
        .as_object()
        .ok_or_else(|| format!("{path}: node must be a JSON object"))?;
    if obj.len() != 1 {
        return Err(format!(
            "{path}: node must be externally tagged with exactly one kind key (got {})",
            obj.len()
        ));
    }
    let (kind, payload) = obj.iter().next().expect("len checked == 1");
    let spec = spec_for(kind)
        .ok_or_else(|| format!("{path}: unknown node kind \"{kind}\""))?;

    let id = *next_id;
    *next_id += 1;
    let mut node = TreeNode::new_unset(id);
    node.kind = spec.kind.to_string();

    match spec.payload {
        PayloadShape::Key => {
            node.key = payload
                .as_str()
                .ok_or_else(|| format!("{path}: {kind} payload must be a string key"))?
                .to_string();
        }
        PayloadShape::KeyQuorum => {
            let p = payload
                .as_object()
                .ok_or_else(|| format!("{path}: {kind} payload must be an object"))?;
            for field in p.keys() {
                if !matches!(field.as_str(), "k" | "keys") {
                    return Err(format!(
                        "{path}: unknown field \"{field}\" in {kind} payload"
                    ));
                }
            }
            node.k = parse_uint(p.get("k"), path, kind, "k")?;
            let keys = p
                .get("keys")
                .and_then(|v| v.as_array())
                .ok_or_else(|| format!("{path}: {kind} payload must carry a keys array"))?;
            for (i, k) in keys.iter().enumerate() {
                node.keys.push(
                    k.as_str()
                        .ok_or_else(|| {
                            format!("{}: key must be a string", key_path(path, kind, i))
                        })?
                        .to_string(),
                );
            }
        }
        PayloadShape::Uint => {
            node.n = parse_uint(Some(payload), path, kind, "payload")?;
        }
        PayloadShape::Hex64 | PayloadShape::Hex40 => {
            node.hex = payload
                .as_str()
                .ok_or_else(|| format!("{path}: {kind} payload must be a hex string"))?
                .to_string();
        }
        PayloadShape::Children2 | PayloadShape::Children3 => {
            let arity = fixed_child_arity(spec.payload).unwrap_or(0);
            let subs = payload
                .as_array()
                .ok_or_else(|| format!("{path}: {kind} payload must be an array"))?;
            if subs.len() != arity {
                return Err(format!(
                    "{path}: {kind} expects exactly {arity} children (got {})",
                    subs.len()
                ));
            }
            for (i, sub) in subs.iter().enumerate() {
                let cpath = child_path(path, kind, i);
                node.children.push(node_from_json(sub, &cpath, depth + 1, next_id)?);
            }
        }
        PayloadShape::ThreshSubs => {
            let p = payload
                .as_object()
                .ok_or_else(|| format!("{path}: thresh payload must be an object"))?;
            for field in p.keys() {
                if !matches!(field.as_str(), "k" | "subs") {
                    return Err(format!(
                        "{path}: unknown field \"{field}\" in thresh payload"
                    ));
                }
            }
            node.k = parse_uint(p.get("k"), path, kind, "k")?;
            let subs = p
                .get("subs")
                .and_then(|v| v.as_array())
                .ok_or_else(|| format!("{path}: thresh payload must carry a subs array"))?;
            for (i, sub) in subs.iter().enumerate() {
                let cpath = child_path(path, kind, i);
                node.children.push(node_from_json(sub, &cpath, depth + 1, next_id)?);
            }
        }
        PayloadShape::Wrap => {
            let p = payload
                .as_object()
                .ok_or_else(|| format!("{path}: wrap payload must be an object"))?;
            for field in p.keys() {
                if !matches!(field.as_str(), "w" | "sub") {
                    return Err(format!("{path}: unknown field \"{field}\" in wrap payload"));
                }
            }
            node.w = p
                .get("w")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("{path}: wrap payload must carry a string w"))?
                .to_string();
            let sub = p
                .get("sub")
                .ok_or_else(|| format!("{path}: wrap payload must carry a sub node"))?;
            let cpath = child_path(path, kind, 0);
            node.children.push(node_from_json(sub, &cpath, depth + 1, next_id)?);
        }
    }
    Ok(node)
}

fn parse_uint(
    value: Option<&serde_json::Value>,
    path: &str,
    kind: &str,
    field: &str,
) -> Result<i64, String> {
    let v = value
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("{path}: {kind} {field} must be a non-negative integer"))?;
    i64::try_from(v).map_err(|_| format!("{path}: {kind} {field} out of range"))
}

// ======================================================================
// Persistence redaction walk (SPEC §1.3)
// ======================================================================

/// The toolkit gate's xprv heuristic (`gate.rs::check_secret_key`,
/// verified GUI-reusable — brainstorm R0-r1 M1): strip an optional
/// `[origin]` prefix via `rsplit(']')`, then extended-private prefixes
/// `xprv`/`tprv`/`yprv`/`zprv`/`uprv`/`vprv` (+ capitals) all have `prv`
/// at byte offset 1..4 (`xpub`/`tpub` have `pub`).
///
/// v0.34.0 (audit I6): this DENY-list heuristic serves the live render-side
/// hint ONLY (`tree_form::xprv_hint` — warn-now semantics, false-negatives
/// tolerable). Persist-redaction uses the POSITIVE allowlist
/// (`is_extended_public_like`), NOT this — WIF / raw-hex / garbage are
/// blanked at persist even though this fn returns false for them.
pub fn is_xprv_like(key: &str) -> bool {
    let key_part = key.rsplit(']').next().unwrap_or(key);
    key_part.is_char_boundary(4) && key_part.as_bytes().get(1..4) == Some(b"prv")
}

/// v0.34.0 (audit I6) — the persist walk's POSITIVE allowlist: true iff the
/// origin-stripped part starts with one of the 10 SLIP-132 extended-PUBLIC
/// prefixes. Everything else (xprv-likes, WIF, raw hex, garbage) is blanked
/// at persist — fail-closed, because this path has NO `from_str` backstop
/// (the toolkit gate's WIF/raw-hex refusal lives in its step-2 type-check,
/// which never runs here). Notes:
/// - Byte 0 is constrained ON PURPOSE: a bare "`pub` at bytes 1..4" probe
///   would keep a `Kpub…`-form WIF in cleartext (base58 contains p/u/b).
/// - The list admits SLIP-132 forms (`ypub`/`zpub`/…) the toolkit's
///   `Xpub::from_str` would refuse anyway — correct trade (public material,
///   no leak direction). Do NOT "tighten" to {xpub,tpub} (back-door
///   behavior change) or mistake this allowlist for a validity check.
/// - Inherited `rsplit(']')` caveat (same as `is_xprv_like`): everything
///   before the LAST `]` goes unexamined, so `xprv…]xpub…` keeps the whole
///   string — tolerated; such a string is not a parseable key anywhere.
fn is_extended_public_like(key: &str) -> bool {
    const XPUB_PREFIXES: &[&[u8; 4]] = &[
        b"xpub", b"tpub", b"ypub", b"Ypub", b"zpub", b"Zpub", b"upub", b"Upub", b"vpub", b"Vpub",
    ];
    let key_part = key.rsplit(']').next().unwrap_or(key);
    match key_part.as_bytes().get(..4) {
        Some(prefix) => XPUB_PREFIXES.iter().any(|p| prefix == &p[..]),
        None => false,
    }
}

/// Blank every `key` / `keys` entry that is NOT positively extended-public,
/// recursively over ALL children (incl. surplus — they persist too).
/// v0.34.0 (audit I6): replaces the deny-list walk (`is_xprv_like`-only),
/// which let WIF / raw-hex private keys survive persist in cleartext. The
/// fail-closed cost: a raw-hex x-only/compressed PUBLIC key is byte-shape-
/// indistinguishable from a raw-hex PRIVATE key, so hex blanks too; xpub-form
/// keys (the dominant input) survive. Hashlock `hex` FIELDS are untouched
/// (different field, not key/keys).
fn blank_non_extended_public_keys(node: &mut TreeNode) {
    if !node.key.is_empty() && !is_extended_public_like(&node.key) {
        node.key.clear();
    }
    for key in &mut node.keys {
        if !key.is_empty() && !is_extended_public_like(key) {
            key.clear();
        }
    }
    // Wave-2 G2 (`tree-xprv-heuristic-only-covers-key-fields`): `hex` (hashlock
    // digest) + `w` (wrapper) are NOT key fields — they have a legitimate
    // non-key shape (64-hex digest / "sv"), so the positive allowlist above
    // would destroy valid data. Apply the NARROW `is_xprv_like` DENY-list:
    // blank ONLY an xprv-SHAPED mis-paste, preserve legit digests/wrap strings.
    // (Asymmetric to the `is_extended_public_like` ALLOWLIST used for key/keys
    // above — correct: key/keys have NO legitimate non-key shape, hex/w do.)
    if !node.hex.is_empty() && is_xprv_like(&node.hex) {
        node.hex.clear();
    }
    if !node.w.is_empty() && is_xprv_like(&node.w) {
        node.w.clear();
    }
    for child in &mut node.children {
        blank_non_extended_public_keys(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC §1.2: GUI-authored path-grammar literals (the toolkit gate
    /// tests carry no `thresh.subs[i]` literal — brainstorm R0-r1 I3; the
    /// LIVE parity gate `tests/tree_path_parity.rs` is the real tether).
    #[test]
    fn child_path_five_productions_literal_pins() {
        // binary + andor share the `{kind}[i]` form.
        assert_eq!(child_path("root", "or_d", 1), "root.or_d[1]");
        assert_eq!(child_path("root", "and_v", 0), "root.and_v[0]");
        assert_eq!(child_path("root", "or_i", 0), "root.or_i[0]");
        assert_eq!(child_path("root", "or_b", 1), "root.or_b[1]");
        assert_eq!(child_path("root", "andor", 2), "root.andor[2]");
        assert_eq!(child_path("root", "thresh", 2), "root.thresh.subs[2]");
        assert_eq!(child_path("root", "wrap", 0), "root.wrap.sub");
        assert_eq!(key_path("root", "multi", 0), "root.multi.keys[0]");
        // The deep consult-probed literal (SPEC §1.2):
        let p = child_path("root", "thresh", 2);
        let p = child_path(&p, "andor", 2);
        assert_eq!(key_path(&p, "multi", 0), "root.thresh.subs[2].andor[2].multi.keys[0]");
        // Nested wrap (toolkit gate test literal `root.or_d[1].and_v[0].wrap.sub`):
        let p = child_path("root", "or_d", 1);
        let p = child_path(&p, "and_v", 0);
        assert_eq!(child_path(&p, "wrap", 0), "root.or_d[1].and_v[0].wrap.sub");
    }

    #[test]
    fn node_paths_covers_nodes_keys_and_in_arity_prefix_only() {
        // or_d( multi(2, [k0, k1]), older ) + one SURPLUS child.
        let tree = TreeNode {
            id: 0,
            kind: "or_d".into(),
            children: vec![
                TreeNode {
                    id: 1,
                    kind: "multi".into(),
                    k: 2,
                    keys: vec!["a".into(), "b".into()],
                    ..Default::default()
                },
                TreeNode { id: 2, kind: "older".into(), n: 144, ..Default::default() },
                TreeNode { id: 3, kind: "after".into(), n: 1, ..Default::default() }, // surplus
            ],
            ..Default::default()
        };
        let paths = node_paths(&tree);
        let expect: Vec<(String, u64)> = vec![
            ("root".into(), 0),
            ("root.or_d[0]".into(), 1),
            ("root.or_d[0].multi.keys[0]".into(), 1),
            ("root.or_d[0].multi.keys[1]".into(), 1),
            ("root.or_d[1]".into(), 2),
        ];
        assert_eq!(paths, expect, "surplus child must NOT be path-addressable");
    }

    #[test]
    fn mark_dirty_clears_diagnostics_and_validate_ok() {
        let mut state = TreeState::fresh();
        state.diagnostics.push(TreeDiag {
            node_path: "root".into(),
            kind: "sigless_branch".into(),
            message: "m".into(),
        });
        state.validate_ok =
            Some(ValidateOk { descriptor: "wsh(...)".into(), cost: serde_json::json!({}) });
        state.strip.push("params: [param] stale".into());
        state.mark_dirty();
        assert!(state.diagnostics.is_empty());
        assert!(state.validate_ok.is_none());
        assert!(state.strip.is_empty(), "the global strip shares the mark_dirty lifetime");
    }

    #[test]
    fn set_kind_materializes_arity_placeholders() {
        let mut state = TreeState::fresh();
        let mut next_id = state.next_id;
        state.root.set_kind("andor", 1, &mut next_id).unwrap();
        state.next_id = next_id;
        assert_eq!(state.root.children.len(), 3);
        assert!(state.root.children.iter().all(|c| c.kind.is_empty()));
        // Fresh unique ids.
        let ids: std::collections::BTreeSet<u64> =
            state.root.children.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), 3);
        assert!(!ids.contains(&state.root.id));
        // thresh starts at 0 children; wrap pads to 1.
        let mut n = TreeNode::new_unset(9);
        let mut nid = 10;
        n.set_kind("thresh", 1, &mut nid).unwrap();
        assert!(n.children.is_empty());
        n.set_kind("wrap", 1, &mut nid).unwrap();
        assert_eq!(n.children.len(), 1, "wrap pads to 1 (preserve keeps thresh's none)");
    }

    #[test]
    fn is_xprv_like_matches_gate_heuristic() {
        assert!(is_xprv_like("xprv9s21ZrQH143K3QTDL4LXw2F"));
        assert!(is_xprv_like("tprvAAAA"));
        assert!(is_xprv_like("[11111111/48h/0h/0h/2h]xprv9s21ZrQH"));
        assert!(!is_xprv_like("xpub661MyMwAqRbcEZVB4dScxMAdx6d"));
        assert!(!is_xprv_like("[11111111/48h]xpub661MyMwAqRbc"));
        assert!(!is_xprv_like(""));
        assert!(!is_xprv_like("926a54995ca48600920a19bf7bc502ca"));
    }
}
