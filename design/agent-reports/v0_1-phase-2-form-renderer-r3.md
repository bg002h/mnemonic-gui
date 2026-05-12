# Phase 2 Form/Argv Review — R3

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `f34b915 fold Phase 2 R2 (1C/0I) — ArgvQuote-canonical cmd_quote + roundtrip cell`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md`
**R2 report:** `design/agent-reports/v0_1-phase-2-form-renderer-r2.md`

## Verdict

**0C / 1I — fold needed** (folded inline; see below)

R2 algorithm verified correct against Colascione's canonical `ArgvQuote`
reference. All 7 algorithm traces match expected encodings; all 7
roundtrip cases parse correctly via the hand-rolled `parse_cmdline_argv_w`.
One stale doc comment named the now-wrong encoding.

---

## R3 Task 1 — Algorithm traces (all PASS)

| Input | Expected | Algorithm | Match |
|------|---------|----------|-------|
| `a"b` | `"a\"b"` | bare `"` → `\"` (rule 4) | ✓ |
| `a\"b` (a,`\`,`"`,b) | `"a\\\"b"` | n=1 + `"` → 3 `\` + `"` (rule 2a) | ✓ |
| `a\\"b` (a,`\`,`\`,`"`,b) | `"a\\\\\"b"` | n=2 + `"` → 5 `\` + `"` (rule 2a) | ✓ |
| `C:\tmp\` | `"C:\tmp\\"` | trailing `\`: end-of-string → 2 `\` (rule 2b) | ✓ |
| `C:\Users\Alice\file.txt` | `"C:\Users\Alice\file.txt"` | interior `\` pass-through (rule 2c) | ✓ |
| `simple` | `"simple"` | no specials | ✓ |
| `with space` | `"with space"` | no specials | ✓ |

## R3 Task 2 — Roundtrip parser (verified)

`parse_cmdline_argv_w` faithfully implements `CommandLineToArgvW` rules:
- n `\` + `"` → emit ⌊n/2⌋ `\`; if n odd → literal `"`, if even → toggle quote mode
- Bare `"` → toggle quote mode
- Whitespace outside quotes → token boundary

All 7 roundtrip cases verify recovery of the original input.

## R3 Task 3 — Reference comparison

Algorithm exactly matches Daniel Colascione's `ArgvQuote` (Microsoft
archive, "Everyone quotes command line arguments the wrong way"). The
only deviation is unconditional wrapping (`Force=true` equivalent),
which is a stricter posture and safe for display-only output.

## R3 Task 4a — Existing tests valid

All 8 Windows test cells verified valid against the post-R2 encoding:
- `windows_simple_argv_double_quoted` — exact match
- `windows_empty_string_renders_as_empty_quotes` — `""` correct (empty loop body)
- `windows_line_continuation_separator_is_caret_crlf_indent` — separator unchanged
- `windows_trailing_backslash_does_not_break_close_quote` — algorithm trace #4
- `windows_interior_backslash_run_unchanged` — algorithm trace #5
- `windows_embedded_double_quote_escaped_as_backslash_quote` — algorithm trace #1
- `windows_backslash_immediately_before_embedded_quote_emits_three_backslashes` — algorithm trace #2
- `windows_double_backslash_before_quote_emits_five_backslashes` — algorithm trace #3

## R3 Task 4b — Empty-string roundtrip skip

Defensible. `assemble_argv` guards empty Text/Path/Dropdown/NodeValueComposite
with `if !v.is_empty()` before emission — no empty argv element reaches
`render_copy_command` via the production GUI path. The empty-string
rendering is independently tested by `windows_empty_string_renders_as_empty_quotes`.

---

## Important findings

### I-1 — `ShellFlavor::WindowsCmd` doc comment names the wrong encoding

**Confidence:** 92
**File:** `src/form/invocation.rs:23` (pre-R3 fold)

Pre-R3 doc comment read: "embedded `\"` becomes `\"\"`". The actual
post-R2 encoding is `\"` (n=0 ArgvQuote case). The comment names the
exact wrong encoding R2 C-1 fixed.

**Impact:** A future maintainer reading this public-API doc comment on
the exported `ShellFlavor` enum would see the wrong encoding and could
"fix" the implementation to match the comment, re-introducing the R2
C-1 regression.

**Fix (folded inline):**
- `src/form/invocation.rs::ShellFlavor::WindowsCmd` doc comment rewritten
  to: "embedded `\"` is encoded as `\\\"` per the `ArgvQuote`
  odd-backslash rule (see `cmd_quote` doc-comment for full rules — `\"\"`
  is NOT a valid literal-`\"` escape under `CommandLineToArgvW`)".
- `tests/copy_command.rs` module-level comment refresh: "POSIX uses
  `shlex::try_quote`; Windows uses ArgvQuote (odd-backslash) encoding
  per `CommandLineToArgvW` convention — see
  `src/form/invocation.rs::cmd_quote` for the full rules."

**Post-fold test totals (unchanged from R2):**
  copy_command: 15/15
  argv_assembler: 10/10
  schema_mirror:   2/2

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| `parse_cmdline_argv_w` empty-token limitation | 55 | Known, documented, not reachable via production path |
| Unconditional wrapping vs Colascione `Force=false` | 20 | Always-wrap is safe; stricter posture fine for display-only |
| `^` caret escaping for cmd.exe metacharacter layer | 15 | Out-of-scope for display-only per R2 disposition |
