# gui-ms1 catch-up — Plan R0 Review (round 0)
**Verdict:** GREEN (0C / 0I)

Plan `design/IMPLEMENTATION_PLAN_gui_v0_22_0.md` (`7f11acb`). Faithfully + implementably realizes the R0-GREEN SPEC; every load-bearing citation independently re-grepped.

## Critical (0) / Important (0) / Minor (3, narrative-only)

## SPEC→plan coverage — COMPLETE
All §4 items 1-11 + §3 + §5 + §6 + §7 + §9 map to tasks. Cargo.toml:42 confirmed inline-table `mnemonic-toolkit = { git=…, tag="mnemonic-toolkit-v0.37.3" }`; Cargo.toml:3 "0.21.3"; pinned-upstream `[mnemonic]`:22 v0.38.0 / `[md]`:38 v0.6.1 / `[ms]`:45 v0.5.0 / `[mk]`:52 v0.6.0, stale comment :32-36; draft slot_editor.rs:44/54/67/82 + secrets.rs:67-68 present; README stale :42/:50-53; banners mnemonic.rs:3452/md.rs:532/mk.rs:476 (ms.rs already current); FOLLOWUPS.md:58/64. §3 decisive: toolkit v0.37.3 SECRET_NODE_TYPES = 8-entry byte-identical to master; SECRET_SLOT_SUBKEYS 5→6 (+ms1) — NODE_TYPES const-assert holds at both pins, only SLOT_SUBKEYS drives the v0.37.3 non-compile.

## pin_coherence (Task 1.1) — TRIAL-VERIFIED COMPILES + WORKS
`toml` is dev-dep (Cargo.toml:73) + runtime dep (:26). The inline-table dep means `cargo["dependencies"]["mnemonic-toolkit"]["tag"].as_str()` reaches the right scalar (form-agnostic for inline + sub-table). `pinned["mnemonic"]["tag"]` correctly disambiguates the `[mnemonic]` table's tag (:22) from the other 3 `tag=` lines (the M2 typed-parse rationale). FAILS on current tree (v0.37.3 != v0.38.0); PASSES after both → v0.41.0. `.expect()` messages name the table path; inline dep exposes `["tag"]` so no confusing index-panic.

## md-repair (Task 1.3) — FIELD-ACCURATE + idiom-correct
`schema/mod.rs`: FlagSchema:64-110 {name,kind,required,repeating,help,secret,default_value,global}; SubcommandSchema:28-48 {name,human_name,flags,positional_args,allows_slots,conditional}; PositionalArgSchema:51-61 {name,required,repeating,help}; FlagKind::Boolean:125 — all match. md-cli v0.6.2 repair.rs:42-43 (md1_strings required+num_args=1..) + :48-49 (json bool). INSPECT_* (md.rs:27-43) is the byte-shape idiom. Append after `address` (SUBCOMMANDS:463-528, address last :520-527) is correct relative order.

## TDD-ordering — SOUND
Branch does NOT compile at v0.37.3 (draft's 6-entry snapshot fires secrets.rs:89-99 E0080). Task 1.2 Step 3 (`cargo build` post-bump) is the FIRST compiling state; FIRST commit is Task 1.2 Step 5 (post-green) — no non-compiling intermediate committed as "done". Task 1.1 writes guard + observes red but explicitly does NOT commit. Phase-0 sizing runs from MASTER (compiles at v0.37.3 — master has the non-draft 5-entry snapshot). pin_coherence red→green is genuine TDD.

## gate — CORRECT
`+1.94.0` justified (GUI transitively ≥1.88; GUI rust-version=1.85 unchanged; CI @stable). `*_BIN` wiring: resolve_bin (schema_mirror.rs:47-50) + schema_check.rs:338-339 honor `<CLI>_BIN`. ci_workflow_snapshot (schema_mirror.rs:163-241) asserts step NAMES + output refs, NOT tag VALUES → pin bump safe. gui_schema_conditional_drift / xpub_search_schema_mirror / schema_mirror_secret_drift all `*_BIN`-honoring, pass at v0.41.0.

## ship — CORRECT
Tag `mnemonic-gui-v0.22.0` matches convention (build.yml prefix-strip, schema_mirror.rs:268-273). Ship sequence (clean tree→checkout master→merge --ff-only→annotated tag→push; then toolkit FOLLOWUP-flip commit) correct. FOLLOWUP flip target `gui-ms1-slot-subkey-pending-pin-bump` (FOLLOWUPS.md:58, Status :64). No GUI README version-marker guard exists → plan correctly omits a readme_version step.

## Minor (3) — narrative-only, folded
- **m1 (Task 1.3 Step 2 framing):** `md repair`'s absence is caught by NO gate (schema_mirror.rs:91-121 iterates only schema-declared subcommands; gui_schema_conditional_drift.rs:231-235 skips binary-only subcommands). The md cell was ALREADY green; adding repair KEEPS it green + closes an ungated coverage gap. Reworded Step 2 accordingly. FOLDED.
- **m2 (Phase 0 framing):** same correction — no failure mode from md repair on master; reworded the sizing-run expectation to "zero flag-NAME drift on every schema-declared cell." FOLDED.
- **m3:** md.rs:1 module-doc header already in Task 2.1 Step 3 (M3) — no gap.

## Verdict rationale
Every SPEC §4 item maps to a source-accurate task; the pin_coherence code compiles + works against the real inline-table Cargo dep + `[mnemonic]`-anchored pinned-upstream tag (fails on live drift, passes after lockstep bump); the md-repair field-set is byte-accurate; TDD ordering commits no non-compiling intermediate; the gate/toolchain/`*_BIN` wiring is correct and no other test breaks on the pin/banner bump; the ship sequence + tag name + FOLLOWUP target are correct. The 3 Minors are narrative-only (an over-tight RED→GREEN framing of the ungated `md repair` coverage gap), folded inline. **R0 GREEN (0C/0I) — implementation may proceed.**
