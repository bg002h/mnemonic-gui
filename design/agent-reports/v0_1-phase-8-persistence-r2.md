# Phase 8 Persistence Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `5084b3f fold Phase 8 R1 (0C/2I) — schema_version stamp + PascalCase leak check`
**R1 report:** `design/agent-reports/v0_1-phase-8-persistence-r1.md`

## Verdict

**0C / 0I — converge**

Both R1 folds verified clean. No new defects.

---

## R1 fold verification

### I-1 — `save()` schema_version stamp — PASS

`src/persistence.rs:130-131`: stamp is applied AFTER `redact_persisted_state` returns the clone, BEFORE `serde_json::to_string_pretty`. `save()` takes `&PersistedState` (immutable); mutation is on local `redacted` binding only — caller state untouched.

`cell_11` regression guard verifies three independent assertions: default `schema_version: 0`, load returns Some with version stamped to SCHEMA_VERSION, no `.bak` created.

### I-2 — cell_2 PascalCase leak check — PASS

`SECRET_SUBKEY_PASCAL = &["Phrase", "Entropy", "Wif", "Xprv"]` is the load-bearing assertion. Lowercase loop retained as defence-in-depth for future `rename_all` annotation.

---

## cell_4 interaction check

`cell_4` writes its stale-version JSON directly via `fs::write` + `serde_json::to_string_pretty`, BYPASSING `save()`. The schema_version stamp has no reach over this path. The stale `999` value survives to `load()`, which correctly renames to `.bak`. No clash.

---

## Test totals (unchanged from R1 post-fold)

  argv_assembler 10, argv_assembler_slot 5, conditional_visibility 13,
  copy_command 15, path_detect 9, persistence 11, runner_integration 3,
  schema_mirror 8, secrets 18.
= 92 total tests across 9 binaries.
