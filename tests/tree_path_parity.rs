//! v0.32.0 (node-tree builder SPEC §4 gate 2) — THE keystone: path-grammar
//! parity against the LIVE binary.
//!
//! The GUI's `child_path` / `key_path` / `node_paths` mirror the toolkit
//! gate's path grammar with no compile-time tether — this gate is the
//! tether. Six plants over the 5 productions (binary + andor share the
//! `{kind}[i]` form), each with a NODE-ADDRESSED diagnostic class pinned
//! per plant (R0-r1 I2 — type/parse-class diagnostics localize to ROOT;
//! recipes probed byte-exact, R0-r2):
//!
//!   1. binary arm  — `or_d(pk, after)`            → sigless_branch @ root.or_d[1]
//!   2. andor arm   — `andor(pk, after, older)`    → sigless_branch @ root.andor[1]
//!      (R0-r2 M1: the naive `andor(pk, older, pk)` VALIDATES)
//!   3. thresh.subs — nested k>n `multi`           → schema_field   @ root.thresh.subs[1]
//!   4. wrap.sub    — short-hex `sha256` under `v:` → schema_field  @ root.and_v[0].wrap.sub
//!   5. root        — bare `older`                 → sigless_branch @ root
//!   6. keys[i]     — planted tprv                 → secret_key     @ root.multi.keys[1]
//!
//! Trees are built via `TreeNode`, serialized with `to_spec_json`, and run
//! through the REAL binary `--spec - --json` (stdin piped TEST-LOCALLY via
//! `std::process::Command` — the test writes the spec; this is NOT the
//! runner, which stays untouched until P2). Assertions: every returned
//! `node_path` equals the GUI-computed path AND matches a live
//! `node_paths()` entry.
//!
//! Skip discipline: skip-if-absent (the `archetype_schema_mirror` pattern).

use std::io::Write;
use std::process::{Command, Stdio};

use mnemonic_gui::form::tree_model::{
    child_path, completeness, key_path, node_paths, to_spec_json, TreeNode,
};

const XPUB_A: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
const XPUB_B: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
/// The keys[i]-class plant (SPEC §4.2: a planted tprv — the gate's
/// secret-key screen is a PREFIX heuristic, fired before any key parse).
const TPRV: &str = "tprvAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn resolve_bin() -> Option<String> {
    match std::env::var("MNEMONIC_BIN").ok() {
        Some(b) => Some(b),
        None => match Command::new("mnemonic").arg("--help").output() {
            Ok(_) => Some("mnemonic".into()),
            Err(_) => {
                eprintln!(
                    "MNEMONIC_BIN unset + PATH lookup failed; \
                     skipping tree path-grammar parity gate"
                );
                None
            }
        },
    }
}

// ---------------------------------------------------------------- builders

fn leaf(kind: &str) -> TreeNode {
    TreeNode { kind: kind.into(), ..Default::default() }
}

fn pk(key: &str) -> TreeNode {
    TreeNode { kind: "pk".into(), key: key.into(), ..Default::default() }
}

fn uint(kind: &str, n: i64) -> TreeNode {
    TreeNode { kind: kind.into(), n, ..Default::default() }
}

fn multi(k: i64, keys: &[&str]) -> TreeNode {
    TreeNode {
        kind: "multi".into(),
        k,
        keys: keys.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

fn parent(kind: &str, children: Vec<TreeNode>) -> TreeNode {
    TreeNode { kind: kind.into(), children, ..Default::default() }
}

/// Assign unique preorder ids (the test builders skip id bookkeeping).
fn assign_ids(node: &mut TreeNode, next: &mut u64) {
    node.id = *next;
    *next += 1;
    for c in &mut node.children {
        assign_ids(c, next);
    }
}

// ---------------------------------------------------------------- harness

/// Run `build-descriptor --spec - --json` with the spec piped via stdin
/// (test-local; SPEC §0 discipline: write_all → drop ChildStdin →
/// wait_with_output).
fn run_spec(bin: &str, spec: &serde_json::Value) -> std::process::Output {
    let mut child = Command::new(bin)
        .args(["build-descriptor", "--spec", "-", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn `mnemonic build-descriptor --spec - --json`");
    {
        let mut stdin = child.stdin.take().expect("stdin piped");
        // BrokenPipe tolerated (child may exit pre-EOF) — degraded to
        // collect-output, matching the §0 discipline.
        let _ = stdin.write_all(spec.to_string().as_bytes());
        // drop(stdin) → EOF.
    }
    child.wait_with_output().expect("wait_with_output")
}

/// One plant: build the tree, send it through the real binary, and assert
/// the returned diagnostics are exactly `expected` (path computed via the
/// GUI's path fns + pinned class) AND that every returned node_path is a
/// live `node_paths()` entry.
fn assert_plant(bin: &str, mut tree: TreeNode, expected: &[(String, &str)]) {
    let mut next = 0u64;
    assign_ids(&mut tree, &mut next);
    // The plants are structurally COMPLETE trees (the GUI gate would let
    // Validate run) — the diagnostics are the toolkit gate's, node-addressed.
    assert!(
        completeness(&tree).is_empty(),
        "plant must be structurally complete; got {:?}",
        completeness(&tree)
    );
    let spec = to_spec_json(&tree);
    let out = run_spec(bin, &spec);
    // Parse contract: stdout parses as {diagnostics:[…]} → node-addressed
    // view. NEVER exit-code-keyed.
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "plant stdout must parse as the diagnostics envelope: {e}\n\
             stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        )
    });
    let diags = json["diagnostics"]
        .as_array()
        .expect("envelope must carry a diagnostics array");
    let returned: Vec<(String, String)> = diags
        .iter()
        .map(|d| {
            (
                d["node_path"].as_str().expect("node_path string").to_string(),
                d["kind"].as_str().expect("kind string").to_string(),
            )
        })
        .collect();
    let expected_owned: Vec<(String, String)> = expected
        .iter()
        .map(|(p, k)| (p.clone(), k.to_string()))
        .collect();
    assert_eq!(
        returned, expected_owned,
        "diagnostic (node_path, kind) drift — the binary's path grammar \
         no longer matches the GUI's child_path/key_path productions",
    );
    // AND every returned path must be a live node_paths() entry (the
    // renderer's tint lookup would find the node).
    let live = node_paths(&tree);
    for (path, _) in &returned {
        assert!(
            live.iter().any(|(p, _)| p == path),
            "returned node_path {path:?} is not a live node_paths() entry; \
             live paths: {:?}",
            live.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>(),
        );
    }
}

// ------------------------------------------------------------------ plants

/// Plant 1 — binary-arm production `{kind}[i]`: `or_d(pk, after)`, the
/// `after` arm is sigless → sigless_branch @ root.or_d[1].
#[test]
fn plant_binary_arm_or_d_sigless() {
    let Some(bin) = resolve_bin() else { return };
    let tree = parent("or_d", vec![pk(XPUB_A), uint("after", 500_000)]);
    let expected = vec![(child_path("root", "or_d", 1), "sigless_branch")];
    assert_plant(&bin, tree, &expected);
}

/// Plant 2 — andor production: `andor(pk, after, older)` → sigless_branch
/// @ root.andor[1]. R0-r2 M1: the naive `andor(pk, older, pk)` VALIDATES —
/// this recipe is the probed working one.
#[test]
fn plant_andor_arm_sigless() {
    let Some(bin) = resolve_bin() else { return };
    let tree = parent(
        "andor",
        vec![pk(XPUB_A), uint("after", 500_000), uint("older", 144)],
    );
    let expected = vec![(child_path("root", "andor", 1), "sigless_branch")];
    assert_plant(&bin, tree, &expected);
}

/// Plant 3 — thresh.subs[i] production: a nested k>n multi →
/// schema_field @ root.thresh.subs[1].
#[test]
fn plant_thresh_subs_schema_field() {
    let Some(bin) = resolve_bin() else { return };
    let tree = TreeNode {
        kind: "thresh".into(),
        k: 1,
        children: vec![pk(XPUB_A), multi(3, &[XPUB_A, XPUB_B])],
        ..Default::default()
    };
    let expected = vec![(child_path("root", "thresh", 1), "schema_field")];
    assert_plant(&bin, tree, &expected);
}

/// Plant 4 — wrap.sub production: a short-hex sha256 under `v:` →
/// schema_field @ root.and_v[0].wrap.sub.
#[test]
fn plant_wrap_sub_schema_field() {
    let Some(bin) = resolve_bin() else { return };
    let sha = TreeNode { kind: "sha256".into(), hex: "deadbeef".into(), ..Default::default() };
    let wrap = TreeNode { kind: "wrap".into(), w: "v".into(), children: vec![sha], ..Default::default() };
    let tree = parent("and_v", vec![wrap, pk(XPUB_A)]);
    let and_v0 = child_path("root", "and_v", 0);
    let expected = vec![(child_path(&and_v0, "wrap", 0), "schema_field")];
    assert_plant(&bin, tree, &expected);
}

/// Plant 5 — root production: a bare `older` root → sigless_branch @ root.
#[test]
fn plant_root_sigless() {
    let Some(bin) = resolve_bin() else { return };
    let tree = uint("older", 144);
    let expected = vec![("root".to_string(), "sigless_branch")];
    assert_plant(&bin, tree, &expected);
}

/// Plant 6 — keys[i] production (a keys-CLASS error, secret_key — node-
/// level key errors attach to the NODE path): a planted tprv →
/// secret_key @ root.multi.keys[1].
#[test]
fn plant_multi_keys_secret_key() {
    let Some(bin) = resolve_bin() else { return };
    let tree = multi(2, &[XPUB_A, TPRV]);
    let expected = vec![(key_path("root", "multi", 1), "secret_key")];
    assert_plant(&bin, tree, &expected);
}

/// Guard: `leaf` is used by no plant today but keeps the builder set
/// uniform — exercise it so clippy/dead-code stays clean.
#[test]
fn builders_smoke() {
    let mut tree = parent("or_i", vec![leaf("older"), leaf("after")]);
    let mut next = 0;
    assign_ids(&mut tree, &mut next);
    assert_eq!(next, 3);
    assert_eq!(node_paths(&tree).len(), 3);
}
