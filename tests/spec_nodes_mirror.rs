//! v0.32.0 (node-tree builder SPEC §4 gate 1) — `spec_nodes_mirror` drift
//! gate.
//!
//! Runs the pinned toolkit binary's `build-descriptor --spec-schema`,
//! parses the `nodes` array, and asserts the GUI's static mirror
//! `schema::nodes::NODE_KIND_SPECS` matches: kind set + ORDER (the array
//! order == the binary's `NODE_KINDS` order, which is also the tree
//! form's kind-dropdown order) and `payload` strings BYTE-EQUAL.
//!
//! Refuse-loud version pins (brainstorm R0-r1 I5 — BOTH axes):
//! `spec_schema_version == 1` (the grammar axis this mirror copies) AND
//! `supported_doc_schema_version == 1` (the document axis the
//! serializers emit).
//!
//! Skip discipline: skip-if-absent (the `archetype_schema_mirror`
//! pattern) — eprintln + return when no binary. CI still runs it because
//! the workflow exports `MNEMONIC_BIN`.

use std::process::Command;

use mnemonic_gui::schema::nodes::NODE_KIND_SPECS;

#[test]
fn node_kind_specs_match_pinned_binary_spec_schema() {
    let bin: String = match std::env::var("MNEMONIC_BIN").ok() {
        Some(b) => b,
        None => match Command::new("mnemonic").arg("--help").output() {
            Ok(_) => "mnemonic".into(),
            Err(_) => {
                eprintln!(
                    "MNEMONIC_BIN unset + PATH lookup failed; \
                     skipping NODE_KIND_SPECS const-vs-binary parity"
                );
                return;
            }
        },
    };
    let out = Command::new(&bin)
        .args(["build-descriptor", "--spec-schema"])
        .output()
        .expect("failed to spawn `mnemonic build-descriptor --spec-schema`");
    assert!(
        out.status.success(),
        "`mnemonic build-descriptor --spec-schema` exited non-zero: {:?}",
        out.status
    );
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("--spec-schema stdout must be JSON");

    // Refuse-loud version pins, BOTH axes (R0-r1 I5).
    assert_eq!(
        json["spec_schema_version"].as_u64(),
        Some(1),
        "spec_schema_version drift — the NODE_KIND_SPECS mirror copies \
         grammar axis v1; re-transcribe src/schema/nodes.rs and revisit \
         the tree builder before bumping",
    );
    assert_eq!(
        json["supported_doc_schema_version"].as_u64(),
        Some(1),
        "supported_doc_schema_version drift — tree_model's serializers \
         emit/import doc schema v1; revisit to_spec_json/from_spec_json \
         before bumping",
    );

    let nodes = json["nodes"]
        .as_array()
        .expect("--spec-schema must carry a `nodes` array");
    assert_eq!(
        nodes.len(),
        NODE_KIND_SPECS.len(),
        "node-kind count drift: binary has {}, NODE_KIND_SPECS has {} — \
         update src/schema/nodes.rs in lockstep with the toolkit pin",
        nodes.len(),
        NODE_KIND_SPECS.len(),
    );
    for (i, (wire, spec)) in nodes.iter().zip(NODE_KIND_SPECS).enumerate() {
        assert_eq!(
            wire["kind"].as_str(),
            Some(spec.kind),
            "node kind drift at index {i} (order is load-bearing — it is \
             the binary's NODE_KINDS order and the kind-dropdown order; \
             compare in declared order, not as a set)",
        );
        assert_eq!(
            wire["payload"].as_str(),
            Some(spec.payload_str),
            "payload drift for `{}` — payload_str is the binary's verbatim \
             payload string, compared byte-equal (the closed vocabulary)",
            spec.kind,
        );
    }

    // Belt-and-suspenders: the flat `node_kinds` list agrees with the
    // `nodes` array (same source, but pins the wire shape both ways).
    let node_kinds = json["node_kinds"]
        .as_array()
        .expect("--spec-schema must carry a `node_kinds` array");
    let kinds: Vec<&str> = node_kinds.iter().filter_map(|v| v.as_str()).collect();
    let mirror: Vec<&str> = NODE_KIND_SPECS.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, mirror, "node_kinds list drift vs NODE_KIND_SPECS");
}
