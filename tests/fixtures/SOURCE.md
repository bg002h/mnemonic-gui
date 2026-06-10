# Vendored test fixtures

Source-of-truth for upstream fixtures consumed by the GUI test
suite. Each file in this directory has a corresponding row below
identifying its upstream origin, the toolkit version it was vendored
from, and the re-vendor procedure when upstream changes.

The vendoring exists to decouple GUI tests from
`MNEMONIC_GUI_UPSTREAM_ROOT` (retired in v0.4.0). Re-vendor when
either the toolkit's emission output for the fixture's input changes
(SPEC §5 / Coldcard format revision) OR a new toolkit cycle ships a
materially different vector.

## Provenance table

| Fixture | Upstream path | Toolkit version | Last re-vendored |
|---|---|---|---|
| `coldcard_generic_bip84_mainnet.json` | `crates/mnemonic-toolkit/tests/export_wallet/coldcard_generic_bip84_mainnet.json` | `mnemonic-toolkit-v0.14.0` | 2026-05-16 |
| `descriptor_builder/decaying-multisig.json` | `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/decaying-multisig.json` | `mnemonic-toolkit-v0.52.0` | 2026-06-10 |
| `descriptor_builder/hashlock-gated.json` | `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/hashlock-gated.json` | `mnemonic-toolkit-v0.52.0` | 2026-06-10 |
| `descriptor_builder/kofn-recovery.json` | `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/kofn-recovery.json` | `mnemonic-toolkit-v0.52.0` | 2026-06-10 |
| `descriptor_builder/simple-timelocked-inheritance.json` | `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/simple-timelocked-inheritance.json` | `mnemonic-toolkit-v0.52.0` | 2026-06-10 |
| `descriptor_builder/tiered-recovery.json` | `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/tiered-recovery.json` | `mnemonic-toolkit-v0.52.0` | 2026-06-10 |

## Re-vendor procedure

```sh
# 1. Clone or update local toolkit checkout to the target tag.
cd /path/to/mnemonic-toolkit
git checkout mnemonic-toolkit-v<NEW_TAG>

# 2. Copy the file into this directory.
cp crates/mnemonic-toolkit/tests/export_wallet/<FIXTURE>.json \
   /path/to/mnemonic-gui/tests/fixtures/<FIXTURE>.json

# 3. Update the provenance table above (toolkit version + date).

# 4. Run the GUI test that consumes it:
cd /path/to/mnemonic-gui
MNEMONIC_BIN=$(which mnemonic) \
  cargo test --release --test runner_integration cell_1_mnemonic_export_wallet_byte_exact

# 5. Commit with reference to the toolkit cycle that drove the re-vendor.
```

## When NOT to re-vendor

- Toolkit version bump that doesn't touch the fixture's source path —
  most minor toolkit releases.
- Coldcard format spec stable; the fixture is a byte-exact pin of a
  stable wire format.

## Test that consumes each fixture

- `coldcard_generic_bip84_mainnet.json` →
  `tests/runner_integration.rs::cell_1_mnemonic_export_wallet_byte_exact`.
  Drives `mnemonic export-wallet --format coldcard` via the GUI's
  argv-assembly + runner; asserts stdout byte-equals the fixture.
- `descriptor_builder/*.json` (the 5 archetype spec goldens, node-tree
  builder SPEC §4 gate 3) → `tests/tree_round_trip.rs`. Round-trip law
  (a) `to(from(j)) == j` per fixture + the live exit-0 leg through
  `build-descriptor --spec - --json` (the staleness tether — fixture
  immutability is CYCLE-scoped per the presets SPEC; if the exit-0 leg
  fails, re-vendor from the toolkit tag and update the table above).
