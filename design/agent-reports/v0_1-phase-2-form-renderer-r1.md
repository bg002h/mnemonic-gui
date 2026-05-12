# Phase 2 Form/Argv Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `95c4a97 Phase 2: form widget + argv assembler + copy-command quoting`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.6 + §B.6.7 + §C Phase 2

## Verdict

**0C / 1I — fold needed**

One important finding. Fold extends to 14 copy_command cells (added 3 backslash-edge cases).

---

## Important findings

### I-1 — `cmd_quote` produces malformed cmd.exe tokens for paths ending in `\`

**Confidence:** 85
**File:** `src/form/invocation.rs:170-173` (pre-fold)

**What:** Pre-fold `cmd_quote` wrapped tokens in `"..."` and doubled embedded `"` but did NOT escape backslashes immediately preceding a `"`. Under the `CommandLineToArgvW` parsing rules used by cmd.exe + PowerShell + the Windows C runtime, a `\` immediately before `"` is treated as an escape: the `"` is captured as a literal and the token is left unclosed. So for input `C:\tmp\`, the pre-fold output `"C:\tmp\"` parses as the unclosed token `C:\tmp"` and merges with subsequent argv tokens.

**Impact:** Windows paths commonly end in `\`. A user copying the rendered command into cmd.exe would get silently corrupted argv — for a display-only contract this is a correctness defect.

**SPEC gap:** §B.6.6 says "Windows. Double-quote each arg; embedded `\"` becomes `\"\"`" but is silent on backslash escaping. The implementation faithfully inherited the SPEC gap.

**Fold:** Replace `cmd_quote` with a `CommandLineToArgvW`-compatible implementation that:
1. Wraps the token in `"..."`.
2. Counts each run of consecutive `\`.
3. If the run is immediately followed by `"` OR end-of-string, doubles the run.
4. If the run is interior (followed by any other char), passes through unchanged.
5. Embedded literal `"` is encoded as `""`.

Added 3 regression cells to `tests/copy_command.rs`:
- `windows_trailing_backslash_does_not_break_close_quote` (input `C:\tmp\` → `"C:\tmp\\"`).
- `windows_interior_backslash_run_unchanged` (input `C:\Users\Alice\file.txt` → `"C:\Users\Alice\file.txt"`).
- `windows_backslash_immediately_before_embedded_quote_is_doubled` + `windows_double_backslash_before_quote_is_doubled_to_four` — the latter pair pins the `\"` and `\\"` interior cases (the reviewer's originally-proposed expectation was wrong; the test was rewritten to actually exercise the `\` immediately-before-`"` case rather than spaced apart).

**Test count post-fold:** copy_command 14/14, argv_assembler 10/10, schema_mirror 2/2.

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| Schema order vs form-state order | — | `cell_7` correctly pins; iterator order verified |
| Repeating flag insertion order | — | `state.values.iter().filter` preserves insertion order |
| `Number` unconditional emission (no omit) | — | SPEC §6.7 specifies no omit rule; correct |
| `cell_6` empty-value NodeValueComposite | — | R3 I-3 fold exercised correctly |
| Path `stdio_sentinel` both branches reachable | 25 | `cell_5` + `cell_5b` cover the schema branches; defensive guard untested but trivially correct |
| POSIX `shlex` NUL-byte fallback | 10 | Pathological input not expected in argv; fallback is POSIX-safe |
| widget.rs unexercised by tests | — | SPEC §14 defers headless egui to v0.2; compilability is the Phase 2 bar |
| `_ =>` silent-skip emitter posture | — | SPEC-endorsed; debug-assert future hardening noted inline |
| Repeating flag with zero entries | 20 | Code path trivially correct; no test pins it |
| Path stdin sentinel defensive skip | — | SPEC chose emit-guard; correct posture |

---

## Hot-spot resolution table

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | Byte-exact emission per FlagKind | Correct |
| 2 | Schema order vs form-state order | Correct |
| 3 | Repeating flag insertion order | Correct |
| 4 | POSIX shell quoting | Correct |
| 5 | Windows cmd.exe quoting — trailing `\` | **Bug → I-1, folded** |
| 6 | argv[0] bare name | Correct per SPEC §6.1 |
| 7 | widget.rs unexercised | Acceptable per Phase 2 scope |
| 8 | Silent-skip emitter posture | SPEC-endorsed |
| 9 | Repeating with zero entries | Trivially correct |
| 10 | Path stdin sentinel | Correct |
