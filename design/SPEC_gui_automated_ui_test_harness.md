# SPEC — mnemonic-gui automated UI-functionality test harness

**Status:** draft → R0 round-2. **Tier:** test-infra / quality-gate (no runtime/funds surface; self-custody secret-hygiene IS in scope).
**Source SHAs:** mnemonic-gui `master` (current). Companion: `design/GUI_TEST_HARNESS_CONSULT.md` (architect consult) + R0 round-1 (`design/agent-reports/gui-ui-test-harness-spec-r0-round-1.md`) — folded 6 Important + 5 Minor.

## 1. Goal & honest value

Automatically exercise **every UI element's functionality** without a human clicking each one,
targeting **functional correctness** ("the element does the right thing — right action/output,
not dead/wrong") and **state/conditional integrity** ("right fields show/enable; state never gets
stuck"). NO known-live bug to fix — `repeating-secret-flags-never-reach-argv` is verified RESOLVED
(v0.31.1). The value is:
1. **Coverage** — ~47 of the **61** subcommands have no full-flow (render→argv) test today.
2. **A routing/redaction regression net** for *classified* secrets (see I3 — corrected; NOT a new
   unclassified-secret detector).
3. **A regression net** so already-fixed bugs (and any the sweep finds) can't silently return.

Scope: 4 CLI tabs / **61 subcommands** (mnemonic 32 + ms 10 + mk 9 + md 10).

## 2. Why this is feasible now (verified TRUE against source in R0 r1)
- **egui 0.31 + `egui_kittest` 0.31** dev-dep, used across ~14 render-driving test files.
- **Headless CI proven:** those kittest tests run under `cargo test --workspace` on `ubuntu-latest`
  in **assertion mode** (AccessKit tree, no GPU) via `schema-mirror.yml`. Inherit it; do NOT enable
  the `wgpu` snapshot feature.
- **Schema-driven UI:** enumerable from `(cli_tab, subcommand, flag)` via `src/schema/` +
  `src/form/widget.rs::render_with_dispatch`. Seam:
  `assemble_argv(schema, sub, form_state) → runner::run_with_stdin → RunResult{argv,exit_code,stdout,stderr}`.
- **Secret routing separate by design:** `FormState::secret_widgets` `#[serde(skip)]` + per-row
  `Zeroizing` (type-level never-persist). Tree `key`/`keys` instead persist-then-redact via
  `redact_for_persistence`. Respect both; do not "fix" them.

## 3. Widget surface (corrected — IMP-2)
**9 `FlagKind` variants** (`src/schema/mod.rs:142-167`): `Text`, `Number{min,max}`, `Dropdown(opts)`,
`Boolean`, `Range`, `Timestamp`, `NodeValueComposite(opts)`, `TaggedOrIndexed(opts)`,
`Path{stdio_sentinel}`. **`SlotEditor` and `Tree` are NOT FlagKinds** — they are separate `FormState`
surfaces (`slots`, `tree`), not enumerable from `(tab,sub,flag)`; they get **dedicated
hand-authored cells**, not the per-flag enumeration.
- **Identity-mapped kinds** (value-in == value-in-argv): `Text`, `Number`, `Dropdown`, `Boolean`,
  `Path`. Candidates for the enumerated I1 round-trip — gated by the §6 P0 spike.
- **Transform kinds** (`Range`, `Timestamp`, `NodeValueComposite`, `TaggedOrIndexed`) + the
  Slot/Tree surfaces: **hand-authored cells** with explicit expected argv (no enumerated identity
  assertion).

## 4. The layered oracle (anti-tautology core — verified non-tautological in R0 r1)
Three **distinct, non-overlapping** oracles, never conflated:
- **Flag NAMES** → already owned by `schema_mirror` (real clap vs schema). The new harness MUST NOT
  re-assert names (the tautology trap).
- **WIRING** → `egui_kittest` **identity round-trip** for identity-mapped kinds (§3): inject a
  distinguishable value through the RENDERED widget, run to stable, `assemble_argv`, assert the
  value appears bound to its flag.
- **FUNCTIONAL** → a small set of **real pinned-CLI** end-to-end cells (the only non-circular oracle
  for "the command does the right thing").

## 5. Invariant classes

### I1 — Form→argv wiring round-trip (render-via-kittest MANDATORY)
Render via kittest, inject a distinguishable value **through the widget**, run to stable,
`assemble_argv`, assert the value is argv-bound to its flag. A pure `assemble_argv(hand-built
state)` test is **structurally BLIND** (the author re-implements the render→store wiring under
test). **Injection discipline (IMP-3):** the flag **under test** must be widget-injected via
kittest; a hand-seeded base `FormState` is permitted ONLY for *context* flags needed to reach a
valid form — never for the value being round-trip-asserted. Boundary: render→store seam = kittest;
strictly downstream of a populated store = the existing `argv_assembler*.rs` cells.

### I2 — Conditional & state integrity (per the 6 Visibility effects — IMP-5)
`Visibility` (`mod.rs:249-270`) has **6** effects with DIFFERENT argv semantics — the invariants
must respect each, not a blanket "disabled ⇒ suppressed":
| Effect | Renderer | Argv |
|---|---|---|
| `Visible` / `Required` | normal (Required adds a red `*`) | EMITS |
| `Hidden` / `Disabled` | hidden / greyed | **SUPPRESSED** |
| `PinValue{v}` | read-only, tooltip | EMITS **replaced** with `v` |
| `DisableOptions{vs}` | options greyed, non-selectable | **schema-time only — does NOT affect argv** (stale residual still emits; CLI rows 10/11 catch it) |

- **Renderer-applies-the-rule:** the rendered enable/visibility state of each flag equals
  `conditional(state)`'s effect for it — checked **per-effect** via the AccessKit tree. (Catches
  renderer↔rule **desync**; does NOT catch a *wrong rule* — that stays with humans /
  `conditional_visibility.rs`. Stated as a limitation.)
- **Value-suppression (fenced to `Hidden`|`Disabled` ONLY):** a value entered then Hidden/Disabled
  must not reach argv — universalized across subcommands. **Do NOT** assert suppression for
  `PinValue` (emits-replaced) or `DisableOptions` (emits-stale-by-design) — that would false-red CI
  on documented-correct behavior.
- **Toggle round-trip (no stuck state) — equivalence DEFINED (IMP-6):** toggling a gating input
  on→off→on returns the form to the same **visibility-state** (the `conditional(state)` projection),
  NOT the same value-state — toggles may legitimately destroy/suppress values. The invariant is over
  the visibility projection only.
- **Scope note:** only **17/61** subcommands declare a `conditional()`; I2 applies to those.
  `conditional()` purity (pure fn of state) is a cheap unit check, not a headline.

### I3 — Classified-secret persistence regression net (CORRECTED — IMP-1; NOT a co-headline class-catcher)
**Honest scope:** the sweep iterates `secret==true` flags only, so it **cannot** catch an
*unclassified* secret rendering as a normal Text widget (the actual v0.31.1 leak class) — that class
is already owned by `tests/schema_mirror_secret_drift.rs` + `secret_taxonomy_pin.rs`, which this
harness **does NOT replace**. I3's real value is a **regression net**: for every *classified* secret
flag × subcommand, inject a FAKE fixture via the real widget, drive the persistence walk **including
`redact_for_persistence`** (tree `key`/`keys` persist-then-redact — assert post-redaction, not just
`serde(skip)`), and assert the fixture is ABSENT from persisted state AND from export/preview
surfaces (masked argv in the confirm modal, `--spec -` stdin). Harness hygiene: FAKE fixtures only;
**never dump the AccessKit tree / widget state on failure** (the undo-ring can hold plaintext) — emit
flag/subcommand coordinates only; respect `Zeroizing`.

### I4 — Functional correctness (real pinned-CLI cells)
A curated, SMALL set of happy-path e2e cells per CLI: drive form → Run against the **pinned** binary
(`MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN`) → assert exit-code + `--json` parses + key fields. The
expensive integration oracle; not exhaustive.

## 6. P0 FEASIBILITY SPIKE (de-risks the universal I1 — IMP-4)
**Before** committing the enumerated I1 to all identity-mapped kinds, a P0 spike MUST prove
`egui_kittest` 0.31 can drive each non-Text identity kind through REAL interaction:
`Dropdown` (ComboBox popup open + option select), `Number` (DragValue set), `Boolean` (toggle),
`Path` (text + sentinel). Only `type_text` into `TextEdit` is proven in-tree today;
`archetype_form.rs:238-247` deliberately substitutes state-mutation for selection — evidence the
non-Text driving is unproven. **Spike outcomes:** kinds the spike proves drivable → enumerated I1;
kinds it cannot → hand-authored cells (explicit, narrower) + a documented gap. If the spike shows a
kind is undrivable AND no hand-cell substitute is sound, that kind's enumerated coverage is descoped
(logged, not silently dropped). The spike is the gate on the "universal" claim's reach.

## 7. Generation strategy & scope realism
Constraints (mutual-exclusion, requires-X) live in `conditional()`, not declarative metadata, so a
naive "fill all flags" generator emits invalid states. Therefore: **hand-seed a minimal VALID base
state per subcommand** (O(flags)-ish; ~800–1500 lines across 61 — NOT ~200/O(1)); sweep/property
code varies only LEAF values atop a valid base, with the under-test value widget-injected (§5 I1).
- **Permanent CI gate = deterministic, table-driven** (fixed cases, no proptest randomness).
- **One-time sweep = proptest** (broad leaf variation + toggle sequences) → file + fix findings →
  the deterministic table absorbs regression cells.

## 8. Two phases (sweep-now + permanent gate — both)
- **Phase 0 — the §6 spike** (gates I1 reach).
- **Phase 1 — one-time sweep (coverage bug-finder):** enumerator + I1 (spike-approved kinds) + I2 +
  I3 as proptest over the 61 subcommands (esp. the ~47 with no full-flow cell). Triage → FOLLOWUPs →
  fix. Honest: a coverage sweep, not a known-bug fix.
- **Phase 2 — permanent CI gate:** deterministic table-driven (I1/I2/I3 + curated I4), wired into the
  existing headless `cargo test --workspace` path. No new CI infra; no `wgpu` feature.

## 9. What it catches / does NOT catch
- **Catches:** mis-wired/dead elements (value entered → wrong/no argv), render↔conditional-rule
  desync (per-effect), stuck visibility-state across toggles, Hidden/Disabled values leaking to argv,
  classified-secret material leaking to persisted/export surfaces (regression), gross functional
  breakage vs the real CLI (I4).
- **Does NOT catch:** an *unclassified* secret (owned by the drift/taxonomy gates — I3 does not
  replace them), a *wrong* conditional rule (rule+renderer agree but rule is wrong), wrong CLI
  semantics (the CLI's own tests), and anything visual/UX (out of scope; separate human/snapshot
  track).

## 10. Determinism / hygiene (corrected — IMP-minor)
The real flake vector is the **multi-frame settle**, not RNG: drive then **run-to-stable** (step
until the AccessKit tree + form-state quiesce) before asserting — NOT a fixed frame count, NOT "RNG
seeds." Inherit `schema-mirror.yml`'s headless assertion path. Secret fixtures fake; failure output
coordinates-only (no state dump). `MNEMONIC_BIN` et al. pin the I4 binaries to the schema-mirror pin.

## 11. Non-goals
Visual/layout/snapshot testing (no `wgpu`); crash-fuzzing as an end (no-panic is a side benefit);
re-proving flag names (owned by `schema_mirror`); detecting *unclassified* secrets (owned by the
drift/taxonomy gates); testing the CLI binaries' own correctness.

## 12. Companion / tracking
FOLLOWUP `gui-automated-ui-functionality-harness` (mnemonic-gui). Phase-1 sweep findings → own
FOLLOWUPs + fixes. Consult: `design/GUI_TEST_HARNESS_CONSULT.md`; R0 r1: `design/agent-reports/gui-ui-test-harness-spec-r0-round-1.md`.
