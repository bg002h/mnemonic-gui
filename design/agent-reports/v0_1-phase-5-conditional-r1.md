# Phase 5 Conditional-Visibility Review — R1

**Date:** 2026-05-12
**Reviewer:** feature-dev:code-reviewer (Sonnet 4.6)
**Scope:** Commit `bfdd6d3 Phase 5: conditional visibility + 11-constraint enumeration`
**Plan ref:** `/home/bcg/.claude/plans/declarative-tumbling-shell.md` §5 + §C Phase 5

## Verdict

**0C / 1I — fold needed** (folded inline)

All 11 clap-level upstream constraints correctly enumerated and tested. One Important finding on the export-wallet `--template` help text overstating enforcement vs. clap.

---

## Constraint inventory verification

| # | Upstream attribute | GUI cell |
|---|--------------------|---------|
| 1 | bundle.rs:25 `--template` required_unless [descriptor, descriptor-file] | cell_01 |
| 2 | bundle.rs:30 `--descriptor` conflicts_with descriptor-file | cell_02 |
| 3 | verify_bundle.rs:28 `--template` required_unless [...] | cell_03 |
| 4 | verify_bundle.rs:32 `--descriptor` conflicts_with descriptor-file | cell_04 |
| 5 | verify_bundle.rs:54 `--ms1` conflicts_with bundle-json | cell_05 (both directions) |
| 6 | verify_bundle.rs:57 `--mk1` required_unless bundle-json | cell_06 |
| 7 | verify_bundle.rs:57 `--mk1` conflicts_with bundle-json | cell_07 (both directions) |
| 8 | verify_bundle.rs:60 `--md1` required_unless bundle-json | cell_08 |
| 9 | verify_bundle.rs:60 `--md1` conflicts_with bundle-json | cell_09 (both directions) |
| 10 | convert.rs:181 `--passphrase-stdin` conflicts_with passphrase | cell_10 (both directions) |
| 11 | export_wallet.rs:43 `--template` conflicts_with descriptor | cell_11 (both directions) |

verify_bundle.rs:67 (`--bundle-json` conflicts_with_all [ms1, mk1, md1]) is the symmetric dual of cells 05/07/09 — covered by the reverse-direction assertions in each. `derive_child.rs` has zero clap constraints; `conditional: None` correct.

---

## Important findings

### I-1 — `export-wallet --template` help overstates clap enforcement; runtime pre-check is the actual enforcement

**Confidence:** 82
**File:** `src/schema/mnemonic.rs` (`EXPORT_WALLET_FLAGS --template.help`)

**What:** GUI help text reads "Mutually-required-one-of with --descriptor" — mirroring the upstream `--help` prose. But upstream `export_wallet.rs:43` has only `conflicts_with = "descriptor"` (no `required_unless_present`). The "one of required" enforcement is the runtime pre-check at `export_wallet.rs:215-219`:

```rust
if args.descriptor.is_none() && args.template.is_none() {
    return Err(ToolkitError::BadInput(
        "export-wallet requires either --template or --descriptor".into(),
    ));
}
```

The pre-fold GUI conditional only modeled the clap-level `conflicts_with`; it did NOT mark either flag as `Required` when both were absent. A user who hit Run with neither populated would get a surprise non-zero exit instead of a pre-Run form-validation marker.

**Fold (more accurate than the reviewer's recommended "edit help string" fix):**

Extend `export_wallet` conditional to mark BOTH `--template` AND `--descriptor` as `Required` when both are absent — modeling the runtime pre-check, not just the clap attribute. The Phase 5 scope was nominally "clap attributes only", but Phase 5's GOAL is "encode upstream constraint semantics into form-validation" — runtime pre-checks like this one are a more accurate match for the user-facing constraint, and the upstream help-text labels the pair "Mutually-required-one-of" which IS the runtime semantic.

Add `cell_12_export_wallet_template_or_descriptor_required_when_neither_set` to pin the new behavior.

Help text in `EXPORT_WALLET_FLAGS --template` is kept as-is (mirrors upstream's own help-text prose; preserves the schema-mirror invariant in spirit).

**Post-fold test count:** 13 cells (was 12; +1 for cell_12).

---

## Confidence-filtered: omitted

| Item | Confidence | Disposition |
|------|------------|-------------|
| Bundle descriptor ↔ template runtime pre-check (bundle.rs:108) | 30 | bundle ALREADY enforces template required_unless [descriptor, descriptor-file] at clap (line 25); no additional runtime check beyond what cell_01 covers |
| cell_03 only tests `with_descriptor`, not `with_descriptor_file` | 40 | cell_04 covers the other side; bundle cell_01 tests all 3 branches |
| `vis_of` first-match semantics for duplicate keys | — | No duplicate keys produced by any conditional fn |

---

## Hot-spot resolution

| # | Hot spot | Verdict |
|---|----------|---------|
| 1 | bundle.rs:25 both branches | Correct, cell_01 |
| 2 | verify-bundle --ms1 optional (not required) | Correct, cell_05 |
| 3 | mk1/md1 dual constraint (required_unless + conflicts_with) | Correct, cells 06+07, 08+09 |
| 4 | has_value() Boolean(true) semantics | Correct, cell_10 |
| 5 | Default Visible for unlisted flags | Correct per SPEC §5 |
| 6 | Bidirectional consistency | Correct (cells 02, 04, 05, 07, 09, 10, 11) |
| 7 | Visibility::Hidden unused | Correct posture for Phase 5 |
| 8 | Empty form state | Safe (cells 01, 03, 12 exercise it) |
| 9 | derive-child no conditional | Correct (coverage cell) |
| 10 | --passphrase-stdin Boolean | Correct, cell_10 |
| 11 | bundle template ↔ descriptor runtime check | Below threshold; covered by bundle.rs:25 clap attribute path |
| 12 | export-wallet template ↔ descriptor pre-check | **→ I-1, folded** |
