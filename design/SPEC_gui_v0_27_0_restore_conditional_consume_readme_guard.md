# SPEC — mnemonic-gui v0.27.0 — consume toolkit restore `conditional_rules` projection + README install-pin coherence guard

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** toolkit FOLLOWUPs `gui-schema-restore-required-unless-md1-projection` (GUI consumption half) + `gui-readme-install-pin-coherence-guard`.
**Source SHA:** mnemonic-toolkit `b74badd` (tag `mnemonic-toolkit-v0.46.2`); GUI base `f6caa20` (tag `mnemonic-gui-v0.26.0`).
**SemVer:** MINOR — toolkit-pin catch-up that activates a drift gate + a new test guard; mirrors the v0.25.0/v0.26.0 catch-up precedent. `0.26.0 → 0.27.0`.

---

## 1. Summary

Two GUI-repo follow-ons to the mnemonic-gui v0.25.0 cycle:

- **(slug 1, GUI half)** `mnemonic-toolkit-v0.46.2` now **projects** restore's `--from required_unless_present="md1"` as a `conditional_rule` (`not(flag_present "--md1") → {--from, required}`). The GUI's hand-authored `conditional::restore` fn (`src/form/conditional.rs`) already emits exactly that shape, but while the GUI pins toolkit v0.46.0 (which emits `conditional_rules: []` for restore), `gui_schema_conditional_drift` **skips** restore — the rule is ungated. Bumping the pin v0.46.0 → v0.46.2 makes the drift gate **exercise** restore (and it passes — verified: drift test GREEN vs the v0.46.2 binary). Add `("restore", 1)` to `SUBCOMMAND_FLOORS` so the now-projected rule can't silently vanish.
- **(slug 2)** mnemonic-gui has no guard asserting its README install-command `--tag` pins match `pinned-upstream.toml` (unlike toolkit's `readme_version_current.rs`); they drifted 3 versions before v0.25.0 backfilled them. Add a pure-logic `tests/readme_pin_coherence.rs`.

**GUI-repo-only.** No toolkit change (Cycle A shipped the projection).

## 2. Empirical baseline (captured pre-implementation)

- Toolkit v0.46.2 binary `mnemonic gui-schema` → restore `conditional_rules` is a **1-element** array: `{"when":{"kind":"not","predicate":{"kind":"flag_present","flag":"--md1"}},"effect":{"flag":"--from","visibility":"required"}}`.
- `gui_schema_conditional_drift` (`+1.94.0`, `MNEMONIC_BIN=`v0.46.2): **GREEN (5/5)** against the current GUI (`conditional::restore` matches the projection; restore not yet in `SUBCOMMAND_FLOORS` → no floor asserted on it; existing floors + `total_rules >= 34` hold).
- **No flag-name delta v0.46.0 → v0.46.2** (v0.46.1 was a pure refactor; v0.46.2 added only the `conditional_rules` projection — no clap flag/value/subcommand). So `schema_mirror` stays GREEN with just the pin bump; no `RESTORE_FLAGS`/schema flag change. The lib re-export const-assert (`SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS`) is unchanged v0.46.0→v0.46.2 (no new secret flag) → compiles.
- Toolchain: CI `dtolnay/rust-toolchain@stable`; run local builds/tests with `+1.94.0` (default nightly ICEs).

## 3. slug 1 (GUI half) — `tests/gui_schema_conditional_drift.rs`

Add `("restore", 1)` to `SUBCOMMAND_FLOORS` (`:300-306`) and bump the total floor `>= 34` → `>= 35` (`:321`, keeping the "sum = 34"→"sum = 35" comment coherent):

```rust
    const SUBCOMMAND_FLOORS: &[(&str, usize)] = &[
        ("bundle", 11),
        ("verify-bundle", 10),
        ("export-wallet", 6),
        ("convert", 4),
        ("derive-child", 3),
        ("restore", 1),   // NEW: toolkit v0.46.2 projects restore's --from required-unless-md1
    ];
```

**Pin-coupling (the RED-without-pin guard):** `("restore", 1)` asserts the PINNED binary emits ≥1 restore rule. Against v0.46.0 it emits `[]` → the floor would FAIL. So this FLOORS entry is ONLY valid once §4's pin bump lands — they MUST move in the same PR (and `pin_coherence` already gates Cargo↔pinned-upstream). No `conditional::restore` GUI-logic change (the fn already matches).

## 4. slug 2 — `tests/readme_pin_coherence.rs` (NEW, pure-logic)

Mirror `tests/pin_coherence.rs`'s `read()`-from-`CARGO_MANIFEST_DIR` style (no binary, no network). Parse each README `cargo install … --tag <TAG> <pkg>` line (whitespace-tolerant — the lines use alignment padding) and assert:

- GUI self-line (`--tag mnemonic-gui-v<X> … mnemonic-gui`): `<X>` == `Cargo.toml` `version`.
- `mnemonic-toolkit` line: `--tag` == `pinned-upstream.toml [mnemonic].tag`.
- `md-cli` line: `--tag` == `[md].tag`. `ms-cli`: `[ms].tag`. `mk-cli`: `[mk].tag`.

`pinned-upstream.toml` carries all four tags (`[mnemonic]:22`, `[md]:39`, `[ms]:46`, `[mk]:53`). Use a small `pkg → (toml-section)` table; assert each README line found + equal. Fail with a clear "README install pin drift: README says X, pinned-upstream/Cargo says Y" message. This closes the slug-2 class (the instance was backfilled in v0.25.0).

## 5. Pin + version bump
- `Cargo.toml` `[dependencies].mnemonic-toolkit.tag`: `v0.46.0` → `v0.46.2` (`:42`).
- `pinned-upstream.toml` `[mnemonic].tag`: `v0.46.0` → `v0.46.2` (`:22`). `pin_coherence` asserts the two agree.
- `Cargo.lock`: regenerate to `mnemonic-toolkit v0.46.2` (stage it).
- `Cargo.toml` `version`: `0.26.0` → `0.27.0` (`:3`).
- `src/schema/mnemonic.rs`: bump ONLY the module-doc (`:1` `mnemonic-toolkit-v0.46.0` → `v0.46.2`) + `pinned_version` (`:3687` `"mnemonic 0.46.0"` → `"mnemonic 0.46.2"`). **Anti-blind-`sed`:** the `:2746` provenance comment (`toolkit v0.46.0: scan a file of candidate passphrases…`) legitimately documents when `--passphrase-candidates-file` was added (v0.46.0) and MUST stay.
- **README install-command pins:** `:42` `mnemonic-gui-v0.26.0` → `v0.27.0`; `:50` `mnemonic-toolkit-v0.46.0` → `v0.46.2` (the new `readme_pin_coherence` guard now enforces these; sibling md/ms/mk lines already current).
- `CHANGELOG.md`: new `## mnemonic-gui [0.27.0]` entry.

## 6. Tests
- `gui_schema_conditional_drift` GREEN vs the v0.46.2 binary with restore now floored at 1 (run with the pinned `MNEMONIC_BIN`).
- `readme_pin_coherence` (NEW) GREEN after §5 (all 5 README tags coherent).
- `pin_coherence` GREEN after §5 (both toolkit pins v0.46.2).
- `schema_mirror` GREEN vs v0.46.2 (no flag delta; run with all four pinned `*_BIN`: mnemonic 0.46.2 / md 0.6.2 / ms 0.7.0 / mk 0.7.0). `schema_mirror_secret_drift` unaffected (no secret flag change). `cli_gui_schema` subcommand-name freeze unaffected (no subcommand add).
- Full `cargo +1.94.0 test -p mnemonic-gui --no-fail-fast` (4 pinned bins) + `cargo +1.94.0 clippy -p mnemonic-gui --all-targets` GREEN.

## 7. Lockstep / ship
- **Toolkit repo:** flip BOTH FOLLOWUPs `gui-schema-restore-required-unless-md1-projection` (`open → resolved (mnemonic-gui-v0.27.0)`) and `gui-readme-install-pin-coherence-guard` (`open → resolved (mnemonic-gui-v0.27.0)`) — separate toolkit-repo commit after the GUI tag exists.
- **manual-gui:** OUT of scope (separate deferred track; `#[ignore]`-gated).

## 8. Phased plan
- **Phase 1 (RED):** baseline captured (§2). The slug-1 FLOORS `("restore", 1)` is RED against a v0.46.0 binary and GREEN against v0.46.2 — demonstrate the coupling once (FLOORS+pin move together); the slug-2 `readme_pin_coherence` is a new additive guard (GREEN once §5's pins are coherent — assert it would be RED if a pin were stale by temporarily checking, optional).
- **Phase 2 (GREEN):** §3 FLOORS add + §4 new guard. Tests vs the v0.46.2 pinned bins + clippy GREEN. Per-phase opus review → persist to `design/agent-reports/`.
- **Phase 3 (pin/version):** §5 pin + version + module-doc/pinned_version + README + CHANGELOG; `pin_coherence` + `readme_pin_coherence` GREEN; full suite + clippy GREEN. Per-phase review.
- **Phase 4 (ship):** clean tree → `git checkout master && ff-merge` → tag `mnemonic-gui-v0.27.0` → push master + tag → watch CI (`build`, `schema-mirror`). Then flip the two toolkit FOLLOWUPs.

## 9. Risk
Low. Pin advances two PATCH versions with NO flag delta (so `schema_mirror` is a clean catch-up, not a flag backfill); the drift-gate activation is verified GREEN against the v0.46.2 binary already; the new README guard is pure-logic and additive. R0 must confirm: (i) no flag-name delta v0.46.0→v0.46.2 (so no `RESTORE_FLAGS`/schema flag change is owed); (ii) the FLOORS `total >= 34 → 35` bump is right (restore adds exactly 1); (iii) the README parser is whitespace-tolerant + maps all 5 lines correctly; (iv) the `:2746` provenance comment is NOT swept by the version bump.
