# SPEC — GUI v0.31.0: data-driven archetype forms (the wizard, archetype-forms stage)

**Status:** R0 GREEN (round 2, 0C/0I; 3 minors folded) — implementation may begin
**Source grounding verified at:** mnemonic-gui `origin/master` = `93902b9` (tag `mnemonic-gui-v0.30.0`); toolkit pin stays `mnemonic-toolkit-v0.52.0` (NO pin change — the `archetypes` schema section shipped in v0.51.0 and the pin is already past it)
**Parent:** the long-deferred GUI wizard ("archetype forms first, recursive node-tree builder deferred" — toolkit `design/BRAINSTORM_descriptor_builder.md` §5 GUI row); architect direction-consult (stream A2: **data-driven, not hand-forms**); toolkit presets SPEC §5 (the `archetypes` section is "the contract a GUI archetype-forms wizard consumes").

## 0. Scope & design stance

When the build-descriptor form's `--archetype` dropdown selects a preset, the generic flag grid is REPLACED (for the param flags) by a **schema-driven archetype param form**: only the selected archetype's parameters render, each with a kind-appropriate widget, required/min annotations, and a live summary line. Deselecting (`"(none)"`) restores the v0.30.0 generic form. argv assembly is UNCHANGED (the param form edits the same `state.values` rows the generic widgets edit — same flags, same assembler).

- **Data-driven, with a static mirror + drift gate (the GUI's standing idiom).** A new GUI-side static table mirrors the toolkit's `archetypes` field-specs; a new **`archetype_schema_mirror`** integration test runs the pinned binary's `build-descriptor --spec-schema` and compares the `archetypes` section field-by-field (the `schema_mirror` pattern — NOT runtime-fetched schema; the GUI never shells out at render time). Hand-maintained per-archetype forms would re-create exactly the drift surface the toolkit eliminated by generating its section from the registry.
- **The node-tree builder stays deferred** (the dominant GUI cost center; out of scope).
- **No toolkit pin change; no `schema_mirror` delta** (zero new flags — this cycle consumes surfaces that already exist).

**Non-goals:** consuming `--json` build output in-GUI (run-modal flow unchanged); the `--emit-spec` preview beyond what the existing Boolean flag already provides (kept as the generic checkbox — visible in archetype mode); fixing `repeating-secret-flags-never-reach-argv` (open FOLLOWUP); any toolkit change.

## 1. The static mirror — `src/schema/archetypes.rs` (new)

```rust
pub struct ArchetypeParamSpec {
    pub flag: &'static str,      // "--key"
    pub kind: &'static str,      // "key" | "threshold" | "blocks" | "absolute_locktime" | "hex_digest"
    pub required: bool,
    pub repeatable: bool,
    pub min: u32,                // wire key "min" (toolkit min_count)
}
pub struct ArchetypeSpec {
    pub id: &'static str,        // == ARCHETYPES const value (minus the "" sentinel)
    pub summary: &'static str,   // toolkit registry summary, shown under the dropdown
    pub params: &'static [ArchetypeParamSpec],
}
pub const ARCHETYPE_SPECS: &[ArchetypeSpec] = &[ /* 5 entries, toolkit order */ ];
```

Transcribed from the v0.52.0 binary's `--spec-schema` `archetypes` section (probed; e.g. kofn-recovery: `--key` key/required/repeatable/min 2; `--threshold` threshold/required/min 1; `--recovery-key` key/required/NON-repeatable/min 1; `--older` blocks/required/min 1). Self-consistency unit test: every `ARCHETYPE_SPECS` id ∈ `ARCHETYPES` (sentinel `""` excluded) and every `flag` ∈ `BUILD_DESCRIPTOR_FLAGS` names — reached via the public `SCHEMA` route (the `tests/build_descriptor_schema.rs` helper pattern) or `pub(crate)` promotion (R0-r1 M2).

## 2. The drift gate — `tests/archetype_schema_mirror.rs` (new)

`MNEMONIC_BIN`-gated with the **skip-if-absent discipline** (the const-vs-binary parity pattern at `tests/schema_mirror.rs:606-619` — eprintln + return when no binary; CI still runs it because the workflow exports `MNEMONIC_BIN`; R0-r1 I4 — NOT the main mirror gate's fail-loud): run `build-descriptor --spec-schema`, parse `archetypes`, assert **field-by-field equality** with `ARCHETYPE_SPECS` — ids (set + order), per-param `flag`/`kind`/`required`/`repeatable`/`min` **in declared param ORDER (load-bearing — §3 renders in schema order; R0-r1 M5c)**, and `summary`. Any toolkit-side preset change (new archetype, param change, summary reword) fails the gate at the next pin bump — the lagging-gate posture `schema_mirror` already established, with the paired-PR rule as the leading discipline.

## 3. The archetype param form (render layer)

**Lib seam (R0-r1 I2):** the archetype form lives in a NEW library module `src/form/archetype_form.rs` exposing `pub fn render(ui, tab, subcommand, state, …)` (and a `pub fn active_archetype(state) -> Option<&ArchetypeSpec>` predicate); `src/main.rs` only dispatches (the SlotEditor precedent at `main.rs:431/:473`) — so kittests drive the REAL form, not a re-implementation. **The generic-loop skip for DECLARED params is a name-set `continue` in the host loop (NOT `Visibility::Hidden` — Hidden suppresses argv, and declared params must emit).** When the subcommand is `build-descriptor` AND `--archetype` has a non-empty value matching an `ARCHETYPE_SPECS` id:

- **Suppress** the generic widgets for the 9 param flags + `--spec` (the conditional's `--spec` handling is UNCHANGED (still `Disabled` — cell_13 pins it; R0-r1 M1); the render suppression is purely host-loop, replacing the disabled ghost row with a cleaner mode switch). The mode-independent flags (`--archetype` itself, `--emit-spec`, `--allow`, `--format`, `--network`, `--json`, `--spec-schema`, `--no-auto-repair` — 8 of the 18; R0-r1 I1) keep their generic widgets (full accounting: 9 params + `--spec` suppressed, 8 mode-independent).
- **Render** the selected archetype's `params` in schema order, each via the ParamKind→widget map (§4), with: the flag label, a required marker, a `(min N)` annotation for repeatable params, and the existing per-flag help affordance — NOTE (R0-r1 M3): `needs_help_icon` grants the `?` icon only to Dropdown/Composite/Tagged/repeating flags; scalar Text/Number params are tooltip-only today and STAY tooltip-only in the bespoke form (status quo accepted).
- **Summary line:** the archetype's `summary` renders directly under the dropdown.
- Params NOT in the selected archetype: their `state.values` rows are **left intact but not rendered** (so switching archetypes back and forth does not destroy entered data) — and since the toolkit refuses inapplicable params, the assembler must skip them: **in archetype mode, `assemble_argv` must emit only the selected archetype's param flags** (plus the mode-independent flags). Mechanism: the conditional fn (§5) marks inapplicable param flags `Hidden` — the assembler already suppresses `Hidden` flags (the established Disabled/Hidden suppression path) — keeping argv assembly declarative, no bespoke assembler branch.

## 4. ParamKind → widget map (5 arms, all reusing v0.30.0 machinery)

| kind | widget |
|---|---|
| `key` (repeatable) | the v0.30.0 repeating-row widget (Text rows, add/remove, header); the `(min N)` annotation renders in the header via a NAMED seam — an `Option<RepeatAnnotation>` parameter on the repeating-header render (default `None`, existing callers unchanged; R0-r1 M4) — **`RepeatAnnotation` carries BOTH the label and the add-suppression bit (one seam serves the `(min N)` and `(exactly 1)` arms; R0-r2 m2); `render_repeating` gets `pub(crate)` so `archetype_form.rs` can call it** |
| `key` (non-repeatable per the ARCHETYPE spec; the underlying FlagSchema stays `repeating: true` for clap) | single Text row when ≤1 row exists; **when >1 row exists (carried over from another archetype — R0-r1 C1), render through the repeating-row widget with "+ add" SUPPRESSED and an `(exactly 1)` annotation** — surplus rows stay visible and removable, because `assemble_argv` emits EVERY matching row off the static `FlagSchema.repeating`; hiding them would emit invisible argv (the GUI shows 1 recovery key, argv carries 2, the toolkit refuses opaquely). What emits is always what renders. |
| `threshold` | Number widget; the Number `min` comes from the STATIC `FlagSchema` (clone the static entry, swap only `max` — the archetype `min` is an occurrence COUNT and feeds ONLY the `(min N)` annotation; R0-r2 m3), **max bound to the LIVE key-row count** of the paired key param — a NEW `NumberMax::FromRowCount(&'static str)` variant (resolve reads `state.values` rows for the named flag, **floored `.max(1)` — the FromSlotCount degenerate-range discipline; R0-r2 m1**; consumed at the existing `max.resolve(state)` site — no new state threading). **Wiring (R0-r1 I3a): the variant is used ONLY in the bespoke-synthesized `FlagSchema` the archetype form constructs; the static `BUILD_DESCRIPTOR_FLAGS` entries keep `Static(20)` (generic-mode behavior unchanged). `FlagKind` gains `#[derive(Clone, Copy)]` to permit synthesis.** Row-count semantics (I3b): ALL rows count, including empty ones (an empty row emits nothing, so max can exceed the emitted key count — acceptable, the CLI gate validates; the §6 clamp-cell expectations use this rule). Clamp-on-render is the existing egui DragValue behavior (`clamp_existing_to_range` default). |
| `blocks` | Number widget, `min..=2_147_483_647` (the gate's `older < 2³¹`) |
| `absolute_locktime` | Number widget, `min..=4_294_967_295` |
| `hex_digest` | Text widget with a 64-hex live-validity hint (non-blocking — the toolkit gate is the validator; the hint is a `⚠ expected 64 hex chars` label, never an input block) |

Threshold↔key pairing is positional convention from the toolkit registry (the `--threshold` param applies to `--key`, `--recovery-threshold` to `--recovery-key`); encode the pairing as a static fn (`paired_key_flag(threshold_flag) -> &str`), unit-tested against `ARCHETYPE_SPECS` (every threshold param's archetype also declares its paired key param).

## 5. Conditional extension

`conditional::build_descriptor` grows archetype-mode awareness: when `--archetype` is a non-empty `ARCHETYPE_SPECS` id, mark each of the 9 param flags NOT declared by that archetype `Hidden` (declared params stay visible — rendered by §3's form; the generic loop skips them per §3 suppression). The existing archetype↔spec mutex is unchanged. Unit cells per archetype: exactly the declared params survive argv assembly (synthesize rows for ALL 9 params, select each archetype, assert only its params emit).

## 6. Tests

- `archetype_schema_mirror` (§2) — the new drift gate, GREEN at the current pin.
- Mirror self-consistency units (§1) + threshold-pairing unit (§4).
- Per-archetype argv cells (§5): all-9-params-populated → only the declared set emits; plus the kofn happy path end-to-end (state → argv == the toolkit preset goldens' argv shape).
- Kittest: select `kofn-recovery` → the param form renders (4 params, summary line, `(min 2)` on `--key`); deselect → generic form returns; entered `--key` rows survive an archetype round-trip (the §3 data-preservation claim).
- Threshold clamp cell: 3 key rows (incl. empty — the I3b ALL-rows rule) → threshold max 3; remove a row → max 2; ZERO rows → max 1 (the m1 floor); an over-max stored value clamps on next render.
- **C1 switch cell:** 2 `--recovery-key` rows entered under tiered-recovery → switch to kofn-recovery → BOTH rows render (repeating widget, add suppressed, `(exactly 1)` annotation) and argv carries both until the user removes one — what emits is what renders.
- Mode-independent visibility cell: `--allow` + `--emit-spec` (+ `--format`/`--network`/`--json`) still render in archetype mode (R0-r1 M5a).
- Full suite with the 4 pinned binaries + clippy clean.

## 7. Release

GUI MINOR v0.31.0: CHANGELOG `[0.31.0]`; version bump + lock; full suite → push → CI green → tag `mnemonic-gui-v0.31.0` → tag-build green. Toolkit follow-ups: `scripts/install.sh:44` GUI pin → v0.31.0 (checklist item); a toolkit FOLLOWUPS note is NOT needed (no toolkit surface consumed beyond the already-shipped schema section) — but the toolkit `descriptor-builder-engine` companion line's "eventual GUI wizard" reference may be annotated as shipped-at-archetype-form-level (node-tree builder still deferred). The `manual-gui` anchor debt continues to accrue against the existing FOLLOWUP (no new flags this cycle → no new anchor debt).

## 8. Source grounding (verified at `93902b9` / toolkit binary 0.52.0)

- `--spec-schema` `archetypes` section: probed (kofn transcribed in §1; 5 entries; wire keys `flag/kind/required/repeatable/min` + `id`/`summary`/`params`).
- `src/main.rs:431/:473` — the SlotEditor bespoke-surface branch (the §3 hosting precedent).
- `src/form/widget.rs` — the v0.30.0 repeating-row widget + `NumberMax::FromSlotCount` resolve (`:102` comment; the §4 generalization seam).
- `src/form/conditional.rs::build_descriptor` — the v0.30.0 mutex this cycle extends; `Hidden` suppression in `assemble_argv` (the established path).
- `src/schema/mnemonic.rs` — `ARCHETYPES`/`ALLOW_RULES`/`BUILD_DESCRIPTOR_FLAGS` (v0.30.0).
- `tests/schema_mirror.rs` — the resolve_bin + MNEMONIC_BIN gating pattern the new gate copies.

---

## Fold log

- **R0 round 1 (YELLOW → folded, 2026-06-09; persisted at `design/agent-reports/gui-v0_31_0-archetype-forms-r0-r1-review.md`):** C1 archetype-non-repeatable keys with surplus carried-over rows render through the repeating widget (add suppressed, `(exactly 1)`) — what emits is what renders; + the tiered→kofn switch cell. I1 `--spec-schema` assigned mode-independent (18-flag accounting closed). I2 lib seam pinned (`src/form/archetype_form.rs` pub render + predicate; main.rs dispatch-only; declared-param skip = host-loop name-set `continue`, NOT Hidden). I3 `NumberMax::FromRowCount` lives ONLY in bespoke-synthesized FlagSchemas (static entries keep Static(20)); FlagKind gains Clone/Copy; ALL-rows count semantics. I4 the drift gate uses the skip-if-absent parity discipline. M1 conditional `--spec` Disabled unchanged. M2 consts via the public SCHEMA route. M3 scalar params stay tooltip-only. M4 `(min N)` via an Option<RepeatAnnotation> header seam. M5 mode-independent-visibility cell + param-ORDER comparison in the gate.
- **R0 round 2 (GREEN 0C/0I, 2026-06-09; persisted at `design/agent-reports/gui-v0_31_0-archetype-forms-r0-r2-review.md`):** all 10 round-1 folds verified consistent across §3/§4/§5/§6 (the declared-skip vs Hidden separation is clean; the 18-flag accounting closes name-for-name). 3 minors folded: m1 FromRowCount floors `.max(1)` (+ zero-rows clamp assertion); m2 RepeatAnnotation carries label + add-suppression, `render_repeating` → pub(crate); m3 the Number `min` comes from the static FlagSchema — the archetype `min` is an occurrence count feeding only the annotation. **Gate satisfied.**
