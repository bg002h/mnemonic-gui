# GUI-form-renders — Leg-1 P2 R0 review — round 1

**Scope:** Leg-1 P2 (the `gui-render` headless structural-form emit binary + the
shared egui-free `render_emit` core) of the GUI-form-renders cycle. Branch
`feat/gui-render-form-emit` @ `a358c03` (P2) + `481e8db` (P1-R0 docs); P1 @ `4718ac8`;
master untouched @ `01520a5`. Plan:
`mnemonic-toolkit/docs/manual-gui/design/IMPLEMENTATION_PLAN_generated_gui_form_renders.md`
(Leg-1 P2); SPEC §2 (render scope + ASCII example) + §6 (determinism / secret hygiene).

**Reviewer:** opus architect, adversarial, verified against source + live builds/runs/tests.

---

## VERDICT: GREEN — 0 Critical / 0 Important

The emit binary is correct, deterministic, and secret-safe. Secret hygiene is not
merely "masked" — it is *structurally unreachable*: the render's value column is never
sourced from form state. The headless `--no-default-features` build is egui-free and now
CI-guarded (the P1 Minor-1 follow-through landed exactly as recommended). The render
faithfully mirrors `main.rs`'s flag-grid gate. I re-ran every gate myself; no rubber-stamp.
GREEN, plainly. One Minor (a reviewed-intended `(required)`-marker over-statement on
at-least-one prompt groups) is recorded with a manual-prose recommendation — it does not
block, because it is the design the SPEC R0 explicitly ratified.

---

## Gate re-verification (run independently)

| Gate | Command | Result |
|---|---|---|
| Full suite | `cargo test -p mnemonic-gui --jobs 2` | **620 passed / 0 failed / 4 ignored** (607 P1 + 13 new) ✓ |
| New emit suite | `--test gui_render_emit` | **13 / 0 / 0** ✓ |
| Clippy (default) | `clippy -p mnemonic-gui --all-targets -- -D warnings` | exit 0, clean ✓ |
| Clippy (headless) | `clippy -p mnemonic-gui --no-default-features -- -D warnings` | exit 0, clean ✓ |
| Headless build | `build -p mnemonic-gui --no-default-features` | `Finished` ✓ |
| Headless closure egui-free | `cargo tree --no-default-features --edges normal \| grep -iE eframe\|egui\|wgpu\|winit` | **EMPTY** ✓ |
| Default `gui-render` build | `build --bin gui-render` | `Finished` ✓ |

The 620/0/4 claim is exact. Working tree left clean; master untouched; HEAD `481e8db`.

---

## SECRET-HYGIENE RULING — PASS (first-class bar met; stronger than "masked")

A leak here would be Critical. There is none, and the design is defense-in-depth:

1. **Values are never state-sourced.** `render_form_from_state` (`render_emit.rs:114-215`)
   reads `state` ONLY for visibility (`conditional(state)`), the tree/archetype mode
   predicates, and `state.slot_count()` (line 178). The flag/positional **value column**
   comes exclusively from `default_flag_value_for_flag(flag)` (schema constants),
   the `MASKED` sentinel, or `<pinned: …>` (a conditional-`PinValue`, the only one being
   `--account → 0`). It never touches `state.values` or `state.secret_widgets`. A secret
   value placed in a fixture is therefore *structurally unreachable for display* — a
   stronger guarantee than masking alone.
2. **Masking is gated on `flag_is_secret` and fires FIRST.** `flag_value_str`
   (`render_emit.rs:289-329`) returns `MASKED` for `is_secret && !is_secret_bool` BEFORE
   evaluating `PinValue` or any default — even though `default_value` is a non-secret schema
   constant anyway. Secret positionals mask in `positional_body` (`render_emit.rs:350`),
   also without reading state.
3. **Secret `*-stdin` Booleans correctly NOT masked.** They carry no payload; rendered as
   the disabled checkbox (`[ ] off [disabled]`, `render_emit.rs:249,273`) the GUI shows
   (`widget.rs:181-194`). Verified against `mnemonic bundle`: `--passphrase → <masked>`,
   `--passphrase-stdin → [ ] off [disabled]`. Mirrors the GUI exactly.
4. **No Debug/`{:?}`/panic secret path.** `json_plain`/`kind_label`/`join_opts` operate on
   schema strings + non-secret `PinValue` JSON only. `SecretLineEdit`'s manual `Debug`
   (len-only) is never invoked here.
5. **Coverage is exhaustive.** `every_secret_flag_renders_masked_never_cleartext`
   (`gui_render_emit.rs:124-168`) iterates `all_forms()` (all 61) × every flag + positional;
   the prefix-with-trailing-space line match plus `contains("-> <masked>")` is sound (the
   trailing space disambiguates `--ms1` from `--ms1-*`; full-width names still keep ≥2
   trailing pad spaces). Non-rendered/suppressed secrets cannot leak (no line → skip).

No code path emits a secret fixture value; the masking holds even for a hypothetical
non-empty secret because values are never state-sourced. **PASS.**

---

## RENDER-FAITHFULNESS ASSESSMENT — faithful (with one documented marker caveat)

Verified the emit against `main.rs`'s real render loop (`main.rs:592-895`) and against
live `gui-render --form` runs (`convert`, `bundle`, `inspect`, `build-descriptor`,
`mk inspect`, `ms inspect`):

- **Render order matches `main.rs` exactly:** header → (build-descriptor) mode-selector
  line (`main.rs:601` ↔ `render_emit.rs:141-152`) → flag grid → (`--archetype`) archetype
  param line (`main.rs:690` ↔ `render_emit.rs:169-171`) → SlotEditor line (`main.rs:718`
  ↔ `render_emit.rs:175-180`) → positionals (`main.rs:832` ↔ `render_emit.rs:183-188`) →
  tree-builder line (`main.rs:886` ↔ `render_emit.rs:191-193`) → `[ Run ]`. Both iterate
  `sub.flags` in declaration order.
- **Suppression/visibility gate is byte-faithful:** `is_render_suppressed`
  (`render_emit.rs:220-241`) reproduces `main.rs`'s three `continue`s (`--slot`-when-slots,
  tree-mode, archetype-mode) + the `Hidden → skip`, calling the SAME egui-free
  `mode_predicates` the PR-#24 harness reuses. Live `mnemonic bundle` confirms `--slot`
  suppressed and the SlotEditor placeholder present.
- **Conditional state is genuinely consulted:** `mnemonic convert` (default fixture) renders
  `--template … [disabled]`, `--path … [disabled]`, `--xpub-prefix [disabled]`, etc. — i.e.
  `conditional(FormState::default())`'s `Disabled` projections, not a static dump.
- **Bespoke sub-surfaces are single labeled placeholder lines** per SPEC §2 (slot editor /
  mode selector / archetype param form / tree builder), correctly OUT of the field-level
  faithfulness scope.
- **Kind column is schema-truthful and richer than the GUI tree** (`dropdown[opts]`,
  `path(stdio)`, `composite[…]`, `tagged-or-indexed[…]`). This is intended: the kind column
  is schema-derived and gated by P5's byte-diff (the plan explicitly routes path-vs-text
  there, not P3 — `Role::TextInput` collapses them in the kittest tree).

**The one caveat (Minor-1 below):** the `(required)` marker is sourced from
`flag.required || matches!(vis, Visibility::Required)` (`render_emit.rs:253`), i.e. it
includes the *conditional* `Visibility::Required`. The live GUI paints a `*` only for the
*static* `flag.required` (`widget.rs:118,143,295,682`); `Visibility::Required` has no
on-screen widget (`main.rs` consumes it only via `add_enabled_ui(!Disabled)` — Required is
not Disabled, so it is a visual no-op today). So for at-least-one prompt groups the emit
shows markers the GUI omits. This is the SPEC-R0-ratified design (see Minor-1), not a drift.

---

## BYTE-DETERMINISM RE-VERIFICATION — PASS

- `gui-render --form mnemonic bundle` twice → **byte-identical**.
- `--emit-all` into two independent dirs → **`diff -r` IDENTICAL**, both wrote **61** files.
- **Headless-build emit == default-build emit:** rebuilt `gui-render` under
  `--no-default-features`, re-emitted, `diff -r` vs the default-build emit → **BYTE-IDENTICAL**.
  This is what makes P5's `verify-examples-gui` regen-diff stable cross-machine.
- **Strictly ASCII, LF, no CR** across all 61: `grep -rlP "[^\x00-\x7F]"` → none;
  `grep -rl $'\r'` → none. The `every_form_…_is_ascii` test additionally bans box-drawing
  glyphs. No RNG/timestamp/`$PATH` input (only static schema + `FormState::default()`).

---

## Census + headless CI + shared-core checks

- **61, dynamically derived.** Live `--emit-all` = 61 = mnemonic 32 + md 10 + ms 10 + mk 9,
  and `emit_all_writes_one_file_per_subcommand` (`gui_render_emit.rs:228-252`) asserts
  `all_forms().len() == Σ schema_for(tab).subcommands.len()` (no hard-coded 61) + a per-form
  on-disk census. ✓
- **Headless CI job is correct + in-pattern** (`build.yml:32-60`): `checkout@v5` +
  `dtolnay/rust-toolchain@stable` (clippy) + `Swatinem/rust-cache@v2`, then
  `build -p mnemonic-gui --no-default-features` and `clippy --no-default-features -- -D
  warnings`. Standalone crate (no `[workspace]`), so the `-p`-less clippy line resolves to
  the package — matching the existing default `clippy` job's form. `--all-targets` is
  correctly OMITTED (the kittest test targets reference `egui::Ui` and only compile with
  `gui` on — exactly P1 Nit-1; with `--all-targets` the headless clippy would false-RED).
  This lands P1's Minor-1 follow-through verbatim. ✓
- **Shared core suitable for P3.** `render_form_from_state(tab, sub, state)` + `all_forms()`
  + `render_fixture` are the single source; P3 can assert `emit == render` over the SAME
  `render_fixture(tab, sub)` with no duplicated fixture/render logic. `FormState::default()`
  is a valid base for all 61 (plan R0-r2 confirmed: `sweep_candidate_bases`'s first element
  for every form); mode forms render their generic/default mode, documented in
  `fixtures.rs:8-14`. ✓

---

## Critical

None.

## Important

None.

## Minor / Nit

- **Minor-1 — the `(required)` marker is conditional-sourced, so at-least-one prompt groups
  render as all-`(required)` though the GUI paints no `*` and the toolkit accepts any one.**
  `render_emit.rs:253` ORs in `Visibility::Required`. For `mnemonic inspect`/`repair`/
  `verify-bundle`, `three_way_card_at_least_one` (`conditional.rs:929-938`) marks all of
  `--ms1`/`--mk1`/`--md1` Required in the default (0-set) fixture (their static `required`
  is `false` — verified in `INSPECT_FLAGS`), and the pinned snapshot
  (`gui_render_emit.rs:37-39`) shows all three `(required)`. A manual reader could read that
  conjunctively ("supply all three") when the semantic is "at least one." **This is the
  reviewed/intended design** — SPEC R0-r1 (`…spec-r0-round-1.md:44`) states the gate-level
  `required` attribute is the one "which `conditional()` does drive," and the plan
  deliberately routes the required-marker out of P3's tree gate (not AccessKit-recoverable)
  to P5's byte-diff. So it is not a defect to block on. **Recommendation (non-blocking):** add
  a one-line caveat in the manual prose adjacent to these renders (or a fixture note) that a
  group of `(required)` cards rendered together denotes an at-least-one/exactly-one prompt,
  not a conjunction — otherwise the documented render slightly over-constrains. (Same family
  fires for any "neither-chosen → both Required" conditional, e.g. `--from-policy`/`--context`,
  `--phrase`/`--hex`.) Caught here because, per the plan, markers are covered by *only* this
  binary + the byte-diff gate — nothing downstream compares them to the live GUI.

- **Nit-1 — latent `DisableOptions` divergence (dead today).** `flag_body` emits the
  `[disabled-options: …]` suffix only when the *primary* `visibility_of` is `DisableOptions`
  (`render_emit.rs:276`), whereas `main.rs:657-666` collects disabled-options *orthogonally*
  across ALL conditional entries for a flag. If a future conditional re-introduces
  `DisableOptions` as a SECONDARY entry behind a primary `Required`/`Disabled`, the emit would
  drop it. **Moot today:** the only `DisableOptions` producer (conditional rows 10/11) was
  reverted in v0.7.2 (`conditional.rs:257`), so no form emits it in any state — confirmed by
  `grep "disabled-options"` over all 61 emitted files (NONE). If `DisableOptions` ever
  returns, mirror `main.rs`'s orthogonal collection here. No action this cycle.

- **Nit-2 — the archetype/tree placeholder branches are unreachable under the canonical
  fixture.** `[ archetype param form ]` (`render_emit.rs:170`) and `[ descriptor tree
  builder ]` (`render_emit.rs:192`) only fire for non-default `build-descriptor` states; the
  generic fixture never exercises them (the committed render shows
  `mode selector: generic`). The code reads correctly and is needed by
  `render_form_from_state` for P3/future per-mode fixtures; just flagging that the committed
  corpus does not cover them. No action.

---

## Bottom line

P2 meets its mandate: a deterministic, ASCII-only, headless-buildable emit binary whose
render faithfully mirrors the GUI flag-grid gate, whose secrets are structurally
unreachable (not just masked), whose count is schema-derived (61), and whose load-bearing
`--no-default-features` gate is now CI-guarded (P1 Minor-1 closed). 620/0/4 == claim; both
clippy gates clean; byte-determinism holds across runs, dirs, and feature-builds.
**GREEN, 0C/0I — proceed to P3.** Track Minor-1 as a manual-prose caveat (at-least-one
groups read conjunctively in the render) and Nit-1 as a comment-or-fix if `DisableOptions`
is ever reintroduced.
