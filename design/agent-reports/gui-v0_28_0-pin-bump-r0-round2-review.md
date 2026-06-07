# R0 Architect Review — mnemonic-gui v0.28.0 pin-bump-v0.47.3 — Round 2

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a74a669393319eb93`). Confirms the round-1 folds + completeness.

---

## VERDICT: 0 Critical / 0 Important (+ 0 Minor)

**GREEN — cleared for implementation.**

## Folds verified
- **I1** — `src/schema/mnemonic.rs:1` reads `mnemonic-toolkit-v0.46.2`; `:3688` reads `pinned_version: "mnemonic 0.46.2"`. Both fold targets live + correct; format matches CHANGELOG precedent.
- **M1** — `README.md:42` reads `mnemonic-gui-v0.27.0`; `readme_pin_coherence.rs:75` asserts the `mnemonic-gui` tag == `mnemonic-gui-v{Cargo.toml version}` → RED if not bumped with the version. Correctly in §3a.
- **M2/M3** — non-load-bearing comment touch-ups; correctly categorized.
- **M4** — §3a says let `cargo update` resolve the tag commit; don't paste recon SHA. Correct.
- **§3d** — GUI repo has no `docs/`; stale prose is in the toolkit repo's `docs/manual-gui/.../45-export-wallet.md:30,340-343` → toolkit FOLLOWUP. Correct disposition.

## Pin-bump site completeness (item 2)
All live `0.46.2`/`v0.46.2` hits accounted for. Bump targets: `Cargo.toml:42`, `Cargo.lock:2296-2297`, `pinned-upstream.toml:22`, `README.md:50`, `src/schema/mnemonic.rs:1`, `src/schema/mnemonic.rs:3688`. NOT bump targets (historical-fact comments): `src/schema/mnemonic.rs:3454`, `src/form/conditional.rs:926`, `tests/gui_schema_conditional_drift.rs:300`, `tests/conditional_visibility.rs:1076`, CHANGELOG. No live site missing from §3a.

## Version-bump site completeness (item 3)
All live `0.27.0` sites: `Cargo.toml:3` (version), `Cargo.lock` (self), `README.md:42` (self-tag, M1), CHANGELOG (new `[0.28.0]`). The `src/schema/mnemonic.rs` "v0.27.0 — …" help strings + `tests/cli_envelope_smoke.rs` comments are historical records of when flags were added — NOT self-version banners, no update needed. No live self-version site missed.

## readme_pin_coherence gates (item 5)
`readme_pin_coherence.rs:74-80` drives five packages: `mnemonic-gui`, `mnemonic-toolkit`, `md-cli`, `ms-cli`, `mk-cli`. After the changes: mnemonic-gui (README:42 v0.28.0 vs Cargo version) GREEN; mnemonic-toolkit (README:50 v0.47.3 vs pinned-upstream tag) GREEN; md/ms/mk unchanged GREEN. No additional gated install line missed.

## Test completeness for the `Now` inversion
`TimestampValue::Now` in tests: `argv_assembler.rs:134` (`cell_3b`, suppression → invert), `argv_assembler.rs:491` (`d33_timestamp_now_at_default_suppresses`, suppression → invert), `widget_unset_sentinel.rs:128` (`seeded_value_for_timestamp_returns_now` — widget click-to-Set path, does NOT consult `default_value`/`is_at_default`/argv → STAYS GREEN unchanged). The two argv-suppression tests are exactly §3c's invert list.

## No fold-induced contradiction (item 4)
§3d out-of-scope is consistent with §3 scope (GUI repo only). §6 ratifications 1–5 consistent with §2a/§3: widget-init (Unset→nothing→toolkit `0`) unchanged; MINIMAL fix (schema `default_value` only) stands; 4-test blast radius complete.

**Implementation may proceed to Phase 1 (RED) per §5.**
