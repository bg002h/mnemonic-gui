# Phase C.3 Pinned-Upstream Upgrade — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** commit `c36db3a` ("feat(v0.2 phase C.3): flip gui-schema-capable + bump pinned tags")
**Plan ref:** `/home/bcg/.claude/plans/v0_2-mnemonic-gui.md` §C Phase C.3; SPEC §7 + §11

## Verdict

**0C / 1I / 2N → FOLD-AND-MERGE.** All four consistency axes
(`pinned-upstream.toml` tags, `schema-mirror.yml` install + clone
tags, `schema/*.rs` `pinned_version` fields, `ci_workflow_snapshot`
`required_tags` array) agree. Test invariants flipped correctly.
Smoke-step Python free of shell injection. Skip-logic in
`schema_check_json_invokes_gui_schema_on_capable_cli` correctly
gates dev-laptop runs while still failing for reachable-but-broken
binaries.

The single Important is a one-line stale doc string in
`src/schema/mk.rs`.

## I-1 — stale module-doc tag in `src/schema/mk.rs`

`src/schema/mk.rs:1` reads `mk-cli-v0.2.0` after the tag-bump commit;
the same file's `SCHEMA::pinned_version` correctly reads `"mk 0.3.0"`
and `pinned-upstream.toml` correctly pins `tag = "mk-cli-v0.3.0"`.
Drift introduced here.

## N-2 — pre-existing stale module-doc tags

`src/schema/mnemonic.rs:1` (`v0.8.1`) and `src/schema/md.rs:1`
(`v0.4.3`) are pre-existing drift. Both fall under the same
cleanliness invariant; folded together with I-1.

## N-1 — skip-logic non-distinguishing failure modes

`tests/schema_mirror.rs::schema_check_json_invokes_gui_schema_on_capable_cli`
treats "binary not found" and "binary found but `gui-schema` missing"
identically (both `eprintln` + skip). Acceptable on dev laptops; CI
always installs all four binaries so the path is never taken. Carry
forward as a future tightening if the population becomes more
heterogeneous.

## Fold at commit `<TBD-fold-sha>`

- `src/schema/mk.rs:1` → `mk-cli-v0.3.0` (I-1).
- `src/schema/mnemonic.rs:1` → `mnemonic-toolkit-v0.9.0` (N-2).
- `src/schema/md.rs:1` → `descriptor-mnemonic-md-cli-v0.5.0` (N-2).

Both `cargo test --workspace` (122 passed, 0 failed across 15
binaries) and `cargo clippy --all-targets -- -D warnings` clean
post-fold.

## R2 self-clear

All three folds are mechanical comment edits that do not affect any
runtime path. Re-review unnecessary. **Disposition: MERGE.**
