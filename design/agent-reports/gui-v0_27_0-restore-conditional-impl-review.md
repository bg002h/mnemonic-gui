# Implementation Review (Phases 2-3) — mnemonic-gui v0.27.0 (restore conditional consume + README pin guard)

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory per-phase review). **Date:** 2026-06-06.
**Branch:** `gui-v0.27.0-restore-conditional-consume-readme-guard`. **Verdict:** **0 Critical / 0 Important / 0 Minor.** **GREEN — cleared for ship (Phase 4).**

> Persisted verbatim per CLAUDE.md. Reviewer found 0C/0I/0Minor statically and withheld only the empirical sign-off (it had no shell). The operator ran all deferred checks GREEN — recorded in the "Operator-run" block below.

---

## VERDICT: 0 Critical / 0 Important / 0 Minor (static) — empirical checks operator-confirmed GREEN

### Statically verified CLEAN (0C/0I)

**Item 4 — all 3 stale comments rewritten + accurate; no 4th survives.**
- `src/form/conditional.rs:926-934` — `restore()` doc rewritten: now states toolkit v0.46.2 projects the rule via `restore_conditional_rules()` and it IS drift-gated. Body (`:935-941`) emits `if !has_value("--md1") { push("--from", Required) }` — correct.
- `src/schema/mnemonic.rs:3454-3458` — restore `SubcommandSchema` comment rewritten to the projected/drift-gated framing with `("restore", 1)` in `SUBCOMMAND_FLOORS`. Accurate.
- `tests/conditional_visibility.rs:1075-1078` — re-pointed to "since toolkit v0.46.2 … now toolkit-projected + drift-gated." Accurate; the two cells call `run_conditional` directly.
- No 4th stale instance. The `mnemonic.rs:3579` hit (`conditional: None`) is `ms-shares-split/combine`, which legitimately emit `[]` — not stale-restore-class.

**Item 5 — pin/version coherence (all sources agree on v0.46.2 / 0.27.0).**
- `Cargo.toml:42` dep tag `mnemonic-toolkit-v0.46.2`; `:3` version `0.27.0`.
- `pinned-upstream.toml:22` `[mnemonic].tag = mnemonic-toolkit-v0.46.2` (== Cargo dep → `pin_coherence` passes).
- `Cargo.lock:2295-2297` `mnemonic-toolkit v0.46.2`, git tag `mnemonic-toolkit-v0.46.2`, SHA `b74badd` (matches SPEC Source SHA).
- `src/schema/mnemonic.rs:1` module-doc `mnemonic-toolkit-v0.46.2`; `:3688` `pinned_version: "mnemonic 0.46.2"`.
- README: self `mnemonic-gui-v0.27.0` (== Cargo version), toolkit `mnemonic-toolkit-v0.46.2`, md/ms/mk current.
- **`:2746` provenance comment NOT swept** — still "toolkit v0.46.0: scan a file of candidate passphrases…" (correct; `--passphrase-candidates-file` shipped v0.46.0). Anti-blind-`sed` respected.

**Item 3 — `readme_pin_coherence` correct + non-vacuous (by code structure).** Parser tokenizes on `split_whitespace` (alignment-tolerant); tag = token after `--tag`, pkg = trailing token; all 5 lines parse. Drives from a fixed 5-element expectations array, `panic`s on missing pkg, `assert_eq!`s each tag against its source of truth (Cargo version for self; pinned-upstream sections for siblings). Cannot vacuously pass.

**Item 2 (binary-independent half) — FLOORS math + GUI/toolkit shape match.** `("restore", 1)` added; total `>= 35`; math 11+10+6+4+3+1 = 35. Toolkit `restore_conditional_rules()` (`cmd/gui_schema.rs:354-372`) emits exactly 1 rule `Not(FlagPresent "--md1") → {--from, Required}`; GUI `conditional: Some(crate::form::conditional::restore)` wired. `synthesize_satisfying(Not(...), default())` → empty state → `restore()` pushes `("--from", Required)` → gate matches.

**Item 7 — `restore()` body emits correct logic** (`--from` Required ⟺ `--md1` absent).

**CHANGELOG** — `[0.27.0] — 2026-06-06` accurate (both slugs, pin bump, no-flag-delta rationale, FOLLOWUP resolutions).

---

### Operator-run empirical checks (the gate the reviewer couldn't close — all GREEN)

With all 4 pinned bins exported (`MNEMONIC_BIN`=v0.46.2, `MS_BIN`=0.7.0, `MK_BIN`=0.7.0, `MD_BIN`=0.6.2), `cargo +1.94.0`:

1. **Full suite `cargo +1.94.0 test -p mnemonic-gui --no-fail-fast` → 0 failures.** (The earlier ms/mk schema_mirror failures were purely the stale-PATH ms 0.4.0/mk 0.4.1; with `MS_BIN`/`MK_BIN` set, `schema_mirror` is **21/21**.)
2. **`$MNEMONIC_BIN gui-schema | jq '…restore…conditional_rules|length'` == 1.** Drift gate exercises restore (floored at 1, not skipped); `gui_schema_conditional_drift` 5/5.
3. **`readme_pin_coherence` 1/1 GREEN + proven non-vacuous** (temporarily desynced the toolkit pin → RED with the exact drift message, reverted).
4. **`pin_coherence` 1/1 GREEN** (Cargo tag == pinned-upstream, both v0.46.2).
5. **clippy `cargo +1.94.0 clippy -p mnemonic-gui --all-targets` → exit 0.**
6. **Diff scope (`git diff --stat f6caa20..HEAD`)** = exactly the 5 src/test files + Cargo.toml/lock + pinned-upstream + schema/mnemonic.rs + README + CHANGELOG + SPEC + 3 agent-reports. No stray edits.
7. **`git diff f6caa20..HEAD -- src/form/conditional.rs`** restore() body diff is EMPTY (only the doc-comment moved; body byte-unchanged).

---

**Bottom line: 0 Critical / 0 Important / 0 Minor. GREEN — cleared for ship (Phase 4).**
