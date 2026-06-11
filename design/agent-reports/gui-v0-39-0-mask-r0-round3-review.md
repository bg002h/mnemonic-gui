# R0 Review — GUI v0.39.0 (mask secret values in every on-screen command display) — ROUND 3 (GREEN)

**Source SHA verified:** `71c7ecd` (= v0.38.0). Re-review after folding all round-2 findings (I1 + M1 + M2 + M3).

**Verdict: 🟢 GREEN (0C / 0I)** — implementation-ready.

---

## I1 fold (RunResult.mask population — option (a)) — SOUND

The three-part wiring is internally consistent:

1. `runner.rs:148-153` struct literal currently has four fields (`argv`, `exit_code`, `stdout`, `stderr`). Adding `mask: Vec::new()` is a clean one-line add; `run_with_stdin` stays mask-oblivious (`Vec::new()` is a zero-cost sentinel).
2. `spawn_and_capture` (`main.rs:1110`) receives `mask: Vec<bool>` and assigns `result.mask = mask` between `run_with_stdin` returning `Ok(result)` and `app.last_run = Some(result)`. Both call sites identified: `:1009` (no-confirm; `mask` in scope from `:898`), `:1031` (confirm-modal Run; mask travels via the grown `pending_confirm_argv` tuple).
3. Options (b)/(c) rejected with correct rationale — `runner.rs` has no schema/secrets imports and shouldn't gain the display concern.

No compile gap: `RunResult` is destructured only at `main.rs:459-465` (D3 display → `render_copy_command_masked(&result.argv, &result.mask, Posix)`) and in `spawn_and_capture`.

## M1 fold (`:303` → `:302`) — CORRECT

`argv.push(value.as_str().to_string())` for the secret positional is at `invocation.rs:302`. SPEC now cites `:302` in both the Problem (item 3) and Design §1. Consistent.

## M3 fold (T-A3 DANGEROUS-direction) — CORRECT

T-A3 mandates an explicit per-case `mask.any() ⟹ should_confirm_run` over every T-A1 vector, with the safe asymmetry (Boolean `*-stdin` greyed → emits nothing → mask all-false, confirm true) documented not asserted. Well-specified and complete.

## M2 fold (Deferred CHANGELOG lines) — CORRECT

CHANGELOG `:1978-1980` (`cell_paste_warn_modal_trigger` over-claim) and `:2196` (non-goals paste-warn reference) confirmed accurate at the current SHA; decay caveat retained.

## Cross-consistency scan — NO fold-introduced drift

- `pending_confirm_argv` grows to `(Vec<String>, Vec<bool>, Option<Vec<u8>>)`; the `:1015` destructure becomes `(argv, mask, stdin)` and the `:1031` call `spawn_and_capture(self, argv, mask, stdin)`. `clone()` on `Vec<bool>` is trivial. Covered.
- Tree-mode append `:917-920`: both `--spec` (flag-name) and `-` (stdin sentinel) tokens are non-secret → both mask bits `false`. Unambiguous from the mask-bit rules.

No new citation contradiction or logic gap. The SPEC is implementation-ready.
