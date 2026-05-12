# Phase 6 Sibling CLI Schemas Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `229353e fold Phase 6 R1 (0C/3I) — three doc/exit-gate/test-bug folds`
**R1 report:** `design/agent-reports/v0_1-phase-6-siblings-r1.md`

## Verdict

**0C / 1I — fold needed** (folded inline)

All three R1 folds verified correct. One new defect: stale `Option<String>` doc-comment on a `String`-returning fn.

---

## R1 fold verification — PASS (all 3)

- **I-1:** `tests/path_detect.rs` cell_2 bare `matches!` is now `assert!(matches!(...))` with R1 I-1 fold attribution.
- **I-2:** Plan §B.3 now contains `PositionalArgSchema` struct + `SubcommandSchema.positional_args` field with fold attribution. §B.6 bullet 8 codifies positional argv emission.
- **I-3:** Plan §C Phase 6 exit gate narrowed; `src/app.rs` ships AppState/CliTab/missing_binary_tooltip data-layer surface; tests cell_8 + cell_9 cover it.

---

## Important findings (R2)

### I-1 — `missing_binary_tooltip` doc-comment falsely claims `Option<String>` return

**Confidence:** 88
**File:** `src/app.rs:103` (pre-R2 fold)

Pre-fold doc comment read: "Returns `None` if the tab IS available (no tooltip needed)." Function signature is `pub fn missing_binary_tooltip(tab: CliTab) -> String` — always returns `String`. Stale remnant from a design iteration where the function was to return `Option<String>`.

**Impact:** Phase 7 callers reading this doc will expect `Option<String>` and may write `.unwrap()` (compile error) or `if let Some(t) = ...` (compile error), or be confused about the contract boundary.

**Fold:** Replace the doc line with: "Call only when `tab_available(tab) == false`; always returns the full tooltip string (never `None` — the prior `Option<String>` design was dropped during Phase 6 R1 fold; R2 I-1 corrects this doc comment to match the actual unconditional-String contract)."

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| `<repo-url>` literal placeholder in tooltip | — | Phase 7 will substitute per-CLI GitHub URL from pinned-upstream.toml |
| No `Display` impl on `CliTab` | — | Phase 7 concern |
| `detect_all()` constructor name vs `scan()` for I/O semantics | — | Style note; below threshold |

---

## Post-fold tests

  cargo build → clean (doc-only edit)

Total counts unchanged from R1:
  argv_assembler         10/10
  argv_assembler_slot     5/5
  conditional_visibility 13/13
  copy_command           15/15
  path_detect             9/9
  runner_integration      3/3
  schema_mirror           5/5

= 60 total tests across 7 binaries.
