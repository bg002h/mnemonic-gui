# Phase 2 (GREEN) Review — mnemonic-gui v0.28.0 pin-bump-v0.47.3

> Persisted verbatim from the opus `feature-dev:code-reviewer` agent
> (`agentId: aa4658ca44597bb59`). Static-only (no Bash in the review env); the
> operator ran the full `cargo +1.94.0 test --no-fail-fast` + clippy + drift
> gates GREEN before/after this review. M2 folded after the review (see note).

---

## VERDICT: 0 Critical / 0 Important (+ 2 Minor)

Reviewing commits `ca77b7f` (Phase 1 RED) + `08a2534` (Phase 2 GREEN) on branch `gui-v0.28.0-pin-bump-v0.47.3`.

## Verified Clean
1. **Schema fix correct + complete.** `src/schema/mnemonic.rs:1038-1044` `--timestamp` `default_value: Some("0")` + help updated; `kind: FlagKind::Timestamp` unchanged; module-doc `:1` = v0.47.3; `pinned_version` `:3688` = "mnemonic 0.47.3". `Some("now")` has zero occurrences in `src/`.
2. **Fix resolves the bug, not vacuous.** `invocation.rs:79-82`: `is_at_default(Now,"0")` = `"0"=="now"` = false → explicit Now emits `--timestamp now`. `widget.rs:184-187` seeds `Timestamp → Unset` → default form emits nothing → toolkit applies `0`. `Unix(_) => false` unchanged. No `is_at_default` change needed/made.
3. **Test inversions discriminating.** `d33_timestamp_now_is_emitted_when_default_is_zero` + `cell_3b_export_wallet_timestamp_now_argv` assert the EMIT behavior (RED under "now", GREEN under "0"). Old `d33_timestamp_now_at_default_suppresses` confirmed deleted. `cell_3` comment updated (R0 M3).
4. **Pin bump lockstep complete.** Cargo.toml tag v0.47.3 + version 0.28.0; Cargo.lock toolkit 0.47.3 + source rev `8502723a…` (the tag commit, NOT recon SHA `d509361`) + self 0.28.0; pinned-upstream.toml v0.47.3; README:42 self `mnemonic-gui-v0.28.0` + README:50 toolkit v0.47.3. md/ms/mk unchanged.
5. **No stray version drift.** All live `v0.46.2`/`0.27.0` hits are historical-fact comments / help-string annotations / frozen design history — not stale pin sites.
6. **FOLLOWUP resolution accurate.** `FOLLOWUPS.md` `gui-timestamp-default-value-drift-v0.47.3` resolved v0.28.0 with correct companion cross-cite; no-manual-gui-change note accurate (GUI repo has no `docs/`); toolkit-repo prose deferred to `manual-gui-export-wallet-timestamp-default-now-stale`.
7. **schema_mirror does not gate default_value** — `schema_mirror.rs:52-54` `schema_flag_names` collects `f.name` only. Confirmed by source. The fix can't break/require schema_mirror.
8. **CHANGELOG [0.28.0] accurate**, not overclaiming.

## Minor
**M1 — toolkit companion FOLLOWUP still `Status: open`** (`mnemonic-toolkit/design/FOLLOWUPS.md` `gui-timestamp-default-value-drift-v0.47.3`). SPEC §3e deferred the toolkit-side flip to "a separate trivial toolkit doc commit." Non-blocking; close at the next toolkit touch.

**M2 — stale "now" in `d33_timestamp_epoch_never_matches_now_default` name + assert message.** SPEC R0 M3 marked this optional; logic correct. Non-blocking.

**Phase 2 is cleared to ship** once the operator confirms the gate suite green (it was).

---

## Operator note (folds + ship)
- **M2 folded:** renamed `d33_timestamp_epoch_never_matches_now_default` → `d33_timestamp_epoch_always_emits` + reworded comment/message (no "now"-default framing); re-run GREEN.
- **M1** (toolkit companion flip → resolved + file `manual-gui-export-wallet-timestamp-default-now-stale`) is a TOOLKIT-repo change, done as a separate toolkit-side commit after the GUI v0.28.0 ships.
- Gate suite (operator-run, `cargo +1.94.0`, 4 pinned bins): full suite 0 failed; clippy 0; schema_mirror + gui_schema_conditional_drift + schema_mirror_secret_drift + xpub_search_schema_mirror + pin_coherence + readme_pin_coherence GREEN. **Phase 2 GREEN — cleared for ship.**
