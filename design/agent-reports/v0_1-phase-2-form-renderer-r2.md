# Phase 2 Form/Argv Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `61b4579 fold Phase 2 R1 (0C/1I) — CommandLineToArgvW-compatible cmd_quote`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md`
**R1 report:** `design/agent-reports/v0_1-phase-2-form-renderer-r1.md`

## Verdict

**1C / 0I — fold not converged at R2** (folded inline; see below)

The R1 I-1 fold correctly addressed the trailing-`\` case but introduced
a critical regression in the embedded-`"` encoding.

---

## Critical findings (R2)

### C-1 — `cmd_quote` used `""` for literal `"`; `CommandLineToArgvW` has no `""` escape

**Confidence:** 97
**File:** `src/form/invocation.rs` (R1-fold body, pre-R2)
**Tests affected:** 3 cells in `tests/copy_command.rs`

**Reviewer trace:**

For input `a"b` (3 chars), the R1 fold emitted `"a""b"`. Parsing under
`CommandLineToArgvW` rules:
- `"` → enter quoted
- `a` → a
- `""` → close-quote then reopen-quote (the `""` form is NOT a literal
  `"` escape in CommandLineToArgvW; that's an MSVCRT-2008+ extension)
- `b` → b
- `"` → close
- Result: `ab` — the literal `"` is silently lost.

For input `a\"b`, the R1 fold emitted `"a\\""b"`. Parsing:
- `"` → quoted
- `a` → a
- `\\` (2 = 2n) + `"` → 1 literal `\` + toggle (now unquoted)
- `"` → reopen
- `b` → b
- `"` → close
- Result: `a\b` — `"` again lost.

The R1 tests passed because they asserted `s.contains(expected)`, not
that the rendered string round-trips through a real CommandLineToArgvW
parser.

**Fix (folded in R2):**

Rewrote `cmd_quote` to Daniel Colascione's canonical `ArgvQuote` rules
(Microsoft "Everyone quotes command line arguments the wrong way"):

1. For each run of `n` backslashes followed by `"` (interior): emit
   `2n+1` `\` + the literal `"`. The odd count is the only universal
   way to encode a literal `"` inside `"..."` for CommandLineToArgvW.
2. For each run of `n` backslashes followed by end-of-string: emit
   `2n` `\` (so the close-`"` is unambiguous).
3. For interior `\` runs not followed by `"`: pass through.
4. A bare `"` (no preceding `\`) is encoded as `\"` (n=0 case).

**Test fold:**

| Test | Old expected | New expected |
|------|--------------|--------------|
| `windows_embedded_double_quote_escaped_as_backslash_quote` (renamed from `..._is_doubled`) | `"a""b"` | `"a\"b"` |
| `windows_backslash_immediately_before_embedded_quote_emits_three_backslashes` (renamed from `..._is_doubled`) | `"a\\""b"` | `"a\\\"b"` (3 `\` + `"`) |
| `windows_double_backslash_before_quote_emits_five_backslashes` (renamed from `..._to_four`) | `"a\\\\""b"` | `"a\\\\\"b"` (5 `\` + `"`) |

Added a `windows_roundtrip_all_embedded_quote_shapes` cell that:
1. Implements `parse_cmdline_argv_w` (a hand-rolled CommandLineToArgvW
   in 30 LoC).
2. Round-trips 7 representative inputs (simple, space, lone `"`, 1`\`+`"`,
   2`\`+`"`, trailing `\`, interior `\`) through `cmd_quote` → parser, asserting recovery.
3. This is the load-bearing test for the C-1 regression class —
   string-shape assertions alone proved insufficient at R1.

Empty-string roundtrip omitted: the GUI's `assemble_argv` layer omits
empty Text/Path/etc. values per SPEC §6.7, so empty argv elements never
reach `render_copy_command` in practice.

**Post-fold test totals:**
  copy_command: 15/15 (was 14/14 at R1; +1 roundtrip cell)
  argv_assembler: 10/10
  schema_mirror:   2/2

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| Some Windows parsers accept `""` as literal `"` (MSVCRT 2008+) | 30 | Not universal; `CommandLineToArgvW` (the canonical Win32 parser) does not. The `\"` form works under both |
| POSIX quoting regression from R2 fold | — | No change to `posix_quote` |
| `cmd.exe` caret `^` preprocessing layer | — | Out of scope for display-only render; caret-escaping is a separate concern |
