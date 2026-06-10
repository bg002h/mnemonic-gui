# Implementation review — GUI v0.33.0 secret flips + pin bump (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Spec @ design/SPEC_gui_v0_33_0_secret_flips_pin_bump.md (R0 GREEN r2). Verdict: GREEN (0 Critical / 0 Important / 1 cosmetic Minor — no fix required). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

1. **Test-name shorthand in docs** — `CHANGELOG.md` ([0.33.0] bullets 3/4) and `FOLLOWUPS.md:78/:88` cite `tests/secret_flips_v0_33_0.rs::t1`/`::t2`/`::t3`, but the actual test fns are `t1_ms_repair_ms1_is_secret_by_deliberate_override`, `t2_checked_secret_stdin_toggles_emit_nothing`, `t3_phrase_is_secret_on_all_three_xpub_search_modes`. The prefixes are unambiguous (grep-resolvable), so this is cosmetic; optionally use full names for citation-decay resistance. No fix required.

## Verdict

**GREEN (0 Critical / 0 Important)** — all six review axes verified empirically:

1. **Pin bump — all 6 sites correct, nothing extra.** `Cargo.toml:42` tag `mnemonic-toolkit-v0.53.1`; `Cargo.lock` rev `87c33c591467cebe4cc8af936af9904d05ec884b` = `git rev-parse mnemonic-toolkit-v0.53.1^{commit}` exactly (verified); `pinned-upstream.toml:22`; `README.md:50` toolkit install line; `src/schema/mnemonic.rs:1` module-doc + `:3949` `pinned_version: "mnemonic 0.53.1"`. Self-version 0.32.0→0.33.0 in `Cargo.toml` + `Cargo.lock` + README self-pin line `:42`. Cargo.lock diff is exactly the 2 stanzas (gui version + toolkit source/version) — no collateral dep churn. The 4 transcription-provenance "v0.52.0" comments (archetypes.rs/nodes.rs/conditional.rs/archetype_form.rs) correctly left untouched per spec §1.

2. **Flips — exactly 9 + 1.** `src/schema/mnemonic.rs` diff is exactly 11 hunks: module-doc, pinned_version, and 9 `secret: false`→`true` at `--phrase`/`--phrase-stdin`/`--ms1-stdin` × PATH_OF_XPUB/ACCOUNT_OF_DESCRIPTOR/PASSPHRASE_OF_XPUB tables (address-of-xpub untouched). Zero collateral flips. `ms.rs:321` flip carries the full override comment (master-secret rationale, no-gate-coverage, FOLLOWUPS cross-cite) matching spec §3.

3. **Tests match spec §6.** T1 pins the ms.rs override with a do-not-reconcile failure message; T2 covers both toggles × all 3 modes with actionable per-pair messages (and is discriminating: pre-flip, a non-secret `Boolean(true)` takes the generic emit path at `invocation.rs:347` → would have emitted); T3 asserts both `flag_is_secret()` (the live widget-dispatch predicate, `main.rs:749` confirms run-confirm wiring is live) and the raw `FlagSchema.secret` bit. The 3 `xpub_search_widgets.rs` conversions seed `secret_widgets` via `SecretLineEdit::from_text` exactly per the prescribed pattern and **retain every pre-existing assert verbatim** (compared against `git show HEAD:` — path-of-xpub keeps both `--phrase` :75 and value :76; account :144 and passphrase :290 keep `--phrase`; no assert weakened or deleted). All 3+8 cells pass.

4. **Gate integrity — empirically proven.** With `MNEMONIC_BIN=…/mnemonic-toolkit/target/release/mnemonic` (`--version` = `mnemonic 0.53.1`), `schema_mirror_secret_drift` PASSES now. Scratch-reverted ONE flip (path-of-xpub `--phrase`) → gate REDs with exactly `only in toolkit: [("xpub-search-path-of-xpub", "--phrase")]` → restored; sha256 of `src/schema/mnemonic.rs` identical before/after (`a2ade88c…`), final `git status`/`diff --stat` match the as-found tree (71 insertions / 36 deletions, same file set).

5. **Docs accurate, no overclaims.** CHANGELOG [0.33.0]: every claim checked against the diff (9 flips, override + t1 pin, census 18→24, converted-not-deleted asserts, redaction-union additions limited to the 2 stdin names; run-confirm claim is live wiring; it correctly does NOT claim a live paste-warn modal). Census arithmetic verified by grep: 23 `secret: true` stdin-toggle sites (12+2+2+1+3+3) + the `ms.rs:275` name-matched `secret: false` `--passphrase-stdin` = 24; 6 names enumerated; the "the pre-v0.33.0 body said 5 while enumerating 4" correction is faithful to the old text. FOLLOWUPS entry headers at :65/:73/:82 — the toolkit entry's `:81→:82` correction is now true of the actual file; audit index line :17 marked resolved. Toolkit-side `gui-secret-mirror-phrase-ms1-stdin` resolution + the structural correction are accurate: the drift gate does compare `FlagSchema.secret` per `(subcommand, flag)` (`tests/schema_mirror_secret_drift.rs:105-112`), and the stale "token-for-token" claim does live in `crates/mnemonic-toolkit/src/secrets.rs` module-doc lines 6-9 (residual-errand note correct). Toolkit working tree carries only the `design/FOLLOWUPS.md` edit.

6. **Suite + clippy.** Full `cargo test` with `MNEMONIC_BIN` + `MS_BIN=/tmp/pinned-sib/bin/ms` (0.7.0) + `MK_BIN=/tmp/pinned-sib/bin/mk` (0.7.0): all test binaries 0 failed; sole ignore is the pre-existing documented `cell_manual_anchor_coverage_against_built_html`. `cargo clippy --all-targets -- -D warnings` clean.
