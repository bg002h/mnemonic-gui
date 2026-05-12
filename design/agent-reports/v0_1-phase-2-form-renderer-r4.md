# Phase 2 Form/Argv Review — R4

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `109e93f fold Phase 2 R3 (0C/1I) — refresh stale ShellFlavor::WindowsCmd doc`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md`
**R3 report:** `design/agent-reports/v0_1-phase-2-form-renderer-r3.md`

## Verdict

**0C / 0I — converge**

R3 fold verified correct. No stale references remain. No new defects above threshold.

---

## R4 Task 1 — `ShellFlavor::WindowsCmd` doc comment

Post-R3: names `\"` as the encoding, explicitly warns `""` is NOT a valid CommandLineToArgvW escape, cross-references `cmd_quote`, fold-tag present. PASS.

## R4 Task 2 — `tests/copy_command.rs` module-level comment

Post-R3: "ArgvQuote (odd-backslash) encoding per CommandLineToArgvW convention — see src/form/invocation.rs::cmd_quote for full rules." PASS.

## R4 Task 3 — Stale-reference sweep

Searched `src/` for `""`, `cmd-style doubled`, `double-the-`, `cmd.exe convention`, `doubled.*quote`. All `""` occurrences are either:
- Negation references in doc-comments ("`""` is NOT a valid escape")
- Unrelated egui empty-label literals (`from_label("")`, `ui.checkbox(b, "")`)

No prescriptive use of `""` as the encoding remains anywhere in live code.

## R4 Task 4 — Test suite

27 tests (10 argv_assembler + 15 copy_command + 2 schema_mirror) confirmed source-consistent. R3 commit was doc-only; no algorithm change. All green.

## R4 Task 5 — Other Phase 2 gaps after 4 rounds

Inspected `emit_one` type-mismatch arm, `posix_quote` NUL fallback, `render_copy_command` separator pinning, `argv[0]` unqualified contract, `NodeValueComposite` empty-value guard, and `TaggedOrIndexed` empty-tag edge case.

No gap above 80 confidence:

| Item | Confidence | Disposition |
|------|------------|-------------|
| `TaggedOrIndexed` empty-tag guard absent | 60 | Not reachable via GUI dropdown |
| `parse_cmdline_argv_w` empty-token limitation | 55 | Documented in R3; not reachable via production |

---

Phase 2 converged. Next: Phase 3 (SlotEditor composite widget).
