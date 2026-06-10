# Impl review — GUI v0.30.0 presets pin bump — round 1

**Verdict: YELLOW** (0C / 1I — one factual census error in the freshly-filed FOLLOWUP; everything else conforms)

## Critical

None.

## Important

**I1 — FOLLOWUP `repeating-secret-flags-never-reach-argv` site census is wrong: `--share` is ×2 affected sites, not ×3 (and the SPEC §8 / R0-r1 M1 claim it transcribes is itself wrong).**
- `SLIP39_COMBINE_FLAGS` `--share` — `FlagKind::Text`, `secret: true`, `repeating: true` (`src/schema/mnemonic.rs:1347-1353`) → **affected** ✓
- `MS_SHARES_COMBINE_FLAGS` `--share` — `Text`, `secret: true`, `repeating: true` (`:1485-1491`) → **affected** ✓
- `SEED_XOR_COMBINE_FLAGS` `--share` — **`FlagKind::NodeValueComposite(PHRASE_ONLY)`**, `secret: true`, `repeating: true` (`:1590-1597`) → **NOT affected**. The widget's secret branch requires `matches!(flag.kind, FlagKind::Text)` (`src/form/widget.rs:76`) — it renders through `state.values` and already emitted fine; as of this commit it gains working multi-row UI.

True census: `--ms1` ×2 (`VERIFY_BUNDLE_FLAGS` `:666-672`, `IMPORT_WALLET_FLAGS` `:2088-2094`) + `--share` ×2 = **4 affected sites**. Exhaustive (no sibling-schema secret+repeating flags; `SECRET_FLAG_NAMES` adds only non-repeating passphrase flags). Error propagates to CHANGELOG [0.30.0] bullet 4 and the SPEC §8 grounding line (three R0 rounds missed the kind mismatch). Fix direction in the FOLLOWUP otherwise sane.

## Minor

**M1 — duplicate cell numbering in `tests/conditional_visibility.rs`** (new `cell_12_…`–`cell_15_…` collide with the existing `cell_12_export_wallet…` numbering in the header comment). Cosmetic.
**M2 — `gui-build-descriptor-presets-pending-pin-bump` still Active while CHANGELOG says "Resolves".** SPEC-conforming (flip at ship) — but the flip + the toolkit `scripts/install.sh:44` GUI-pin commit (8 releases stale) are now an untracked ship-checklist dependency.
**M3 — row-index ComboBox salts migrate after a removal** (positional row_idx re-keys rows above; transient popup state may attach wrong for a frame). Within SPEC (per-row uniqueness delivered); cosmetic.
**M4 — the "(none)" closed-button display site is popup-pinned only** (egui registers the combo button with an empty accesskit label); both sites share one mapping expression. Accepted residual.

## SPEC-conformance checklist

- **§1 pin sites (6/6):** Cargo.toml:42 v0.52.0; Cargo.lock rev `2cad4b71a6f4…` == tag commit; pinned-upstream.toml:22; README.md:50; module-doc :1; banner :3949 ("mnemonic 0.52.0"). Residue grep: exactly the 3 historical comments survive (line-shifted by the +12 insert), zero strays. ✓
- **§2 the 12 entries field-by-field** (`mnemonic.rs:3462-3615`): all kinds/repeating/bounds/default_value per the SPEC table; `--archetype` the ONLY defaulted entry; ARCHETYPES sentinel-first + CliArchetype order; ALLOW_RULES byte-equal the binary. ✓
- **§3 widget:** secret check first (:76), repeating branch second (:95); scalar path byte-unchanged (render → render_row(None); chrome + 2-tuple salts gated on row.is_none()); per-frame required-zero-rows seed (:185-190); add-row seeds Text("")/Dropdown("") (:228-233); row_idx salted into all 3 ComboBox sites; "(none)" both display sites value-untouched (:449-453, :459-460); collect-then-apply removal (:213-219); unconditional header (:170-182). ✓
- **§4 conditional:** has_value mutex both directions (conditional.rs:640-649); `flag_value_is_present` treats `Dropdown("")` as absent; registered (:3709); coverage list grown. ✓
- **§6 tests:** all cells present + discriminating (order, seeds both directions, respawn, emission, "(none)" round-trip with argv assertion; 18-flag + value pins). ✓
- **§5/§7:** FOLLOWUP filed (census → I1); CHANGELOG otherwise accurate; version 0.30.0; README self-pin → v0.30.0 correct pre-tag (readme_pin_coherence forces self-pin == Cargo.toml version; v0.29.0 precedent).

## Empirical probes run

1. Binary 0.52.0 possible-values byte-equal ARCHETYPES[1..]/ALLOW_RULES.
2. Cargo.lock rev == `git rev-parse mnemonic-toolkit-v0.52.0^{commit}`.
3. Full suite, 4 binaries: **377 passed / 0 failed**. clippy `-D warnings`: clean.
4. repeating_rows 10/10 verbose; build_descriptor_schema 6/6; conditional cells pass.
5. I1 census probes: kind/secret/repeating of all 5 claimed sites + SECRET_FLAG_NAMES + sibling-schema sweep.
