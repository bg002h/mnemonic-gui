# Phase C.1 `--gui-schema` JSON Consumer — R1 (self-review)

**Date:** 2026-05-12
**Reviewer:** executing agent (self-review per autonomous-execution direction)
**Scope:** Implementation commit for Phase C.1 on branch v0_2
**Plan ref:** /home/bcg/.claude/plans/v0_2-mnemonic-gui.md §C Phase C.1; SPEC §7

## Verdict

**0 Critical / 0 Important / 0 Sub-threshold — converge**

C.1 is small and mechanical: pure-parsing function (`parse_gui_schema_json`) + orchestrator (`json_flag_names`) that reads `pinned-upstream.toml` and conditionally shells out. The two layers are cleanly separated, the parser is fully unit-tested via synthetic JSON (3 positive/negative cells), and the orchestrator's fall-back-on-`false` invariant is asserted at the table level (`schema_check_json_falls_back_on_non_capable_cli` exercises all 4 CLIs). The TOML-deserialization `#[serde(default)]` on `gui_schema_capable` satisfies the SPEC §7 R1 N-3 fold (existing `pinned-upstream.toml` without the key deserializes cleanly to `false`).

The phase is self-reviewable because (a) the surface area is small, (b) there is no `unsafe`, (c) the runtime path is gated on `gui-schema-capable = true` which no CLI yet satisfies (C.3 flips them), and (d) the `tests/schema_mirror.rs` mirror loop will continue to use the v0.1 regex path until C.3.

## Hot spots reviewed

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | TOML schema for `gui-schema-capable` | CORRECT — `#[serde(default, rename = "gui-schema-capable")]` aligns with the kebab-case field; existing files deserialize cleanly. |
| 2 | JSON `version` field validation | CORRECT — non-1 versions return `None` with `tracing::warn!`; gate cell asserts. |
| 3 | Subcommand miss handling | CORRECT — returns `None`; gate cell asserts. |
| 4 | Binary lookup (`<CLI>_BIN` env var override) | CORRECT — matches existing `resolve_bin` convention. |
| 5 | Non-zero exit handling | CORRECT — logs warn and returns `None`. |
| 6 | Pure parser vs orchestrator separation | CORRECT. |
| 7 | `placeholder()` stub retention | OK — backward-compat; no caller depends. |
| 8 | `toml` dep promoted to runtime | NECESSARY — schema_check.rs is library code. |
| 9 | All 4 CLIs `gui-schema-capable = false` at C.1 | CORRECT — invariant cell asserts. |
| 10 | Mirror loop behavior unchanged | DEFERRED to C.3. |

## Exit gate

| Item | Status |
|------|--------|
| 3 RED cells GREEN | PASS (+2 additional negative cells) |
| Graceful version/missing-CLI handling | PASS |
| All 4 CLIs `gui-schema-capable = false` | PASS |
| Mirror CI unchanged | PASS |
| 0C / 0I | PASS |

Phase C.1 closed via self-review. Next phase, C.2, opens cross-repo PRs in `bg002h/{descriptor-mnemonic, mnemonic-secret, mnemonic-key, mnemonic-toolkit}` and is the gate where autonomous progression pauses for user-merge action.
