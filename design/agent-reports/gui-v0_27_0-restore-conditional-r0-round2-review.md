# R0 Architect Review (round 2) — SPEC mnemonic-gui v0.27.0 (restore conditional consume + README pin guard)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `gui-v0.27.0-restore-conditional-consume-readme-guard`. **Verdict:** **0 Critical / 1 Important** (+ 2 Minor: M1 folded, M2 pre-existing).

> Persisted verbatim per CLAUDE.md BEFORE the fold. I2 (NEW): a THIRD stale-class comment the I1/M1 fold missed — the "fix the CLASS, hunt for the 2nd/3rd occurrence" discipline. Fold → re-dispatch round 3.

---

## VERDICT: `0 Critical / 1 Important` (+ 2 Minor: M1 folded, M2 pre-existing)

**NOT YET GREEN.** The round-1 I1 + M1 folds are correct and complete, but verifying the fold's *class* surfaced a **third live instance** of the same stale-comment class that the fold did not catch — and §5's wording affirmatively instructs leaving it stale. One new Important. Fold it and re-dispatch round 3.

---

## Important

**I2 (NEW) — §5 omits the third stale `restore emits []` comment, and its "ONLY" wording affirmatively instructs leaving it stale: `src/schema/mnemonic.rs:3454-3457`.**

The comment sitting directly on the `SubcommandSchema { name: "restore", conditional: Some(crate::form::conditional::restore) }` entry (`:3458-3464`) makes two clauses that the pin bump turns FALSE:
- `:3454-3456` — *"The toolkit gui-schema still emits `conditional_rules: []` for restore (hand-encoded allowlist has no restore arm)"* — false: the arm now exists at toolkit `gui_schema.rs:344` (`"restore" => restore_conditional_rules()`), emitting exactly 1 rule.
- `:3457` — *"(not drift-gated, like repair/inspect)"* — false: activating that drift-gating is slug 1's entire purpose.

This is the **same class** as I1 (`conditional.rs`) and M1 (`conditional_visibility.rs`), and it is **production code on the most load-bearing spot** — the very `SubcommandSchema` linchpin that round-1 Item 2 named as the FLOORS-path entry point (`:3459` name, `:3464` `conditional: Some`). Round 1 rated the parallel `conditional.rs` doc-comment **Important**; the same logic forces this to Important, not Minor (M1 was Minor only because it's a test-file comment).

The defect is sharpened by §5 line 58, which says bump *"**ONLY** the module-doc (`:1`) + `pinned_version` (`:3687`)"*. That word **ONLY** does not merely omit `:3454-3457` — it affirmatively instructs the implementer (already editing this file) to leave it. An implementer following §5 literally ships a stale, self-contradictory comment on the linchpin restore entry.

**Fix:** Expand the §5 `src/schema/mnemonic.rs` bullet to also rewrite `:3454-3457` (toolkit v0.46.2 now projects the restore rule → restore IS drift-gated by `gui_schema_conditional_drift` with `("restore", 1)` in `SUBCOMMAND_FLOORS`; drop "not drift-gated, like repair/inspect"). Soften the "ONLY" wording so the bullet is not contradicted by its own edit-list. No logic change (the `conditional: Some(...)` wiring already matches).

---

## What verified clean (with confirming citations)

**I1 fold — correct + complete (CLEAN).** The actual doc-comment at `conditional.rs:916-941` matches the SPEC's description: all four false clauses present. SPEC §5's `:926-934` range covers all four. The proposed replacement is factually accurate against toolkit `b74badd`: `gui_schema.rs:344` arm exists; `:354-371` `restore_conditional_rules()` returns exactly one rule. The fn body `:935-941` unchanged. Correct.

**M1 fold — correct (CLEAN).** `tests/conditional_visibility.rs:1076-1078` carries the exact stale-class text; §5 directs the one-line touch-up; the test stays GREEN (calls `run_conditional` directly). Correct.

**No new drift from the fold; version-site list complete (CLEAN).** Re-grep of `0.46.0` (excl. `design/`) = exactly the sites §5 lists; MUST-NOT-TOUCH `:2746` provenance + `CHANGELOG.md:8` historical confirmed; §3/§4/§5/§7/§8 mutually consistent.

**Core premise (fresh adversarial pass) (CLEAN):** (4a) no flag delta v0.46.0→v0.46.2 → schema_mirror clean catch-up; (4b) restore emits exactly 1 rule → FLOORS `("restore",1)` + total `>=34→>=35` right; (4c) GUI `conditional::restore` matches; (4d) slug-2 README parser sound (5 lines + 4 pinned-upstream tags exist, split_whitespace required); (4e) FLOORS+pin coupling (RED without pin). Round 1 correct on all.

**Class fully enumerated (item 5).** 3 live src/test sites — `conditional.rs:926-934` (I1), `conditional_visibility.rs:1076-1078` (M1), `mnemonic.rs:3454-3457` (I2 NEW). 2 folded, 1 missed → exactly one new finding. Out-of-scope hits correct: `mnemonic.rs:3578` is `ms-shares-split` (legitimately `[]`); `CHANGELOG.md` historical. No other test breaks on the pin bump or FLOORS change.

**M2 (pre-existing, out of scope).** `compare-cost` feeds `total_rules` but is unfloored; the `>= 35` lower bound is satisfied by the 6 floored subcommands.

---

## Tooling limitation (does not block — consistent with round 1)
Reviewer had no shell; substituted toolkit source ground-truth at `b74badd` (`restore_conditional_rules()` returns one rule) + the §2 operator-captured `gui_schema_conditional_drift` GREEN 5/5 + CHANGELOG. Same deliberate substitution round 1 used.

---

## Gate decision
Fold **I2** (expand the §5 `src/schema/mnemonic.rs` bullet to rewrite `:3454-3457`; soften the "ONLY" wording) and **re-dispatch round 3**. **Not GREEN yet — 1 Important open.**
