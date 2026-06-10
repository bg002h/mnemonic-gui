# R0 review — SPEC_gui_v0_31_0_archetype_forms — round 2
**Verdict: GREEN** (0C/0I)

## Round-1 fold verification (C1, I1-I4, M1-M5)
- **C1 — RESOLVED.** §4 non-repeatable-key arm renders through the repeating widget when >1 row (add suppressed, `(exactly 1)`, removable); §6 carries the tiered→kofn switch cell. Arity divergence re-probed real; the fix is the only shape keeping render==argv.
- **I1 — RESOLVED.** Accounting closes at 18 (9 params + `--spec` + 8 mode-independent), name-for-name vs `BUILD_DESCRIPTOR_FLAGS` (`mnemonic.rs:3440-3647`).
- **I2 — RESOLVED, §5 consistent.** Lib seam pinned; declared-skip = host-loop name-set continue (NOT Hidden); undeclared→Hidden via conditional; no remaining conflation; `--spec` correctly in the name-set (Disabled does NOT render-skip).
- **I3 — RESOLVED.** FromRowCount bespoke-only; static keeps Static(20); ALL-rows semantics; FlagKind Copy-able (every field Copy; resolve takes self by value).
- **I4 — RESOLVED.** Skip-if-absent at `schema_mirror.rs:606-619` verified; CI exports MNEMONIC_BIN.
- **M1-M5 — RESOLVED** (conditional --spec Disabled unchanged; public SCHEMA route; scalars tooltip-only; Option<RepeatAnnotation> seam; mode-independent + C1 cells + param-ORDER comparison).

## Critical
None.
## Important
None.
## Minor
**m1 —** `FromRowCount` needs the `.max(1)` degenerate-range floor its sibling `FromSlotCount` documents (`mod.rs:148-154,181`); §6 clamp cell could add a 0-rows assertion.
**m2 —** the C1 arm needs two widget knobs; say `RepeatAnnotation` carries both the label AND the add-suppression bit (one seam). `render_repeating` is private (`widget.rs:155`) — `pub(crate)` promotion for archetype_form.rs.
**m3 —** §4's "min from the schema" conflates min_count with a value bound: the Number `min` comes from the static `FlagSchema` (clone the static entry, swap only `max`); the archetype `min` feeds ONLY the `(min N)` annotation.

## Empirical probes run
1. GUI local == 93902b9; only SPEC + r1 report untracked.
2. Live `--spec-schema` 5-archetype dump — transcription byte-faithful; arity divergence confirmed; ids match ARCHETYPES order; pairing holds (no counter-case).
3. 18-flag hand-count matches §3.
4. invocation.rs:160 (Hidden|Disabled argv suppression), main.rs:434-436 (Hidden render-skip), :431/:473 (SlotEditor precedent).
5. FlagKind Copy feasibility (mod.rs:114-139); resolve by value (:178).
6. schema_mirror.rs:606-619 + CI MNEMONIC_BIN export; build_descriptor_schema public-SCHEMA route.
7. widget.rs needs_help_icon :27-32; render_repeating private, unconditional add at :170.
8. install.sh GUI pin currently v0.30.0 — §7 follow-up valid.
