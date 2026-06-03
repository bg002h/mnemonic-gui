# gui-ms1 catch-up — SPEC R0 Review (round 0)
**Verdict:** GREEN (0C / 0I)

SPEC `design/SPEC_gui_v0_22_0_pin_catchup_ms1.md` (`bdaa91b`). Every citation independently re-grepped against live source (GUI branch `bundle-slot-ms1-gui`, toolkit master v0.41.0, md-cli v0.6.2).

## Critical (0) / Important (0) / Minor (4)

## Citation verification — ALL ACCURATE (no DRIFTED/WRONG)
- GUI `Cargo.toml:3` (version 0.21.3), `:42` (toolkit tag v0.37.3) ✓.
- `pinned-upstream.toml`: `[mnemonic].tag` v0.38.0 `:22`; `[md]` v0.6.1 `:38`; `[ms]` v0.5.0 `:45`; `[mk]` v0.6.0 `:52`; stale `[md]` comment `:32-36` ✓.
- `src/secrets.rs`: real `pub use` at `:34` (`:7` is a doc mention); `v0_3_canonical_fallback` mod `:37`, NODE_TYPES snapshot `:42-54` (8), SLOT_SUBKEYS snapshot `:67-68` (6 incl ms1); two const-asserts NODE_TYPES `:78-88` / SLOT_SUBKEYS `:89-99` ✓.
- `pinned_version` banners: mnemonic.rs `:3452` "mnemonic 0.38.0"; mk.rs `:476` "mk 0.6.0"; ms.rs `:529` "ms 0.7.0" (already current); md.rs `:532` "md 0.5.0" ✓.
- `src/form/slot_editor.rs` draft Ms1 picker: variant `:44`, ALL `:59`, as_str `:72`, is_secret_bearing `:88` — set+ordering byte-mirror toolkit `slot_input.rs` ✓.
- `tests/schema_mirror.rs`: set-equality docstring `:1-9`, extractor `:20-44`, assert `:110-119` (`:112`) ✓.
- `README.md`: stale gui pin `:42`, "match pinned-upstream.toml" claim `:46-47`, 4 stale sibling install pins `:50-53` ✓.
- Toolkit `secret_taxonomy.rs` master: SECRET_NODE_TYPES `:76-85` (8), SECRET_SLOT_SUBKEYS `:111` (6 incl ms1) ✓.
- md-cli `repair.rs`: `md1_strings: Vec<String>` `:42-43` (required, num_args=1..) + `json: bool` `:48-49` ✓.

## md repair field-set — FIELD-ACCURATE (not just a sketch)
`src/schema/mod.rs`: `FlagSchema` `:64-110` = `{name,kind,required,repeating,help,secret,default_value,global}`; `SubcommandSchema` `:28-48` = `{name,human_name,flags,positional_args,allows_slots,conditional}`; `PositionalArgSchema` `:51-61` = `{name,required,repeating,help}` — all EXACTLY match the SPEC §5 block. `FlagKind::Boolean` is a unit variant `:125`. The `inspect`/`decode` idiom (md.rs `:27-43`,`:185-201`) confirms the pattern. `md repair` is uncfg-gated upstream (`main.rs:222`; `--json` flag NAME always present, only the emit path is cfg=json) and is the SOLE md schema gap (9 binary vs 8 schema). Appending `repair` last (after `address`) is the correct relative order (schema omits the hidden `gui-schema`).

## SECRET_NODE_TYPES-unchanged proof (§3) — DECISIVE
The const-asserts compare the toolkit-imported consts (resolved at the pinned tag) vs the v0.3.3 snapshots. At the v0.41.0 pin the imported values = master: NODE_TYPES 8 (`:76-85`), SLOT_SUBKEYS 6 incl ms1 (`:111`) — byte-identical to the draft snapshots → both asserts COMPILE at v0.41.0. CHANGELOG dates the only SLOT_SUBKEYS delta (ms1) to v0.41.0; no NodeType added v0.38–v0.41. Draft's SLOT_SUBKEYS-only snapshot bump is sufficient; NODE_TYPES snapshot needs NO edit.

## pin_coherence (§6) — feasible + real guard
`toml` is both runtime dep (`Cargo.toml:26`) AND dev-dep (`:73`) → typed parse available (M2: prefer it; pinned-upstream has 4 `tag=` lines so anchor on `[mnemonic]`). Guard FAILS today (Cargo v0.37.3 `:42` != pinned-upstream mnemonic v0.38.0 `:22`); PASSES after the lockstep bump to v0.41.0. TDD-exercisable per §8 P1.

## Completeness / gate / SemVer
- `ci_workflow_snapshot` (schema_mirror.rs:163-241) asserts step NAMES + `<cli>_tag` output refs, NOT literal tag VALUES (the v0.5.1 cleanup removed literal-pin asserts) → bumping pinned-upstream does NOT break it.
- All gui-schema/help-consuming tests honor `*_BIN` (resolve_bin `:46-50`); none runs against a stale `$PATH` binary when `*_BIN` set. `gui_schema_conditional_drift`/`xpub_search_schema_mirror`/template-groups parity all pass against v0.41.0 (no conditional/template-group surface changed v0.38→v0.41).
- Only `mnemonic_toolkit::` code import is `secrets.rs:34` → v0.41.0 pin bump compile-safe (no module move/rename).
- md `--features cli-compiler` keeps `default=["json"]` on → `md gui-schema` + `md repair --json` work.
- `+1.94.0` correct (GUI transitively needs ≥1.88; GUI CI `@stable`); GUI `rust-version=1.85` unchanged, no newer-edition GUI code added.
- MINOR/0.22.0 justified (CHANGELOG convention: new SubcommandSchema = MINOR, v0.20.0/v0.21.0). Single-cycle all-pins-together is the minimal set for CI green (schema_mirror runs one installed binary SET), NOT scope creep.
- Sibling tags exist locally for the §7 build (mk-cli-v0.7.0/ms-cli-v0.7.0/md-cli-v0.6.2/mnemonic-toolkit-v0.41.0).

## Minor (4) — non-blocking, folded into the SPEC
- **M1** §1 narrative: banners WERE silently bumped too (not schema files alone) — prose tightened.
- **M2** §6: prefer the already-present `toml` dev-dep over string-scan (4 `tag=` lines hazard) — folded.
- **M3** §4 item 8: also bump cosmetic module-doc headers (mnemonic.rs:1/md.rs:1/mk.rs:1 + schema_mirror.rs:402) — folded.
- **M4** §7: pre-flight `git ls-remote` the toolkit v0.41.0 tag (git-dep resolves against GitHub, not local) — folded.

## Verdict rationale
Every load-bearing citation ACCURATE; the §3 proof decisive (const-asserts compile at v0.41.0); the §5 md-repair field-set field-accurate against the live schema definitions + the sole md gap; the §6 guard feasible/real; the §7 gate toolchain-correct with no other test breaking on the pin bump; MINOR/single-cycle scope justified. No Critical/Important. The 4 Minors are cosmetic/operational/narrative, folded without materially editing the edit list → no re-dispatch needed. **R0 GREEN (0C/0I) — implementation may proceed.**
