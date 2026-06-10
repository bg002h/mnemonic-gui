# SPEC — GUI v0.32.0: the recursive node-tree builder

**Status:** R0 GREEN (round 2, 0C/0I; 3 minors folded) — implementation may begin
**Source grounding verified at:** GUI `origin/master` = `c59f388`-pending (brainstorm committed on `d2fe58b` = tag v0.31.1); toolkit binary v0.52.0 (`11d36e5`)
**Foundation:** `design/BRAINSTORM_node_tree_builder.md` (R0 GREEN, 3 rounds — every locked decision + fold binds this SPEC; §0 ground truth incorporated by reference, re-grep at edit time)
**Shape:** one GUI MINOR (v0.32.0), three phases, P3 cuttable to v0.32.1. No toolkit involvement; no flag-name change → no `schema_mirror` delta.

## 0. Inherited contracts (brainstorm-locked; restated normatively)

- **Mode mutex:** tree enabled ⇒ `--spec` + `--archetype` `Disabled`, ALL TEN `requires="archetype"` flags `Hidden` (9 params + `--emit-spec`); render dispatch is mode-aware (neither the archetype param form nor the `--spec` row renders in tree mode). Mode-independent flags (`--allow`, `--format`, `--network`, `--json`, `--spec-schema`, `--no-auto-repair`) render normally.
- **Parse contract:** stdout-parses-as-`{diagnostics:[…]}` → node-addressed view; else stderr → the global error strip. NEVER exit-code-keyed.
- **stdin discipline:** `write_all` → drop `ChildStdin` → `wait_with_output`; `BrokenPipe` degrades to collect-output; specs ≤ ~2 KB (stated license for the unthreaded writer).
- **Path grammar:** `child_path()` mirrors the gate's 4 `child_paths` arms + `root` + `keys[i]`; the LIVE parity gate is the tether (6 plants over 5 productions — binary + andor share the `{kind}[i]` form; the `keys[i]` plant uses a keys-CLASS error, `secret_key`).
- **Persistence:** `FormState.tree: Option<TreeState>` `#[serde(default)]`, persisted; `redact_for_persistence` blanks xprv-matching key fields (the `gate.rs:281` heuristic: strip `[origin]`, bytes 1..4 == `"prv"`); hashlock hex deliberately not redacted; diagnostics NEVER persist (`#[serde(skip)]`).

## 1. P1 — model + contracts (UI-free, fully TDD-able)

### 1.1 `src/schema/nodes.rs` (new) — the grammar mirror

```rust
pub enum PayloadShape { Key, KeyQuorum, Uint, Hex64, Hex40, Children2, Children3, ThreshSubs, Wrap }
pub struct NodeKindSpec { pub kind: &'static str, pub payload: PayloadShape, pub payload_str: &'static str }
pub const NODE_KIND_SPECS: &[NodeKindSpec] = &[ /* 17 entries, NODE_KINDS order */ ];
```

`payload_str` is the binary's verbatim `payload` string (e.g. `"{k:uint, keys:[key]}"`) — compared byte-equal by the drift gate; `PayloadShape` is the GUI-side closed enum (9 variants) driving widget arms + arity. Self-test: the 17 kinds are unique; every `payload_str` maps to exactly one shape.

### 1.2 `src/form/tree_model.rs` (new)

```rust
pub struct TreeState {
    pub enabled: bool,
    pub next_id: u64,
    pub root: TreeNode,
    #[serde(skip)] pub diagnostics: Vec<TreeDiag>,   // transient; cleared on ANY tree mutation
    #[serde(skip)] pub validate_ok: Option<ValidateOk>, // descriptor + cost summary, transient
}
pub struct TreeNode {
    pub id: u64,
    pub kind: String,          // "" = unset placeholder
    pub key: String, pub k: i64, pub keys: Vec<String>,
    pub n: i64, pub hex: String, pub w: String,
    pub children: Vec<TreeNode>,
}
pub struct TreeDiag { pub node_path: String, pub kind: String, pub message: String }
```

- **Wide node:** kind switches never destroy payload data; serialization projects ONLY the active kind's fields.
- **Kind-switch children policy (brainstorm §3 → decided): preserve-and-flag.** Children are kept verbatim on any kind switch; the renderer shows children beyond the new kind's arity under a `surplus — will not emit` amber flag with per-child remove (the v0.31.0 "what emits is what renders" inversion: what does NOT emit must be VISIBLY flagged). Serialization emits only the in-arity prefix.
- **Arity materialization:** selecting a fixed-arity kind pads `children` with unset placeholders up to arity (2/3/1 for Children2/Children3/Wrap; ThreshSubs starts at 0 + "add branch").
- **Unset semantics (decided): GUI-side completeness gate.** `completeness(root) -> Vec<String /*paths*/>` lists unset/incomplete nodes (kind `""`, empty key on a key kind, k<1 on quorums — the UNSET-sentinel reading of the wide-node `k: i64` default 0, NOT semantic validation (R0-r1 M3: the GUI checks STRUCTURAL emptiness only; k≤n, hex validity, key format are the toolkit gate's, returned node-addressed — the no-second-validation-path ethos); empty hex, empty `w`, in-arity unset children); Validate/Run are DISABLED while non-empty, with a count + the first few paths shown (the CLI's bad-arity stderr remains the backstop per the parse contract — never relied on for UX).
- **Serializers:** `to_spec_json(&TreeNode) -> serde_json::Value` (wrapped `{schema_version:1, wrapper:"wsh", root}`) and `from_spec_json(&Value) -> Result<TreeNode, String>` (full 17-kind grammar; ids assigned 0..n during the walk; **the IMPORT INVARIANT (R0-r1 I3): every populate-from-import site recomputes `next_id = max_id(root) + 1`** — a named helper + cell, else post-import adds collide with imported ids and break egui identity; rejects unknown kinds/fields with a path-bearing message; **checks `schema_version == 1` and `wrapper == "wsh"` on import — runtime parity with the test-time refuse-loud pin, R0-r1 M5**). Round-trip laws (R0-r1 I1 — the naive `from(to(t)) == t` is FALSE for wide nodes, whose inactive fields + surplus children are deliberately NOT serialized): **(a) `to(from(j)) == j` value-equal for any valid spec; (b) `from(to(t)) == projection(t)`** where projection = active-kind fields + in-arity child prefix — equivalently `to(from(to(t))) == to(t)` (to-idempotence). The §4.3 goldens test (a); a unit cell with a stale-data wide node tests (b).
- **`child_path(parent, kind, i)`** + `node_paths(root) -> Vec<(String, id)>` per the §0 grammar; unit-pinned with GUI-authored literals incl. `root.thresh.subs[2].andor[2].multi.keys[0]`.
- **Depth posture (brainstorm I2): `MAX_TREE_DEPTH = 64`** — add-child/paste-import refuse beyond it (amber message); the serializers assert it defensively. A node-count strip (`N nodes`) renders in the tree header; no hard node cap (the toolkit gate caps what matters).

### 1.3 Persistence + redaction

`FormState.tree: Option<TreeState>` (+ `#[serde(default)]`); `redact_for_persistence`'s struct literal gains the field (the no-Clone forcing function): tree is persisted with every node's `key`/`keys` entries BLANKED when xprv-matching (recursive walk; the heuristic above). Cells: an xprv in a tree key field does not survive a redact round-trip; xpubs + hex digests do; diagnostics/validate_ok absent by type.

## 2. P2 — the form + runner + Validate

### 2.1 `runner::run_with_stdin(argv, stdin: Option<Vec<u8>>)`

Per the §0 discipline; existing `run` delegates with `None` (byte-identical behavior incl. `MNEMONIC_FORCE_TTY`). Unit cell: a child that reads stdin to EOF gets the bytes; a child that exits immediately (clap error) does not error the parent (BrokenPipe degraded); output still collected.

### 2.2 `src/form/tree_form.rs` (new, lib-hosted; main.rs dispatch-only)

- **Mode selector** at the top of the build-descriptor form: `Generic / spec file · Archetype · Tree builder`. **State model (R0-r1 I4): ONLY the tree bit is stored (`TreeState.enabled`); Generic-vs-Archetype remains dropdown-DERIVED exactly as v0.31.0** (the selector is a view; gestures R0-r2 M2: selecting Tree sets `enabled`; selecting Generic or Archetype CLEARS `enabled` first (the dropdown doesn't render in tree mode — clear-then-act); Generic-click additionally sets the dropdown to `"(none)"` — the v0.31.0-native gesture. The never-destroys guarantee is scoped to VALUES/params/tree-NODES; the dropdown SELECTION is exempt). v0.31.0 behavior is therefore provably unchanged, and a pre-v0.32.0 `state.json` (no `tree` field) loads `None` → non-tree mode (migration cell, R0-r1 M6a). Switching modes never destroys any mode's state (values/archetype params/tree all persist).
- **Tree render:** recursive walk with `push_id(node.id)`; per node: CollapsingHeader (kind + a one-line summary), kind ComboBox (17 + `"(choose…)"` unset sentinel — the `""`→display-label mapping reuses the v0.30.0 IDIOM, not a shared mechanism (that mapping is inline in the FlagValue Dropdown arm); extract a tiny `display_or(label, value)` helper both sites call — R0-r1 M2), payload widgets by `PayloadShape` (Key→Text w/ xprv amber hint; KeyQuorum→k-Number + repeating key rows reusing the v0.30.0 row idiom; Uint→Number; Hex64/Hex40→Text + length hint; ThreshSubs→k-Number + child list + "add branch"; Wrap→`w` free-Text + a hint listing `a s c d v j n l u t` (free text because wrappers compose, e.g. `"sv"`) ); per-child remove (collect-then-apply); surplus-children amber flags; diagnostic tint + message under any node whose path matches `diagnostics`.
- **Every mutating action clears `diagnostics` + `validate_ok`** (one `mark_dirty(&mut TreeState)` chokepoint called by all widget WRITES — CollapsingHeader collapse/expand lives in egui memory, not TreeState, and MUST NOT clear (cell, R0-r1 M6b)).
- **Validate button:** disabled while incomplete; runs FIXED argv `["mnemonic", "build-descriptor", "--spec", "-", "--json"]` (argv[0] = the CLI binary, the runner contract — R0-r1 M1) via `run_with_stdin(to_spec_json bytes)`; result parsed per the §0 contract into `diagnostics`/`validate_ok`/global strip; **a diagnostic whose `node_path` matches NO live node (drift, the `"params"` sentinel, stale paths) renders in the global strip — fail-soft, never dropped, never a panic (R0-r1 I5 restores the brainstorm rule; cell: an envelope with `node_path: "params"` and one with `"root.bogus[9]"` both land in the strip).** (Fixed argv: `--network` only affects the human view, not `--json`; the user's other flags belong to Run.) **Validate does NOT write `last_run` (R0-r1 I6): the bottom output panel remains Run's surface; Validate's results live entirely in the tree view — two surfaces, no clobbering either way.** **Validate argv ALSO carries the user's `--allow` occurrences (R0-r2 M3 — `--allow` changes the VERDICT, not a view: a deliberately-overridden tree must not show permanently-red Validate while Run succeeds; the banner lands on stderr → the strip shows the override note).**
- **Run button (the existing one):** in tree mode the host appends `["--spec","-"]` to the assembled argv (mode-independent flags included) and routes through `run_with_stdin`. The argv-Copy buttons are annotated `(spec via stdin — use Copy spec JSON)` in tree mode; a **Copy spec JSON** button sits beside them (gated by the same completeness check — `to_spec_json` of an unset placeholder is undefined; R0-r1 M4).
- **Conditional arm** per §0 (the 10-flag Hidden set + 2 Disabled), in `conditional::build_descriptor`.

### 2.3 Version-skew posture (brainstorm risk 4)

No runtime version probe. A too-old binary fails Validate with clap's unknown-argument stderr → the global strip (the parse contract). One cell pins the strip rendering for a non-envelope failure.

## 3. P3 (cuttable to v0.32.1)

- **"Edit as tree…"** button in archetype mode: runs `--emit-spec` with current params via the runner; exit 0 → `from_spec_json` → populate `TreeState`, switch mode; failure → stderr strip, stay. NEVER re-implements lowering.
- **POSIX pipeline copy** in tree mode: `printf '%s' '<json>' | mnemonic build-descriptor --spec - …` (posix_quote); Windows copy stays argv + separate JSON copy.

## 4. Drift gates (all skip-if-absent, the archetype_schema_mirror posture)

1. `tests/spec_nodes_mirror.rs`: `NODE_KIND_SPECS` vs the binary's `nodes` array — kind set+order, `payload_str` byte-equal; `spec_schema_version == 1` AND `supported_doc_schema_version == 1` refuse-loud.
2. `tests/tree_path_parity.rs` (THE keystone): 6 plants over 5 productions, **diagnostic CLASS pinned per plant (R0-r1 I2 — type/parse-class diagnostics localize to ROOT, probed; node-addressed classes only):** binary-arm child = `sigless_branch` (e.g. `or_d[1]` an `after`); andor arm = `sigless_branch` at `andor[1]` (recipe R0-r2 M1: `andor(pk, after, older)` — NOTE the naive `andor(pk, older, pk)` VALIDATES, probed); `thresh.subs[i]` = `schema_field` via a nested k>n `multi`; `wrap.sub` = `schema_field` via a short-hex `sha256` under `v:`; root = `sigless_branch` via a bare `older` root; `multi.keys[i]` = `secret_key` via a planted tprv — trees built via `TreeNode`, serialized, run through the REAL binary `--spec - --json`; every returned `node_path` must equal the GUI-computed path AND match a live `node_paths()` entry.
3. `tests/tree_round_trip.rs`: the 5 vendored archetype fixtures (immutability cycle-scoped — the live exit-0 leg is the staleness tether) `from→to` value-identical + exit-0 through the binary; plus a `TreeState` serde persistence round-trip + the xprv redaction cells.

## 5. Other tests

- Kind-switch preservation matrix cells (multi→thresh keeps k; pk→pkh keeps key; surplus-children flag renders + does not emit).
- Completeness gate cells (each incompleteness class blocks; complete tree unblocks).
- Mode-mutex cells: tree mode argv contains NO `--spec <path>` / `--archetype` / param flags / `--emit-spec` even with all of them stale-populated; switching back restores them.
- Validate flow kittest: build a sigless tree (complete), Validate → the offending node tints + message; edit the node → tint clears (the mutation chokepoint); fix → Validate → validate_ok summary renders.
- Migration cell: a pre-v0.32.0 `state.json` (no `tree` field) loads to `None`/non-tree mode (R0-r1 M6a); collapse/expand-does-not-clear cell (M6b); the wide-node projection-law cell (I1b); the next_id import-invariant cell (I3); the fail-soft strip cells (I5).
- Depth-cap cell; ComboBox kind-picker driven via kittest if the A2 pattern holds (state-level seam fallback acceptable — note which).
- Full suite (4 pinned binaries) + clippy clean.

## 6. Release

GUI MINOR v0.32.0: CHANGELOG; version bump + lock + README self-tag; full suite → push → CI green → tag → tag-build green; toolkit `scripts/install.sh:44` pin → v0.32.0; resolve nothing toolkit-side (no toolkit FOLLOWUP exists for this — the toolkit BRAINSTORM §5 GUI row + the engine FOLLOWUP companion note get a "node-tree builder SHIPPED" annotation in a toolkit follow-up commit, closing the LAST wizard layer).

---

## Fold log

- **R0 round 1 (YELLOW → folded, 2026-06-10; persisted at `design/agent-reports/gui-v0_32_0-node-tree-r0-r1-review.md`):** I1 the round-trip law restated correctly (to∘from identity on valid specs; from∘to = projection; to-idempotence) — the naive law was false for exactly the trees preserve-and-flag creates. I2 plant classes pinned per plant (node-addressed classes only; type-class localizes to root — probed). I3 the next_id import invariant (max_id+1 recompute, named helper + cell). I4 the mode model pinned: only the tree bit is stored, Generic/Archetype stays dropdown-derived — v0.31.0 provably unchanged + the clean migration leg. I5 the unmatched-path fail-soft rule restored + cells. I6 Validate does not write last_run (two surfaces). M1 argv[0]. M2 display_or helper (idiom, not mechanism). M3 structural-emptiness-only rationale. M4 Copy gated by completeness. M5 import version/wrapper checks. M6 migration + collapse cells.
- **R0 round 2 (GREEN 0C/0I, 2026-06-10; persisted at `design/agent-reports/gui-v0_32_0-node-tree-r0-r2-review.md`):** all 12 round-1 folds verified (all six plant classes re-probed byte-exact). 3 minors folded: M1 the andor plant recipe pinned (`andor(pk, after, older)` — the naive variant VALIDATES); M2 selector gestures (clear-enabled-first; Generic resets the dropdown; never-destroys scoped); M3 Validate argv carries the user's `--allow` (verdict-relevant). **Gate satisfied.**
