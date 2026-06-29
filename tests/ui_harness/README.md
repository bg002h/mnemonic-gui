# mnemonic-gui automated UI-functionality harness

Schema-enumerated `egui_kittest` tests that exercise UI element functionality without
manual clicking. Design: `design/SPEC_gui_automated_ui_test_harness.md` +
`design/IMPLEMENTATION_PLAN_gui_ui_test_harness.md` (+ `design/agent-reports/gui-ui-test-harness-*`).

## What it gates (runs in CI via `cargo test --workspace`)
The permanent, **deterministic** gate — no proptest randomness, no flake:

| File | Invariant | What it catches |
|---|---|---|
| `spike_widget_drivers.rs` | P0 drive primitives | that kittest can drive each widget kind (ComboBox/Checkbox/TextEdit/DragValue) |
| `ui_harness_i1_roundtrip.rs` | **I1** form→argv wiring round-trip (slice) | a value entered via the real widget reaches argv bound to the right flag (mis-wired/dead elements) |
| `ui_harness_i2_conditional.rs` | **I2** conditional/state (17 conditional subs) | renderer↔`conditional()` desync (per the 6 `Visibility` effects); Hidden/Disabled value-suppression; toggle round-trip (no stuck visibility-state) |
| `ui_harness_i3_secret_nopersist.rs` | **I3** classified-secret never-persist regression | a classified secret reaching persisted state / masked-argv preview / spec-stdin (regression net; does NOT replace `schema_mirror_secret_drift`/`secret_taxonomy_pin`) |
| `ui_harness_i4_realcli.rs` | **I4** real pinned-CLI functional cells | the GUI-assembled argv produces correct CLI output (env-gated: skips cleanly when `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN` unset; CI sets them) |
| `ui_harness_sweep.rs` :: `i1_wiring_sweep_all_61`, `sweep_census` | **coverage gate** | 61/61 subcommands round-trip ≥1 identity flag (census REDs if a sub drops to 0) |

`tests/ui_harness/mod.rs` is the shared engine (enumerator, per-subcommand seed table, drive
dispatch, `render_one_flag` / `render_whole_form` helpers). `#![allow(dead_code)]` because a
shared test module trips clippy `dead_code` per consuming binary.

## The one-time SWEEP (NOT in the every-commit gate)
`ui_harness_sweep.rs` also has three `#[ignore]` proptest finders (`i1_leaf_value_proptest`,
`i2_render_faithfulness_proptest`, `i3_secret_fixture_proptest`) — broad random leaf/toggle
variation over all 61 subs. They are the **one-time coverage bug-finder**, kept out of CI (proptest
randomness → no flake/shrink in the gate). Run on demand:

```sh
cargo test --test ui_harness_sweep -- --ignored
```

`failure_persistence: None` → no `proptest-regressions` file is written. `proptest` is a
`[dev-dependencies]` — it never enters the shipped GUI dependency graph.

## Running locally
```sh
# the deterministic gate (what CI runs):
cargo test --workspace --jobs 2          # --jobs 2 avoids a linker OOM on argv_assembler_slot under full parallel linking

# exercise the I4 real-CLI cells against installed binaries:
MNEMONIC_BIN=$HOME/.cargo/bin/mnemonic MD_BIN=$HOME/.cargo/bin/md \
  MS_BIN=$HOME/.cargo/bin/ms MK_BIN=$HOME/.cargo/bin/mk \
  cargo test --test ui_harness_i4_realcli
```

## Scope (what it does NOT do — by design)
Not visual/layout/UX ("is it confusing" needs a human / a separate snapshot track); not
crash-fuzzing as an end; does not re-prove flag NAMES (owned by `schema_mirror`); does not detect
*unclassified* secrets (owned by `schema_mirror_secret_drift` + `secret_taxonomy_pin`); does not
test the CLI binaries' own correctness.
