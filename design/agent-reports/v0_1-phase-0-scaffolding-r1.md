# Phase 0 Scaffolding Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Single commit — `scaffold mnemonic-gui v0.1 Phase 0 (repo skeleton + dependency pins)`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §C Phase 0
**Exit-gate status:** PASS (`cargo build` clean; `cargo test` 0 tests; `pinned-upstream.toml` resolves all four sibling repos)

## Verdict

**0C / 2I — fold needed**

Two important findings. Neither blocks the Phase 0 exit gate (already passed). Both require resolution before Phase 1 begins: I-1 is a SPEC document fix; I-2 is a one-line README fix.

---

## Important findings

### I-1: `src/lib.rs` and `src/form/mod.rs` absent from SPEC §B.2 normative source tree

**Confidence:** 85
**Files:** `src/lib.rs` (new), `src/form/mod.rs` (new)
**SPEC ref:** §B.2 — "Source tree (normative; deviations from this layout require SPEC amendment)"

**What:** SPEC §B.2's source tree does not list `src/lib.rs` or `src/form/mod.rs`. Two scaffolded files are therefore unlisted structural deviations by the SPEC's own rule.

- `src/lib.rs`: dual-target lib+bin pattern is correct (integration tests under `tests/` cannot reach module internals via a pure `[[bin]]` crate — they require a `[lib]` target). The SPEC §B.2 tree must be amended to list `src/lib.rs`.
- `src/form/mod.rs`: §B.2 explicitly lists `src/schema/mod.rs` but silently omits `src/form/mod.rs`. The `form/` directory requires a `mod.rs` for `rustc` to compile it.

**Impact:** Without SPEC amendment, every future phase reviewer must independently adjudicate whether these two files are expected.

**Fix:** Fold a §B.2 amendment into the plan (inline, no new review round required):
```
src/lib.rs              — library crate root; dual-target lib+bin for
                          integration-test accessibility.
src/form/mod.rs         — form module root.
```

---

### I-2: README.md hardcodes a local machine absolute path

**Confidence:** 95
**File:** `README.md` line 37
**Text:** `` See `/home/bcg/.claude/plans/declarative-tumbling-shell.md` ``

**What:** Design section points to a file on the author's local machine. The plan is not committed to the repo and the path is meaningless on any other machine or in GitHub's rendered README.

**Impact:** First-impression orientation document carries a broken reference; will cause confusion at Phase 10 push.

**Fix:** Replace with a pointer to checked-in artifacts (`design/agent-reports/`), or commit the plan into `design/` per the constellation's convention. The Phase 0 fold uses the first option.

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|-----------|-------------|
| `toml = "0.8"` in `[build-dependencies]` not mentioned in IMPL_PLAN Phase 0 step 3 | 55 | Anticipatory; SPEC §B.11 mandates TOML parsing by `build.rs`. Not a bug. |
| `tracing-subscriber` `env-filter` feature not mentioned in Phase 0 IMPL_PLAN | 50 | Required by Phase 4 step 5; anticipatory. |
| `main.rs` `let _ = mnemonic_gui::app::placeholder;` idiom | 40 | Sound function-item coercion; replaced at Phase 4. |
| `[ms].workspace-member-path = "crates/ms-cli"` | — | Verified correct against local `mnemonic-secret` checkout. |
| `design/agent-reports/` not pre-created in commit | — | Created by writing this report. Not a defect. |
| No remote in `.git/config` | — | Local-first; remote added at Phase 10. |

---

## Deliberation on the six raised points

1. **`src/lib.rs` SPEC deviation** — Real deviation requiring amendment (see I-1).
2. **`toml = "0.8"` in build-deps before Phase 7** — Correct anticipatory addition. IMPL_PLAN Phase 0 step 3 mentions only `syn` (silent on `toml`).
3. **`[ms].workspace-member-path = "crates/ms-cli"`** — Correct; verified.
4. **`main.rs` linking idiom** — Acceptable Phase 0 stub.
5. **`directories = "5"` pin** — Matches SPEC §C Phase 0 step 3.
6. **Missing files check** — All §B.2 files present; two extras (`lib.rs`, `form/mod.rs`) → I-1.
7. **`Cargo.lock` committed** — Confirmed retained per plan.
