# R0 Architect Review (round 1) — SPEC mnemonic-gui v0.27.0 (restore conditional consume + README pin guard)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `gui-v0.27.0-restore-conditional-consume-readme-guard` (off master `f6caa20`). **Verdict:** **0 Critical / 1 Important** (+ 2 Minor).

> Persisted verbatim per CLAUDE.md BEFORE the fold. I1 (stale doc-comment on the central `restore()` fn) must be folded into §5 → re-dispatch. Empirical drift-gate GREEN-vs-v0.46.2 was run by the operator (reviewer had no shell); design soundness confirmed via source + CHANGELOG + FLOORS-path hand-trace.

---

### VERDICT: `0 Critical / 1 Important` (+ 2 Minor)

**NOT YET GREEN.** One Important doc-accuracy omission must be folded into the SPEC's §5 edit-list before implementation. Everything substantive (the pin bump, drift-gate activation, FLOORS arithmetic, slug-2 guard design, version-site map) verified sound.

---

### Important

**I1 — SPEC §5 omits the now-stale `conditional::restore` doc-comment rewrite (`src/form/conditional.rs:926-934`).**
The doc-comment on the central `restore()` fn — the very function this cycle drift-gates — makes four clauses that the pin bump turns FALSE:
1. `:927-929` "hand-encoded allowlist ... with **no `restore` arm**" — false: the arm now exists at `gui_schema.rs:344` (`"restore" => restore_conditional_rules()`).
2. `:929-930` "so restore emits **`conditional_rules: []`**" — false: v0.46.2 emits exactly 1 rule.
3. `:930-931` "this rule is therefore **not covered by `gui_schema_conditional_drift`**" — false: activating that coverage IS slug 1's purpose.
4. `:933-934` "Promotion ... **tracked as a toolkit FOLLOWUP**" — the FOLLOWUP `gui-schema-restore-required-unless-md1-projection` is resolved this cycle (SPEC §7).

This project demonstrably gates on doc accuracy (README-version guard `readme_version_current.rs`, the manual-mirror invariant, and a recurring stale-doc FOLLOWUP history per MEMORY). The central function's doc directly contradicting the cycle's purpose should not ship unlisted.
**Fix:** Add to SPEC §5 a bullet rewriting `conditional.rs:926-934` to state that toolkit v0.46.2 NOW projects the restore rule (`gui_schema.rs:344`), it is drift-gated by `gui_schema_conditional_drift` with `("restore", 1)` in `SUBCOMMAND_FLOORS`, and drop the "tracked as a FOLLOWUP" sentence (resolved). No logic change — the fn body (`:935-941`) already matches.

---

### Minor

**M1 — Stale comment in `tests/conditional_visibility.rs:1076-1078`** ("hand-encoded allowlist with no restore arm ... so this is not drift-gated") becomes factually stale post-bump. The test itself stays GREEN (it invokes `run_conditional` directly, not via the drift path), so this is cosmetic — but worth a one-line touch-up alongside I1 for the same reason.

**M2 — `compare-cost` feeds `total_rules` but is unfloored** (pre-existing). The `total_rules >= 35` lower bound is satisfied by the 6 floored subcommands' sum (35), so compare-cost's rules are headroom. Pre-existing, out of scope; noting only so the floor arithmetic isn't misread as exact.

---

### What verified clean (with confirming citations)

**Item 1 — No flag delta v0.46.0→v0.46.2; `schema_mirror` is a clean catch-up (CLEAN).**
- Toolkit `CHANGELOG.md:18-24` (v0.46.1): "No CLI-surface change." `:9-14` (v0.46.2): "No clap flag/value/subcommand change → no `schema_mirror` ... stays v5."
- The `--passphrase-candidates-file` flag (only v0.46.x CLI add) landed in v0.46.0, already pinned + in GUI schema at `schema/mnemonic.rs:2750`. No `RESTORE_FLAGS`/schema flag backfill owed.

**Item 2 — Drift-gate activation correct; restore emits EXACTLY 1 rule; FLOORS arithmetic right (CLEAN).**
- Toolkit `gui_schema.rs:354-372` `restore_conditional_rules()` returns one `ConditionalRule`: `when: Not(FlagPresent "--md1") → effect: {--from, Required}`.
- GUI `conditional.rs:935-941` `restore()` emits `if !state.has_value("--md1") { vis.push(("--from", Required)); }` — shape-identical.
- **Linchpin:** `schema/mnemonic.rs:3459` `name: "restore"` + `:3464` `conditional: Some(crate::form::conditional::restore)` → FLOORS path (`drift.rs:231` Some, `:236` Some → `:247` `insert("restore",1)`) → `1 >= 1` passes.
- `SUBCOMMAND_FLOORS` (`drift.rs:300-306`) + `total_rules >= 34` (`:320-321`) accurate. New total 11+10+6+4+3+1 = **35** → `>= 34 → >= 35` correct.

**Item 3 — FLOORS+pin coupling sound (CLEAN).** v0.46.0 → restore `[]` → `0 >= 1` RED. So `("restore", 1)` valid ONLY with the v0.46.2 pin; SPEC §3/§8 couple them. `pin_coherence` (`tests/pin_coherence.rs:24-37`) asserts Cargo.tag == pinned-upstream[mnemonic].tag; both v0.46.2 → GREEN.

**Item 4 — slug-2 guard well-specified; README parses (CLEAN).** All 5 install lines exist (README `:42` self, `:50` toolkit, `:51` md, `:52` ms, `:53` mk). `pinned-upstream.toml` carries `[mnemonic]:22`, `[md]:39`, `[ms]:46`, `[mk]:53`. pkg→section mapping correct; GUI self-tag → `Cargo.toml.version`. Lines use alignment padding → impl must use `split_whitespace()`. Mirror `tests/pin_coherence.rs` style. Recommend the new guard demonstrably go RED on a deliberately-stale pin.

**Item 5 — Version-bump sites accurate; anti-blind-sed confirmed (CLEAN).** Repo-wide `0.46.0` grep (excl. `design/`) = exactly 8 sites: BUMP `Cargo.toml:42`, `pinned-upstream.toml:22`, `Cargo.lock:2296-2297`, `schema/mnemonic.rs:1`, `:3687`, README `:50`; GUI-version bump README `:42`, `Cargo.toml:3`; MUST-NOT-TOUCH `schema/mnemonic.rs:2746` (provenance) + `CHANGELOG.md:8` (historical v0.26.0). SPEC §5 edit-list complete.

**Item 6 — SemVer + scope correct (CLEAN).** v0.24→v0.25→v0.26 all MINOR catch-up cycles; 0.26→0.27 MINOR mirrors. GUI-repo-only. manual-gui out of scope. Both toolkit FOLLOWUPs flip post-tag.

**Item 7 — lib const-assert compiles; no other version-pin test breaks (CLEAN).** No new secret flag → `secrets.rs` re-export unchanged. `secret_taxonomy_pin.rs` min-membership unaffected. `conditional_visibility.rs` restore cells invoke the fn directly, stay GREEN. `cli_gui_schema` subcommand freeze unaffected.

---

### Tooling limitation (does not block — R0 is design-soundness)
Reviewer had no shell; `gui_schema_conditional_drift`/`schema_mirror` not run empirically by the reviewer — substituted source-of-truth at toolkit `b74badd` + CHANGELOG + FLOORS-path hand-trace. (Operator independently ran `gui_schema_conditional_drift` GREEN 5/5 vs the v0.46.2 binary pre-SPEC.)

---

### Gate decision
Fold **I1** (add the `conditional.rs:926-934` doc-rewrite to SPEC §5; M1's `conditional_visibility.rs:1076-1078` comment touch-up rides along) and re-dispatch R1. **Not GREEN yet — 1 Important open.**
