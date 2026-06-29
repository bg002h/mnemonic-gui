# R0 review — SPEC_gui_automated_ui_test_harness.md (round 2)

**Reviewer:** opus architect (adversarial). **Gate:** 0 Critical / 0 Important.
**Target:** `design/SPEC_gui_automated_ui_test_harness.md` (R0 round-2 revision).
**Mandate:** confirm round-1 convergence (0C/6I/5m, all folded), hunt fold-introduced drift.
**Verified against:** live `mnemonic-gui master` source (not the spec's claims).

---

## Verdict

**GREEN — 0 Critical / 0 Important / 1 Minor.**

The six Important findings and five Minors from round-1 are **genuinely resolved**, not gestured.
I re-verified every load-bearing factual claim the folds rest on against current source — the 9
`FlagKind`s, the 6 `Visibility` effects and their exact argv semantics, the 17/61 conditional
count, and the tree persist-then-redact path — and they all check out. The folds introduced **no
new Critical/Important**, no dangling cross-refs, and no number drift. One carryover precision nit
remains (round-1 IMP-5's sub-point (a)), but the authoritative per-effect table the fold added
already states the correct behavior, so it is prose-tightening for the plan-doc, not a spec
correctness defect. **Converged. Proceed to plan-doc.**

---

## Source re-verification (the facts the folds assert)

| Claim in revised spec | Source | Result |
|---|---|---|
| 9 `FlagKind`: Text, Number{min,max}, Dropdown, Boolean, Range, Timestamp, NodeValueComposite, TaggedOrIndexed, Path{stdio_sentinel} | `src/schema/mod.rs:142-167` | **EXACT** ✓ |
| 6 `Visibility`: Visible, Hidden, Required, Disabled, PinValue, DisableOptions | `src/schema/mod.rs:249-270` | **EXACT** ✓ |
| Hidden/Disabled SUPPRESS; PinValue emits-REPLACED; DisableOptions no-argv-effect (stale still emits) | `src/form/invocation.rs:189` (`suppresses = matches!(v, Hidden \| Disabled)`), `:200-245`, doc `mod.rs:243-247` | **EXACT** ✓ |
| 17/61 subcommands declare `conditional()` | `grep "conditional: Some"` = 17 (mnemonic 12 + md 3 + ms 1 + mk 1) | ✓ |
| 61 = mnemonic 32 + ms 10 + mk 9 + md 10 | per-file `SubcommandSchema {` (the 2 surplus grep hits are `archetypes.rs`/`mod.rs` doc-structs, not CLI subs) | ✓ |
| Tree `key`/`keys` persist-then-redact (assert post-redaction, not serde-skip) | `form/tree_model.rs:180` `redacted_for_persistence` blanks xprv-shaped; `main.rs:368` maps `redact_for_persistence` over persisted state | ✓ |

The §5-I2 per-effect **argv column is accurate on every row**, including the two the prompt
flagged: PinValue = emits-replaced (✓ `invocation.rs:208-245`), DisableOptions = no-argv-effect /
stale-residual-emits (✓ `invocation.rs:181-189`, comment "does NOT join the suppress set").

---

## Per-finding convergence audit

**IMP-1 (I3 false "by construction" co-headline) — RESOLVED.** §1.2 now reads "regression net for
*classified* secrets (see I3 — corrected; NOT a new unclassified-secret detector)." §I3 header:
"NOT a co-headline class-catcher"; body honestly states the sweep "iterates `secret==true` flags
only, so it **cannot** catch an *unclassified* secret … that class is already owned by
`schema_mirror_secret_drift.rs` + `secret_taxonomy_pin.rs`, which this harness **does NOT
replace**." §9 ("Does NOT catch: an *unclassified* secret (owned by the drift/taxonomy gates — I3
does not replace them)") and §11 Non-goals agree. **No surviving sentence implies I3 catches the
unclassified class or replaces the drift gates.** Honest.

**IMP-2 (widget taxonomy) — RESOLVED.** §3 now lists the 9 real `FlagKind`s with the
`mod.rs:142-167` citation, correctly states SlotEditor/Tree are *not* FlagKinds (separate FormState
surfaces → dedicated hand-authored cells), and classifies Path as identity-mapped. Matches source
exactly.

**IMP-3 (§gen reintroduced I1 blindness) — RESOLVED.** §5 I1 "Injection discipline (IMP-3)": the
flag **under test** must be widget-injected via kittest; hand-seeded base state permitted ONLY for
*context* flags. §7 echoes ("the under-test value widget-injected (§5 I1)"). The render→store seam
vs downstream-of-store boundary is preserved.

**IMP-4 (non-Text injection unproven) — RESOLVED.** §6 P0 spike added, per-kind (Dropdown ComboBox
popup, Number DragValue, Boolean toggle, Path text+sentinel), with a clear pass/fail and an
explicit descope path ("undrivable AND no hand-cell substitute → enumerated coverage descoped —
logged, not silently dropped"). The rest of the spec treats universal-I1 as spike-conditional: §3
"Candidates … gated by the §6 spike", §8 Phase 0 = the spike (gates I1 reach), Phase 1 "I1
(spike-approved kinds)". Not asserted unconditionally anywhere.

**IMP-5 (I2 vs the Visibility effects) — RESOLVED for the load-bearing part (b); residual prose nit
on (a), see Minor.** Sub-point (b) — the false-red risk — is fully closed: the suppression property
is fenced to "`Hidden`|`Disabled` ONLY" mirroring `invocation.rs:189`, with explicit "Do NOT assert
suppression for PinValue (emits-replaced) or DisableOptions (emits-stale-by-design)." The per-effect
table is argv-accurate (verified above). Sub-point (a) (renderer-faithfulness AccessKit
queryability for PinValue/DisableOptions) is addressed by the table's Renderer column but the prose
bullet still says "checked per-effect via the AccessKit tree" — see m1.

**IMP-6 (toggle equivalence undefined) — RESOLVED.** §5 I2 defines it as **visibility-state**
equivalence (the `conditional(state)` projection), "NOT the same value-state — toggles may
legitimately destroy/suppress values." Well-defined and immune to legitimate value destruction.

**Minors — all RESOLVED.** MIN-1: §10 reframes the flake vector as multi-frame settle /
run-to-stable, "NOT 'RNG seeds.'" MIN-2: §5 scope note "only 17/61 … I2 applies to those" (count
verified). MIN-3: purity demoted to "a cheap unit check, not a headline." MIN-4: §I3 asserts
"through `redact_for_persistence` (tree `key`/`keys` persist-then-redact — assert post-redaction,
not just `serde(skip)`)" — matches source. MIN-5: §1 labels "~47" approximate with the tilde.

---

## Fold-introduced drift sweep (round-2 specific)

- **Number consistency:** "61" (§1, §1-scope, §7, §8), "17/61" (§5), "~47" (§1.1, §8) are mutually
  consistent — 17/61 (conditional declarations) and ~47/61 (no full-flow test) measure different
  things; no contradiction. No stray "10 widget kinds" / "5 effects" survivors from round-1.
- **Identity partition coherence:** §3 identity-mapped = {Text, Number, Dropdown, Boolean, Path}
  (5); §6 spike drives the 4 non-Text members (Text already proven); §5 I1 round-trips identity
  kinds. The three sections agree — no kind is both identity and transform, none orphaned.
- **I3 reframe introduced no new contradiction:** "classified" qualifier is applied consistently in
  §1/§9/§11; the harness-does-not-replace-the-drift-gates statement appears in §I3, §9, §11 without
  conflict.
- **No invariant both claimed and disclaimed:** the I2 renderer gate's "does NOT catch a *wrong
  rule*" and I3's "does NOT catch unclassified" are scope *limitations*, not self-contradictions.
- **Citations grounded:** `archetype_form.rs:238-247` (spike's "substitutes state-mutation for
  selection" evidence) and the `redact_for_persistence`/tree path are real and say what the spec
  claims.

No new Critical or Important introduced by the folds.

---

## Minor / Nit (non-blocking)

- **m1 (carryover of round-1 IMP-5(a), prose-precision).** §5 I2's renderer-faithfulness bullet
  still reads "the rendered enable/visibility state of each flag equals `conditional(state)`'s
  effect for it — **checked per-effect via the AccessKit tree.**" That phrasing is clean for
  `Hidden` (node absent) and `Disabled` (`.is_disabled()`), but loose for the other two: `PinValue`
  renders *read-only* (an AccessKit read-only-property query, unproven in-tree — not enable/
  visibility), and `DisableOptions` is a *per-option* greyout inside an unopened ComboBox popup (no
  per-*flag* tree node reflects it; it rides the §6 spike's popup-open primitive). The fold's own
  per-effect table already states these correctly (PinValue → "read-only, tooltip"; DisableOptions →
  "options greyed, non-selectable" / schema-time-only), so the table neutralizes the prose — no
  gate is wrong as a result. Recommend (plan-doc, not a spec blocker): scope the bullet to
  "`{Hidden, Disabled}` verified via the AccessKit node; PinValue read-only and DisableOptions
  per-option greyout verified best-effort (PinValue's argv is already covered by the
  emission cell; DisableOptions render-check, if attempted, depends on the §6 spike's popup-open)."
  This keeps the I2 renderer-check from being read as a per-flag query on a per-option effect (which
  would invite the vacuous assertion §4 warns against). Minor because the argv semantics — the
  funds/false-red-CI-load-bearing half — are correct, and the authoritative table already says the
  right thing.

---

## Rubric grade

- **Altitude / scope:** correct (test-infra quality gate; secret-hygiene in scope, funds out). ✓
- **Factual premises:** re-verified TRUE against live source (9 kinds, 6 effects + argv, 17/61,
  61-count, persist-then-redact). ✓
- **Anti-tautology core (§4):** intact; layered oracle preserved; I1 widget-injection mandate
  restored. ✓
- **Honesty (§1/§9/§11):** I3 reframe is honest; catches/does-not-catch is accurate. ✓
- **Feasibility de-risking:** §6 spike concretely gates universal-I1 reach with a descope path. ✓
- **Convergence:** 6I + 5m folded without introducing new C/I or drift. ✓

---

## Bottom line

**GREEN (0C/0I).** Clean convergence. All six round-1 Importants and five Minors are genuinely
closed and source-verified; the folds introduced no new Critical/Important, no number drift, and no
dangling cross-references. The single residual (m1) is a prose-precision carryover that the fold's
own correct per-effect table already neutralizes — close it in the plan-doc. **The spec is cleared
to proceed to the implementation plan-doc** (which itself enters its own R0 loop). First plan action
should be Phase 0 / the §6 feasibility spike, since it gates the I1 reach the plan will size.
