# Phase 7 Secrets + build.rs Codegen Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** R1 fold commit `c77c5a6 fold Phase 7 R1 (1C/1I)`
**R1 report:** `design/agent-reports/v0_1-phase-7-secrets-r1.md`

## Verdict

**0C / 0I — converge**

Both R1 folds verified clean. No fold-introduced defects.

---

## R1 fold verification

### C-1 — rerun-if-changed gap — PASS

`build.rs` lines 79-80 emit `cargo:rerun-if-changed=` for both resolved upstream files, AFTER the `is_file()` existence check at lines 65-72. All stub-fallback paths return before line 79 — no spurious watch directives on the stub path. R2-task-5 failure mode (root resolves, file missing) correctly caught by `is_file()` guard.

### I-1 — test-side parser fence — PASS

Call-graph trace confirms five helpers all live and called:
- `extract_secret_variants_from_block_with_type` (called from `parse_secret_set`)
- `extract_as_str_map_from_block_with_type` (called from `parse_secret_set`)
- `extract_secret_variants_from_block_filtered` (called from the `_with_type` variant)
- `two_segment_guard` (called from both `_with_type` variants)
- `collect_variants_filtered` (called recursively + from `accept` closure)

Zero dead code from the refactor. `two_segment_guard` is structurally identical to `build.rs::extract_variant_ident` (`segs.len() == 2 && segs[0] ∈ {target_type, "Self"}`).

---

## Hot-spot resolution

All 15 R1 hot spots remain at their prior verdicts. No new hot spots introduced.

---

## Test totals

81 tests across 8 binaries, no warnings.
