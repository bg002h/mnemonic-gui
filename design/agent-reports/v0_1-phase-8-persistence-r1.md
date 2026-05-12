# Phase 8 Persistence Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `46d121d Phase 8: persistence + state.json round-trip + never-persist audit`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.10 + §C Phase 8

## Verdict

**0C / 2I — fold needed** (both folded inline)

Security invariant (no secret payload on disk) is correct and well-exercised. Two gaps:

---

## Important findings

### I-1 — `save()` does not stamp `schema_version`; default state writes version 0

**Confidence:** 90
**File:** `src/persistence.rs` (pre-fold save())

`PersistedState` derives `Default` → `schema_version: 0`. Pre-fold `save()` serialized the caller's value as-is. A natural Phase 9+ call pattern
```rust
let state = PersistedState::default();
// ...populate...
save(&state, &path)?;
```
would write `"schema_version": 0`. Next `load()`: `0 != SCHEMA_VERSION (1)` → rename to `.bak` + return `None`. State silently lost on every cold start.

**Fold:** Harden `save()` to stamp `redacted.schema_version = SCHEMA_VERSION` unconditionally. Makes save() self-contained. Added `cell_11_save_stamps_current_schema_version_even_with_default_input` as regression guard.

### I-2 — `cell_2` slot-subkey PascalCase leak check absent; lowercase-only check is vacuously true

**Confidence:** 88
**File:** `tests/persistence.rs` (pre-fold cell_2 slot-subkey assertion block)

`SlotSubkey` derives `Serialize` without `#[serde(rename_all = "snake_case")]` → serde emits PascalCase (`"Phrase"`, `"Wif"`). Pre-fold cell_2 iterated `SECRET_SLOT_SUBKEYS` (lowercase) and asserted `!on_disk.contains("\"phrase\"")`. The lowercase form never appears in serde output, so the assertion is vacuously true — a redaction-logic regression would NOT be caught by this assertion alone (the known-value checks elsewhere in cell_2 would catch the payload leak, but not the structural subkey-name leak).

**Fold:** Added PascalCase variant-name checks (`"Phrase"`, `"Entropy"`, `"Wif"`, `"Xprv"`) which match the actual serialization path. Lowercase check retained as defence-in-depth for a future `#[serde(rename_all)]` migration.

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | redact_for_persistence completeness | Correct (Path/Number/etc. not secret-bearing) |
| 2 | SlotSubkey PascalCase | **→ I-2, folded** |
| 3 | default_state_path None | Caller pattern documented; below threshold |
| 4 | to_string_pretty | Stylistic; not a defect |
| 5 | with_extension("json.bak") | Correct (`state.json` → `state.json.bak`) |
| 6 | form_state_per_subcommand key shape | Documented; Phase 9 wires consistent |
| 7 | Missing serde(default) on String/BTreeMap fields | 78 conf; below threshold |
| 8 | save() schema_version default = 0 | **→ I-1, folded** |
| 9 | serde_json dep | Already pinned |
| 10 | tempfile dev-dep | Already pinned |
| 11 | No fsync | Below threshold for v0.1 |
| 12 | Silent I/O errors in load() | Phase 4 tracing infrastructure; below threshold |

---

## Post-fold test totals

  argv_assembler         10/10
  argv_assembler_slot     5/5
  conditional_visibility 13/13
  copy_command           15/15
  path_detect             9/9
  persistence            11/11 (was 10; +1 cell_11 regression guard)
  runner_integration      3/3
  schema_mirror           8/8
  secrets                18/18

= 92 total tests across 9 binaries.
