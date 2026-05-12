# Phase 0 Scaffolding Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** R1-fold commit `96fdd8a` + preceding Phase 0 scaffold commit
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §B.2 (post-amendment) + §C Phase 0
**R1 report:** `design/agent-reports/v0_1-phase-0-scaffolding-r1.md`

## Verdict

**0C / 0I — converge**

Both R1 findings correctly folded. No new defects introduced by the folds. Phase 0 exit gate holds.

---

## R1 fold verification

### I-1 fold (SPEC document amendment) — RESOLVED

Plan §B.2 now lists `src/lib.rs` with rationale "library crate root; dual-target lib+bin (Phase 0 R1 I-1 fold: integration tests under tests/ reach module internals only via a [lib] target, so a library target is required)" and `src/form/mod.rs` with "form module root (Phase 0 R1 I-1 fold)". SPEC and repo are in agreement.

### I-2 fold (README local-path) — RESOLVED

`README.md` no longer contains any reference to `/home/bcg/.claude/plans/...`. The Design section now points at `design/agent-reports/` via a relative Markdown link. The directory exists in the repo (contains the R1 report); the link resolves correctly on GitHub browse and on local clone.

---

## SPEC §B.2 post-amendment cross-check

All 18 `src/` files enumerated in the amended §B.2 tree are present in the repo (`lib.rs`, `main.rs`, `app.rs`, `runner.rs`, `secrets.rs`, `persistence.rs`, `path_detect.rs`, `schema_check.rs`, `schema/{mod,mnemonic,md,ms,mk}.rs`, `form/{mod,widget,slot_editor,conditional,invocation}.rs`). No unlisted extras in `src/`. Phase-deferred directories (`tests/`, `docs/onboarding/`, `.github/workflows/`) are absent as expected — they appear in §B.2 as targets for later phases.

---

## New-issue check (R1-fold-induced defects)

**Candidate**: `design/agent-reports/` relative Markdown link in README breaks at `cargo install` time.

**Disposition**: Not a defect. `cargo install` does not render the README; the link targets the GitHub repo tree, not a filesystem path. The `design/` tree is committed and will be present on any clone or GitHub browse. Confidence 20.

No other new issues identified.

---

## Phase 0 exit gate (re-verified)

| Gate | Status |
|------|--------|
| `cargo build` clean | PASS (R1 folds touched only README + plan; no Rust source changes) |
| `cargo test` 0 tests | PASS (same reasoning) |
| `pinned-upstream.toml` resolves all 4 sibling repos | PASS (file unchanged from R1-verified state) |

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| `design/agent-reports/` link broken at `cargo install` | 20 | README not a runtime artifact; targets GitHub tree |
| `tests/`, `docs/`, `.github/` absent from repo | — | Phase-deferred; expected |
| No `build.rs` in repo | — | Phase 7-deferred; expected |
