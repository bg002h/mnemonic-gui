# Architect consult — automated UI testing of `mnemonic-gui`

**Question:** judge the proposed "C→A" schema-driven invariant harness (vertical
slice on the form→argv round-trip, proving-red a known bug, then expanding to all
subcommands + conditional/state invariants + a CI gate).

**Verdict (up front):** **PURSUE-WITH-CHANGES.** The altitude is right and the tool
(schema enumeration over `egui_kittest`) is right — but the proposal's *anchor* is
based on a stale fact, it under-counts scope by ~5×, and three of the six design
questions turn on distinctions the spec must get exactly right or the gate becomes
either tautological or blind. Details below.

---

## Load-bearing facts I verified in-repo (not from the brief)

1. **The cited "live bug" is RESOLVED and pinned GREEN — the vertical-slice premise
   is false as written.** `repeating-secret-flags-never-reach-argv` is marked
   **resolved `mnemonic-gui-v0.31.1` (2026-06-10)** in `FOLLOWUPS.md:141`. The fix
   already shipped, and `tests/repeating_secret_rows.rs` already drives the **real**
   widget through `egui_kittest` and passes:
   `cell_live_path_import_wallet_two_typed_ms1_rows_reach_argv_in_row_order` is
   GREEN today (`cargo test --test repeating_secret_rows` → `8 passed`). You cannot
   "point the engine at the known bug and prove it red, then fix, then green" — the
   bug is gone and the canonical kittest cell for it exists. The brief's "OPEN" tag
   is stale.
2. **Much of the proposed harness already exists, hand-written.** 14 test files
   already drive real renderers through `egui_kittest::Harness`
   (`widget_interaction.rs`, `repeating_secret_rows.rs`, `kittest_import_wallet_form.rs`,
   `repeating_rows.rs`, `xpub_search_widgets.rs`, `tree_form.rs`, `archetype_form.rs`,
   `greyout_stdin_toggles_v0_37_0.rs`, …). `conditional_visibility.rs` enumerates 11+
   conditional constraints; `argv_assembler*.rs` cover the assembler in isolation;
   `argv_assembler_disabled_suppression.rs` + `argv_assembler_visibility.rs` already
   gate hidden/disabled-value suppression. **The novel contribution is not the
   harness mechanism — it is ENUMERATION (auto-cover all subcommands) + a UNIVERSAL
   secret-never-persist sweep.** Frame the spec honestly around that.
3. **Scope is ~61 subcommands, not "40+":** mnemonic 32 + ms 10 + mk 9 + md 10.
4. **Headless CI is already proven.** The 14 kittest files run under
   `cargo test --workspace` on `ubuntu-latest` in `.github/workflows/schema-mirror.yml`
   with no xvfb/GPU. The suite uses assertion-mode kittest (the AccessKit tree), not
   the `wgpu` snapshot/`.png` backend. The new sweep inherits a proven-headless path —
   **do NOT enable egui_kittest's `wgpu` snapshot feature** (that one needs a GPU).
5. **Secret routing is, by design, a SEPARATE store.** Post-v0.31.1, secret Text
   flags render into `state.secret_widgets` (`BTreeMap<String, Vec<SecretLineEdit>>`,
   `#[serde(skip)]`, per-row `Zeroizing`); the assembler's secret branch is
   *kind-gated* to read from there (`widget.rs:30,38,74`; `invocation.rs:262-336`).
   This is load-bearing for never-persist — the harness must respect it, not "fix" it.

---

## The six design questions

### Q1 — Sound + the right tool?
Yes to both. Schema enumeration over `egui_kittest` is the correct altitude: every
widget is addressable from `(cli_tab, subcommand, flag, kind)`, the assertion seam
(`assemble_argv` → `RunResult`) is clean, and the subprocess is env-var-mockable.
A general invariant sweep is strictly more valuable than more hand cells. **But the
"C→A vertical slice that proves a live bug red" is built on a dead bug** (fact 1).
Replace that anchor — see Refinement 1. This is why the verdict is WITH-CHANGES, not
AS-IS.

### Q2 — Does form→argv REQUIRE render-via-kittest, or can it be `assemble_argv(hand-built FormState)`? (the crux)
**Render-via-kittest is MANDATORY. A pure `assemble_argv(hand-built state)` test is
structurally blind to this entire bug class — provably.** The bug was *wiring*: the
widget wrote to `secret_widgets`; the old assembler read `state.values`. A test
author building a `FormState` by hand puts the value where *they* think it goes. If
they mirror the assembler's read (put it in `state.values`), the assembler test
passes — which is **exactly** what happened: the pre-v0.31.1 "masking" cells
synthesized `state.values` directly (e.g. `cell_import_wallet_repeating_ms1_argv`)
and stayed GREEN *while the live form emitted nothing*. `repeating_secret_rows.rs`
says this in its own header: "The pre-v0.31.1 cells masked the bug by synthesizing
`state.values` entries directly (assembler-half only)."

A hand-built `FormState` is the test author *re-implementing the wiring under test* —
it checks the assembler against the author's mental model, never against the
renderer's actual writes. The only way to catch a render→store→assemble mismatch is
to let the **real renderer populate the store** (type into the real widget via
kittest) and assemble from *that* state.

**The boundary, stated precisely:**
- **Spans the render→store seam** ("where does the widget WRITE?") → **needs kittest.**
  The form→argv ROUND-TRIP, to be non-vacuous, MUST begin at a real keystroke
  (kittest `focus()` + `type_text()` / click / select-option) and end at argv,
  reading back the renderer-populated `state` after `harness.run()`.
- **Strictly downstream of a *given* store** (`assemble_argv(state) → argv`,
  `is_at_default`, kind→token emission) → **logic-only.** Keep these in
  `argv_assembler*.rs`; do not pay kittest cost for them.

So the sweep's form→argv cells are kittest cells whose *input-driving step* is the
load-bearing part — and it is per-`FlagKind` nontrivial (Text=type, Dropdown=select
by AccessKit role, Boolean=click, Number=type, Range/Timestamp/composite/TaggedOrIndexed
= harder). That is also where the "~200 lines / O(1) per subcommand" estimate breaks
(Q6).

### Q3 — The functional-correctness ORACLE (tautology risk). LAYER it; don't conflate three things.
The round-trip property (*choose X, set X via the real UI, assert token `--flag X`
in argv*) is the **strongest non-tautological wiring invariant available**, because X
is a free variable chosen by the test, not derived from the mapping under test. It
cross-checks three independently-authored links (render-write → store → assembler-emit)
without re-encoding any single one. That is genuinely non-tautological *for the
wiring* — it is what found the real bug.

What the round trip does **NOT** catch, and the correct oracle for each:
- **Wrong flag NAME in the schema** (schema `--ms1` but CLI wants `--ms-1`): the round
  trip uses the schema name on *both* the set and expect side → tautological, passes.
  **Non-circular NAME oracle = the existing `schema_mirror` test vs the real pinned
  clap `gui-schema`/`--help`.** It already exists. The round trip must NOT try to
  re-prove names — that is the tautology trap.
- **Wrong value TRANSFORM** (UI "mainnet" → CLI "bitcoin"; off-by-one Number): if a
  transform exists, the "expected" must encode it → re-encoding risk. **Mitigation:
  restrict the generic property to IDENTITY-mapped kinds** (Text / Number / Dropdown
  passthrough) and assert the typed value appears **verbatim**; **exclude
  transform-bearing kinds** from the generic property and cover them as hand cells.
  The honest non-tautological core is *identity round-trip for identity-mapped kinds.*
- **Semantic correctness** ("does this element DO the right thing"): out of scope for
  any GUI-internal oracle. The only non-circular functional oracle is **the CLI
  itself** — a small set of **real-pinned-CLI cells** asserting `exit_code`/`stdout`.

**Layered oracle (write this into the spec):**
| Property | Oracle | Status |
|---|---|---|
| Flag NAME correct | real clap `gui-schema` (`schema_mirror`) | exists — reuse |
| WIRING (render→store→argv) | kittest identity round-trip, mock CLI | the new sweep |
| FUNCTIONAL (argv→right behavior) | real pinned CLI exit/stdout | a few curated cells |

### Q4 — The conditional/state ORACLE (circularity). Gate the RENDERER-applies-the-rule, not the rule-against-itself.
There are two distinct objects; conflating them is the trap.
- **The rule** (`conditional::export_wallet(state) → FlagVisibility`): asserting it
  returns map M for state S is fine *only if M is hand-authored from the SPEC* — that
  is `conditional_visibility.rs` today (non-circular). Generating M by *calling the
  fn* is fully circular and worthless.
- **The renderer applying the rule:** the meaningful, non-circular property is
  **"rendered visibility/enablement == `conditional(state)`"** — drive a state via
  kittest, query the AccessKit tree for which fields are present/enabled, assert it
  matches the fn. **Catches:** renderer ignores the rule, render/state desync (the real
  "wrong fields show/enable" class). **Cannot catch:** the rule being *wrong* (both
  sides are the same fn). Say so explicitly; don't oversell.

The genuinely NEW, fully non-circular invariants — make these the headline state-
integrity gate:
- **Purity:** `conditional(state)` is deterministic (same state → same map; no hidden
  hysteresis).
- **Toggle round-trip:** toggle A on then off returns visibility to baseline (catches
  "state stuck/inconsistent" directly — the metamorphic property).
- **Disabled/hidden value suppression:** a hidden/disabled field's stale value never
  reaches argv. Generalize the existing `argv_assembler_disabled_suppression.rs` /
  `argv_assembler_visibility.rs` to **all** subcommands — this is funds-safety-adjacent
  (a hidden field silently contributing to argv is exactly a "wrong output" bug).

### Q5 — Secret-bearing subtlety. Elevate, don't footnote.
The harness will type into `SecretLineEdit` (lands in `Zeroizing<Vec<u8>>` — good).
Hazards + obligations:
- **Use obviously-fake fixtures** (`"SECRET_FIXTURE_ms1_row0"`), never real seed
  phrases, so the input literals are harmless and greppable.
- **The never-persist invariant needs its OWN universal assertion** — do not assume
  it. After driving each secret flag, call `redact_for_persistence(state)` / serialize
  and assert **no fixture token appears in the bytes.** This already exists as *hand
  cells* (`secrets.rs::form_state_secret_widgets_never_serialized`,
  `persist_redaction_v0_34_0.rs::t1b`) — **the single highest-value thing the SWEEP
  adds is making it UNIVERSAL:** for every secret flag on every one of ~61 subcommands,
  type a fixture, serialize, assert-absent. The two leaks fixed *incidentally* in
  v0.31.1 (`xpub-search-inline-phrase-not-secret-classified`,
  `ms-repair-ms1-not-secret-classified`) were both "a secret field that wasn't
  classified as secret leaks into persistence" — a universal serialize-and-grep sweep
  catches that class **by construction**. Make this a co-headline invariant beside
  form→argv.
- **Don't dump harness/AccessKit state on failure.** egui's TextEdit undo ring holds
  plaintext `String` snapshots (documented caveat `gui-secret-buffer-allocator-residue`);
  assert on booleans / token-absence, never by printing `state`. Zeroize/drop
  `FormState` between cells.

### Q6 — Scope realism + CI fit.
- **"~200 lines" and "O(1) per subcommand" are both wrong.** The mechanism is ~200
  lines; the cost is the per-`FlagKind` **input drivers** + per-subcommand **valid
  seed states**. It is **O(flags) per subcommand**, because a naive "set every flag"
  generator produces INVALID states (mutually-exclusive flags, e.g. `bundle`'s
  `--template` xor `--descriptor`) for which the "expected argv" is ill-defined (clap
  would reject). The conflict constraints live in the `conditional()` fn, **not** in
  declarative schema metadata a generator can read — so constraining the generator with
  `conditional()` re-introduces circularity for the conditional invariant.
- **Recommended generation model:** hand-author a **minimal VALID base state per
  subcommand** (~61 seeds, a few lines each), then **property-VARY only leaf values**
  (Text content, Number within `min..max`, Dropdown choice) holding structure fixed.
  This keeps every generated state valid and the round-trip well-defined. Realistic
  size: **~800–1500 lines**, not 200. Still very worth it.
- **Determinism > proptest for the PERMANENT gate.** This project is flake-averse
  (mlock g4_a history). Randomized proptest + a render harness shrinks slowly and may
  not re-render identically (egui IDs/focus). **Use deterministic table-driven cells
  for the CI gate; reserve randomized proptest for the one-time SWEEP.**
- **Headless CI: confirmed working** (fact 4). Keep assertion-mode kittest; never the
  wgpu snapshot feature.

---

## Verdict

**PURSUE-WITH-CHANGES.** The 2–4 refinements that matter most:

1. **Kill the dead-bug anchor.** `repeating-secret-flags-never-reach-argv` is resolved
   (v0.31.1) and already pinned green by a real kittest cell — you cannot "prove it red
   then fix." Either (a) run the one-time SWEEP **first** as the genuine bug-finder
   (the ~47 subcommands with no full-flow cell are the live hunting ground — that
   one-time pass is where new wiring/leak bugs will actually surface), or (b) reframe
   the slice honestly as "lift the `repeating_secret_rows` pattern to a schema-enumerated
   sweep," with no pretense of re-finding a dead bug.
2. **Render-via-kittest is MANDATORY for form→argv; a pure `assemble_argv(hand-built
   state)` test is structurally blind to the whole class** (the author re-implements
   the wiring — exactly how the old masking cells stayed green while the live form
   emitted nothing). The invariant must start at a real keystroke and end at argv.
   Boundary: render→store seam = kittest; strictly-downstream-of-a-store = logic-only.
3. **Layer the oracle; never let the round trip re-prove flag NAMES** (tautology) —
   names are owned by `schema_mirror` vs real clap. The round trip proves WIRING via
   **identity** round-trip for identity-mapped kinds only; transform kinds are
   hand-celled; FUNCTIONAL correctness is proven only by the few **real-pinned-CLI**
   cells (exit/stdout), which are non-circular because the oracle is the CLI itself.
4. **Conditional gate = "renderer faithfully applies `conditional(state)`"** (catches
   render desync; cannot catch a wrong rule — say so). Add the truly non-circular
   metamorphic invariants: purity, toggle round-trip (no stuck state), and
   **disabled/hidden-value suppression generalized to all subcommands**.

Plus one elevation: make a **universal secret never-persist sweep** (type fake
fixture → serialize → assert-absent, for every secret flag on every subcommand) a
co-headline invariant — it catches the "unclassified secret leaks to disk" class by
construction, which is the bar a self-custody tool must hold.

Scope correction for the spec: ~61 subcommands; budget ~800–1500 lines; O(flags) per
subcommand via hand-seeded valid base states + leaf-value variation; deterministic
table-driven for the gate, proptest only for the one-time sweep; headless CI proven,
no wgpu snapshot feature.

*Advisory only — not implemented.*
