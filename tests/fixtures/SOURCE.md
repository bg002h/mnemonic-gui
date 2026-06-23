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
| `wallet_import/bsms-2line-multi-2of3.txt` | `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-multi-2of3.txt` | `mnemonic-toolkit-v0.70.0` | 2026-06-22 |
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

## Wave-3 wire-shape goldens (`tests/wire_shape_snapshot.rs`)

The W3-1 wire-shape regression cells (`tests/wire_shape_snapshot.rs`) do NOT
read vendored JSON envelope fixtures — they capture each `--json` envelope LIVE
from the pinned toolkit binary and assert its full structural key-set. Two
input blobs feed those cells:

- `wallet_import/bsms-2line-multi-2of3.txt` (vendored above) → the
  `import-wallet --format bsms --json` multisig cell.
- `coldcard_generic_bip84_mainnet.json` (vendored above) → the
  `import-wallet --format coldcard --json` single-sig cell.

The xpub-search cells embed canonical xpubs/descriptors inline (abandon×11+about
test seed, no real funds) rather than fixtures.

**Maintenance contract (the LEADING drift gate the slug wants):** the captured
goldens are valid ONLY at the `Cargo.toml` `[dependencies] mnemonic-toolkit` pin
(currently `mnemonic-toolkit-v0.70.0`). Every future toolkit-pin bump MUST
re-run the 8 `wire_shape_snapshot` cells against the new binary IN THE SAME
cycle (`MNEMONIC_BIN=$(which mnemonic) cargo test --test wire_shape_snapshot`);
a changed key-set is an intentional wire-shape evolution → update the assertion
+ bump the GUI in lockstep. CI runs these cells against the cargo-installed
pinned binary (`schema-mirror.yml` `cargo-test-full-suite`), so a stale golden
REDs in CI even if it would pass a hand-written offline assertion.

> **Retired (Wave-3, v0.49.0):** the loose v0.27.0 envelope smoke fixtures
> (`v0_27_0_envelopes/*`, `wallet_import/envelope_v0_27_0.json`) and their
> consumer `tests/cli_envelope_smoke.rs` were DELETED — superseded by the
> live-capture module above. They encoded the OLD wire shape (e.g.
> `path/template/account: null` on a `no_match`, since corrected to key
> OMISSION) and never ran the binary.

## When NOT to re-vendor

- Toolkit version bump that doesn't touch the fixture's source path —
  most minor toolkit releases.
- Coldcard format spec stable; the fixture is a byte-exact pin of a
  stable wire format.

## Test that consumes each fixture

- `coldcard_generic_bip84_mainnet.json` →
  `tests/runner_integration.rs::cell_1_mnemonic_export_wallet_byte_exact`.
  Drives `mnemonic export-wallet --format coldcard` via the GUI's
  argv-assembly + runner; asserts stdout byte-equals the fixture. ALSO →
  `tests/wire_shape_snapshot.rs::wireshape_import_wallet_coldcard_singlesig`
  (live `import-wallet --format coldcard --json` envelope-shape capture).
- `wallet_import/bsms-2line-multi-2of3.txt` →
  `tests/wire_shape_snapshot.rs::wireshape_import_wallet_bsms_multisig`
  (live `import-wallet --format bsms --json` envelope-shape capture).
- `descriptor_builder/*.json` (the 5 archetype spec goldens, node-tree
  builder SPEC §4 gate 3) → `tests/tree_round_trip.rs`. Round-trip law
  (a) `to(from(j)) == j` per fixture + the live exit-0 leg through
  `build-descriptor --spec - --json` (the staleness tether — fixture
  immutability is CYCLE-scoped per the presets SPEC; if the exit-0 leg
  fails, re-vendor from the toolkit tag and update the table above).
