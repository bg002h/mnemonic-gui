# Phase C.2 Sibling-Repo gui-schema PRs — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** 4 sibling-repo PRs implementing SPEC §7 gui-schema subcommand
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase C.2; SPEC §7

## Verdict overall

| PR | Repo | Status | Verdict |
|----|------|--------|---------|
| #8 | mk-cli v0.3.0 | MERGED (pre-review) | RETROACTIVE OK — 0C/0I/1N |
| #5 | ms-cli v0.2.0 | MERGED (pre-review) | RETROACTIVE OK — 0C/0I/1N |
| #14 | mnemonic-toolkit v0.9.0 | OPEN | MERGE — 0C/0I/1N |
| #29 | md-cli v0.5.0 | OPEN | FOLD-AND-RE-REVIEW — 1C/2I/2N → folded at d725b48 → R2 self-clear |

The two pre-existing merges (#8, #5) were squash-merged by the original autonomous agents before the user's "architect-review-each-PR" direction landed; user subsequently OK'd post-merge review.

## Cross-PR consistency check

All four emit the same JSON shape (`version` + `cli` + `subcommands{name, flags, positionals}`). All correctly exclude `gui-schema` and `help`. All bump version, add CHANGELOG, carry Companion line.

Three drift items, all in PR #29 pre-fold: pretty-printed vs compact JSON (I-1); fragile Debug-string heuristics for kind classification (I-2); Cargo.toml not bumped to match CHANGELOG (C-1). All three folded at d725b48.

---

## PR #8 (mk-cli v0.3.0, MERGED) — RETROACTIVE OK

Contract correct: version 1, cli "mk", compact JSON, TypeId-based path detection, SetTrue/SetFalse → "boolean", restricted-value-parser → "dropdown". 7 integration tests cover exit-0, JSON parse, envelope, encode-flag spot-check, self-reference exclusion, kind classification, repeating-positional. N-1: no numeric flags in surface; mapping to "text" documented. **Disposition: RETROACTIVE OK.**

## PR #5 (ms-cli v0.2.0, MERGED) — RETROACTIVE OK

Contract correct: version 1, cli "ms", compact JSON. 11 integration tests including hyphenated `chinese-simplified`/`chinese-traditional` Chinese-language dropdown assertion (matches D.1 finding #1). N-1: `ArgAction::Append` not in positional repeating check; latent gap only (no Append-positional in ms-cli). **Disposition: RETROACTIVE OK.**

## PR #14 (mnemonic-toolkit v0.9.0, OPEN) — MERGE

Most complete implementation. TypeId-based numeric detection over 14 primitives (u8-u64, i8-i64, f32, f64, usize, isize), TypeId-based path detection. Alphabetic sort for determinism. 16 integration tests including exact subcommand-list equality + dropdown choice-list assertions. **Disposition: MERGE.**

## PR #29 (md-cli v0.5.0, OPEN) — FOLD-AND-RE-REVIEW → MERGE post-fold

R1 returned 1C/2I/2N:
- **C-1:** Cargo.toml still 0.4.3 vs CHANGELOG 0.5.0 (one-line). Reverted by git stash round-trip during initial commit cycle.
- **I-1:** `to_string_pretty` vs siblings' compact output.
- **I-2:** Debug-string `.contains("path_buf"|"other(")` heuristics — clap internal not contract-stable.

**Fold at d725b48:**
- C-1: Cargo.toml line 3 → `version = "0.5.0"`.
- I-1: `serde_json::to_string_pretty` → `serde_json::to_string`. Tests parse JSON; no test-side change.
- I-2: Debug-string heuristics → `TypeId` comparisons. Path: `TypeId::of::<PathBuf>()` post-ValueHint branch. Numeric: 10 primitive integer TypeIds. Matches mk-cli + mnemonic-toolkit.

**Side effect of I-2:** `vectors --out` (declared `Option<String>` in md-cli) now classifies as "text" rather than "path". This is MORE accurate per SPEC §7 (String-typed, not PathBuf). Previous "path_buf" debug-string match was matching inner parser repr, not actual type. Test at `cmd_gui_schema.rs:110` (kind-in-valid-set sweep) still passes.

**R2 (self-review):** all 3 folds mechanical and R1-pre-approved. `cargo test --release -p md-cli` all green; `cargo clippy --release -p md-cli -- -D warnings` clean. **Disposition: MERGE.**

---

## Cross-PR consistency (post-fold)

| Aspect | mk #8 | ms #5 | mnemonic #14 | md #29 |
|--------|-------|-------|--------------|--------|
| version=1, cli string | ✓ | ✓ | ✓ | ✓ |
| Compact JSON | ✓ | ✓ | ✓ | ✓ (post-fold) |
| Self-reference excluded | ✓ | ✓ | ✓ | ✓ |
| TypeId-based kind | ✓ | ✓ (no path/numeric) | ✓ | ✓ (post-fold) |
| Cargo.toml bumped | ✓ | ✓ | ✓ | ✓ (post-fold) |
| CHANGELOG + Companion | ✓ | ✓ | ✓ | ✓ |
| Test coverage | 7 | 11 | 16 | adequate post-fold |

All four PRs satisfy the SPEC §7 contract. PRs #14 and #29 safe to merge + tag.
