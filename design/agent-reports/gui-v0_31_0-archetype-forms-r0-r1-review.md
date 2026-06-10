# R0 review — SPEC_gui_v0_31_0_archetype_forms — round 1
**Verdict: YELLOW**

The central mechanism survives adversarial verification: `Visibility::Hidden` exists (`src/schema/mod.rs:199-204`), the render loop skips it (`src/main.rs:434-436`), and `assemble_argv` suppresses it (`src/form/invocation.rs:160,176`). The §1 kofn transcription is byte-faithful to the v0.52.0 binary (probed), the threshold↔key pairing is sound against all 5 archetypes, and §7 is correct. But there is one design gap in the §3/§4 interaction (arity divergence across archetypes meets data preservation meets the all-rows assembler) that produces invisible argv emission, plus a cluster of mechanism ambiguities the implementer would have to guess at.

## Critical

**C1 — Declared-but-non-repeatable key params: surplus rows from another archetype emit invisibly.** The underlying `FlagSchema`s for `--key`/`--recovery-key` are `repeating: true` (`src/schema/mnemonic.rs:3486,3510` — they must be, for clap Append), but the archetypes diverge in arity: `--key` is non-repeatable in `hashlock-gated`/`simple-timelocked-inheritance`, and `--recovery-key` is non-repeatable in `kofn-recovery`/`hashlock-gated`/`simple-timelocked-inheritance` (probed). Walk the SPEC's rules: user enters 2 `--recovery-key` rows in `tiered-recovery`, switches to `kofn-recovery`. §3 preserves both rows; §5 does NOT hide `--recovery-key` (declared); §4's "key (non-repeatable) → single Text row" renders exactly one row (the scalar `position()` lookup finds the FIRST); but `assemble_argv` keys repeating emission off the **static** `FlagSchema.repeating` and emits EVERY matching row (`invocation.rs:255-258`). Result: the GUI shows one recovery key, argv carries two, the toolkit refuses, and the user cannot see why. The §5 argv cells as specified cannot catch this. Fix: render archetype-non-repeatable keys through the repeating-row widget whenever >1 row exists (rows visible + removable, "+ add" suppressed, an "(exactly 1)" annotation), so what emits is always what renders; plus an argv cell for the tiered→kofn switch.

## Important

**I1 — The §3 flag accounting omits `--spec-schema`.** build-descriptor has **18** flags; §3 enumerates 17. Assign `--spec-schema` a mode (presumably mode-independent status quo). Decide and state it.

**I2 — The §6 kittest is not implementable with the SPEC's stated hosting.** main.rs is the **binary** — integration kittests can only drive library fns. The mode-switch dispatch must be lib-hosted (e.g. `pub fn` in `src/form/` consumed by main.rs). Without this the kittest re-implements main.rs logic and pins nothing — the masking class the v0.30.0 cycle filed. Specify the lib seam. Related: the generic-loop skip for DECLARED params **must not** be `Visibility::Hidden` (Hidden suppresses argv — declared params must emit); it has to be a name-set `continue` in the host loop. §5's wording reads dangerously close to "the conditional handles it."

**I3 — §4's threshold-max resolve: wiring site and semantics unspecified.** Verified: `NumberMax::resolve(&FormState)` (`mod.rs:178-183`) consumed at render (`widget.rs:417`) — a new variant carrying the paired key flag needs no new state threading; clamp-on-render is real (egui DragValue `clamp_existing_to_range` default true). Missing decisions: (a) does the new variant replace `Static(20)` in the static entries (changes generic-mode behavior) or live only in a bespoke-synthesized `FlagSchema`? Note `FlagKind` derives nothing — synthesizing kinds needs a Clone/Copy derive or inline construction; (b) row-count semantics: counting ALL rows (incl. empty) lets max exceed the emitted key count — acceptable (CLI is the gate) but state it.

**I4 — §2's "the `schema_mirror` resolve_bin pattern" names the wrong precedent.** The main mirror gate fails LOUD when the binary is missing; the const-vs-binary parity tests use a skip-if-absent discipline (`schema_mirror.rs:606-619`). The new gate is a const-mirror-vs-binary parity test — pin it to the `:608` skip pattern explicitly (CI still runs it: the workflow exports MNEMONIC_BIN), or declare fail-loud; decide.

## Minor

**M1 —** keep the conditional returning `Disabled` for `--spec` (cell_13 pins exactly that); render suppression at the host loop; say the conditional is unchanged for `--spec`.
**M2 —** `ARCHETYPES`/`BUILD_DESCRIPTOR_FLAGS` are module-private; the §1 units reach them via the public `SCHEMA` route or `pub(crate)`. Note the sentinel exclusion.
**M3 —** `needs_help_icon` grants icons only to Dropdown/Composite/Tagged/repeating — scalar params are tooltip-only today; accept status quo or extend; adjust the claim.
**M4 —** the "(min N)" header annotation needs a named seam (signature extension on the repeating header, default None) so the implementer doesn't fork the widget.
**M5 —** add: a mode-independent-flags-visible cell; the C1 switch cell; the §2 gate compares per-archetype param ORDER explicitly (render order is load-bearing).

## Citation audit
- §1 kofn transcription — VERIFIED byte-for-byte. Kind vocabulary + wire keys VERIFIED (toolkit `descriptor_builder/schema.rs:62,75`).
- §3 SlotEditor precedent (`main.rs:430-431/:473`) — VERIFIED. §8 `widget.rs:102` FromSlotCount comment — VERIFIED (resolve at `:417`).
- Hidden suppression (`invocation.rs:160,176`) + render-skip (`main.rs:434-436`) — VERIFIED.
- §2 "resolve_bin pattern" — IMPRECISE (I4). §7 no-pin-change premise — VERIFIED. §4 bounds — VERIFIED.
- Unstated: 18 flags incl. `--spec-schema` (I1).

## Empirical probes run
1. GUI local == 93902b9 (tag v0.30.0); SPEC the only untracked file.
2. Binary 0.52.0 `--spec-schema` full 5-archetype dump (decaying 8 / hashlock 4 / kofn 4 / simple 3 / tiered 5) — fidelity + pairing + the C1 arity-divergence finding (every declared threshold pairs with a repeatable key in the same archetype; no counter-case).
3. egui 0.31.1 DragValue clamp default true.
4. conditional_visibility cells 12-15 use vis_of lookups — §5's Hidden entries won't break them.
5. gui_schema_conditional_drift skips empty-rule subcommands — no drift-gate delta.
