# R0 review — SPEC_gui_v0_32_0_node_tree_builder — round 1
**Verdict: YELLOW** (0C / 6I / 6M — inheritance faithful; gaps are SPEC-internal underdefinitions, one false normative law, two delegated decisions unmade)

## Critical
None.

## Important
**I1.** The round-trip law `from(to(t)) == t` is FALSE for wide nodes (projection loses inactive fields + surplus children — exactly what preserve-and-flag creates). Right law: `to(from(j)) == j` for valid specs + `from(to(t)) == projection(t)` (equivalently to-idempotence).
**I2.** Plant classes unpinned for 3 of 6 — and class choice matters: type-class plants localize to ROOT (probed: thresh-sub type error → root). Verified-working classes: root=sigless (bare older); andor[1]=sigless; thresh.subs[i]=schema_field (nested k>n); wrap.sub=schema_field (short hex); keys[i]=secret_key; binary arm=sigless. Rule: node-addressed classes only.
**I3.** from_spec_json can't mint globally fresh ids (no next_id access); stale-low next_id after import collides ids (egui identity breakage). Thread &mut next_id or define post-import next_id = max_id(root)+1 + cell.
**I4.** Mode-selector state model undefined — the one place v0.31.0 behavior could change. If only the tree bit is stored and Generic/Archetype stays dropdown-derived, v0.31.0 is provably unchanged and old state.json migrates clean (None → non-tree). Say exactly that.
**I5.** The unmatched-node_path fail-soft rule (brainstorm decision 6) was dropped — weakened by omission; restate + cell ("params"/"root.bogus[9]" → strip, never floor/panic).
**I6.** Validate ↔ last_run interplay delegated-but-undecided: does Validate write the bottom output panel? Decide.

## Minor
**M1.** Fixed argv missing argv[0] ("mnemonic"). **M2.** "(choose…)" reuses the IDIOM not a mechanism (the v0.30.0 mapping is an inline two-liner in the FlagValue Dropdown arm) — extract a helper or reword. **M3.** k<1 = unset-sentinel rationale (fence off semantic creep; NO k≤n GUI-side — the gate owns it, probe-confirmed addressed). **M4.** Copy spec JSON gated by the same completeness gate. **M5.** from_spec_json checks schema_version==1 + wrapper=="wsh" on import (runtime parity with the test-time refuse-loud pin). **M6.** Cells: old state.json → None/non-tree; collapse/expand does NOT clear diagnostics.

## Inheritance audit
All brainstorm decisions land OK (table in full review); the single weakening = I5 (omission). Shape mapping verified byte-for-byte (17 kinds/9 shapes, order + verbatim payload strings). serde-skip Default non-issue. install.sh currently v0.31.1 (bump target right).

## Empirical probes run
Schema dump (17/9/both-1); Probe A root sigless@root; B andor sigless@root.andor[1]; C tprv secret_key@root.multi.keys[1]; D NEGATIVE type-class → root (drives I2); E nested k>n schema_field@root.thresh.subs[1]; G short-hex schema_field@root.and_v[1].and_v[0].wrap.sub; H --format+--json compose exit 0.
