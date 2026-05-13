# Phase A.3 egui_kittest Scaffold — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit bb50ac9 on branch v0_2
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase A.3; SPEC §6

## Verdict

**0C / 3I — fold needed**

The harness pattern is mechanically correct: `Harness::new_ui_state` matches the egui_kittest 0.31.1 API, the click/run/state() sequence is the canonical kittest idiom, `kittest::Queryable` is correctly imported to bring `get_by_label` into scope, and `vis_for` over `Vec<(&'static str, Visibility)>` is sound. `tests/widget_secret.rs` is correctly absent. Cargo churn (~761 lines) is proportionate and contains no surprising entries. Three Important findings require plan folds before the phase closes: exit gate wording mismatch (deviation b), cell 1 value-emission steps omitted, and SPEC §6 cell 2 aspirational assertions absent with no plan phase to GREEN them.

---

## Critical findings

None.

---

## Important findings

### I-1 — Exit gate wording violated: cells PASS, gate requires "fail RED with assertion errors"

**Confidence:** 88
**File/ref:** Plan line 760; `tests/widget_interaction.rs` (both test functions)

The Phase A.3 exit gate (plan line 760) reads: "both cells 1 and 2 fail RED with assertion errors." Both cells currently pass. The commit rationalizes this as the pre-bb50ac9 compile-fail state counting as "RED," which is defensible in spirit but conflicts with the exit gate's explicit "assertion errors" language — which implies the binary compiles and the assertions themselves fail. The root cause is that both cells assert already-implemented v0.1 behavior: cell 1's `to_slot_argv()` empty-value omission rule ships in v0.1; cell 2's mutual-exclusion logic (`conditional::export_wallet()`) ships in v0.1. Neither assertion was going to fail RED once `egui_kittest` compiled.

**Required fold:** Update plan line 760 to replace "fail RED with assertion errors" with language acknowledging that the pre-bb50ac9 compile-fail is the RED state for the harness introduction, and that cell assertions are against pre-existing v0.1 behavior (GREEN at landing). Cite I-2 and I-3 for the deferred aspirational assertion scope.

---

### I-2 — Cell 1 fidelity gap: value-setting and byte-exact argv steps omitted

**Confidence:** 85
**File:** `tests/widget_interaction.rs` lines 35-88; Plan line 751

Plan line 751 specifies: "add row → set subkey `xpub` → set value → remove → add again; call `assemble_argv()`; assert byte-exact argv." The commit delivers: add row → remove → add again; assert `to_slot_argv()` is empty (value is empty). The ComboBox subkey selector interaction, the TextEdit value-field interaction, and `assemble_argv()`'s byte-exact `--slot @0.xpub=<value>` assertion are all absent. The file-level doc comment (lines 12-15) correctly describes the reduced flow; the plan spec is not met.

The gap matters for Phase B: `widget_secret.rs` will reuse this harness pattern. If the ComboBox and TextEdit interaction code paths remain unverified in A.3, Phase B inherits an untested harness surface for the more complex `SecretLineEdit` widget.

**Required fold:** Either (a) deliver the full flow in A.3 R2 (kittest ComboBox accessible-label API requires exploration); or (b) amend plan line 751 to match the reduced scope and move the byte-exact argv assertion to an explicitly named later sub-phase. If (b), Phase B.1 is a natural vehicle since it already extends `widget_interaction.rs` scope via `widget_secret.rs`.

---

### I-3 — SPEC §6 cell 2 aspirational assertions absent; no plan phase GREENs them

**Confidence:** 82
**File:** `tests/widget_interaction.rs` lines 90-166; Plan line 530 (SPEC §6 table); Plan line 752

SPEC §6 table (plan line 530) specifies cell 2 should assert: `--threshold` hidden on bip84; required on sparrow; `--wallet-name` absent on bip84. These assertions are not in the commit. The commit's cell 2 asserts only the existing mutual-exclusion logic (`--template` ↔ `--descriptor`). Neither `--threshold` nor `--wallet-name` visibility rules exist anywhere in `conditional::export_wallet()` (src/form/conditional.rs:114-133). No current phase in the plan adds them: Phase D.3 handles md schema entries; Phase B handles `SecretLineEdit`; no phase extends `export_wallet` conditionals. This is both a SPEC/code mismatch and a plan gap.

**Required fold:** Amend the plan with one of: (a) drop the `--threshold`/`--wallet-name` aspirational assertions from SPEC §6 cell 2 (update SPEC §6 table accordingly; cell 2 is scoped to mutual-exclusion only); or (b) add a named phase or sub-phase that extends `conditional::export_wallet()` with threshold/wallet-name rules and updates cell 2. Without this, SPEC §6 is permanently violated.

---

## Sub-threshold notes

### N-1 — Comment inaccuracy: default SlotRow subkey stated as Phrase, is actually Xpub

**Confidence:** 97 (documentation only, no functional impact)
**File:** `tests/widget_interaction.rs` lines 60-62

Lines 60-61 read: "subkey = SlotSubkey::Phrase (first variant of the const ALL — see slot_editor.rs:39-46)". `SlotSubkey::ALL[0]` is `Phrase`, but `SlotRow::default()` (slot_editor.rs:83-90) hard-codes `subkey: SlotSubkey::Xpub`. The default impl does not delegate to `ALL[0]`. No test assertion reads or checks the subkey value, so behavior is unaffected. Fix comment at next touch of this file.

---

### N-2 — Accesskit production binary side effect warrants a FOLLOWUPS entry

**Confidence:** 80
**File:** `Cargo.toml` line 13; `Cargo.lock` (accesskit_* and atspi-* entries)

`egui_kittest 0.31.1` → `kittest 0.1.0` → `accesskit 0.17.1`; cargo feature unification activates `egui/accesskit`; `egui-winit 0.31.1` links `accesskit_winit 0.23.1`, which includes `accesskit_unix` (atspi-* on Linux), `accesskit_macos`, and `accesskit_windows` for all platforms. The production binary now has an active accessibility tree. This is not a security issue, but the user did not request accessibility integration; it is an involuntary consequence of the test harness choice. No FOLLOWUPS entry for this side effect currently exists. A `gui-accesskit-production-side-effect` entry should record the root cause and disposition (accept / revisit / alternative harness).

---

## Deviation rulings

### Deviation (a) — accesskit feature added to eframe: ACCEPTED

Technical necessity confirmed. `kittest 0.1.0` depends on `accesskit 0.17.1`; cargo feature unification activates `egui/accesskit`; `egui-winit 0.31.1`'s `PlatformOutput` is destructured exhaustively and requires the feature at compile time. The `"accesskit"` addition to eframe's feature list in Cargo.toml line 13 is the minimum correct fix. No 0.31.x version of `egui_kittest` avoids the dep (kittest integration shipped with 0.31). No cargo mechanism scopes features to dev builds only. No accesskit-free egui 0.31 testing harness exists. Deviation accepted; N-2 documents the side effect.

### Deviation (b) — Cells PASS instead of fail RED: PARTIAL ACCEPT, plan fold required per I-1/I-2/I-3

The compile-fail-before-bb50ac9 interpretation of "RED" is valid for the file-does-not-exist state, and plan line 751 ("Fails RED until egui_kittest dep is in") supports it. However, the exit gate (line 760) explicitly says "assertion errors," and the cells were designed to assert aspirational behavior (byte-exact argv in cell 1; threshold/wallet-name in cell 2) that v0.1 had not implemented. The commit avoids RED by narrowing assertions to already-implemented behavior. This is pragmatically sound but three plan folds are required (I-1, I-2, I-3) before the phase can close.

### Cell 1 interaction fidelity gap: FOLD REQUIRED

The ComboBox subkey selector and TextEdit value-field interaction were deferred as requiring API exploration beyond the scaffolding scope. The reduced flow (add/remove lifecycle) is a correct subset. The ruling matches I-2: fold the byte-exact argv assertion into a documented later scope (A.3 R2 or Phase B.1 addendum).

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | `Harness::new_ui_state` API usage | CORRECT — matches egui_kittest 0.31.1 signature: `fn new_ui_state(app: impl FnMut(&mut Ui, &mut State) + 'a, state: State) -> Self`. `kittest::Queryable` import is required and present. |
| 2 | `vis_for` helper | CORRECT — linear scan over `Vec<(&'static str, Visibility)>`. `*k == flag` compares `&'static str` to `&str` via `PartialEq<str>`; correct. Two-element max result size makes linear scan appropriate. |
| 3 | click() + run() ordering | CORRECT — kittest click() enqueues pointer events into a locked queue; events are not processed until run() is called. Pattern is canonical. |
| 4 | ScrollArea traversal | CORRECT — accesskit traverses scroll area contents; `"+ Add slot"` and `"✕"` inside `ScrollArea::vertical().show()` are reachable via `get_by_label`. |
| 5 | Cell 2 values accumulation | CORRECT — "clear-form" calls `state.values.clear()` before the descriptor path; no stale accumulation. |
| 6 | `tests/widget_secret.rs` absence | CONFIRMED ABSENT — R1 C-1 fold correctly executed. |
| 7 | No snapshot `.png` files | CONFIRMED — SPEC §6 assertion-only posture correctly implemented. |
| 8 | `egui_kittest` in `[dev-dependencies]` only | CORRECT — Cargo.toml line 35; not in `[dependencies]`. |
| 9 | Cargo.lock churn (~761 lines) | PROPORTIONATE — accesskit family + atspi-* are expected transitives of `accesskit_unix`. No unexpected or security-concerning entries. atspi crates are the Linux AT-SPI2 accessibility bridge — standard. |
| 10 | `SlotRow::default()` subkey in cell 1 | Comment bug only (N-1). No assertion reads the subkey; test logic unaffected. |
| 11 | Cell naming (`cell_1_*` / `cell_2_*`) | CORRECT — matches plan numbering and lexicographic binary ordering. |

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| `run()` redundant after final click at line 78-79 | 40 | run() is required to process queued click events before reading `harness.state()`; correct. |
| `FlagValue::Dropdown` kind mismatch for `--template` | 35 | `--template` uses `FlagKind::Dropdown`; `FlagValue::Dropdown(...)` matches correctly. No issue. |
| Probe-button cell 2 vs real form renderer | 25 | Probe widget is adequate for testing the conditional function; driving the full form renderer would conflate scope. Justified in doc comment. |

---

## Exit gate checklist

| Gate item | Status |
|-----------|--------|
| `cargo test --test widget_interaction` compiles clean | PASS |
| Both cells fail RED with assertion errors | FAIL — both PASS (deviation b; I-1 fold required) |
| `egui_kittest = "0.31"` in `[dev-dependencies]` | PASS — Cargo.toml line 35 |
| No snapshot `.png` files | PASS |
| `tests/widget_secret.rs` absent | PASS — R1 C-1 fold correctly executed |
| 0C / 0I | FAIL — 3I outstanding; fold needed |

Sources:
- egui_kittest 0.31.1 docs: <https://docs.rs/egui_kittest/0.31.1/>
- kittest 0.1.0 crate: <https://crates.io/crates/kittest>
