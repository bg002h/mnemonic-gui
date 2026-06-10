//! v0.32.0 P2 (node-tree builder SPEC §5) — tree-form cells.
//!
//! Layers (the repo's established split — `tests/archetype_form.rs` /
//! `tests/repeating_rows.rs` are the references):
//!   - state-level argv cells: the three-way mode mutex through
//!     `assemble_argv` (tree mode emits NO `--spec <path>` / `--archetype`
//!     / param flags / `--emit-spec` even with all of them
//!     stale-populated; switching back restores) + the fixed Validate
//!     argv incl. the `--allow` pass-through (R0-r2 M3);
//!   - parse-contract cells: `apply_validate_result` over synthesized
//!     `RunResult`s (success / failure envelope / non-envelope; the
//!     fail-soft strip for `"params"` + `"root.bogus[9]"` — R0-r1 I5);
//!   - UI-path kittest cells driving the LIB seam (`tree_form::render` +
//!     `tree_form::render_mode_selector` — main.rs only dispatches):
//!     the Validate flow against the REAL binary (skip-if-absent, the
//!     `tree_path_parity` posture), the `--allow` override leg, the
//!     collapse-does-not-clear cell (R0-r1 M6b), the mutation-chokepoint
//!     edit-clears leg, the selector gestures (R0-r2 M2), and the
//!     depth-cap refusal message.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

use mnemonic_gui::form::invocation::assemble_argv;
use mnemonic_gui::form::tree_form::{
    self, MODE_ARCHETYPE_LABEL, MODE_GENERIC_LABEL, MODE_TREE_LABEL,
};
use mnemonic_gui::form::tree_model::{
    max_id, to_spec_json, TreeDiag, TreeNode, TreeState, MAX_TREE_DEPTH,
};
use mnemonic_gui::runner::RunResult;
use mnemonic_gui::schema::{self, FlagValue, FormState, Visibility};

const XPUB_A: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";

fn subcommand(name: &str) -> &'static schema::SubcommandSchema {
    schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("subcommand {} not in schema", name))
}

fn build_descriptor_argv(state: &FormState) -> Vec<String> {
    assemble_argv(&schema::mnemonic::SCHEMA, subcommand("build-descriptor"), state)
}

fn occurrences(argv: &[String], flag: &str) -> Vec<String> {
    argv.windows(2)
        .filter(|w| w[0] == flag)
        .map(|w| w[1].clone())
        .collect()
}

fn resolve_bin() -> Option<String> {
    match std::env::var("MNEMONIC_BIN").ok() {
        Some(b) => Some(b),
        None => match std::process::Command::new("mnemonic").arg("--help").output() {
            Ok(_) => Some("mnemonic".into()),
            Err(_) => {
                eprintln!(
                    "MNEMONIC_BIN unset + PATH lookup failed; skipping tree-form \
                     Validate cell"
                );
                None
            }
        },
    }
}

// ------------------------------------------------------------ tree builders

fn pk(key: &str) -> TreeNode {
    TreeNode { kind: "pk".into(), key: key.into(), ..Default::default() }
}

fn uint(kind: &str, n: i64) -> TreeNode {
    TreeNode { kind: kind.into(), n, ..Default::default() }
}

fn parent(kind: &str, children: Vec<TreeNode>) -> TreeNode {
    TreeNode { kind: kind.into(), children, ..Default::default() }
}

fn assign_ids(node: &mut TreeNode, next: &mut u64) {
    node.id = *next;
    *next += 1;
    for c in &mut node.children {
        assign_ids(c, next);
    }
}

/// An enabled `TreeState` around `root` (ids assigned, next_id recomputed).
fn enabled_tree(mut root: TreeNode) -> TreeState {
    let mut next = 0u64;
    assign_ids(&mut root, &mut next);
    let mut state = TreeState::fresh();
    state.enabled = true;
    state.root = root;
    state.recompute_next_id();
    state
}

/// Structurally complete but gate-SIGLESS tree: `or_d(pk, after)` →
/// `sigless_branch @ root.or_d[1]` (the plant-1 recipe from
/// `tests/tree_path_parity.rs`).
fn sigless_root() -> TreeNode {
    parent("or_d", vec![pk(XPUB_A), uint("after", 500_000)])
}

/// FormState with ALL THREE modes' inputs stale-populated (the §5
/// mode-mutex matrix worst case): a `--spec` path, a selected
/// `--archetype`, all 9 params, a checked `--emit-spec`, PLUS the
/// mode-independent `--allow`/`--network`/`--json` — and the tree.
fn state_with_everything_stale(tree_enabled: bool) -> FormState {
    let mut state = FormState::from_pairs(vec![
        ("--spec", FlagValue::Path("/tmp/stale-spec.json".into())),
        ("--archetype", FlagValue::Dropdown("kofn-recovery".into())),
        ("--key", FlagValue::Text("xpubKEY1".into())),
        ("--key", FlagValue::Text("xpubKEY2".into())),
        ("--threshold", FlagValue::Number(2)),
        ("--recovery-key", FlagValue::Text("xpubREC".into())),
        ("--recovery-threshold", FlagValue::Number(1)),
        ("--final-key", FlagValue::Text("xpubFINAL".into())),
        ("--older", FlagValue::Number(144)),
        ("--recovery-older", FlagValue::Number(288)),
        ("--after", FlagValue::Number(800_000)),
        (
            "--hash",
            FlagValue::Text(
                "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".into(),
            ),
        ),
        ("--emit-spec", FlagValue::Boolean(true)),
        ("--allow", FlagValue::Dropdown("sigless-branch".into())),
        ("--network", FlagValue::Dropdown("testnet".into())),
        ("--json", FlagValue::Boolean(true)),
    ]);
    let mut tree = enabled_tree(sigless_root());
    tree.enabled = tree_enabled;
    state.tree = Some(tree);
    state
}

fn run_result(stdout: &str, stderr: &str, exit: i32) -> RunResult {
    RunResult {
        argv: vec!["mnemonic".into()],
        exit_code: Some(exit),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

// ─── mode-mutex cells (SPEC §0 / §5) ─────────────────────────────────────

#[test]
fn cell_tree_mode_argv_suppresses_all_other_mode_flags() {
    // Tree mode with EVERYTHING stale-populated: argv must carry NO
    // --spec <path> / --archetype / param flags / --emit-spec; the
    // mode-independent flags keep emitting.
    let state = state_with_everything_stale(true);
    let argv = build_descriptor_argv(&state);
    for name in [
        "--spec",
        "--archetype",
        "--key",
        "--threshold",
        "--recovery-key",
        "--recovery-threshold",
        "--final-key",
        "--older",
        "--recovery-older",
        "--after",
        "--hash",
        "--emit-spec",
    ] {
        assert!(
            !argv.contains(&name.to_string()),
            "tree mode must suppress stale {name}: {argv:?}"
        );
    }
    assert_eq!(occurrences(&argv, "--allow"), ["sigless-branch"], "{argv:?}");
    assert_eq!(occurrences(&argv, "--network"), ["testnet"], "{argv:?}");
    assert!(argv.contains(&"--json".to_string()), "{argv:?}");
}

#[test]
fn cell_switching_back_from_tree_mode_restores_the_other_modes() {
    // enabled=false with the archetype still selected → the v0.31.0
    // archetype rules return verbatim: --archetype + its declared params
    // emit again (--spec stays suppressed by the archetype↔spec mutex).
    //
    // The stale --spec row is dropped for this leg: a state with BOTH
    // --spec and --archetype populated trips the v0.30.0 mutex in both
    // directions (neither emits) — a state the live form can't reach
    // (the --spec widget is Disabled the moment an archetype is selected;
    // see archetype_form.rs). Restoration is asserted per live-reachable
    // shape: archetype-selected first, then generic with --spec.
    let mut state = state_with_everything_stale(false);
    state.values.retain(|(k, _)| k != "--spec");
    let argv = build_descriptor_argv(&state);
    assert_eq!(occurrences(&argv, "--archetype"), ["kofn-recovery"], "{argv:?}");
    for declared in ["--key", "--threshold", "--recovery-key", "--older"] {
        assert!(
            argv.contains(&declared.to_string()),
            "declared param {declared} must emit again after leaving tree mode: {argv:?}"
        );
    }
    assert!(
        !argv.contains(&"--spec".to_string()),
        "--spec stays suppressed by the archetype mutex: {argv:?}"
    );

    // Deselect the archetype ("" sentinel — the Generic gesture) and
    // restore the --spec row: it emits again (generic / spec-file mode).
    if let Some(row) = state.values.iter_mut().find(|(k, _)| k == "--archetype") {
        row.1 = FlagValue::Dropdown(String::new());
    }
    state
        .values
        .push(("--spec".to_string(), FlagValue::Path("/tmp/stale-spec.json".into())));
    let argv = build_descriptor_argv(&state);
    assert_eq!(
        occurrences(&argv, "--spec"),
        ["/tmp/stale-spec.json"],
        "generic mode restores the --spec row's emission: {argv:?}"
    );
}

#[test]
fn cell_tree_mode_visibility_map_disables_two_hides_ten() {
    // The conditional arm itself (SPEC §0): --spec + --archetype Disabled;
    // the 9 params + --emit-spec Hidden.
    let state = state_with_everything_stale(true);
    let vis = mnemonic_gui::form::conditional::build_descriptor(&state);
    let vis_of = |name: &str| {
        vis.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or(Visibility::Visible)
    };
    assert_eq!(vis_of("--spec"), Visibility::Disabled);
    assert_eq!(vis_of("--archetype"), Visibility::Disabled);
    for name in [
        "--key",
        "--threshold",
        "--recovery-key",
        "--recovery-threshold",
        "--final-key",
        "--older",
        "--recovery-older",
        "--after",
        "--hash",
        "--emit-spec",
    ] {
        assert_eq!(vis_of(name), Visibility::Hidden, "{name} must be Hidden");
    }
    // Mode-independent flags carry no override.
    for name in ["--allow", "--format", "--network", "--json", "--spec-schema"] {
        assert_eq!(vis_of(name), Visibility::Visible, "{name} must stay Visible");
    }
}

#[test]
fn cell_suppressed_in_tree_mode_is_exactly_the_twelve() {
    // Render-half accounting: --spec + --archetype + 9 params +
    // --emit-spec suppressed; the 6 mode-independent flags render.
    let sub = subcommand("build-descriptor");
    let (suppressed, independent): (Vec<&str>, Vec<&str>) = sub
        .flags
        .iter()
        .map(|f| f.name)
        .partition(|n| tree_form::suppressed_in_tree_mode(n));
    assert_eq!(suppressed.len(), 12, "2 mode flags + 9 params + --emit-spec: {suppressed:?}");
    assert_eq!(
        independent,
        ["--spec-schema", "--allow", "--format", "--network", "--json", "--no-auto-repair"],
        "the 6 mode-independent flags"
    );
}

// ─── Validate argv cells (SPEC §2.2 / R0-r2 M3) ──────────────────────────

#[test]
fn cell_validate_argv_fixed_prefix_plus_allow_occurrences() {
    // Fixed argv (argv[0] = the CLI binary name — R0-r1 M1) + the user's
    // --allow rows in row order; empty (added-untouched) rows skipped.
    let state = FormState::from_pairs(vec![
        ("--allow", FlagValue::Dropdown("sigless-branch".into())),
        ("--allow", FlagValue::Dropdown(String::new())), // untouched row
        ("--allow", FlagValue::Dropdown("timelock-mixing".into())),
        ("--network", FlagValue::Dropdown("testnet".into())), // NOT Validate's
        ("--json", FlagValue::Boolean(true)),                 // already fixed
    ]);
    assert_eq!(
        tree_form::validate_argv(&state),
        [
            "mnemonic",
            "build-descriptor",
            "--spec",
            "-",
            "--json",
            "--allow",
            "sigless-branch",
            "--allow",
            "timelock-mixing",
        ],
        "fixed prefix + --allow pass-through only (the other flags belong to Run)"
    );
    assert_eq!(
        tree_form::validate_argv(&FormState::default()),
        ["mnemonic", "build-descriptor", "--spec", "-", "--json"],
    );
}

// ─── parse-contract cells (apply_validate_result) ────────────────────────

#[test]
fn cell_fail_soft_unmatched_paths_land_in_the_strip() {
    // R0-r1 I5: a diagnostic whose node_path matches NO live node (the
    // "params" sentinel, drift like "root.bogus[9]") renders in the global
    // strip — never dropped, never a panic. Matched paths tint normally.
    let mut tree = enabled_tree(sigless_root());
    let envelope = serde_json::json!({
        "diagnostics": [
            { "node_path": "params", "kind": "param",
              "message": "kofn-recovery requires --key (missing)", "flag": "--key" },
            { "node_path": "root.bogus[9]", "kind": "schema_field",
              "message": "drifted path" },
            { "node_path": "root.or_d[1]", "kind": "sigless_branch",
              "message": "this subtree can be spent without any signature" },
        ]
    });
    tree_form::apply_validate_result(&mut tree, &run_result(&envelope.to_string(), "", 2));
    assert_eq!(
        tree.diagnostics,
        vec![TreeDiag {
            node_path: "root.or_d[1]".into(),
            kind: "sigless_branch".into(),
            message: "this subtree can be spent without any signature".into(),
        }],
        "only the live-path diagnostic is node-addressed"
    );
    assert_eq!(tree.strip.len(), 2, "both unmatched diagnostics fail-soft to the strip");
    assert!(tree.strip[0].contains("params") && tree.strip[0].contains("--key (missing)"));
    assert!(tree.strip[1].contains("root.bogus[9]"));
    assert!(tree.validate_ok.is_none());
}

#[test]
fn cell_non_envelope_failure_routes_stderr_to_the_strip() {
    // §2.3 version-skew posture: spec-parse errors / clap errors emit
    // EMPTY stdout + stderr text (probed) — the strip, never a panic.
    let mut tree = enabled_tree(sigless_root());
    tree_form::apply_validate_result(
        &mut tree,
        &run_result("", "error: spec JSON parse error: missing field `root`", 2),
    );
    assert!(tree.diagnostics.is_empty() && tree.validate_ok.is_none());
    assert_eq!(tree.strip, ["error: spec JSON parse error: missing field `root`"]);

    // Degenerate: no stdout AND no stderr still surfaces something.
    tree_form::apply_validate_result(&mut tree, &run_result("", "", 2));
    assert_eq!(tree.strip.len(), 1);
    assert!(tree.strip[0].contains("exit 2"), "{:?}", tree.strip);
}

#[test]
fn cell_success_envelope_sets_validate_ok_and_allow_note_strips() {
    // The success envelope ALSO carries "diagnostics": [] (probed against
    // v0.52.0) — the descriptor arm must win. With --allow, the override
    // banner lands on stderr → the strip (R0-r2 M3).
    let mut tree = enabled_tree(sigless_root());
    let envelope = serde_json::json!({
        "descriptor": "wsh(or_d(pk(...),after(500000)))#checksum",
        "cost": { "conditions": [
            { "wsh_vbytes": 73 }, { "wsh_vbytes": 41 },
        ] },
        "diagnostics": [],
        "allowed_rules_fired": ["sigless-branch"],
    });
    tree_form::apply_validate_result(
        &mut tree,
        &run_result(
            &envelope.to_string(),
            "WARNING: sanity rules OVERRIDDEN by --allow and FIRED: sigless-branch.",
            0,
        ),
    );
    let ok = tree.validate_ok.as_ref().expect("descriptor arm must set validate_ok");
    assert_eq!(ok.descriptor, "wsh(or_d(pk(...),after(500000)))#checksum");
    assert!(tree.diagnostics.is_empty(), "empty diagnostics array must not tint");
    assert_eq!(tree.strip.len(), 1, "the --allow override note lands in the strip");
    assert!(tree.strip[0].contains("OVERRIDDEN by --allow"));

    // Clean success (no stderr): no strip entry.
    tree_form::apply_validate_result(&mut tree, &run_result(&envelope.to_string(), "", 0));
    assert!(tree.strip.is_empty());
    assert!(tree.validate_ok.is_some());
}

// ─── selector gesture cells (R0-r2 M2) ───────────────────────────────────

fn selector_harness(initial: FormState) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        |ui, state: &mut FormState| {
            tree_form::render_mode_selector(ui, state);
        },
        initial,
    )
}

#[test]
fn cell_selector_gestures_tree_archetype_generic() {
    // Start in archetype mode (dropdown-derived) with no tree yet.
    let initial = FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("kofn-recovery".into())),
        ("--key", FlagValue::Text("xpubA".into())),
    ]);
    let mut harness = selector_harness(initial);
    harness.run();

    // Tree click: creates + enables the tree; values untouched.
    harness.get_by_label(MODE_TREE_LABEL).click();
    harness.run();
    assert!(tree_form::tree_enabled(harness.state()), "Tree click sets enabled");
    // Give the tree a node so never-destroys is observable.
    {
        let tree = harness.state_mut().tree.as_mut().unwrap();
        let mut next = tree.next_id;
        tree.root.set_kind("pk", 1, &mut next).unwrap();
        tree.next_id = next;
        tree.root.key = XPUB_A.into();
    }

    // Archetype click: clears enabled FIRST; the dropdown selection is NOT
    // touched (it derives Archetype mode again), tree nodes preserved.
    harness.get_by_label(MODE_ARCHETYPE_LABEL).click();
    harness.run();
    let state = harness.state();
    assert!(!tree_form::tree_enabled(state), "Archetype click clears enabled");
    assert_eq!(
        state.values.iter().find(|(k, _)| k == "--archetype").map(|(_, v)| v.clone()),
        Some(FlagValue::Dropdown("kofn-recovery".into())),
        "Archetype click must NOT reset the dropdown"
    );
    assert_eq!(state.tree.as_ref().unwrap().root.kind, "pk", "tree nodes preserved");

    // Back to Tree: the SAME tree returns (never-destroys).
    harness.get_by_label(MODE_TREE_LABEL).click();
    harness.run();
    assert!(tree_form::tree_enabled(harness.state()));
    assert_eq!(harness.state().tree.as_ref().unwrap().root.key, XPUB_A);

    // Generic click: clears enabled FIRST and resets the dropdown to the
    // "" "(none)" sentinel (the v0.31.0-native gesture); values + tree
    // nodes survive (never-destroys is scoped to values/params/nodes —
    // the dropdown SELECTION is exempt).
    harness.get_by_label(MODE_GENERIC_LABEL).click();
    harness.run();
    let state = harness.state();
    assert!(!tree_form::tree_enabled(state), "Generic click clears enabled");
    assert_eq!(
        state.values.iter().find(|(k, _)| k == "--archetype").map(|(_, v)| v.clone()),
        Some(FlagValue::Dropdown(String::new())),
        "Generic click resets the dropdown to \"(none)\""
    );
    assert!(
        state.values.iter().any(|(k, v)| k == "--key" && *v == FlagValue::Text("xpubA".into())),
        "param values survive"
    );
    assert_eq!(state.tree.as_ref().unwrap().root.key, XPUB_A, "tree nodes survive");
}

// ─── tree-form kittest cells ─────────────────────────────────────────────

/// Harness rendering the lib-seam tree form exactly as main.rs dispatches
/// it (`tree_form::render(ui, state, bin)`).
fn tree_form_harness(initial: FormState, bin: String) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            tree_form::render(ui, state, &bin);
        },
        initial,
    )
}

fn form_with_tree(tree: TreeState) -> FormState {
    FormState { tree: Some(tree), ..Default::default() }
}

#[test]
fn cell_collapse_expand_does_not_clear_diagnostics() {
    // R0-r1 M6b: collapse/expand lives in egui memory, not TreeState —
    // it must NOT route through mark_dirty. Inject a diagnostic, toggle
    // the root header twice, diagnostics survive both frames.
    let mut tree = enabled_tree(sigless_root());
    tree.diagnostics.push(TreeDiag {
        node_path: "root.or_d[1]".into(),
        kind: "sigless_branch".into(),
        message: "planted".into(),
    });
    // bin is never spawned (Validate not clicked) — any string works.
    let mut harness = tree_form_harness(form_with_tree(tree), "mnemonic".into());
    harness.run();
    assert!(
        harness.query_by_label("⚠ [sigless_branch] planted").is_some(),
        "the node-addressed message renders under the matched node"
    );

    // The root header label ("or_d — 2 branches" — node_summary is
    // deliberately distinct from the bare kind the ComboBox shows).
    harness.get_by_label("or_d — 2 branches").click();
    harness.run();
    assert_eq!(
        harness.state().tree.as_ref().unwrap().diagnostics.len(),
        1,
        "collapse must NOT clear diagnostics"
    );
    harness.get_by_label("or_d — 2 branches").click();
    harness.run();
    assert_eq!(
        harness.state().tree.as_ref().unwrap().diagnostics.len(),
        1,
        "expand must NOT clear diagnostics either"
    );
}

#[test]
fn cell_completeness_banner_gates_validate() {
    // An incomplete tree (unset root): the banner renders with the count +
    // path, and clicking Validate is impossible (button disabled) — the
    // state stays untouched.
    let tree = enabled_tree(TreeNode::new_unset(0));
    let mut harness = tree_form_harness(form_with_tree(tree), "mnemonic".into());
    harness.run();
    assert!(
        harness.query_by_label("⚠ 1 incomplete node(s): root").is_some(),
        "the completeness banner renders count + first paths"
    );
    let validate = harness.get_by_label("Validate");
    assert!(validate.is_disabled(), "Validate must be disabled while incomplete");
    assert!(
        tree_form::spec_stdin_bytes(harness.state()).is_none()
            && tree_form::spec_json_pretty(harness.state()).is_none(),
        "Run/Copy-spec share the completeness gate (R0-r1 M4)"
    );
}

#[test]
fn cell_validate_flow_diagnose_edit_clears_fix_green() {
    // SPEC §5 Validate-flow kittest (REAL binary, skip-if-absent): a
    // complete sigless tree → Validate → the offending node carries the
    // message; a WIDGET edit clears it (the mutation chokepoint); fixing
    // the tree → Validate → the validate_ok summary renders.
    let Some(bin) = resolve_bin() else { return };
    let tree = enabled_tree(sigless_root());
    let mut harness = tree_form_harness(form_with_tree(tree), bin);
    harness.run();

    harness.get_by_label("Validate").click();
    harness.run();
    {
        let tree = harness.state().tree.as_ref().unwrap();
        assert_eq!(tree.diagnostics.len(), 1, "strip: {:?}", tree.strip);
        assert_eq!(tree.diagnostics[0].node_path, "root.or_d[1]");
        assert_eq!(tree.diagnostics[0].kind, "sigless_branch");
        assert!(tree.validate_ok.is_none(), "Validate red — no green summary");
        // The message renders under the matched node (tint + message).
        let expected_label =
            format!("⚠ [{}] {}", tree.diagnostics[0].kind, tree.diagnostics[0].message);
        assert!(
            harness.query_by_label(&expected_label).is_some(),
            "the node-addressed message must render: {expected_label}"
        );
    }

    // EDIT via a real widget (the per-child ✕ on the first branch): the
    // structural change routes through mark_dirty and clears the stale
    // tint (sibling removal shifts [i] indices — the funds-safety rule).
    let removes: Vec<_> = harness.get_all_by_label("✕").collect();
    assert!(!removes.is_empty(), "per-child remove buttons render");
    removes[0].click();
    drop(removes);
    harness.run();
    {
        let tree = harness.state().tree.as_ref().unwrap();
        assert!(
            tree.diagnostics.is_empty() && tree.strip.is_empty(),
            "a widget edit must clear diagnostics via the chokepoint"
        );
        assert_eq!(tree.root.children.len(), 1, "the ✕ removed one branch");
    }

    // FIX: replace with a gate-valid tree (wsh(pk(X)) — probed exit 0),
    // then Validate again → the green summary renders.
    {
        let tree = harness.state_mut().tree.as_mut().unwrap();
        tree.root = pk(XPUB_A);
        let mut next = 0;
        assign_ids(&mut tree.root, &mut next);
        tree.recompute_next_id();
    }
    harness.run();
    harness.get_by_label("Validate").click();
    harness.run();
    {
        let tree = harness.state().tree.as_ref().unwrap();
        let ok = tree
            .validate_ok
            .as_ref()
            .unwrap_or_else(|| panic!("fixed tree must validate green; strip: {:?}", tree.strip));
        assert!(ok.descriptor.starts_with("wsh(pk("), "{}", ok.descriptor);
        assert!(tree.diagnostics.is_empty());
    }
    assert!(
        harness.query_by_label("✓ gate passed").is_some(),
        "the green Validate summary renders"
    );
}

#[test]
fn cell_validate_allow_pass_through_green_with_override_note() {
    // R0-r2 M3 (REAL binary): a deliberately-sigless tree + an --allow
    // sigless-branch row → Validate goes GREEN (the argv carried the
    // user's --allow) and the override banner (stderr) shows in the strip.
    let Some(bin) = resolve_bin() else { return };
    let mut state = form_with_tree(enabled_tree(sigless_root()));
    state
        .values
        .push(("--allow".to_string(), FlagValue::Dropdown("sigless-branch".into())));
    let mut harness = tree_form_harness(state, bin);
    harness.run();

    harness.get_by_label("Validate").click();
    harness.run();
    let tree = harness.state().tree.as_ref().unwrap();
    assert!(
        tree.validate_ok.is_some(),
        "--allow must flip the Validate verdict (strip: {:?})",
        tree.strip
    );
    assert!(tree.diagnostics.is_empty());
    assert_eq!(tree.strip.len(), 1, "the override note lands in the strip");
    assert!(
        tree.strip[0].contains("sigless-branch"),
        "the note names the overridden rule: {:?}",
        tree.strip
    );
}

#[test]
fn cell_kind_picker_combobox_drives_set_kind_with_arity_padding() {
    // SPEC §5: the kind ComboBox IS kittest-drivable (the A2 pattern from
    // repeating_rows::cell_archetype_unset_sentinel_displays_as_none_label
    // holds — open the combo by role, click the option label). Selecting
    // `andor` on the unset root routes through set_kind: arity padding
    // materializes 3 unset placeholders with fresh ids.
    let tree = enabled_tree(TreeNode::new_unset(0));
    let mut harness = tree_form_harness(form_with_tree(tree), "mnemonic".into());
    harness.run();
    assert!(
        harness.query_by_label("(unset node)").is_some(),
        "the unset root renders the placeholder header"
    );

    harness.get_by_role(egui::accesskit::Role::ComboBox).click();
    harness.run();
    harness.get_by_label("andor").click();
    harness.run();

    let tree = harness.state().tree.as_ref().unwrap();
    assert_eq!(tree.root.kind, "andor");
    assert_eq!(tree.root.children.len(), 3, "Children3 padding materializes");
    assert!(tree.root.children.iter().all(|c| c.kind.is_empty()));
    let ids: std::collections::BTreeSet<u64> =
        tree.root.children.iter().map(|c| c.id).collect();
    assert_eq!(ids.len(), 3, "fresh unique child ids");
    assert!(tree.next_id > *ids.iter().max().unwrap(), "next_id bookkeeping kept");
    assert_eq!(
        harness.query_all_by_label("(unset node)").count(),
        3,
        "the three placeholder children render"
    );
}

#[test]
fn cell_depth_cap_add_branch_refusal_lands_in_strip() {
    // The MAX_TREE_DEPTH refusal message (SPEC §1.2 render leg): a thresh
    // node AT the cap refuses "+ add branch" with the amber notice in the
    // strip. Driven at the form level with a deep wrap chain; the harness
    // is sized tall enough that the deepest widgets stay clickable.
    let mut node = TreeNode { kind: "thresh".into(), k: 1, ..Default::default() };
    node.children.push(pk(XPUB_A));
    for _ in 0..(MAX_TREE_DEPTH - 1) {
        node = TreeNode {
            kind: "wrap".into(),
            w: "v".into(),
            children: vec![node],
            ..Default::default()
        };
    }
    let tree = enabled_tree(node);
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(2400.0, 16000.0))
        .build_ui_state(
            move |ui, state: &mut FormState| {
                tree_form::render(ui, state, "mnemonic");
            },
            form_with_tree(tree),
        );
    harness.run();

    harness.get_by_label("+ add branch").click();
    harness.run();
    let tree = harness.state().tree.as_ref().unwrap();
    assert!(
        tree.strip.iter().any(|m| m.contains("maximum tree depth")),
        "the depth-cap refusal must land in the strip: {:?}",
        tree.strip
    );
    // And the tree itself is unchanged (the refusal is not a mutation).
    let mut deepest = &tree.root;
    while !deepest.children.is_empty() && deepest.kind == "wrap" {
        deepest = &deepest.children[0];
    }
    assert_eq!(deepest.kind, "thresh");
    assert_eq!(deepest.children.len(), 1, "no branch was added past the cap");
}

// ─── P3: "Edit as tree…" cells (SPEC §3) ─────────────────────────────────

/// The vendored kofn-recovery fixture's remaining keys (XPUB_A above IS
/// the fixture's first key).
const XPUB_B: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
const XPUB_C: &str = "[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
const XPUB_D: &str = "[44444444/48h/0h/0h/2h]xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa";

fn fixture(name: &str) -> serde_json::Value {
    let path = format!(
        "{}/tests/fixtures/descriptor_builder/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse fixture {path}: {e}"))
}

/// An archetype-mode FormState whose kofn-recovery params reproduce the
/// vendored `kofn-recovery.json` fixture (the params it was generated
/// from), PLUS stale mode-independent flags that must NOT enter the
/// `--emit-spec` argv (`--json`/`--format` are clap-conflicting with it).
fn kofn_archetype_form() -> FormState {
    FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("kofn-recovery".into())),
        ("--key", FlagValue::Text(XPUB_A.into())),
        ("--key", FlagValue::Text(XPUB_B.into())),
        ("--key", FlagValue::Text(XPUB_C.into())),
        ("--threshold", FlagValue::Number(2)),
        ("--recovery-key", FlagValue::Text(XPUB_D.into())),
        ("--older", FlagValue::Number(52_560)),
        // Mode-independent flags that must NOT ride along.
        ("--json", FlagValue::Boolean(true)),
        ("--format", FlagValue::Dropdown("descriptor".into())),
        ("--network", FlagValue::Dropdown("testnet".into())),
        ("--allow", FlagValue::Dropdown("sigless-branch".into())),
    ])
}

fn edit_as_tree_harness(initial: FormState, bin: String) -> Harness<'static, FormState> {
    Harness::new_ui_state(
        move |ui, state: &mut FormState| {
            tree_form::render_edit_as_tree(ui, state, &bin);
        },
        initial,
    )
}

#[test]
fn cell_emit_spec_argv_archetype_params_only() {
    // Purpose-built argv (SPEC §3): --archetype + the declared param rows
    // in schema order + --emit-spec; NO mode-independent flags (--json /
    // --format clap-conflict with --emit-spec — probed) and no --spec.
    let state = kofn_archetype_form();
    let spec = mnemonic_gui::schema::archetypes::find("kofn-recovery").expect("kofn");
    assert_eq!(
        tree_form::emit_spec_argv(&state, spec),
        [
            "mnemonic",
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            XPUB_A,
            "--key",
            XPUB_B,
            "--key",
            XPUB_C,
            "--threshold",
            "2",
            "--recovery-key",
            XPUB_D,
            "--older",
            "52560",
            "--emit-spec",
        ],
        "archetype + declared params (schema order, all rows) + --emit-spec; \
         nothing else"
    );

    // Empty Text rows skip (the emit_one rule); missing params are not
    // pre-gated — the binary's [param] diagnostics are the validator.
    let sparse = FormState::from_pairs(vec![
        ("--archetype", FlagValue::Dropdown("kofn-recovery".into())),
        ("--key", FlagValue::Text(String::new())), // added-but-blank row
    ]);
    assert_eq!(
        tree_form::emit_spec_argv(&sparse, spec),
        ["mnemonic", "build-descriptor", "--archetype", "kofn-recovery", "--emit-spec"],
    );
}

#[test]
fn cell_edit_as_tree_happy_path_populates_tree_and_switches_mode() {
    // SPEC §3 happy path (REAL binary, skip-if-absent): kofn params →
    // "Edit as tree…" → the tree holds the kofn fixture AST modulo ids,
    // tree mode is on, next_id satisfies the import invariant.
    let Some(bin) = resolve_bin() else { return };
    let mut harness = edit_as_tree_harness(kofn_archetype_form(), bin);
    harness.run();

    harness.get_by_label("Edit as tree…").click();
    harness.run();

    let state = harness.state();
    assert!(
        state.edit_as_tree_error.is_none(),
        "happy path must not error: {:?}",
        state.edit_as_tree_error
    );
    assert!(tree_form::tree_enabled(state), "success switches to tree mode");
    let tree = state.tree.as_ref().expect("tree populated");
    // AST modulo ids: to_spec_json projects ids away — value-equal to the
    // vendored fixture (the binary's lowering IS the fixture's source).
    assert_eq!(
        to_spec_json(&tree.root),
        fixture("kofn-recovery"),
        "the lowered spec must match the vendored kofn fixture AST"
    );
    // The import invariant (R0-r1 I3): next_id = max_id(root) + 1 — sane
    // for post-import adds (no egui-identity collisions).
    assert_eq!(tree.next_id, max_id(&tree.root) + 1, "next_id recomputed on import");
    // The imported tree is complete → the tree-mode gates open.
    assert!(tree_form::spec_stdin_bytes(state).is_some(), "imported tree is complete");
    // The archetype params survive (never-destroys — switching back is
    // one selector click).
    assert!(
        state.values.iter().any(|(k, v)| k == "--key" && *v == FlagValue::Text(XPUB_A.into())),
        "archetype params stay populated"
    );
}

#[test]
fn cell_edit_as_tree_failure_stays_archetype_and_surfaces_stderr() {
    // SPEC §3 failure path (REAL binary): an archetype with NO params →
    // the binary refuses ([param] diagnostics on stderr, empty stdout) →
    // archetype mode unchanged, no tree created, the error label renders.
    let Some(bin) = resolve_bin() else { return };
    let initial = FormState::from_pairs(vec![(
        "--archetype",
        FlagValue::Dropdown("kofn-recovery".into()),
    )]);
    let mut harness = edit_as_tree_harness(initial, bin);
    harness.run();

    harness.get_by_label("Edit as tree…").click();
    harness.run();

    let state = harness.state();
    assert!(!tree_form::tree_enabled(state), "failure must stay in archetype mode");
    assert!(state.tree.is_none(), "failure must not create/populate a tree");
    let err = state
        .edit_as_tree_error
        .as_ref()
        .expect("the stderr must be surfaced in the transient error");
    assert!(
        err.contains("requires --key (missing)"),
        "the binary's [param] diagnostics surface verbatim: {err}"
    );
    // ... and the label renders next to the button.
    assert!(
        harness
            .query_by_label(&format!("edit as tree failed: {err}"))
            .is_some(),
        "the transient error label must render"
    );

    // The error never persists (#[serde(skip)] — transient by type).
    let raw = serde_json::to_string(harness.state()).expect("serialize FormState");
    assert!(
        !raw.contains("edit_as_tree_error"),
        "edit_as_tree_error must be absent from the wire"
    );
}

#[test]
fn cell_edit_as_tree_renders_only_with_an_archetype_selected() {
    // No archetype (the "" sentinel / no row): the button does not render.
    let mut harness = edit_as_tree_harness(FormState::default(), "mnemonic".into());
    harness.run();
    assert!(
        harness.query_by_label("Edit as tree…").is_none(),
        "no archetype selected → no button"
    );
}

// ─── P3: POSIX pipeline copy cells (SPEC §3) ─────────────────────────────

#[test]
fn cell_posix_pipeline_copy_quoting_flags_and_completeness_gate() {
    // A complete tree + mode-independent flags; argv assembled exactly as
    // the main.rs host does (assemble_argv + the appended `--spec -`).
    let mut state = form_with_tree(enabled_tree(sigless_root()));
    state
        .values
        .push(("--network".to_string(), FlagValue::Dropdown("testnet".into())));
    state
        .values
        .push(("--allow".to_string(), FlagValue::Dropdown("sigless-branch".into())));
    let mut argv = build_descriptor_argv(&state);
    argv.push("--spec".to_string());
    argv.push("-".to_string());

    let pipeline = tree_form::posix_pipeline_command(&state, &argv)
        .expect("complete tree in tree mode → Some");

    // Quoting law: shlex round-trips the pipeline; the token after
    // `printf '%s'` is EXACTLY the compact spec JSON Run pipes (the
    // spec_stdin_bytes parity), `|` is its own token, and the argv tail
    // carries the user's flags + `--spec -`.
    let tokens = shlex::split(&pipeline).expect("the pipeline must be POSIX-parseable");
    assert_eq!(tokens[0], "printf");
    assert_eq!(tokens[1], "%s");
    let expected_json =
        String::from_utf8(tree_form::spec_stdin_bytes(&state).unwrap()).unwrap();
    assert_eq!(tokens[2], expected_json, "the quoted JSON must round-trip byte-exact");
    assert_eq!(tokens[3], "|");
    assert_eq!(tokens[4], "mnemonic");
    assert_eq!(tokens[5], "build-descriptor");
    for pair in [["--network", "testnet"], ["--allow", "sigless-branch"]] {
        assert!(
            tokens.windows(2).any(|w| w == pair),
            "mode-independent flag {pair:?} must ride the pipeline: {tokens:?}"
        );
    }
    assert_eq!(&tokens[tokens.len() - 2..], ["--spec", "-"], "spec via stdin");

    // Completeness gate (R0-r1 M4 parity with Copy spec JSON).
    let incomplete = form_with_tree(enabled_tree(TreeNode::new_unset(0)));
    assert!(
        tree_form::posix_pipeline_command(&incomplete, &argv).is_none(),
        "incomplete tree → no pipeline copy"
    );

    // Mode gate: a complete but DISABLED tree (non-tree mode) → None.
    let mut disabled = form_with_tree(enabled_tree(sigless_root()));
    disabled.tree.as_mut().unwrap().enabled = false;
    assert!(
        tree_form::posix_pipeline_command(&disabled, &argv).is_none(),
        "non-tree mode → no pipeline copy"
    );
}

#[test]
fn cell_posix_pipeline_executes_through_a_real_shell() {
    // The pasted one-liner is the product — prove it RUNS: sh -c the
    // pipeline against the real binary and expect the same descriptor the
    // direct Validate path yields. Skip-if-absent on both sh and the
    // binary; an --allow row flips the sigless tree green.
    let Some(bin) = resolve_bin() else { return };
    if !std::path::Path::new("/bin/sh").exists() {
        eprintln!("/bin/sh not available; skipping pipeline-exec cell");
        return;
    }
    let mut state = form_with_tree(enabled_tree(sigless_root()));
    state
        .values
        .push(("--allow".to_string(), FlagValue::Dropdown("sigless-branch".into())));
    let mut argv = build_descriptor_argv(&state);
    argv.push("--spec".to_string());
    argv.push("-".to_string());
    // The copy string carries the contract argv[0] "mnemonic"; for the
    // exec leg substitute the resolved binary path (what a user with the
    // binary on PATH gets for free).
    argv[0] = bin;
    let pipeline = tree_form::posix_pipeline_command(&state, &argv).expect("complete");

    let out = std::process::Command::new("/bin/sh")
        .args(["-c", &pipeline])
        .output()
        .expect("spawn sh");
    assert!(
        out.status.success(),
        "the pasted pipeline must run exit-0:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("wsh(or_d("),
        "the pipeline must yield the descriptor: {stdout}"
    );
}
