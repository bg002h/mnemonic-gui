# R0 Review — GUI v0.39.0 (mask secret values in every on-screen command display) — ROUND 2

**Source SHA reviewed:** `71c7ecd` (= v0.38.0, current `origin/master`). All file:line citations verified against working tree. This round reviews the RE-SCOPED Part-A-only spec (round 1 reviewed the earlier combined Part A + Part B spec).

**Verdict: YELLOW — 0 Critical / 1 Important / 3 Minor**

---

## Critical

None.

---

## Important

**I1 — `RunResult.mask` population path is unspecified: compile-break without guidance.**

The SPEC specifies adding `mask: Vec<bool>` to `runner::RunResult` (`runner.rs:18-25`) and threading it through `spawn_and_capture` (`main.rs:1110`). But `RunResult` is constructed via struct literal at `runner.rs:148-153`:

```rust
let result = RunResult {
    argv,
    exit_code,
    stdout: output.stdout,
    stderr: output.stderr,
};
```

`run_with_stdin` has no access to the mask (which is a GUI-layer display concept, not a runner-layer concept). Adding a `mask` field to `RunResult` without specifying how `runner.rs:148-153` is updated causes an unconditional compile error. The SPEC gives no resolution strategy.

There are three viable options and the SPEC must commit to one:

**(a) Post-construction assignment** (recommended — cleanest, zero runner-layer leakage): `spawn_and_capture` receives `mask: Vec<bool>`, calls `run_with_stdin`, then assigns `result.mask = mask` before storing in `app.last_run`. Requires `RunResult.mask` to be `pub Vec<bool>`, initialized to a default-empty in `runner.rs:148` (`mask: Vec::new()`), overwritten by the caller. The runner.rs literal adds `mask: Vec::new()` — one line, unambiguous.

**(b) Separate `MaskedRunResult` wrapper** in `spawn_and_capture` — unnecessary complexity.

**(c) `mask` parameter on `run_with_stdin`** — leaks display concern into the runner layer; wrong.

The SPEC must specify option (a) explicitly: `runner.rs:148` adds `mask: Vec::new()`, and `spawn_and_capture` overwrites it after the call. Without this, the implementer has no obvious fix path and may choose (c) and compromise the runner layer's abstraction.

**Fix:** Add to the D3 wiring paragraph: "`RunResult` gains `pub mask: Vec<bool>` initialised `Vec::new()` in `run_with_stdin`'s struct literal (`runner.rs:148`). `spawn_and_capture` receives the `mask: Vec<bool>` parameter and assigns `result.mask = mask` immediately after the `run_with_stdin` call returns `Ok(result)` — before storing in `app.last_run`."

---

## Minor

**M1 — Line citation drift: secret positional push is at line 302, not 303.**

The SPEC (Problem section, item 3, and Design §1) cites `invocation.rs:303` for the secret positional `argv.push`. The actual push is at line 302: `argv.push(value.as_str().to_string())` (the `let value = w.as_string();` is line 301, `argv.push` is line 302). Line 303 is a closing `}`. Not a security gap but will confuse an implementer or auditor who greps to verify.

**Fix:** Change `:303` → `:302` in the two occurrences in the Problem and Design sections.

**M2 — The deferral of round-1 I1 (CHANGELOG over-claim) carries a live falsehood for another cycle.**

`CHANGELOG.md:1978-1980` (in the v0.2-era entry) says `cell_paste_warn_modal_trigger` "validates the paste-warn modal text and behavior on `SecretLineEdit` paste events." The test's own comment (widget_secret.rs:18-24) documents that the live app-state check is deferred — the test only validates the `should_warn_on_paste` predicate. `PASTE_WARN_MODAL_TEXT` and `should_warn_on_paste` are dead code with no live caller in `src/`.

The SPEC's deferral rationale ("until then the claim stays a documented-but-not-wired item") is defensible: the claim has been stale since v0.2, one more cycle doesn't change the severity, and correcting the claim while the wiring is absent makes the CHANGELOG state "broken" rather than "aspirational." The v0.40.0 cycle where wiring lands is the right place to make the claim true AND honest simultaneously.

However, the SPEC should explicitly note that `CHANGELOG.md:1978-1980` and `CHANGELOG.md:2196` (the paste-warn modal reference in the non-goals section) are the specific stale lines — the line numbers are stated in the round-1 review as snapshots and the SPEC's current text just says "the v0.31.1 partial self-correction already flags half." Giving the implementer precise line numbers prevents re-grepping and possible miss.

**Fix (advisory, not gate):** Add to the §Deferred paragraph: "The specific stale CHANGELOG lines are `1978-1980` (widget_secret.rs description) and `2196` (non-goals paste-warn modal reference) — re-verify at v0.40.0 fold time, as these line numbers decay."

**M3 — T-A3 test wording: the "safe asymmetry" pin does not check the DANGEROUS direction.**

T-A3 asserts `mask.any() ==> should_confirm_run` but the test wording only asks to check that the asymmetry exists (the Boolean `*-stdin` case where `should_confirm_run = true` but `mask.all_false`). The SPEC does not specify a negative test for the dangerous direction: a state where `mask.any() = true` but `should_confirm_run = false` should be IMPOSSIBLE. This should be asserted as a universal property over all T-A1 test vectors, not just documented prose.

The test as described is: "for each populated-secret state in T-A1, `mask.iter().any()` is true AND `should_confirm_run` is true." This IS the dangerous-direction check implicitly (if both are true, the dangerous scenario `mask.any() && !should_confirm_run` is excluded for those states). But it only checks the T-A1 specific states, not exhaustively.

**Fix:** Add an explicit assertion: for every T-A1 state where `mask.any() = true`, ALSO assert `should_confirm_run(sub, state) = true` (making the dangerous direction explicit as a per-case invariant, not just prose). The test spec already implicitly does this but should call it out: "Assert no masked-token state passes `should_confirm_run = false` — this is the dangerous direction."

---

## Confirmations (R0 gates answered)

**Tree-mode deferral safety confirmed:** All 17 `build-descriptor` node kinds (`pk`, `pkh`, `multi`, `sortedmulti`, `older`, `after`, `sha256`, `hash256`, `hash160`, `ripemd160`, `and_v`, `or_d`, `or_i`, `or_b`, `andor`, `thresh`, `wrap`) have payloads that are keys, hashes, unsigned integers, or child-node references (`nodes.rs:63-104`). None are in `SECRET_NODE_TYPES = ["phrase", "entropy", "xprv", "wif", "ms1", "bip38", "electrum-phrase", "seedqr"]` (`secrets.rs:42-53`). Deferring tree-mode pipeline masking to a FOLLOWUP is **safe**.

**Mask completeness confirmed:** The four secret-token sources in the SPEC are exhaustive against `should_confirm_run`'s four arms. The T-A3 asymmetry (`mask.any() ==> should_confirm_run` but NOT `<=>`) is correct: Boolean `*-stdin` flags trigger `should_confirm_run` (arm 1 via `flag_is_secret && has_value`) but emit no token (suppressed at `invocation.rs:278`) → mask stays all-false. The dangerous direction (`mask.any() && !should_confirm_run`) is structurally impossible because every `mask[i]=true` site corresponds to a source covered by one of `should_confirm_run`'s four arms.

**PinValue is not a secret leak:** The only live `PinValue` use is `--account → PinValue(0)` (`conditional.rs:241`), a Number flag with value 0. Not secret-bearing. The `pin_value_to_argv_token` path at `invocation.rs:179-188` correctly gets `mask.push(false)` per the SPEC.

**`argv_posix` alias-split is correct:** After masking, `preview = render_copy_command_masked(...)`. The SPEC correctly identifies that `argv_posix = preview.clone()` at `:934` would alias the masked string into the copy path. The fix (recompute `argv_posix = render_copy_command(&argv, Posix)` as the real command) breaks the alias. The tree-mode POSIX pipeline at `tree_form.rs:130` independently calls `render_copy_command(argv, Posix)` on the real argv — this path is already the reveal path (copy is intentional) and requires no change.

**SemVer MINOR confirmed.** No flag-name additions, no `secret:` bit changes, no subcommand additions. The `schema_mirror` gate is flag-name–parity only (per CLAUDE.md); purely internal function renames and new private functions (`assemble_argv_with_secret_mask`, `render_copy_command_masked`, `to_slot_argv_masked`) do not affect the clap surface. No manual impact. MINOR is correct.

**`pending_confirm_argv` readers are fully enumerated:** 6 sites in `main.rs` only (`grep` confirmed: lines 108, 314, 897, 1007, 1015, 1030, 1034). No other file references it. The 3-tuple expansion `(Vec<String>, Vec<bool>, Option<Vec<u8>>)` touches exactly these sites.
