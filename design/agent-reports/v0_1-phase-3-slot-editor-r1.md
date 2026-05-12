# Phase 3 SlotEditor Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `1ac1fb1 Phase 3: SlotEditor composite widget`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.4 + §6.4 + §C Phase 3

## Verdict

**0C / 1I — fold needed** (folded inline)

Upstream parity verified across all 3 surfaces (variant set, `as_str` token map, `is_secret_bearing` arm set). All 5 test cells valid. Hot-spot sweep clean except for one misleading doc comment.

---

## Upstream parity

| Check | GUI | Upstream | Match |
|-------|-----|----------|-------|
| SlotSubkey variants (declaration order) | 8 (Phrase, Entropy, Xpub, MasterXpub, Fingerprint, Path, Wif, Xprv) | slot_input.rs:13-28 same order | ✓ |
| `as_str()` token map | 8 tokens, exact match | slot_input.rs:44-55 | ✓ |
| `is_secret_bearing()` arm set | Phrase \| Entropy \| Wif \| Xprv | slot_input.rs:56-58 (arm order differs but `matches!` is order-independent) | ✓ |

---

## Important findings

### I-1 — `rows_sorted()` doc comment states wrong upstream BTreeMap key shape

**Confidence:** 82
**File:** `src/form/slot_editor.rs:98-99` (pre-fold)

Pre-fold comment claimed: "matches `cmd::bundle::resolve_slots` BTreeMap semantics where (index, subkey) pairs are the key". Upstream `resolve_slots` actually uses `BTreeMap<u8, Vec<&SlotInput>>` — keyed on `u8` index alone; per-subkey deduplication is done by `validate_slot_set`, not BTreeMap-key collision.

**Impact:** Code behavior is correct; only the comment is wrong. A developer who reads the comment and looks up the upstream source will be confused. If they use the comment to reason about duplicate-row handling, they will reach the wrong conclusion about the mechanism.

**Fold:** Replace with: "rows with the same `index` preserve insertion order. Upstream `cmd::bundle::resolve_slots` keys its `BTreeMap<u8, Vec<&SlotInput>>` by `u8` index alone; duplicate `(index, subkey)` pairs for the same slot are rejected by `validate_slot_set` (`duplicate-subkey` error), NOT by BTreeMap-key collision."

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | `rows_sorted()` sort stability / BTreeMap collision | Behavior correct; doc comment wrong → I-1 |
| 2 | `to_slot_argv()` empty-value omission | Symmetric with SPEC §6.7; UX deferred to Phase 5 |
| 3 | `u8` index `0..=15` range | Correct (BIP-388 N≤16 cosigners → indices 0..=15) |
| 4 | `--slot` in `values` silent drop | Intended design — SlotEditor is sole owner |
| 5 | `allows_slots == false` slot leak (cell_5) | Defense complete |
| 6 | `persistable_rows()` insertion order | Correct for JSON round-trip |
| 7 | `SlotRow::default()` subkey = Xpub | Safe UX default (watch-only); not a bug |
| 8 | Render `remove_idx` mutation | Correct egui single-per-frame pattern |
| 9 | `FormState::from_pairs` no-slots path | Exercised by all non-slot argv tests |
| 10 | `cell_3` `is_secret_bearing()` ↔ persistable | Logically equivalent over closed 8-variant set; cell_3 exhaustively verifies |

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| DragValue 0..=15 vs upstream 0..=255 | 30 | Correct for BIP-388 cap |
| `--slot` values-map silent drop | 20 | Intended design |
| `cell_5` slot leak | 10 | Defense complete |
| `persistable_rows` order | 15 | Correct for round-trip |
| `SlotRow::default()` Xpub | 25 | Safe UX choice; not Phase 3 scope |
| `remove_idx` render | 10 | Correct single-per-frame |
