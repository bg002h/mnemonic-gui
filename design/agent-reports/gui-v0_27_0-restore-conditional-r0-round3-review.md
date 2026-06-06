# R0 Architect Review (round 3) — SPEC mnemonic-gui v0.27.0 (restore conditional consume + README pin guard)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `gui-v0.27.0-restore-conditional-consume-readme-guard`. **Verdict:** **0 Critical / 0 Important.** **GREEN — implementation may proceed.**

> Persisted verbatim per CLAUDE.md. Convergence round: I2 fold correct + complete; stale-comment CLASS fully enumerated (exactly 3 live in-scope sites, all in §5; no 4th); no new drift.

---

## VERDICT: 0 Critical / 0 Important (+ 0 new Minor; M2 pre-existing, out of scope)

**GREEN — implementation may proceed.**

The round-2 I2 fold is correct and complete, the stale-comment class is fully enumerated (exactly 3 live in-scope sites, all now in §5; no 4th instance), and the two folds introduced no new drift or self-contradiction.

---

### Item 1 — I2 fold correct + complete (CLEAN)

SPEC §5 now directs **rewriting** `src/schema/mnemonic.rs:3454-3457` (the stale `conditional_rules: []` / "not drift-gated, like repair/inspect" comment on the `restore` `SubcommandSchema` linchpin), with the correct replacement (toolkit v0.46.2 now projects the rule → restore IS drift-gated via `("restore", 1)` in `SUBCOMMAND_FLOORS`). Verified against the live comment.

The round-2 self-contradiction is resolved: the "ONLY" wording was re-scoped — now governs *don't touch the provenance comment* (`:2746`), not *don't touch `:3454`*. Version sites preserved: `:1` module-doc, `:3687` pinned_version; `:2746` provenance MUST-NOT-TOUCH still present.

### Item 2 — Stale-comment CLASS fully enumerated; NO 4th instance (CLEAN — the round's central job)

Phrase-based content grep (`conditional_rules: \[\]` / `no restore arm` / `not drift-gated` / `GUI-authored` / `hand-encoded allowlist`) over the whole repo, corroborated by a broader-net grep.

**Exactly 3 live in-scope stale sites, all now directed in §5:**
1. `src/form/conditional.rs:926-934` — I1, §5 directs rewrite. 4 false clauses present; body `:935-941` matches (no logic change).
2. `tests/conditional_visibility.rs:1075-1078` — M1, §5 directs touch-up; test stays GREEN (calls `run_conditional` directly at `:1084`).
3. `src/schema/mnemonic.rs:3454-3457` — I2, §5 directs rewrite.

**Out-of-scope hits correctly excluded:** `mnemonic.rs:3575-3578` (`ms-shares-split`, genuine `conditional: None` / accurate `[]`); `CHANGELOG.md:8/12/16` (historical); `design/` docs + the prior reviews. **No 4th in-scope stale site exists.**

### Item 3 — No new drift; SPEC internally consistent (CLEAN)

§3/§4/§5/§7/§8 re-read; no contradiction. Version-site list matches the round-1 8-site map. Additionally examined the floor-comment block at `tests/gui_schema_conditional_drift.rs:285-299`: `:299` "Total floor: 36 -> 34 in lockstep" is **chronological changelog narration of the v0.7.2 revert** (paired with `:293-298`), NOT a live current-total assertion — stays true after a later +1. The live total assertion is `:318/:321` `(sum = 34)` / `>= 34`, which SPEC §3 correctly directs bumping to 35 / `>= 35`. So `:299` is correctly out of scope; not a finding.

### Item 4 — Core premise re-confirmed (CLEAN)

- No flag delta v0.46.0→v0.46.2 → `schema_mirror` clean catch-up.
- Restore emits exactly 1 rule (toolkit `cmd/gui_schema.rs:354-372` returns single `ConditionalRule`; dispatch arm `:344`).
- FLOORS `("restore",1)` + total `>=34 → >=35` correct (11+10+6+4+3+1 = 35).
- GUI `conditional::restore` (`:937-939`) shape-identical to the projection.
- FLOORS+pin coupling: RED against v0.46.0 (`[]` → `0 >= 1`), GREEN only with the v0.46.2 pin; `pin_coherence` gates Cargo↔pinned-upstream.

Rounds 1-2's clean findings all still stand.

---

**GREEN — 0 Critical / 0 Important. Implementation may proceed.**
