# Phase 5 Conditional-Visibility Review — R2

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `30b8d79 fold Phase 5 R1 (0C/1I) — export-wallet runtime pre-check`
**R1 report:** `design/agent-reports/v0_1-phase-5-conditional-r1.md`

## Verdict

**0C / 0I — converge**

R1 fold verified correct. No fold-introduced bugs. No additional flag-presence-only runtime pre-checks missed.

---

## Fold verification

### `export_wallet` post-fold logic

Three mutually-exclusive paths; no duplicate keys in returned vec:

| Form state | Path | `--template` | `--descriptor` |
|------------|------|--------------|----------------|
| both absent | `!has_descriptor && !has_template` only | Required | Required |
| descriptor set | `if has_descriptor` only | Disabled | Visible (default) |
| template set | `if has_template` only | Visible (default) | Disabled |

The "both absent" guard only fires when neither Disabled branch has fired, so a key cannot appear twice for the same input.

### cell_12 assertions

- `empty` → both Required ✓
- `with_template` → `--template` not Required, `--descriptor` Disabled ✓

---

## Other runtime pre-checks evaluated

Grepped `BadInput` returns across `cmd/{bundle,verify_bundle,convert,export_wallet,derive_child}.rs`. Findings:

| Location | Constraint | GUI-modelable as Phase 5 conditional? |
|----------|------------|-----------------------------------------|
| `export_wallet.rs:185` | tr-multi-a/tr-sortedmulti-a templates require `--taproot-internal-key` | Value-dependent (dropdown content); out of scope |
| `export_wallet.rs:202` | `--taproot-internal-key` only valid with taproot templates | Value-dependent; out of scope |
| `export_wallet.rs:208-213` | descriptor + template both set | Already modeled (cell_11) |
| `convert.rs:588-601` | `--from` primary-node count 0 or >1 | Input-count/content; out of scope |
| `convert.rs:609` | `--passphrase-stdin` + `--from <node>=-` conflict | Two flags + value content `-`; not flag-presence-only |
| `bundle.rs:473/479` | Slot-resolution failures | Value-content; out of scope |
| `verify_bundle.rs:561/568` | JSON envelope shape validation | Runtime data validation; out of scope |
| `derive_child.rs:98/193/248` | Value-range/format validation | Value-content; out of scope |

None are flag-presence-only constraints — all require value-content inspection beyond Phase 5's `has_value()` lookup. They belong to a v0.2 phase or to the form widget's per-`FlagKind` validator surface.

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| Module-level "11 cells" comment in `tests/conditional_visibility.rs` is now stale (count is 13) | 70 | Documentation-only; no runtime impact; below threshold |

---

## Test suite expectation

48 tests across 6 integration test binaries:
- argv_assembler 10/10
- argv_assembler_slot 5/5
- conditional_visibility 13/13 (R1: 12/12)
- copy_command 15/15
- runner_integration 3/3
- schema_mirror 2/2

Phase 5 converged. Next: Phase 6 (sibling CLI schemas + path-detect greying).
