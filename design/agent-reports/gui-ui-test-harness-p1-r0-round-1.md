# R0 review — GUI UI-test harness, Phase P1 (round 1)

**Reviewer:** opus architect (mandatory per-phase R0 gate; adversarial; verified against source + live test runs)
**Scope:** branch `feat/ui-harness-p0-spike` @ `153aa27c` (P1) over `master`@`da47994`. Diff is **TESTS ONLY**
(`tests/spike_widget_drivers.rs` P0, `tests/ui_harness/mod.rs` P1 enumerator/seed/render/drive,
`tests/ui_harness_i1_roundtrip.rs` P1 I1 cells). No `src/` change.
**Authoritative:** `design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` P1; `design/SPEC_gui_automated_ui_test_harness.md` §4/§5/§6.

---

## Verdict: GREEN — 0 Critical / 0 Important

P1 ships a faithful, non-tautological I1 vertical slice. The isolation-render choice is **correct and, for robust
per-flag targeting, necessary** (ruling below). 2 Minor + 2 Nit, all forward-guidance for P5/P6; none gate-blocking.

### Gates (all re-run live, this session)
- `cargo test --test ui_harness_i1_roundtrip --test spike_widget_drivers --jobs 2` → **10 passed / 0 failed** (I1)
  + **6 passed / 0 failed** (P0 spike).
- `cargo clippy --all-targets -- -D warnings` → **exit 0, clean**.
- `cargo test --test schema_mirror --jobs 2` → **21 passed / 0 failed**.
- Broad `cargo test --jobs 2` → **563 passed, 0 failed** across all 60+ binaries (incl. `argv_assembler_slot`,
  which ran clean under `--jobs 2` — no linker OOM).
- `#![allow(dead_code)]` present (`tests/ui_harness/mod.rs:1`, plan m6). NO `src/` change. Diff touches `tests/` only
  (no broad `cargo fmt`; no existing-file churn) — matches the GUI no-fmt-CI-gate constraint.

---

## RULING on the isolation-render choice (primary judgment): ACCEPT — faithful, not a weakening

P1 renders the under-test flag **in isolation** (one widget) via `render_one_flag` → `render_with_dispatch`
(`tests/ui_harness/mod.rs:181-228`) rather than the whole subcommand form. I verified this exercises the **same
render→store→argv seam** the real form uses, and hides nothing in I1's scope:

1. **`render_one_flag` is byte-faithful to the real per-flag loop body.** It reproduces
   `src/main.rs:582-686`: `vis = sub.conditional.map(|f| f(state)).unwrap_or_default()` (identical),
   the `find(k==name) | Visible` lookup (identical to `visibility_of`), the `Hidden → skip`, the
   `DisableOptions` filter/filter_map/flatten extraction (identical), and the `add_enabled_ui(!Disabled, …)`
   wrap — then calls `render_with_dispatch(ui, tab, sub.name, flag, state, &disabled_options)` with the
   **identical** argument tuple `src/main.rs:677-684` passes.
2. **`render_with_dispatch` is self-contained per flag** (`src/form/widget.rs:214-223`): it reads
   `state.values[flag.name]` (or `default_flag_value_for_flag`), renders, and writes the local `value` back to
   `state.values`. `assemble_argv` reads **only** `FormState`. Each flag owns its own `state.values` entry; there
   is no shared mutable aliasing between distinct flags' scalar widgets. So a one-widget render fully exercises that
   flag's render→store wiring — rendering siblings cannot change *this* flag's store path.
3. **Isolation removes only out-of-scope concerns:** (a) cross-flag *conditional interaction* — explicitly P2/I2,
   and the plan already **mandates** the whole-form render there (`IMPLEMENTATION_PLAN…:47-48`, m3); (b) argv
   *ordering* — `assemble_argv` emits `--flag value` adjacently and the cells assert adjacency (`pos+1`), so order
   among siblings is irrelevant; (c) egui ID collisions — IDs are salted by `flag.name`
   (`src/form/widget.rs:567`), so cross-flag collisions cannot occur regardless, and within-flag repeating-row
   collisions are out of P1's scalar scope.
4. **Isolation is, in fact, necessary** for a robust per-flag handle: egui attaches no label↔input association, so a
   faithful whole-form render surfaces many same-`Role` nodes (`SpinButton`/`TextInput`/`ComboBox`/`CheckBox`) with
   no unique `get_by_role` match. The "Set", option-label, and checkbox locators the drive uses
   (`tests/ui_harness/mod.rs:280-332`) are only unambiguous because exactly one widget renders.

A render→store wiring bug in any of the seven slice flags **would** RED (verified per-cell below). The isolation
does not tautologize the guarantee.

---

## Anti-tautology — verified per cell (none passes if render→store wiring breaks)

The assertion substance is the **widget-produced value**, with the flag name used only as a locator; flag-NAME
parity stays owned by `schema_mirror`. The cell is additionally tied to the **enumerator**
(`tests/ui_harness_i1_roundtrip.rs:54-59`), so the under-test flag is proven identity-mapped/non-secret/non-repeating
from the schema, not hardcoded-asserted.

- **Number `--account` (strongest):** default `Unset` → "Set" → `Number(0)`, and `0` **is** the schema default
  (`default_value: Some("0")`, `src/schema/mnemonic.rs:3834`) ⇒ `is_at_default` suppresses it. So clicking "Set"
  alone yields **no** `--account` token → `position()` panics. GREEN **requires** the AccessKit `SetValue(4242)` to
  flow through the DragValue's `&mut i64` (`src/form/widget.rs:542-544`) into the write-back at
  `widget.rs:220-223`. Empirical proof of real wiring, not a store mutation in disguise (the test never touches
  `state_mut`). `4242 ∈ [0, 2147483647]`, distinct from default. ✓
- **Dropdown `--network` / `--language` / `--address-type`:** opts[0] = `mainnet`/`english`/`p2pkh`
  (`mnemonic.rs:29`, `ms.rs:17-28`, `mk.rs:331`); injected `regtest`/`korean`/`p2tr` are present and **distinct**
  from opts[0]. A no-op drive leaves `--network mainnet` (or absent) → `pos+1 != regtest` (or `position()` panic).
  Load-bearing. ✓
- **Text `--from` / Path `--out`:** under-test flag stripped + empty default; a broken type-drive yields absent or
  empty-value argv → panic or `pos+1` mismatch. ✓
- **Boolean `--json`:** explicit absent-before / present-after; a no-op flip fails the present-after assert. Proves
  toggle→presence-emission, not name validity. ✓

---

## Number `SetValue` through-path — independently re-verified

`render_with_dispatch` clones `state.values[--account]` into a local `value`; `render`→`render_row` binds
`egui::DragValue::new(n)` with `n: &mut i64` **into that local** (`widget.rs:542-544`); the pushed
`Event::AccessKitActionRequest{ SetValue, NumericValue(4242) }` (`mod.rs:304-313`) is consumed by the DragValue's
own action handler, writing through `n`; `render_with_dispatch` writes `value` back to `state.values`
(`widget.rs:220-223`); `assemble_argv` reads only `FormState`. Injecting `4242` and observing `--account 4242` in
argv is therefore proof of the real DragValue write path — **not** a `state_mut` bypass. ✓

---

## Enumerator + seed-table discipline — verified

- `identity_flags` (`mod.rs:106-115`) excludes secret via **`mnemonic_gui::secrets::flag_is_secret`** — the **same
  function** `render_with_dispatch`'s secret branches key on (`src/form/widget.rs:89,175`; `secrets.rs:151-153` =
  `flag.secret || SECRET_FLAG_NAMES.contains(name)`). It also excludes `flag.repeating` (matches the
  `render_repeating` route) and transform kinds (`identity_kind` → `None`). The `enumerator_excludes_secret_passphrase`
  + `enumerator_yields_only_identity_nonsecret_nonrepeating` tests pin this across all 4 CLIs.
- **Injection discipline (§5 IMP-3) holds:** `base_state` seeds CONTEXT only (`mnemonic addresses`: `--from`,
  `--network`; empty for the positional-input subs — `mod.rs:156-173`), and `assert_roundtrip` **unconditionally
  strips** the under-test flag (`base.values.retain(k != flag_name)` + `secret_widgets.remove`,
  `tests/ui_harness_i1_roundtrip.rs:63-65`) before driving. The asserted value is solely widget-injected, never
  seeded. ✓
- **Reusable by P5/P6:** the enumerator is generic over any `SubcommandSchema` and the sanity tests already walk all
  subs of all 4 CLIs. ✓ (One forward caveat on the *render helper* — Minor 1.)

## Slice adequacy — verified
≥1 sub per CLI (4 tabs, `slice_covers_all_five_identity_kinds` asserts `clis.len()==4`); all 5 identity kinds
covered (Text `--from`, Number `--account`, Dropdown `--network`/`--language`/`--address-type`, Boolean `--json`,
Path `--out`); all four chosen subs are `conditional: None` (`mnemonic.rs:4307`, `md.rs:593`, `ms.rs:562`,
`mk.rs:519`) and `allows_slots: false` — so the conditional gate cannot suppress the under-test flag and
`render_one_flag`'s omission of the slot/tree/archetype `continue`s is provably inert for the slice. ✓

---

## Critical
None.

## Important
None.

## Minor
- **M1 (forward-guidance for P5/P6):** `render_one_flag` reproduces the *visibility / disabled_options /
  add_enabled_ui* subset of the form loop but **not** the three mode `continue`s (`--slot`/`allows_slots`,
  tree-mode-suppressed, archetype-suppressed) nor the post-render archetype hook (`src/main.rs:624-647,690-709`).
  For P1's `conditional:None`, non-slot, non-`build-descriptor` slice this is inert. But P5's sweep extends the seed
  table to all 61 subcommands. If P5 reuses `render_one_flag` verbatim on `build-descriptor` / `allows_slots` subs,
  a seed state that activates archetype/tree mode would make `render_one_flag` render a flag the **real** form
  suppresses → a possible false-GREEN. P5 must either (a) ensure its seed table never activates a suppressing mode
  for the under-test flag, or (b) extend the render helper to replicate the mode `continue`s. Recommend a one-line
  note in P5's plan/seed-table docstring. (Not P1-blocking.)
- **M2 (precision):** the `identity_flags` doc (`mod.rs:99-103`) says it "Mirrors the *render dispatch's* own
  predicate." The dispatch routes to the secret path only for `flag_is_secret && (Text|Boolean)` kinds
  (`widget.rs:89,175`); the enumerator excludes **all** `flag_is_secret` flags regardless of kind. The enumerator is
  thus strictly *more* conservative — which is safe (excluding a flag can only under-cover, never false-GREEN; and
  the live secret set is Text/Boolean only, so no identity flag is actually lost today). Worth a half-sentence so a
  future reader doesn't "tighten" the enumerator to match the dispatch's kind-AND and accidentally route a
  secret-but-non-Text flag into `state.values`-backed I1.

## Nit
- **N1:** `render_one_flag`'s docstring cites `src/main.rs:624-710` (`mod.rs:179-180`), a range that *includes* the
  slot/tree/archetype `continue`s it does not implement (see M1). Narrow the citation to the
  vis/disabled_options/add_enabled_ui span (`~648-686`) to avoid implying drop-in parity for all subs.
- **N2:** `IdentityKind` derives `PartialOrd, Ord` (`mod.rs:68`) used only to build the `BTreeSet` ordering in
  `slice_covers_all_five_identity_kinds`. Fine as-is; noting it's load-bearing only for that one test so it isn't
  "simplified" away.

---

## Misleads-P2–P6 check
Only M1 (render-helper reuse on mode-bearing subcommands) could mislead the P5 sweep; captured above as
forward-guidance, not a P1 defect. The plan already draws the I1-wiring vs I2-conditional boundary
(`IMPLEMENTATION_PLAN…:47-48`), and P1 stays correctly on the I1 side.

**GREEN. No fold required to proceed to P2.** (M1/M2/N1/N2 are optional polish; if folded, re-run the two harness
binaries — no re-dispatch needed for doc-only edits.)
