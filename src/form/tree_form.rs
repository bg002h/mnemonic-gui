//! v0.32.0 P2 (node-tree builder SPEC §2.2) — the recursive tree form +
//! mode selector + Validate round-trip for `mnemonic build-descriptor`.
//!
//! Lib seam (the `archetype_form` precedent): this module is the REAL
//! form — `src/main.rs` only dispatches — so kittests drive these `pub`
//! fns rather than re-implementing main.rs logic.
//!
//! Mode model (R0-r1 I4): ONLY the tree bit is stored
//! (`TreeState.enabled`); Generic-vs-Archetype remains dropdown-derived
//! exactly as v0.31.0. Selector gestures (R0-r2 M2): selecting Tree sets
//! `enabled`; selecting Generic or Archetype CLEARS `enabled` FIRST (the
//! dropdown doesn't render in tree mode — clear-then-act); Generic-click
//! additionally resets the dropdown to the `""` "(none)" sentinel. The
//! never-destroys guarantee is scoped to VALUES/params/tree-NODES; the
//! dropdown SELECTION is exempt.
//!
//! Mutation chokepoint (R0-r1 I4 lifetime rule): EVERY widget write routes
//! through `TreeState::mark_dirty` — implemented as a structural
//! before/after diff of the root around the render walk, so NO write can
//! be missed (a stale tint on the wrong node after sibling removal is a
//! funds-tool correctness bug). CollapsingHeader collapse/expand lives in
//! egui memory, not `TreeState`, and therefore does NOT trip the diff
//! (cell: R0-r1 M6b).
//!
//! Parse contract (SPEC §0, probed against the v0.52.0 binary): the
//! SUCCESS envelope also carries `"diagnostics": []`, so the discriminator
//! order is (1) a `"descriptor"` string → `ValidateOk`; (2) a
//! `"diagnostics"` array → the node-addressed view, with any diagnostic
//! whose `node_path` matches NO live node (drift, the `"params"` sentinel,
//! stale paths) rendered in the global strip — fail-soft, never dropped,
//! never a panic (R0-r1 I5); (3) anything else (spec-parse error, version
//! skew, clap error: empty stdout) → stderr in the strip. NEVER
//! exit-code-keyed.

use eframe::egui;

use crate::form::archetype_form;
use crate::form::invocation::{posix_quote, render_copy_command, ShellFlavor};
use crate::form::tree_model::{
    completeness, emitted_child_count, from_spec_json, is_xprv_like, node_paths,
    to_spec_json, TreeDiag, TreeNode, TreeState, ValidateOk,
};
use crate::form::widget::display_or;
use crate::runner::{self, RunResult};
use crate::schema::archetypes::{ArchetypeSpec, ARCHETYPE_PARAM_FLAGS};
use crate::schema::nodes::{spec_for, PayloadShape, NODE_KIND_SPECS};
use crate::schema::{FlagValue, FormState};

const AMBER: egui::Color32 = egui::Color32::from_rgb(220, 165, 0);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 60, 60);
const GREEN: egui::Color32 = egui::Color32::from_rgb(60, 180, 75);

/// Selector labels (the §2.2 three-way row).
pub const MODE_GENERIC_LABEL: &str = "Generic / spec file";
pub const MODE_ARCHETYPE_LABEL: &str = "Archetype";
pub const MODE_TREE_LABEL: &str = "Tree builder";

/// The tree-mode predicate: `FormState.tree` exists AND its `enabled` bit
/// is set (the ONLY stored mode state — R0-r1 I4).
pub fn tree_enabled(state: &FormState) -> bool {
    state.tree.as_ref().is_some_and(|t| t.enabled)
}

/// True iff `flag_name` is suppressed from the generic flag loop in TREE
/// mode (SPEC §0 mode mutex render leg): `--spec` + `--archetype` (their
/// argv suppression is the conditional's `Disabled`; their ROWS don't
/// render — the mode selector + tree form replace them) + the 10
/// `requires = "archetype"` flags (9 params + `--emit-spec`; argv-
/// suppressed via the conditional's `Hidden`, listed here for the render
/// dispatch). The 6 mode-independent flags (`--spec-schema`, `--allow`,
/// `--format`, `--network`, `--json`, `--no-auto-repair`) render normally.
pub fn suppressed_in_tree_mode(flag_name: &str) -> bool {
    flag_name == "--spec"
        || flag_name == "--archetype"
        || flag_name == "--emit-spec"
        || ARCHETYPE_PARAM_FLAGS.contains(&flag_name)
}

/// Structural completeness gaps for the CURRENT tree mode. Empty when not
/// in tree mode (non-tree modes are never completeness-gated) — the host
/// gates Validate / Run / Copy-spec on this being empty.
pub fn completeness_gaps(state: &FormState) -> Vec<String> {
    state
        .tree
        .as_ref()
        .filter(|t| t.enabled)
        .map(|t| completeness(&t.root))
        .unwrap_or_default()
}

/// The compact spec-JSON bytes piped to `--spec -` on Run. `Some` ONLY in
/// tree mode with a structurally complete tree (`to_spec_json` of an unset
/// placeholder is undefined — R0-r1 M4 gates Copy and Run alike).
pub fn spec_stdin_bytes(state: &FormState) -> Option<Vec<u8>> {
    let t = state.tree.as_ref()?;
    if !t.enabled || !completeness(&t.root).is_empty() {
        return None;
    }
    Some(to_spec_json(&t.root).to_string().into_bytes())
}

/// Pretty spec JSON for the "Copy spec JSON" button. Same completeness
/// gate as [`spec_stdin_bytes`] (R0-r1 M4).
pub fn spec_json_pretty(state: &FormState) -> Option<String> {
    let t = state.tree.as_ref()?;
    if !t.enabled || !completeness(&t.root).is_empty() {
        return None;
    }
    serde_json::to_string_pretty(&to_spec_json(&t.root)).ok()
}

/// v0.32.0 P3 (SPEC §3) — the tree-mode POSIX pipeline copy string:
/// `printf '%s' '<json>' | mnemonic build-descriptor … --spec -`.
/// `argv` is the host-assembled Run argv (the user's mode-independent
/// flags + the appended `--spec -`), quoted by the same
/// `render_copy_command` the argv copies use; the spec JSON is COMPACT
/// (`spec_stdin_bytes` parity — the pasted pipeline feeds the binary the
/// exact bytes Run pipes) and `posix_quote`d (single-quote-safe for
/// arbitrary key content). `Some` ONLY in tree mode with a structurally
/// complete tree — the same completeness gate as Copy spec JSON
/// (R0-r1 M4). The Windows copy deliberately stays argv + the separate
/// "Copy spec JSON" button (no `CommandLineToArgvW`-safe pipeline
/// one-liner exists for arbitrary JSON under cmd.exe quoting).
pub fn posix_pipeline_command(state: &FormState, argv: &[String]) -> Option<String> {
    let json_bytes = spec_stdin_bytes(state)?;
    let json = String::from_utf8(json_bytes).expect("serde_json output is UTF-8");
    Some(format!(
        "printf '%s' {} | {}",
        posix_quote(&json),
        render_copy_command(argv, ShellFlavor::Posix),
    ))
}

// ───────────────────────────── P3: "Edit as tree…" (archetype lowering) ──

/// v0.32.0 P3 (SPEC §3) — the `--emit-spec` argv for the CURRENTLY
/// selected archetype + its param rows: `["mnemonic", "build-descriptor",
/// "--archetype", <id>, <declared param rows in schema order>,
/// "--emit-spec"]` (`argv[0]` = the CLI binary name, the runner contract).
///
/// PURPOSE-BUILT rather than `assemble_argv` reuse: `--emit-spec` is
/// `conflicts_with_all = [--format, --json]` (probed against v0.52.0), so
/// the user's mode-independent flags must NOT ride along — the lowering
/// input is the archetype + params ONLY. Every matching `state.values`
/// row emits (parity with the archetype-mode Run argv, which emits every
/// row off the static `FlagSchema.repeating` — the v0.31.0 C1 "what emits
/// is what renders" corollary); empty Text rows skip (the `emit_one`
/// rule). Missing/incomplete params are NOT pre-gated — the binary's
/// `[param]` diagnostics are the validator (the no-second-validation-path
/// ethos), surfaced by [`render_edit_as_tree`]'s error label.
pub fn emit_spec_argv(state: &FormState, spec: &'static ArchetypeSpec) -> Vec<String> {
    let mut argv: Vec<String> = vec![
        "mnemonic".to_string(),
        "build-descriptor".to_string(),
        "--archetype".to_string(),
        spec.id.to_string(),
    ];
    for param in spec.params {
        for (_, value) in state.values.iter().filter(|(k, _)| k == param.flag) {
            match value {
                FlagValue::Text(s) if !s.is_empty() => {
                    argv.push(param.flag.to_string());
                    argv.push(s.clone());
                }
                FlagValue::Number(n) => {
                    argv.push(param.flag.to_string());
                    argv.push(n.to_string());
                }
                // Param flags are Text (key / hex_digest) or Number
                // (threshold / blocks / absolute_locktime) by schema;
                // anything else is a type-shape mismatch — skip, the
                // binary's param diagnostics report the absence.
                _ => {}
            }
        }
    }
    argv.push("--emit-spec".to_string());
    argv
}

/// v0.32.0 P3 (SPEC §3) — apply one `--emit-spec` subprocess result:
/// stdout parses as a spec doc (`from_spec_json`, the full grammar +
/// version/wrapper checks) → populate the tree via the import chokepoint
/// (`TreeState::import_root` recomputes `next_id` — the R0-r1 I3 import
/// invariant) and switch to tree mode (`enabled = true`); anything else →
/// `Err(message)` and the state is UNTOUCHED (stay in archetype mode).
/// NEVER re-implements lowering — the binary's emitted spec is installed
/// verbatim. Parse-keyed, not exit-code-keyed (the §0 contract posture):
/// a refusal emits EMPTY stdout + human stderr (probed), which fails the
/// parse and routes the stderr into the message.
pub fn apply_emit_spec_result(
    state: &mut FormState,
    result: &RunResult,
) -> Result<(), String> {
    let parsed = serde_json::from_slice::<serde_json::Value>(&result.stdout)
        .map_err(|e| e.to_string())
        .and_then(|doc| from_spec_json(&doc));
    match parsed {
        Ok(root) => {
            let tree = state.tree.get_or_insert_with(TreeState::fresh);
            tree.import_root(root);
            tree.enabled = true;
            Ok(())
        }
        Err(parse_err) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stderr = stderr.trim();
            Err(if stderr.is_empty() {
                format!(
                    "--emit-spec produced no parseable spec (exit {}): {parse_err}",
                    result
                        .exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "killed".to_string())
                )
            } else {
                stderr.to_string()
            })
        }
    }
}

/// v0.32.0 P3 (SPEC §3) — the archetype-mode "Edit as tree…" button + its
/// transient error label. Renders whenever an archetype is selected
/// (param completeness is NOT pre-gated — simplest posture, decided:
/// failure is handled by the binary's diagnostics); on click, runs
/// [`emit_spec_argv`] via the runner (no stdin) and applies
/// [`apply_emit_spec_result`]. Success switches to tree mode (the
/// archetype dropdown + params stay — never-destroys; switching back is
/// one selector click). Failure stays in archetype mode and surfaces the
/// stderr in `FormState.edit_as_tree_error`, rendered RED under the
/// button; the label lives until the next attempt overwrites or clears it
/// (transient by type — `#[serde(skip)]`, never persisted). `bin` is the
/// spawn target ("mnemonic" from main.rs; the pinned-binary path from
/// tests — the argv CONTRACT keeps `argv[0] = "mnemonic"`).
pub fn render_edit_as_tree(ui: &mut egui::Ui, state: &mut FormState, bin: &str) {
    let Some(spec) = archetype_form::active_archetype(state) else {
        return;
    };
    let clicked = ui
        .button("Edit as tree…")
        .on_hover_text(
            "runs `build-descriptor --archetype … --emit-spec` and loads the \
             lowered spec into the tree builder (your archetype params are \
             kept; switch back anytime)",
        )
        .clicked();
    if clicked {
        let mut argv = emit_spec_argv(state, spec);
        argv[0] = bin.to_string();
        state.edit_as_tree_error = match runner::run(argv) {
            Ok(result) => apply_emit_spec_result(state, &result).err(),
            // OS spawn failure (binary missing): the label, not a crash.
            Err(e) => Some(format!("could not spawn `{bin}`: {e}")),
        };
    }
    if let Some(err) = &state.edit_as_tree_error {
        ui.colored_label(RED, format!("edit as tree failed: {err}"));
    }
}

/// The FIXED Validate argv (SPEC §2.2): `["mnemonic", "build-descriptor",
/// "--spec", "-", "--json"]` (`argv[0]` = the CLI binary name, the runner
/// contract — R0-r1 M1) PLUS the user's `--allow` occurrences (R0-r2 M3 —
/// `--allow` changes the VERDICT, not a view: a deliberately-overridden
/// tree must not show permanently-red Validate while Run succeeds). The
/// user's OTHER flags belong to Run (`--network` only affects the human
/// view, not `--json`).
pub fn validate_argv(state: &FormState) -> Vec<String> {
    let mut argv: Vec<String> = ["mnemonic", "build-descriptor", "--spec", "-", "--json"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for (k, v) in &state.values {
        if k == "--allow" {
            if let FlagValue::Dropdown(rule) = v {
                if !rule.is_empty() {
                    argv.push("--allow".to_string());
                    argv.push(rule.clone());
                }
            }
        }
    }
    argv
}

/// Apply one Validate subprocess result to the tree per the parse
/// contract (module doc). Clears all transient result state first; never
/// panics on drifted/missing fields (fail-soft — R0-r1 I5).
pub fn apply_validate_result(tree: &mut TreeState, result: &RunResult) {
    tree.diagnostics.clear();
    tree.validate_ok = None;
    tree.strip.clear();

    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&result.stdout) {
        // (1) Success envelope: a "descriptor" string (probed: it ALSO
        // carries "diagnostics": [], so this arm must run first).
        if let Some(descriptor) = json.get("descriptor").and_then(|d| d.as_str()) {
            tree.validate_ok = Some(ValidateOk {
                descriptor: descriptor.to_string(),
                cost: json.get("cost").cloned().unwrap_or(serde_json::Value::Null),
            });
            // R0-r2 M3: with --allow, the override banner lands on stderr
            // → the strip shows the override note alongside the green
            // verdict.
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stderr = stderr.trim();
            if !stderr.is_empty() {
                tree.strip.push(stderr.to_string());
            }
            return;
        }
        // (2) Failure envelope: {"diagnostics": [...]} — node-addressed
        // view. Unmatched node_path (drift, "params", stale) → the strip.
        if let Some(diags) = json.get("diagnostics").and_then(|d| d.as_array()) {
            let live = node_paths(&tree.root);
            for d in diags {
                let node_path = d.get("node_path").and_then(|p| p.as_str()).unwrap_or("");
                let kind = d.get("kind").and_then(|p| p.as_str()).unwrap_or("");
                let message = d.get("message").and_then(|p| p.as_str()).unwrap_or("");
                if live.iter().any(|(p, _)| p == node_path) {
                    tree.diagnostics.push(TreeDiag {
                        node_path: node_path.to_string(),
                        kind: kind.to_string(),
                        message: message.to_string(),
                    });
                } else {
                    tree.strip.push(format!("{node_path}: [{kind}] {message}"));
                }
            }
            return;
        }
    }
    // (3) Non-envelope failure (spec-parse error, version skew, clap
    // error) → stderr in the strip (§2.3 version-skew posture).
    let stderr = String::from_utf8_lossy(&result.stderr);
    let stderr = stderr.trim();
    tree.strip.push(if stderr.is_empty() {
        format!(
            "validate failed (exit {}) with no parseable output",
            result
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "killed".to_string())
        )
    } else {
        stderr.to_string()
    });
}

/// One-line cost note for the Validate summary (SPEC §2.2: "descriptor
/// string + a one-line cost note"). Derived from the envelope's
/// `cost.conditions` array; degrades gracefully on drift.
fn cost_note(cost: &serde_json::Value) -> String {
    let Some(conditions) = cost.get("conditions").and_then(|c| c.as_array()) else {
        return "cost: (no preview)".to_string();
    };
    let vbytes: Vec<i64> = conditions
        .iter()
        .filter_map(|c| c.get("wsh_vbytes").and_then(|v| v.as_i64()))
        .collect();
    match (vbytes.iter().min(), vbytes.iter().max()) {
        (Some(min), Some(max)) if min == max => {
            format!("cost: {} spend condition(s), {} wsh vB each", conditions.len(), min)
        }
        (Some(min), Some(max)) => format!(
            "cost: {} spend condition(s), {}–{} wsh vB",
            conditions.len(),
            min,
            max
        ),
        _ => format!("cost: {} spend condition(s)", conditions.len()),
    }
}

// ─────────────────────────────────────────────────────── mode selector ──

/// The §2.2 three-way mode-selector row. View-state only (R0-r1 I4):
/// Tree sets `TreeState.enabled`; Generic/Archetype clear `enabled`
/// FIRST; Generic also resets the `--archetype` dropdown to the `""`
/// "(none)" sentinel (the v0.31.0-native gesture). Values, archetype
/// params, and tree nodes are never destroyed by mode switches.
pub fn render_mode_selector(ui: &mut egui::Ui, state: &mut FormState) {
    let in_tree = tree_enabled(state);
    let in_archetype = !in_tree && archetype_form::active_archetype(state).is_some();
    let in_generic = !in_tree && !in_archetype;

    ui.horizontal(|ui| {
        ui.label("Mode:");
        if ui.selectable_label(in_generic, MODE_GENERIC_LABEL).clicked() {
            // Clear-enabled-first (R0-r2 M2), then the Generic gesture:
            // reset the dropdown to "(none)" (stored "").
            if let Some(t) = state.tree.as_mut() {
                t.enabled = false;
            }
            if let Some(row) = state.values.iter_mut().find(|(k, _)| k == "--archetype") {
                row.1 = FlagValue::Dropdown(String::new());
            }
        }
        if ui
            .selectable_label(in_archetype, MODE_ARCHETYPE_LABEL)
            .clicked()
        {
            // Clear-enabled-first only — the dropdown (which renders
            // again once tree mode exits) drives Archetype-vs-Generic.
            if let Some(t) = state.tree.as_mut() {
                t.enabled = false;
            }
        }
        if ui.selectable_label(in_tree, MODE_TREE_LABEL).clicked() {
            match state.tree.as_mut() {
                Some(t) => t.enabled = true,
                None => {
                    let mut fresh = TreeState::fresh();
                    fresh.enabled = true;
                    state.tree = Some(fresh);
                }
            }
        }
    });
}

// ──────────────────────────────────────────────────────── tree render ──

/// Render the tree form: node-count strip, the recursive node walk, the
/// completeness banner, the Validate button + summary, and the global
/// error strip. `bin` is the CLI binary the Validate run spawns
/// (`"mnemonic"` from main.rs; the pinned-binary path from tests — the
/// argv CONTRACT keeps `argv[0] = "mnemonic"`, see [`validate_argv`];
/// only the spawn target is substituted).
pub fn render(ui: &mut egui::Ui, state: &mut FormState, bin: &str) {
    let Some(tree) = state.tree.as_ref() else { return };
    if !tree.enabled {
        return;
    }

    // Pre-walk (immutable): diagnostic id-map + the dirty-diff snapshot.
    // Key-row paths (root.multi.keys[i]) map to the OWNING node's id via
    // node_paths, so key-class diagnostics tint the quorum node.
    let live = node_paths(&tree.root);
    let mut diag_by_id: std::collections::BTreeMap<u64, Vec<String>> =
        std::collections::BTreeMap::new();
    for d in &tree.diagnostics {
        if let Some((_, id)) = live.iter().find(|(p, _)| p == &d.node_path) {
            diag_by_id
                .entry(*id)
                .or_default()
                .push(format!("[{}] {}", d.kind, d.message));
        }
        // No else: an unmatched path can only enter `diagnostics` via
        // external construction — apply_validate_result routes those to
        // the strip (fail-soft). Unmatched here would simply not tint.
    }
    let before = tree.root.clone();

    ui.separator();
    ui.horizontal(|ui| {
        ui.strong("Policy tree");
        ui.weak(format!("{} nodes", tree.root.node_count()));
    });

    // Mutable walk. Split borrows so set_kind/try_add_child can allocate
    // ids while nodes are being edited; refusal notices collect locally.
    let mut notices: Vec<String> = Vec::new();
    {
        let tree = state.tree.as_mut().expect("checked Some above");
        let TreeState { root, next_id, .. } = tree;
        render_node(ui, root, 1, next_id, &diag_by_id, &mut notices);
    }

    // THE mutation chokepoint (R0-r1 I4): any structural/payload change in
    // the walk routes through mark_dirty. Collapse/expand lives in egui
    // memory, not TreeState — it cannot trip this diff (M6b).
    {
        let tree = state.tree.as_mut().expect("checked Some above");
        if tree.root != before {
            tree.mark_dirty();
        }
        // Depth-cap refusals surface in the strip AFTER the dirty clear
        // (a refusal is not a mutation; it must not be eaten by it).
        tree.strip.extend(notices);
    }

    // Completeness banner (SPEC §1.2): count + the first few paths;
    // non-empty gates Validate (and the host's Run / Copy-spec).
    let gaps = completeness_gaps(state);
    let complete = gaps.is_empty();
    if !complete {
        let shown: Vec<&str> = gaps.iter().take(3).map(String::as_str).collect();
        ui.colored_label(
            AMBER,
            format!(
                "⚠ {} incomplete node(s): {}{}",
                gaps.len(),
                shown.join(", "),
                if gaps.len() > 3 { ", …" } else { "" }
            ),
        );
    }

    // Validate (SPEC §2.2): completeness-gated; fixed argv + the user's
    // --allow occurrences; spec piped via stdin. Does NOT write last_run
    // (R0-r1 I6) — results live entirely in TreeState.
    let mut validate_clicked = false;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(complete, egui::Button::new("Validate"))
            .on_hover_text(
                "runs `build-descriptor --spec - --json` (+ your --allow rows) \
                 and maps diagnostics onto the tree; does not touch the output panel",
            )
            .clicked()
        {
            validate_clicked = true;
        }
    });
    if validate_clicked {
        let mut argv = validate_argv(state);
        argv[0] = bin.to_string();
        let stdin = state
            .tree
            .as_ref()
            .map(|t| to_spec_json(&t.root).to_string().into_bytes());
        match runner::run_with_stdin(argv, stdin) {
            Ok(result) => {
                let tree = state.tree.as_mut().expect("checked Some above");
                apply_validate_result(tree, &result);
            }
            Err(e) => {
                // OS spawn failure (binary missing): the strip, not a crash.
                let tree = state.tree.as_mut().expect("checked Some above");
                tree.strip = vec![format!("validate could not spawn `{bin}`: {e}")];
            }
        }
    }

    // Validate summary + the global strip.
    let tree = state.tree.as_ref().expect("checked Some above");
    if let Some(ok) = &tree.validate_ok {
        ui.colored_label(GREEN, "✓ gate passed");
        ui.monospace(&ok.descriptor);
        ui.weak(cost_note(&ok.cost));
    }
    for msg in &tree.strip {
        ui.colored_label(RED, msg);
    }
}

/// One-line node summary for the CollapsingHeader title (kind + detail).
/// Deliberately distinct from the bare kind string the ComboBox shows, so
/// kittest label queries can target the header unambiguously.
fn node_summary(node: &TreeNode) -> String {
    let Some(spec) = spec_for(&node.kind) else {
        return "(unset node)".to_string();
    };
    let detail = match spec.payload {
        PayloadShape::Key => {
            if node.key.is_empty() {
                "key unset".to_string()
            } else {
                truncate(&node.key, 24)
            }
        }
        PayloadShape::KeyQuorum => format!("{}-of-{}", node.k, node.keys.len()),
        PayloadShape::Uint => node.n.to_string(),
        PayloadShape::Hex64 | PayloadShape::Hex40 => {
            if node.hex.is_empty() {
                "hex unset".to_string()
            } else {
                truncate(&node.hex, 16)
            }
        }
        PayloadShape::Children2 => "2 branches".to_string(),
        PayloadShape::Children3 => "3 branches".to_string(),
        PayloadShape::ThreshSubs => {
            format!("{} of {} branches", node.k, node.children.len())
        }
        PayloadShape::Wrap => {
            if node.w.is_empty() {
                "wrapper unset".to_string()
            } else {
                format!("{}:", node.w)
            }
        }
    };
    format!("{} — {}", spec.kind, detail)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max).collect();
        format!("{prefix}…")
    }
}

/// The recursive node renderer. `depth` is THIS node's depth (root = 1 —
/// the tree_model convention); `next_id` feeds arity padding + add-branch;
/// `notices` collects depth-cap refusal messages for the strip.
fn render_node(
    ui: &mut egui::Ui,
    node: &mut TreeNode,
    depth: usize,
    next_id: &mut u64,
    diag_by_id: &std::collections::BTreeMap<u64, Vec<String>>,
    notices: &mut Vec<String>,
) {
    ui.push_id(node.id, |ui| {
        let diags = diag_by_id.get(&node.id);
        let title = if diags.is_some() {
            egui::RichText::new(node_summary(node)).color(RED)
        } else {
            egui::RichText::new(node_summary(node))
        };
        egui::CollapsingHeader::new(title)
            .default_open(true)
            .show(ui, |ui| {
                // Diagnostic tint + message under path-matched nodes.
                if let Some(msgs) = diags {
                    for m in msgs {
                        ui.colored_label(RED, format!("⚠ {m}"));
                    }
                }

                // Kind picker: 17 kinds + the "(choose…)" unset sentinel,
                // display-mapped via the shared display_or helper
                // (R0-r1 M2). Selection routes through set_kind so arity
                // padding + the depth cap apply (SPEC §1.2).
                let mut selected = node.kind.clone();
                egui::ComboBox::from_id_salt("tree_kind")
                    .selected_text(display_or("(choose…)", &selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, String::new(), "(choose…)");
                        for spec in NODE_KIND_SPECS {
                            ui.selectable_value(
                                &mut selected,
                                spec.kind.to_string(),
                                spec.kind,
                            );
                        }
                    });
                if selected != node.kind {
                    if let Err(msg) = node.set_kind(&selected, depth, next_id) {
                        notices.push(msg);
                    }
                }

                // Payload widgets by shape (SPEC §2.2).
                if let Some(spec) = spec_for(&node.kind) {
                    render_payload(ui, node, spec.payload, depth, next_id, notices);
                }

                // Children: the emitted in-arity prefix renders plainly;
                // children beyond it carry the surplus amber flag
                // (preserve-and-flag — what does NOT emit must be VISIBLY
                // flagged). Per-child remove, collect-then-apply.
                let emit_n = emitted_child_count(node);
                let mut remove_at: Option<usize> = None;
                for (i, child) in node.children.iter_mut().enumerate() {
                    ui.push_id(("child_row", i), |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .small_button("✕")
                                .on_hover_text("remove this branch")
                                .clicked()
                            {
                                remove_at = Some(i);
                            }
                            if i >= emit_n {
                                ui.colored_label(AMBER, "surplus — will not emit");
                            }
                        });
                        render_node(ui, child, depth + 1, next_id, diag_by_id, notices);
                    });
                }
                if let Some(i) = remove_at {
                    node.children.remove(i);
                }
            });
    });
}

/// Payload-shape widget arms (SPEC §2.2 / §1.1 PayloadShape table).
fn render_payload(
    ui: &mut egui::Ui,
    node: &mut TreeNode,
    shape: PayloadShape,
    depth: usize,
    next_id: &mut u64,
    notices: &mut Vec<String>,
) {
    match shape {
        PayloadShape::Key => {
            ui.horizontal(|ui| {
                ui.label("key");
                // cycle-15 Lane G slug-4: mask a mis-pasted xprv-shaped key on
                // screen, gated on the SAME `is_xprv_like` predicate `xprv_hint`
                // keys off (mask + amber hint co-fire). A watch-only xpub (the
                // canonical input) stays readable.
                let key_is_xprv = crate::form::tree_model::is_xprv_like(&node.key);
                ui.add(egui::TextEdit::singleline(&mut node.key).password(key_is_xprv));
            });
            xprv_hint(ui, &node.key);
        }
        PayloadShape::KeyQuorum => {
            ui.horizontal(|ui| {
                ui.label("k");
                // range floor 0: the wide-node k default 0 is the UNSET
                // sentinel (completeness flags k < 1) — a min-1 clamp
                // would silently "complete" a node the user never set.
                ui.add(egui::DragValue::new(&mut node.k).range(0..=100_000));
                if ui.button("+ add key").clicked() {
                    node.keys.push(String::new());
                }
            });
            let mut remove_at: Option<usize> = None;
            for i in 0..node.keys.len() {
                ui.push_id(("key_row", i), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("keys[{i}]"));
                        // cycle-15 Lane G slug-4: same xprv-shaped masking as the
                        // single Key arm above, per keys[i] row.
                        let k_is_xprv = crate::form::tree_model::is_xprv_like(&node.keys[i]);
                        ui.add(
                            egui::TextEdit::singleline(&mut node.keys[i]).password(k_is_xprv),
                        );
                        if ui.small_button("✕").on_hover_text("remove key row").clicked()
                        {
                            remove_at = Some(i);
                        }
                    });
                    xprv_hint(ui, &node.keys[i]);
                });
            }
            if let Some(i) = remove_at {
                node.keys.remove(i);
            }
        }
        PayloadShape::Uint => {
            ui.horizontal(|ui| {
                ui.label("n (blocks / locktime)");
                ui.add(egui::DragValue::new(&mut node.n).range(0..=4_294_967_295_i64));
            });
        }
        PayloadShape::Hex64 | PayloadShape::Hex40 => {
            let want = if shape == PayloadShape::Hex64 { 64 } else { 40 };
            ui.horizontal(|ui| {
                ui.label("hex digest");
                ui.text_edit_singleline(&mut node.hex);
                ui.weak(format!("({want} hex chars)"));
            });
            if !node.hex.is_empty()
                && (node.hex.len() != want
                    || !node.hex.chars().all(|c| c.is_ascii_hexdigit()))
            {
                ui.colored_label(AMBER, format!("⚠ expected {want} hex chars"));
            }
        }
        PayloadShape::ThreshSubs => {
            ui.horizontal(|ui| {
                ui.label("k");
                ui.add(egui::DragValue::new(&mut node.k).range(0..=100_000));
                if ui.button("+ add branch").clicked() {
                    if let Err(msg) = node.try_add_child(depth, next_id) {
                        notices.push(msg);
                    }
                }
            });
        }
        PayloadShape::Wrap => {
            ui.horizontal(|ui| {
                ui.label("w");
                ui.text_edit_singleline(&mut node.w);
            });
            // Free text because wrappers compose (e.g. "sv") — SPEC §2.2.
            ui.weak("wrappers: a s c d v j n l u t (compose, e.g. \"sv\")");
        }
        PayloadShape::Children2 | PayloadShape::Children3 => {
            // Children-only payloads: the child walk below renders them.
        }
    }
}

/// The live xprv amber hint (SPEC §1.3): the persistence layer BLANKS
/// xprv-matching keys, and the toolkit gate refuses them — warn at type
/// time, never block (the gate is the validator).
///
/// v0.34.0 (audit I6) widening note: the persist layer now blanks EVERY
/// non-extended-public key/keys entry (WIF, raw hex — incl. legit hex
/// pubkeys — and garbage), not just xprv-likes; this hint still fires only
/// on the xprv shape, so those other classes blank silently at persist.
/// Extending the hint is render-side polish, deliberately out of the
/// v0.34.0 scope.
fn xprv_hint(ui: &mut egui::Ui, key: &str) {
    if is_xprv_like(key) {
        ui.colored_label(
            AMBER,
            "⚠ looks like an extended PRIVATE key — the builder is watch-only; \
             this value will not be saved and the gate will refuse it",
        );
    }
}
