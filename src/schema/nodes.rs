//! v0.32.0 (node-tree builder SPEC §1.1) — static mirror of the toolkit's
//! `build-descriptor --spec-schema` `nodes` array (the 17-kind policy-node
//! grammar the recursive tree builder renders + serializes).
//!
//! Transcribed from the pinned v0.52.0 binary's `--spec-schema` output
//! (probed 2026-06-10). Wire keys per node: `kind` / `payload` / `renders`
//! / `tag`; the GUI mirrors `kind` + `payload` (verbatim, byte-compared by
//! the drift gate) and adds the GUI-side closed `PayloadShape` discriminant
//! (9 shapes — brainstorm R0-r1 I1) that drives widget arms + child arity.
//!
//! The payload strings are a closed vocabulary matched EXACTLY (the
//! archetype-kind-vocabulary precedent) — never structurally parsed at
//! runtime.
//!
//! Drift gate: `tests/spec_nodes_mirror.rs` runs the pinned binary's
//! `build-descriptor --spec-schema` and compares this table (kind set +
//! ORDER, `payload` byte-equal) plus refuse-loud pins on BOTH
//! `spec_schema_version == 1` and `supported_doc_schema_version == 1`
//! (brainstorm R0-r1 I5). Any toolkit-side grammar change fails the gate
//! at the next pin bump — the lagging-gate posture; the leading discipline
//! is the paired-PR rule.

/// GUI-side closed payload-shape discriminant — the 9 distinct payload
/// vocabularies across the 17 node kinds. Drives the tree form's widget
/// arm per node (P2) and the model's child-arity rules
/// (`fixed_child_arity` in `form/tree_model.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PayloadShape {
    /// `string(key)` — `pk` / `pkh`. Wide-node field: `key`.
    Key,
    /// `{k:uint, keys:[key]}` — `multi` / `sortedmulti`. Fields: `k`, `keys`.
    KeyQuorum,
    /// `uint` — `older` / `after`. Field: `n`.
    Uint,
    /// `string(hex64)` — `sha256` / `hash256`. Field: `hex`.
    Hex64,
    /// `string(hex40)` — `hash160` / `ripemd160`. Field: `hex`.
    Hex40,
    /// `[node, node]` — `and_v` / `or_d` / `or_i` / `or_b`. Arity 2.
    Children2,
    /// `[node, node, node]` — `andor`. Arity 3.
    Children3,
    /// `{k:uint, subs:[node]}` — `thresh`. Fields: `k` + variable children.
    ThreshSubs,
    /// `{w:string, sub:node}` — `wrap`. Fields: `w` + exactly 1 child.
    Wrap,
}

/// One node-kind spec: the wire `kind`, the GUI shape discriminant, and
/// the binary's verbatim `payload` string (drift-gate compared byte-equal).
pub struct NodeKindSpec {
    /// Wire `kind` (== the externally-tagged JSON key, e.g. `"or_d"`).
    pub kind: &'static str,
    /// GUI-side closed shape driving widget arms + arity.
    pub payload: PayloadShape,
    /// The binary's verbatim `payload` string (e.g. `"{k:uint, keys:[key]}"`).
    pub payload_str: &'static str,
}

/// The 17 node kinds in the binary's `NODE_KINDS` order (== the
/// `--spec-schema` `nodes` array order == the `node_kinds` list).
/// Transcribed verbatim from the v0.52.0 binary.
pub const NODE_KIND_SPECS: &[NodeKindSpec] = &[
    NodeKindSpec { kind: "pk", payload: PayloadShape::Key, payload_str: "string(key)" },
    NodeKindSpec { kind: "pkh", payload: PayloadShape::Key, payload_str: "string(key)" },
    NodeKindSpec {
        kind: "multi",
        payload: PayloadShape::KeyQuorum,
        payload_str: "{k:uint, keys:[key]}",
    },
    NodeKindSpec {
        kind: "sortedmulti",
        payload: PayloadShape::KeyQuorum,
        payload_str: "{k:uint, keys:[key]}",
    },
    NodeKindSpec { kind: "older", payload: PayloadShape::Uint, payload_str: "uint" },
    NodeKindSpec { kind: "after", payload: PayloadShape::Uint, payload_str: "uint" },
    NodeKindSpec { kind: "sha256", payload: PayloadShape::Hex64, payload_str: "string(hex64)" },
    NodeKindSpec { kind: "hash256", payload: PayloadShape::Hex64, payload_str: "string(hex64)" },
    NodeKindSpec { kind: "hash160", payload: PayloadShape::Hex40, payload_str: "string(hex40)" },
    NodeKindSpec {
        kind: "ripemd160",
        payload: PayloadShape::Hex40,
        payload_str: "string(hex40)",
    },
    NodeKindSpec { kind: "and_v", payload: PayloadShape::Children2, payload_str: "[node, node]" },
    NodeKindSpec { kind: "or_d", payload: PayloadShape::Children2, payload_str: "[node, node]" },
    NodeKindSpec { kind: "or_i", payload: PayloadShape::Children2, payload_str: "[node, node]" },
    NodeKindSpec { kind: "or_b", payload: PayloadShape::Children2, payload_str: "[node, node]" },
    NodeKindSpec {
        kind: "andor",
        payload: PayloadShape::Children3,
        payload_str: "[node, node, node]",
    },
    NodeKindSpec {
        kind: "thresh",
        payload: PayloadShape::ThreshSubs,
        payload_str: "{k:uint, subs:[node]}",
    },
    NodeKindSpec {
        kind: "wrap",
        payload: PayloadShape::Wrap,
        payload_str: "{w:string, sub:node}",
    },
];

/// Look up the spec for a node kind. `None` for the `""` unset placeholder
/// and any unknown kind.
pub fn spec_for(kind: &str) -> Option<&'static NodeKindSpec> {
    NODE_KIND_SPECS.iter().find(|s| s.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    /// SPEC §1.1 self-test: the 17 kinds are unique (and there are 17).
    #[test]
    fn seventeen_unique_kinds() {
        assert_eq!(NODE_KIND_SPECS.len(), 17, "the grammar has 17 node kinds");
        let set: BTreeSet<&str> = NODE_KIND_SPECS.iter().map(|s| s.kind).collect();
        assert_eq!(set.len(), 17, "node kinds must be unique");
    }

    /// SPEC §1.1 self-test: every `payload_str` maps to exactly one shape
    /// (the closed-vocabulary invariant), and the 9 payload strings / 9
    /// shapes are in bijection with all 9 `PayloadShape` variants covered.
    #[test]
    fn payload_str_to_shape_mapping_consistent() {
        let mut by_str: BTreeMap<&str, PayloadShape> = BTreeMap::new();
        for spec in NODE_KIND_SPECS {
            if let Some(prev) = by_str.insert(spec.payload_str, spec.payload) {
                assert_eq!(
                    prev, spec.payload,
                    "payload_str {:?} maps to two shapes ({:?} vs {:?}) — \
                     the vocabulary is closed, one shape per payload string",
                    spec.payload_str, prev, spec.payload,
                );
            }
        }
        assert_eq!(by_str.len(), 9, "9 distinct payload strings");
        let shapes: BTreeSet<String> =
            by_str.values().map(|s| format!("{s:?}")).collect();
        assert_eq!(shapes.len(), 9, "the 9 payload strings cover all 9 shapes");
    }

    #[test]
    fn spec_for_resolves_known_and_rejects_unset() {
        assert_eq!(spec_for("wrap").map(|s| s.payload), Some(PayloadShape::Wrap));
        assert!(spec_for("").is_none(), "the unset placeholder has no spec");
        assert!(spec_for("bogus").is_none());
    }
}
