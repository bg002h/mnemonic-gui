# SPEC — mnemonic-gui v0.28.0: bump toolkit pin v0.46.2 → v0.47.3 + timestamp default_value drift fix

**Resolves (toolkit FOLLOWUP, GUI companion):** `gui-timestamp-default-value-drift-v0.47.3`.
**Toolkit source SHA at recon:** `d509361` (toolkit master, post-v0.47.3).
**Cycle type:** GUI **MINOR** v0.27.0 → **v0.28.0** (pin-bump convention: v0.22/0.25/0.26/0.27 were all MINOR; this carries a behavior-correctness fix).
**GUI nightly:** build/test with `cargo +1.94.0` (default nightly ICEs).

---

## 1. Recon (measured, not assumed)

Current GUI pins (HEAD `4b83a9f`, version `0.27.0`):
- toolkit: `mnemonic-toolkit-v0.46.2` (Cargo.toml:42 + Cargo.lock:2296 + pinned-upstream.toml `[mnemonic]`).
- md `descriptor-mnemonic-md-cli-v0.6.2`, ms `ms-cli-v0.7.0`, mk `mk-cli-v0.7.0` — **all current** (match `md 0.6.2` / `ms 0.7.0` / `mk 0.7.0`); NOT bumped this cycle.

**schema_mirror MEASURED GREEN against the v0.47.3 toolkit binary.** Ran `schema_mirror` + `gui_schema_conditional_drift` + `schema_mirror_secret_drift` + `xpub_search_schema_mirror` with `MNEMONIC_BIN`=v0.47.3 (+ MD/MS/MK debug bins): all pass. The toolkit releases v0.46.3→v0.47.3 added **no** flag-NAME / dropdown value-enum / conditional-rule / secret-projection change:
- v0.46.3 origin-extraction dedup (internal), v0.47.0 `addresses --from electrum-phrase` (`--from` is a free `String` NodeType, not a clap value-enum → not gated), v0.47.1 synthesize dedup (internal), v0.47.2 quick-wins (no CLI surface), v0.47.3 timestamp **default_value** flip (not gated by schema_mirror).
- All three historical `*-pending-pin-bump` flag items are already resolved (GUI v0.22.0/v0.25.0/v0.26.0), consistent with schema_mirror GREEN.

**So the ONLY pin-necessitated change is the timestamp `default_value` drift** (`gui-timestamp-default-value-drift-v0.47.3`), which `schema_mirror` cannot catch (it gates flag-NAMES + value-enums only, not `default_value`; `GuiSchemaFlag` deserializes `name` only).

## 2. The drift (toolkit R0 I2 from the v0.47.3 cycle)

toolkit v0.47.3 flipped `export-wallet --timestamp` default `now → 0` (genesis rescan). The GUI schema (`src/schema/mnemonic.rs:1044`) still declares `FlagKind::Timestamp, default_value: Some("now")`. The D33 default-suppression (`src/form/invocation.rs:78`) is `TimestampValue::Now => default_str == "now"`. At the v0.47.3 pin, an **explicit** `Now` selection would be suppressed from argv (matching the stale `"now"` default) → the toolkit then applies its NEW `0` default → **the user's explicit `now` is silently discarded.**

### 2a. Widget-init verification (decisive — the fix is MINIMAL)
`src/form/widget.rs:166-188` (`default_flag_value_for_flag`): `FlagKind::Timestamp` → `FlagValue::Unset` (the default form seeds **Unset**, NOT `Now`). `Unset` emits nothing → the default export-wallet form naturally yields the toolkit's `0` default. `seeded_value_for` (click-to-Set) → `Now`. So:
- **Default form** (`Unset`): emits no `--timestamp` → toolkit applies `0`. ✓ Correct after the flip — no "forces now" regression.
- **Explicit `Now`** (user clicked Set): after the fix `is_at_default(Now, "0")` = `"0"=="now"` = false → NOT suppressed → emits `--timestamp now`. ✓ Bug fixed.
- **Explicit `Unix(n)`**: `Unix(_) => false` (always emits). ✓ unchanged.

Therefore **`is_at_default` needs NO change** — only the schema `default_value`. The `Unset` default form is the reason the minimal fix is complete.

## 3. Changes

### 3a. Toolkit pin bump v0.46.2 → v0.47.3 (lockstep, `pin_coherence` + `readme_pin_coherence` gated)
- `Cargo.toml:42` — `tag = "mnemonic-toolkit-v0.47.3"`.
- `Cargo.lock:2296-2297` — regenerate via `cargo +1.94.0 update -p mnemonic-toolkit` → version `0.47.3` + the **actual v0.47.3 tag commit** in `source` (**R0 M4:** let cargo resolve it; do NOT hand-paste the recon SHA `d509361`).
- `pinned-upstream.toml` `[mnemonic].tag` — `mnemonic-toolkit-v0.47.3` (cross-cite parity; `pin_coherence` gated).
- `README.md:50` — toolkit install pin `mnemonic-toolkit-v0.47.3` (`readme_pin_coherence` guard).
- **(R0 I1, ungated but every prior cycle bumps these):** `src/schema/mnemonic.rs:3688` — `pinned_version: "mnemonic 0.46.2"` → `"mnemonic 0.47.3"` (action-bar "Pinned:" banner); `src/schema/mnemonic.rs:1` module-doc header — `mnemonic-toolkit-v0.46.2` → `v0.47.3`.
- **(R0 M1):** `README.md:42` — GUI self-install tag `mnemonic-gui-v0.27.0` → `v0.28.0` (`readme_pin_coherence` gated; moves with the version bump in §4).
- md/ms/mk pins **unchanged** (current).

### 3b. Timestamp `default_value` fix (the drift)
- `src/schema/mnemonic.rs:1044` — `default_value: Some("now")` → `Some("0")` (matches the v0.47.3 toolkit default; keep `kind: FlagKind::Timestamp`). Update the adjacent `help:` string if it implies a `now` default (it currently reads "Bitcoin Core `timestamp` field. `now` or unix seconds." — reword to note `0` default).
- `src/form/invocation.rs` — **NO change** (per §2a; `Unset` default form + `Now => default_str=="now"` already yield correct behavior once `default_str` is `"0"`).

### 3c. Test updates (the 2 that pin the OLD `"now"` default — they invert)
- `tests/argv_assembler.rs::d33_timestamp_now_at_default_suppresses` (`:486`) — **invert**: with the schema default now `"0"`, an explicit `Timestamp::Now` is NO LONGER suppressed; it MUST emit `--timestamp now`. Rename to `…now_is_not_suppressed_when_default_is_zero` (or similar) + assert `--timestamp now` IS present. This is the discriminating regression guard for the bug fix.
- `tests/argv_assembler.rs::cell_3b_export_wallet_timestamp_now_argv` (`:126`) — update: `Timestamp::Now` now emits → expect `argv == ["mnemonic","export-wallet","--timestamp","now"]` (was `["mnemonic","export-wallet"]`); update the stale `// --timestamp now is the toolkit v5 default` comment.
- `tests/argv_assembler.rs::d33_timestamp_epoch_never_matches_now_default` (`:507`) — assertion (`Unix(0)` emits) STAYS GREEN (`Unix(_)=>false` unchanged); update the comment/name only for accuracy (default is now `0`, but epoch values still always emit since `is_at_default` Unix arm is `false`).
- `tests/argv_assembler.rs::cell_3_export_wallet_range_timestamp_argv` (`:89`) — `Unix(1_700_000_000)` still emits → STAYS GREEN; **(R0 M3)** update the stale `:97` comment ("emits regardless of the `--timestamp now` schema default" → the default is now `0`).
- **(R0 M2)** `src/form/invocation.rs:75-76` comment (`// Timestamp: Now matches "now"; …`) — optional accuracy touch-up; the logic is unchanged (the arm correctly still compares `default_str == "now"`, which is now false for export-wallet). Not load-bearing.
- Sweep `tests/` for any other assertion depending on the timestamp `"now"` default — **R0 confirmed the only `Some("now")` in the codebase is `src/schema/mnemonic.rs:1044`; the 4 argv_assembler tests are the complete set.** Re-grep at impl time to confirm.

### 3d. NO manual-gui change in THIS (GUI) repo — but a TOOLKIT-repo loose end exists
**The GUI repo has no `docs/` at all** (R0 verified) — so nothing to change here. **Correction to the toolkit v0.47.3 R0 M2:** `docs/manual-gui/` lives in the **TOOLKIT** repo (the GUI user manual, separate pinned cadence ~v0.3.0), and it DOES carry stale `now`-default prose: `mnemonic-toolkit/docs/manual-gui/src/40-mnemonic/45-export-wallet.md:30` ("default `now`") + `:340-343` ("`now` (the default; emits the literal string `"now"`…)"). The `expected_gui_schema_inventory.json` `--timestamp` entry has NO `default_value` field (not stale, not gated). This is OUT of scope for this GUI-repo cycle → **file a TOOLKIT FOLLOWUP** `manual-gui-export-wallet-timestamp-default-now-stale` (fix at the next manual-gui cadence touch) and note it in the `gui-timestamp-default-value-drift-v0.47.3` resolution.

### 3e. FOLLOWUP
Add `gui-timestamp-default-value-drift-v0.47.3` to `mnemonic-gui/FOLLOWUPS.md` as **resolved** this cycle (the placeholder branch `followup-timestamp-default-value-drift` is superseded — delete it; the resolved entry lands on master via this cycle). Update the toolkit-side companion line accordingly (separate trivial toolkit doc commit, or note for the next toolkit touch).

## 4. SemVer / version
GUI **MINOR** v0.27.0 → **v0.28.0** (pin-bump-with-behavior-fix convention). Bump `Cargo.toml` version + `Cargo.lock` self + CHANGELOG `[0.28.0]` + README pin (3a).

## 5. Phasing / gates
- **Phase 1 (RED):** invert `d33_timestamp_now_at_default_suppresses` + `cell_3b` to assert the FIXED behavior (`--timestamp now` emitted). RED against the current `default_value:"now"` schema.
- **Phase 2 (GREEN):** apply 3a (pin) + 3b (schema default_value) + finalize 3c tests + 3e FOLLOWUP + 4 version. Gates (all `cargo +1.94.0`, 4 pinned bins via `*_BIN`): full GUI test suite + `schema_mirror`/`gui_schema_conditional_drift`/`schema_mirror_secret_drift`/`xpub_search_schema_mirror` + `pin_coherence` + `readme_pin_coherence` + clippy `--all-targets`. Per-phase opus review.
- **Phase 3 (ship):** ff-merge to master → tag `mnemonic-gui-v0.28.0` → push → watch CI (build + schema-mirror).

## 6. R0 decisions (RATIFIED round 1)
1. **MINIMAL fix correct** (schema `default_value` only; NO `is_at_default` change). ✅ R0-ratified — `widget.rs` seeds Timestamp→`Unset` (default form emits nothing→toolkit `0`); explicit `Now` emits `--timestamp now`; `Unix(n)` always emits.
2. **GUI MINOR v0.28.0** (vs PATCH). ✅ R0-ratified (v0.22–v0.27 pin-bump convention; conservative for a behavior-correctness fix).
3. **No manual-gui change in the GUI repo.** ✅ R0-confirmed (GUI repo has no `docs/`). The toolkit-repo `docs/manual-gui/` staleness is tracked separately (§3d → toolkit FOLLOWUP).
4. **Test inversions** are the discriminating regression guards; the full set is the 4 argv_assembler tests (only `Some("now")` in the codebase is `schema/mnemonic.rs:1044`). ✅ R0-confirmed.
5. **md/ms/mk pins** correctly unchanged (current). ✅ R0-confirmed.

## 7. Out of scope
- md/ms/mk pin bumps (current).
- The 5 other open `gui-*` FOLLOWUPs (manual cross-refs, localization, run-confirm-modal companion, bsms-token repeating mirror, mnemonic-gui-schema-mirror tracker) — independent, not pin-necessitated.
- Any `is_at_default` semantic change (the `Unset` default form makes it unnecessary).
