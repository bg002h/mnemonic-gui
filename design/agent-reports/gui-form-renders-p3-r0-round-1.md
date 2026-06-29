# GUI-form-renders — Leg-1 P3 R0 review — round 1

**Scope:** Leg-1 P3 (the egui_kittest FAITHFULNESS gate + the shared ASCII/projection
core refactor of `render_emit.rs`) of the GUI-form-renders cycle. Branch
`feat/gui-render-form-emit` @ `99dc48a` (P3); P2 `a358c03`; P1 `4718ac8`; P1-docs `481e8db`;
master untouched @ `01520a5`. Plan Leg-1/P3; SPEC §2 (render scope) + §3 (faithfulness
anchor) + §6 (determinism / secret hygiene). Tests run `--jobs 2` (local linker-OOM bound).

**Reviewer:** opus architect, adversarial. Every claim verified against source + live
`gui-render` runs + the real egui_kittest crate source + a teeth re-run. Branch left clean.

---

## VERDICT: RED — 0 Critical / 1 Important

The faithfulness gate itself is sound, has real teeth (103-divergence proof reproduced
exactly), is secret-safe, covers all 61 forms, and all release gates are green. But the
implementer's escalated divergence is a genuine **manual-fidelity defect**, and the correct
resolution is **Option A** (seed → conditional → re-gate disabled), which is an **Important**:
it reopens P2's pinned ASCII and re-gates the disabled axis in P3. Per the gate standard, an
open Important blocks GREEN. The implementer was RIGHT to escalate rather than paper it.

---

## THE DISABLED-AXIS RULING: **(A)** — the emit must seed flag defaults into FormState
## before evaluating `conditional()`. This is an **Important** (revise P2 ASCII + re-gate P3).

### Why A, empirically

The emit is **internally self-inconsistent today**, and it depicts screens the user never
sees. Verified live:

```
$ gui-render --form mnemonic bundle
  --template              dropdown[bip44,…]  (required) -> bip44      ← value column: bip44 (single-sig)
  --multisig-path-family  dropdown[bip48,bip87]  -> bip48              ← ENABLED (no [disabled])
  --threshold             number  -> <unset>                          ← ENABLED (no [disabled])

$ gui-render --form mnemonic export-wallet
  --template    …  (required) -> bip44
  --descriptor  text  (required) -> <empty>                           ← marked REQUIRED + ENABLED
  --threshold / --multisig-path-family                                ← ENABLED
```

The VALUE column already shows `--template -> bip44` (the schema default, via
`default_flag_value_for_flag`, `render_emit.rs:517`). bip44 ∈ `SINGLE_SIG_TEMPLATES`
(`conditional.rs:30`). A single-sig template has no threshold and no multisig path family —
yet the emit shows those flags editable. **The emit's own value column contradicts its own
disabled column.** A manual reader sees "template=bip44 AND fill in threshold/path-family,"
which is incoherent.

**Root cause** (confirmed by reading the real production paths): the emit evaluates
`sub.conditional(state)` over the UNSEEDED `FormState::default()` (`render_emit.rs:146`,
`fixtures.rs:26-29`), where `dropdown_value("--template")` returns `None`, so
`template_is_single_sig` is false (`conditional.rs:199, 253-256`). The GUI, however,
**auto-seeds** every non-secret non-repeating flag's default into `state.values` on the FIRST
render frame via `render_with_dispatch`'s write-back (`widget.rs:220-229`:
`None => state.values.push((flag.name, default_flag_value_for_flag(flag)))`). On the next
frame `conditional` reads `--template = bip44` and greys the multisig-only flags. The egui
event loop runs this to a fixed point in ~1 frame at 60fps, so **the user sees the greyed
state on load** (≈16 ms). This is PRODUCTION behavior, not a kittest artifact:
`egui_kittest::Harness::from_builder` calls `harness.run_ok()` in its constructor
(`egui_kittest-0.31.1/src/lib.rs:133` "Run the harness until it is stable") — kittest merely
reproduces the real event loop's settle. So the (B)-justifying "settling-artifact" hypothesis
is FALSE; verified at the crate source.

The manual's entire raison d'être is to depict the screen the user actually sees. Today the
ASCII pins a screen the user never sees. That is a fidelity bug in the OUTPUT, not merely a
gating gap — so (B) (keep the carve-out) leaves the manual permanently wrong. → **Rule A.**

### The precise seeding model the emit MUST adopt (so it matches the GUI EXACTLY)

The question "does the GUI auto-seed ALL flag defaults, or only some kinds?" — traced through
`widget.rs` + `main.rs`:

- **Non-secret, non-repeating flags: ALL kinds are seeded** with
  `default_flag_value_for_flag(flag)` (`widget.rs:220-229`) — `Dropdown(default)`,
  `Text(default|"")`, `Path(default|"")`, `Boolean(false)`, empty `NodeValueComposite`, and
  **`Unset`** for Number/Range/Timestamp/TaggedOrIndexed. (For conditional purposes only the
  *non-empty* Dropdown/Text/Path seeds are load-bearing: `flag_value_is_present(Unset)=false`,
  empty Text/Path and `Boolean(false)` also read absent — `schema/mod.rs:496-510`. So the
  numeric/empty seeds are conditional-neutral; seeding them is harmless but the mirror should
  match the GUI to stay obviously-faithful.)
- **Required REPEATING non-secret flags: ONE row seeded** with the default
  (`render_repeating`, `widget.rs:310-315`). **Optional repeating flags seed NOTHING.** This
  matters: `convert --to` is a required-repeating Dropdown that auto-seeds `Dropdown("phrase")`,
  which `convert`'s conditional reads — so the model is load-bearing beyond template.
- **Secret flags are NEVER seeded into `state.values`** (Text → `secret_widgets`,
  `widget.rs:95-169`; `*-stdin` Boolean → early return, `widget.rs:181-194`). A conditional
  reading `has_value` on a secret flag must keep seeing it absent.
- **Mode-suppressed (`--slot`/tree/archetype) and conditional-`Hidden` flags are NOT rendered,
  hence NOT seeded** (`main.rs` `continue`s; mirrored by `is_render_suppressed`,
  `render_emit.rs:436-457`).
- The GUI applies this **per frame with `conditional` re-evaluated at the top**, and
  `state.values` is **monotone** (a flag that later goes Hidden keeps its already-stored value).
  The steady state is therefore the **monotone fixed point** of "seed every currently-rendered
  non-secret flag → re-evaluate `conditional` → seed any newly-revealed flag → repeat." It
  converges in ≤ |flags| iterations.

So the emit must adopt an **egui-free seed simulator** that is the render-gated, secret-
excluding, optional-repeating-excluding fixed point — NOT a blanket "seed every flag once."
A blanket seed would OVER-seed (Hidden / secret / optional-repeating flags) and a one-shot seed
would UNDER-converge; either fabricates NEW divergences (exactly the trap the prompt warns of).
Concretely: replace the conditional-input state in `form_elements`/`project_form`/
`render_form_from_state` with `seeded_fixture(tab, sub)` = that fixed point. The displayed
value column is unchanged (it already shows the same defaults) — only the `conditional`
*input* and thus the `[disabled]` column move into agreement with the value column.

### What A changes, exactly (the P2 reopen)

The 6 disabled-axis flags that gain `[disabled]` (= the implementer's count, verified against
the conditional bodies):

| sub | flags gaining `[disabled]` under seeded bip44 | source |
|---|---|---|
| `bundle` | `--threshold`, `--multisig-path-family` | `conditional.rs:253-256` |
| `verify-bundle` | `--threshold` | `conditional.rs:414-416` |
| `export-wallet` | `--descriptor`, `--threshold`, `--multisig-path-family` | `conditional.rs:600, 620-622` |

Bonus: A also drops the spurious `export-wallet` `(required)` markers on `--template` /
`--descriptor` (under seeded `has_template`, `conditional.rs:605` no longer fires) — i.e. A
partially closes **P2 Minor-1**'s over-statement for the auto-seeded (non-secret) flags. (The
at-least-one secret-card groups — `inspect`/`repair`/`verify-bundle` `--ms1`/`--mk1`/`--md1` —
stay Required under A, correctly, since secret flags are not auto-seeded.)

### How P3 re-gates the disabled axis (and why this self-guards the seed-sim)

Re-gate exactly as `ui_harness_i2_conditional` already reads disabled: for each present flag,
compare the emit's `FlagProjection.disabled` against the SETTLED whole-form harness's
flag-name-LABEL node `is_disabled()` (`ui_harness/mod.rs:549` `label_disabled`). Both sides now
reflect the seeded steady state, so they agree. Critically, **this re-gate is the anti-tautology
guard on the seed-sim itself**: if the emit's seed produces a different `conditional` outcome
than the real settled GUI, the disabled axis diverges → RED. So A's added mirror surface is
self-policing — the main risk of A (seed-sim drifting from `widget.rs`) is caught by the very
gate A enables. That removes the only serious objection to A.

### A also removes a LATENT fragility in the current presence gate

Today presence agrees across all 61 forms only because, for these fixtures, the `Hidden` set
happens to be identical under `conditional(unseeded)` and `conditional(seeded)`. That is an
empirical coincidence, not an invariant. A future conditional that HIDES a flag only under a
seeded value (e.g. hide `--foo` when `template=bip44`) would make the settled harness drop it
while the unseeded emit still shows it → the presence gate REDs on a NON-bug (the emit
faithfully drawing an unseeded screen the user never sees). Under A both sides are seeded, so
presence is robust too. Another reason A is the right architecture, not just a patch.

---

## Faithfulness-test SOUNDNESS / anti-tautology — VALID (teeth reproduced)

- **Side (1) is the REAL render, read off AccessKit — not a 2nd schema derivation.**
  `render_extended_form_harness` (`gui_render_faithfulness.rs:127-140`) renders the production
  `render_whole_form` (real `render_with_dispatch`) + `render_positionals` + `render_action_bar`
  inside `egui_kittest::Harness`, then reads `accesskit::Role` / label nodes
  (`has_role`/`has_label`/`observe_control`, lines 160-204). Control-class is read from ISOLATED
  single-flag renders (`render_flag_harness`) because the whole form surfaces many same-Role
  widgets with no per-flag handle — a sound, documented constraint inherited from PR #24.
- **Teeth proof reproduced EXACTLY.** I temporarily mutated `control_class`'s
  `FlagKind::Boolean` arm to `TextInput` (`render_emit.rs:366`) and re-ran: the gate fired
  **103 divergences**, every one `real control=CheckBox, emit control=TextInput` — i.e. side (1)
  reads `Role::CheckBox` off the rendered tree wholly independently of the emit's prediction.
  Reverted; `grep` confirms the arm is back to `CheckBox` and `git status` is clean. **The gate
  is not a tautology.**
- **`render_positionals` / `render_action_bar` are byte-faithful to `main.rs`.** The positional
  label format `"{} {}{}"` (name / `*` / `...`), the secret→`SecretLineEdit::show` branch, and
  the non-secret `ui.label` + `text_edit_singleline(&mut state.positionals[i])` path
  (`gui_render_faithfulness.rs:74-106`) mirror `main.rs:832-879` verbatim (it omits only the
  multi-row +/✕ chrome, which is irrelevant to presence/masking). `render_action_bar`
  (`add_enabled(true, Button::new("Run"))`) mirrors `main.rs:1023`'s `add_enabled(run_enabled,
  Button::new("Run"))`.

---

## m4 carve-outs are legitimately covered elsewhere (NOT coverage holes) — with ONE exception

- **`(required)` marker** — not AccessKit-recoverable (`Visibility::Required` paints no widget;
  `main.rs` consumes it only via `add_enabled_ui(!Disabled)`); covered by the P2 `gui_render_emit`
  ASCII snapshots + P5 byte-diff. Legit (P2 Minor-1 already on record). ✓
- **path-vs-text** — `Role::TextInput` collapses `Path`/`Text` in AccessKit; covered by the
  schema-truthful kind column under P5's byte-diff. ✓
- **default/placeholder TEXT** — the value column; covered by P5 byte-diff. ✓
- **sub-surface INTERNALS** — single placeholder line per SPEC §2; field-level out of scope. ✓
- **disabled axis** — currently carved out, "covered indirectly" by `ui_harness_i2_conditional`
  + the P2 ASCII snapshots. **This is the exception**: `i2_conditional` verifies the renderer
  applies `conditional(state)` for a GIVEN seeded state — it does NOT verify that the emit's
  DEPICTED screen equals the user's on-load (auto-seeded) screen. So manual-OUTPUT fidelity on
  the disabled axis is genuinely uncovered, which is precisely the gap Ruling A closes.

---

## Census / secret hygiene / gates — all PASS

- **Census = 61.** `gui_render_faithfulness.rs:211-216` asserts `all_forms().len()==61` AND
  `covered==61` (`:347`); `all_forms()` is schema-derived (32+10+10+9), no hard-coded 61. ✓
- **Secret hygiene (first-class).** Fixture is `FormState::default()` — FAKE, no secret ever
  set. Divergences are COORDINATE-ONLY (`bin/sub/flag-name + coarse class`,
  `:252,287,316,334,339`); the AccessKit tree / form state are never dumped. Masking is checked
  by `Role::PasswordInput` presence (`:331`), never by reading a buffer. ✓
- **Gates (re-run independently):**

| Gate | Command | Result |
|---|---|---|
| Full suite | `cargo test -p mnemonic-gui --jobs 2` | **621 / 0 / 4** ✓ (matches claim) |
| Faithfulness | `--test gui_render_faithfulness` | **1 / 0 / 0** ✓ |
| Clippy default | `clippy --all-targets -- -D warnings` | exit **0** ✓ |
| Clippy headless | `clippy --no-default-features -- -D warnings` | exit **0** ✓ |
| Headless build | `build -p mnemonic-gui --no-default-features` | `Finished` ✓ |
| `gui-render` default | `build --bin gui-render` | exit **0** ✓ |
| `gui-render` headless | `build --bin gui-render --no-default-features` | exit **0** ✓ |
| fmt / mlock | diff vs `01520a5` touches no `mlock`/mass-reformat | clean ✓ |

(Note: `gui-render` is a BINARY, not a feature — there is no `gui-render` feature in
`Cargo.toml`; the binary is egui-free and builds under both default and `--no-default-features`,
which is the load-bearing headless path. `--features gui-render` would (correctly) error.)

---

## Critical

None.

## Important

- **I1 — Adopt Ruling A: seed flag defaults into the conditional-input state, then re-gate the
  disabled axis. (Reopens P2 + extends P3.)** The emit currently depicts on-load screens the
  user never sees (`bundle`/`verify-bundle`/`export-wallet`: value column shows the seeded
  single-sig `--template -> bip44` while the disabled column behaves as if no template were
  chosen, leaving `--threshold`/`--multisig-path-family`/`--descriptor` falsely enabled — 6
  flags / 3 subs, table above). The GUI auto-seeds defaults on its first frame
  (`widget.rs:220-229`) and `egui_kittest` reproduces that via the constructor's `run_ok()`
  (`egui_kittest-0.31.1/src/lib.rs:133`), so this is the REAL screen, not a settling artifact —
  (B) is not defensible for a fidelity-mandated manual. **Required work:** (a) add an egui-free
  `seeded_fixture` = the render-gated monotone fixed point defined above (NOT a blanket seed —
  must exclude secret + optional-repeating + suppressed/Hidden flags, seed required-repeating's
  first row, mirror `default_flag_value_for_flag`); feed it as the `conditional` input in
  `form_elements`/`project_form`/`render_form_from_state`; (b) regenerate + re-pin the P2
  `gui_render_emit` ASCII (the 6 flags gain `[disabled]`; `export-wallet` loses the spurious
  `--template`/`--descriptor` `(required)` markers — folding part of P2 Minor-1); (c) re-gate
  the disabled axis in P3 by comparing `FlagProjection.disabled` against the settled harness's
  per-flag label `is_disabled()` (`ui_harness/mod.rs:549`) — this self-guards the new seed-sim.
  Re-run the FULL `-p` suite afterward (CLI/schema/ASCII lints ripple beyond the one test).

  **Sub-finding (correct as part of I1): the gate's own comment misdescribes egui_kittest.**
  `gui_render_faithfulness.rs:227-248` claims "We read the CONSTRUCTION frame (no `run()`), NOT
  a settled multi-frame state … `new_ui_state` likewise renders one frame … so the disabled axis
  compares like-for-like." This is FALSE and **directly contradicts the sibling comment at
  `:264-282`** ("egui_kittest's `Harness` instead always settles (`run_ok`)…"), which is the
  correct one. The harness settles in its constructor; the test already reads the post-auto-seed
  steady state. The gate is unaffected (presence/control are settle-stable or settle-invariant),
  but the false rationale must be removed when I1 lands, or it will mis-guide the seed-sim work.

## Minor / Nit

- **Minor-1 — the action-bar check is effectively always-true on both sides.** `render_action_bar`
  hard-codes `add_enabled(true, …)` and the gate asserts `has_label("Run")`; a button renders its
  label whether enabled or disabled, and `form_elements` always emits `Run`, so both sides are
  unconditionally true for all 61 forms. It verifies "a Run button exists" (non-zero value) but
  the `run_enabled` mirror of `main.rs:1023` is never actually exercised. Harmless; if a future
  fixture wants the enabled/greyed Run state gated, compute `run_enabled` rather than `true`.
- **Nit-1 — `render_one_positional` uses `or_insert_with(|| vec![SecretLineEdit::new()])` while
  `main.rs:838` uses the same seed but the test omits the per-row ✕ / "+ add" repeating chrome.**
  Intentional and documented (presence/masking only); noted for completeness.

---

## Bottom line

The P3 gate is well-built: real-AccessKit ground truth, 103-divergence teeth reproduced exactly,
single shared `form_elements` core (ASCII ⟷ projection cannot drift), 61-form census,
coordinate-only secret hygiene, all release gates green (621/0/4, both clippy clean, headless
builds). The implementer correctly escalated the disabled divergence instead of papering it —
and the correct resolution is **Ruling A**: the emit must seed flag defaults before evaluating
`conditional`, because the auto-seed is the REAL on-load screen (verified at the egui_kittest
crate source), the current emit is self-contradictory (value column vs disabled column), and A
both fixes manual fidelity AND yields a self-guarding disabled re-gate. That is an **Important**
(reopen P2 ASCII + extend P3 + delete the contradictory settling comment). **RED — 0C / 1I.**
Fold I1 → re-pin P2 → re-gate P3 → re-dispatch this review to convergence before proceeding.
