# Phase 6 Sibling CLI Schemas Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `5d88c1d Phase 6: sibling CLI schemas (md/ms/mk) + path-detect tests`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.3 + §B.6 + §C Phase 6

## Verdict

**0C / 3I — fold needed** (all folded inline)

Three important findings folded:

---

## Important findings

### I-1 — Bare `matches!` in `cell_2_mnemonic_only_others_missing` is a no-op assertion

**Confidence:** 97
**File:** `tests/path_detect.rs:76-79` (pre-fold)

Pre-fold cell_2 had:
```rust
matches!(detect_in("mnemonic", Some(path_env.clone()), None), Detected::Found(_));
```
`matches!` returns `bool`; without `assert!`, the result is discarded. The positive-arm check for `mnemonic` being Found is silently a no-op; only the negative arms for `md`/`ms`/`mk` actually fired.

**Fold:** Wrap in `assert!(matches!(...), "mnemonic must be Found when present in PATH")`. R1 I-1 fold attribution comment added.

### I-2 — SPEC §B.3 + §B.6 require amendment for `PositionalArgSchema`

**Confidence:** 90
**File:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.3 + §B.6

Phase 6 introduced `PositionalArgSchema` + `SubcommandSchema.positional_args` + positional-emission rule (emit at end, skip empty strings). None appeared in the normative §B.3 / §B.6 blocks. The Phase 0 R1 precedent (§B.2 amendment for `src/lib.rs`) establishes that normative type-grammar additions require SPEC amendments in the same commit.

**Fold:**
- §B.3: `SubcommandSchema` gains `positional_args: &'static [PositionalArgSchema]` field; `PositionalArgSchema` struct definition appended with R1 I-2 fold attribution.
- §B.6: new bullet 8 codifies "positional argv emission at end of argv in form-state order; empty strings skipped; repeating-positional schema index may have multiple state entries."

### I-3 — Phase 6 exit gate overstates delivery; tab-strip GUI deferred

**Confidence:** 85
**File:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §C Phase 6 exit gate

Pre-fold exit gate read: "All four CLI tabs render; missing-CLI greying verified; schema-mirror tests pass for all four CLIs." The commit deferred `src/app.rs` tab-strip wiring with `path_detect::NotFound` greying. The deferral was technically justified (eframe loop not yet wired) but the gate text was unchanged.

**Fold (two-part):**
- Repo-side: landed minimal `src/app.rs` with `AppState`, `CliTab`, per-CLI `Detected` slots, `tab_available()`, and `missing_binary_tooltip()` (SPEC §8 byte-exact tooltip text). Added `cell_8` + `cell_9` to `tests/path_detect.rs` covering this DATA-LAYER surface.
- Plan-side: §C Phase 6 exit gate narrowed to "data-layer wired + path_detect tests pass; GUI tab-strip rendering deferred to Phase 7+ (eframe-loop prerequisite)." Explicit deferral makes the contract honest.

---

## Hot-spot evaluations (all other items below threshold)

| Hot spot | Disposition |
|---------|-------------|
| 2: positionals-after-flags ordering | clap permissive; no issue |
| 5: flat positionals Vec for multi-positional subcommands | Phase 7+ concern; no v0.1 multi-positional subcommand exists |
| 6: Windows PATHEXT case-sensitivity in cell_4 | Production gated by `cfg!(windows)`; test override comment correct |
| 7: `is_executable_file_impl` visibility | private fn; correct |
| 8: positionals-emission cell in argv_assembler.rs | Coverage gap, not a bug; below threshold |
| 10: md v0.3.0→v0.4.3 flag drift | schema_mirror live-verifies against v0.4.3 binary; safe |
| 11: argv_assembler regression with empty positionals | `Vec::new()` iterates 0 times; no regression |

---

## Post-fold test totals

  argv_assembler         10/10
  argv_assembler_slot     5/5
  conditional_visibility 13/13
  copy_command           15/15
  path_detect             9/9   (was 7; +2 AppState data-layer cells)
  runner_integration      3/3
  schema_mirror           5/5

Total: 60 tests across 7 binaries.
