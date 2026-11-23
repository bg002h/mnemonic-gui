# FIX sec-secret-drift-gate-fail-closed: the secret-flag drift gate fails instead of passing on bad input

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23. Track: small. Class: test gate (no holder-visible change).
Base: mnemonic-gui `origin/master` 922a0b38 (`tests/schema_mirror_secret_drift.rs` unchanged since 0ba4aa0, so the
grader's 336e544 line numbers hold). Closes security-sweep unit `secret-drift-gate-fail-closed` (coordinator
`reports/security-sweep/grade/routing.md:31`, grade row tsv387, secret id `gui-secret-drift-gate-silent-skip`,
regraded M -> I) and repo `FOLLOWUPS.md:81` `secret-drift-gate-version-skip-silent`.

## 1. Defect (at 922a0b38)

- `tests/schema_mirror_secret_drift.rs:61-82` `fetch_v5_schema` returns `None` when the binary cannot be spawned
  (`.output().ok()?`, `:63`), exits non-zero (`:64-70`), prints non-UTF-8 (`:71`), prints text that is not a
  gui-schema document (`:72`, silent), or reports `version < 5` (`:73-80`).
- The gate test (`:85-91`) treats every `None` as SKIP and returns, so the test PASSES.
- CI runs this test with `MNEMONIC_BIN: mnemonic` (`.github/workflows/schema-mirror.yml:196-203`, the
  `cargo-test-full-suite` step). A broken, regressed or mis-installed pinned binary therefore turns the
  secret-taxonomy drift gate into a false PASS. The only other check on that binary,
  `schema-mirror.yml:173` `smoke-gui-schema-mnemonic`, asserts `version>=1`.
- `MNEMONIC_BIN` unset falls back to `mnemonic` on `PATH` (`:54-56`) with the same silent skips.

## 2. Spec delta (the rules the build implements)

No SPEC in `design/` states this gate's skip contract (`git grep schema_mirror_secret_drift -- design/*.md` finds
only CHANGELOG mentions and agent reports). The rules live in the test file's module doc, which the build rewrites
(see "Doc-comment changes" below).

Rule B (which binary). `MNEMONIC_BIN` is "set" when `std::env::var_os("MNEMONIC_BIN")` is `Some`, whatever its
value (an empty value is set and fails at spawn). Set: the gate runs that value. Unset: the gate runs `mnemonic`
through `PATH`.

Rule S (the only skip). The gate SKIPS (test passes) only when `MNEMONIC_BIN` is unset AND spawning `mnemonic`
fails with `std::io::ErrorKind::NotFound`. It prints this one line to stderr and returns:

    schema_mirror_secret_drift: SKIP: MNEMONIC_BIN is unset and `mnemonic` is not on PATH; set MNEMONIC_BIN to the pinned toolkit binary to run this gate

Rule F (fail closed). In every other case the gate test FAILS (panics) when the binary does not yield a v5+
document, with exactly one of these messages. `<bin>` is the binary as run, rendered with `Path::display`
(the `MNEMONIC_BIN` value, or `mnemonic` when unset); `<err>` is the error's `Display`; `<status>` is the
`ExitStatus` `Display` (`exit status: 3`, or `signal: 9 (SIGKILL)`):

| case | message |
|---|---|
| F1 spawn fails (set: any error; unset: any error except NotFound) | ``schema_mirror_secret_drift: cannot run `<bin> gui-schema`: <err>`` |
| F2 exits non-zero or by signal | ``schema_mirror_secret_drift: `<bin> gui-schema` failed: <status>`` |
| F3 stdout is not UTF-8 | ``schema_mirror_secret_drift: `<bin> gui-schema` stdout is not UTF-8`` |
| F4 stdout does not deserialize as the gate's v5 root (`version`, `cli`, `subcommands`) | ``schema_mirror_secret_drift: `<bin> gui-schema` stdout is not a gui-schema JSON document: <err>`` |
| F5 `version < 5` | ``schema_mirror_secret_drift: `<bin> gui-schema` reports schema version <v>; this gate needs version 5 or later (per-flag `secret` field)`` |

The F4 message never includes stdout (gui-schema output is not secret, but the message stays coordinate-only like
`ui_harness_i4_realcli.rs`). Checks run in the order F1..F5; the first that applies is the message. A v5+ document
proceeds to the existing `cli == "mnemonic"` assertion and set comparison, both unchanged (texts at `:92`, `:117-123`).

Why Rule S keeps the `PATH` fallback: it is the skip rule of every sibling real-binary gate in this repo
(`archetype_schema_mirror.rs:27`, `canonicity_drift.rs:25`, `bundle_restore_independent_oracle.rs:37`,
`schema_mirror_defaults_drift.rs:34`), and it is stricter than skipping on unset alone: a stale `mnemonic` on a dev
`PATH` now fails loudly instead of being ignored. The routing row requires failure when `MNEMONIC_BIN` is set; Rule F
also fails the unset-but-found case, which the row does not forbid.

Doc-comment changes the build makes (code comments, not holder text):
- Module doc `:19-22`: replace "If the binary cannot be located OR the output isn't v5+, the cell SKIPS (so dev-laptop
  runs without the v5 binary don't spuriously fail)." with Rules B, S and F in at most four sentences, citing this
  FIX. Keep the first sentence of that paragraph, but drop "mirrors `schema_check::json_flag_names`" (it no longer
  does: `json_flag_names` falls back to `pinned-upstream.toml`'s `bin` and returns `None` on failure).
- `fetch_v5_schema` doc `:58-60` and the SKIP comment `:87-90`: state Rule S as the only skip.

Printed texts and exit codes: no CLI, GUI or holder-visible text changes. The five F messages and the S line are
test-harness output (seen by a developer or in a CI log); the test exit is libtest's / nextest's (nextest exit 100
on a failure). WORDING_REVIEW proposals: none (no holder-visible wording; the texts above are the record).
Toolkit manual lockstep: none (no `mr`/`md-cli` flag or printed text; `git grep schema_mirror_secret_drift
origin/master -- docs` in mnemonic-toolkit finds nothing). No `.github/` change: CI already sets `MNEMONIC_BIN`, so
Rule F is live in `cargo-test-full-suite` once the build lands.

## 3. Production change (what, not how)

No production (`src/`) code changes. `tests/schema_mirror_secret_drift.rs` only: `fetch_v5_schema` reports the three
outcomes (document, skip, failure message) instead of `Option`, and the gate test panics with the failure message.
The function's shape is the builder's choice; the texts and the rules above are not.

Author's feasibility check (not committed): a 30-line prototype of Rules B, S, F on 922a0b38 turned the nine
section-4 tests to 9 passed (with `MNEMONIC_BIN` unset and with `MNEMONIC_BIN=mnemonic`, local toolkit 0.107.1), and
`cargo clippy --locked --all-targets -- -D warnings` stayed clean. Mutant "skip on NotFound even when set" (drop the
`unset` condition from Rule S) fails `drift_gate_fails_when_bin_set_and_absent` (measured: 8 passed, 1 failed).
Mutant "always fail" fails `drift_gate_passes_on_matching_v5_document` and
`drift_gate_skips_loudly_when_bin_unset_and_not_on_path` (by construction: both assert a child exit 0).

## 4. RED rows

The tests ship as a patch next to this file: `design/FIX_sec_secret_drift_gate_fail_closed.red.patch` (adds module
`gate_contract`, `#[cfg(unix)]`, to `tests/schema_mirror_secret_drift.rs`: 8 tests). Each test re-runs the existing
gate test in a child process (`std::env::current_exe()` with `--exact
secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode --nocapture`) with `MNEMONIC_BIN` or `PATH` pointed at a
fake `mnemonic` (`/bin/sh` script in a `tempfile` dir; `tempfile` is already a dev-dependency), and asserts the
child's exit and its output. No signet or mainnet data is involved. Run on the branch head, from the worktree root:

    git apply design/FIX_sec_secret_drift_gate_fail_closed.red.patch
    PATH=$HOME/.cargo/bin:$PATH env -u MNEMONIC_BIN cargo nextest run --locked --no-fail-fast --test schema_mirror_secret_drift

Expected on 922a0b38 (measured 2026-11-23, local `mnemonic` 0.107.1 on PATH), exit 100:

    Summary [...] 9 tests run: 2 passed, 7 failed, 0 skipped

Every failing row's assertion message ends `child success=true, output:` (the child gate passed). Paths below are the
measured temp dirs.

| row (test name, `gate_contract::`) | 922a0b38 | after the fix | case |
|---|---|---|---|
| `drift_gate_fails_when_bin_set_and_absent` | FAIL, ``gate must FAIL printing "schema_mirror_secret_drift: cannot run `/tmp/.tmpD3y469/absent-mnemonic gui-schema`: "; child success=true`` | PASS | F1, set |
| `drift_gate_fails_when_bin_exits_nonzero` (script `exit 3`) | FAIL, ``gate must FAIL printing "schema_mirror_secret_drift: `/tmp/.tmpbjlOYy/mnemonic gui-schema` failed: exit status: 3"; child success=true``; child printed `` `...` exited ExitStatus(unix_wait_status(768)); skipping`` | PASS | F2 |
| `drift_gate_fails_when_stdout_not_utf8` (script `printf '\377'`) | FAIL, ``gate must FAIL printing "schema_mirror_secret_drift: `/tmp/.tmpDtvqyo/mnemonic gui-schema` stdout is not UTF-8"; child success=true`` | PASS | F3 |
| `drift_gate_fails_when_stdout_not_json` (script `echo not-json`) | FAIL, ``gate must FAIL printing "schema_mirror_secret_drift: `/tmp/.tmp25gBbt/mnemonic gui-schema` stdout is not a gui-schema JSON document: "; child success=true`` | PASS | F4 |
| `drift_gate_fails_when_schema_version_below_5` (script prints `{"version":4,...}`) | FAIL, ``gate must FAIL printing "schema_mirror_secret_drift: `/tmp/.tmpLy3Jso/mnemonic gui-schema` reports schema version 4; this gate needs version 5 or later (per-flag `secret` field)"; child success=true``; child printed `gui-schema version 4 (< 5); skipping drift gate` | PASS | F5, set |
| `drift_gate_fails_when_bin_unset_and_path_mnemonic_is_pre_v5` (`MNEMONIC_BIN` removed, `PATH` = dir holding the v4 script) | FAIL, ``gate must FAIL printing "schema_mirror_secret_drift: `mnemonic gui-schema` reports schema version 4; ..."; child success=true`` | PASS | F5, unset (Rule B fallback) |
| `drift_gate_skips_loudly_when_bin_unset_and_not_on_path` (`MNEMONIC_BIN` removed, `PATH` = empty dir) | FAIL, ``gate must PASS printing "schema_mirror_secret_drift: SKIP: MNEMONIC_BIN is unset and `mnemonic` is not on PATH; set MNEMONIC_BIN to the pinned toolkit binary to run this gate"; child success=true`` (the child passed but printed nothing) | PASS | Rule S |
| `drift_gate_passes_on_matching_v5_document` (script `cat`s a v5 doc built from `schema::mnemonic::SCHEMA`) | PASS | PASS | guard: a good document still passes |

The ninth test is the existing gate (`secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode`), PASS on both
(here via the local `mnemonic` on PATH).

The build applies the patch as its RED commit, sees the summary above, then makes the section-3 change and sees
`9 tests run: 9 passed, 0 skipped`. With `MNEMONIC_BIN=mnemonic` pointing at a non-pinned local toolkit the ninth
test may fail on a real secret-set difference; that is the gate working, not a regression (it passed at 0.107.1).

## 5. Tests that change

- Added (the patch): the eight `gate_contract::` tests above.
- Edited: `secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode` and `fetch_v5_schema` (section 3); the
  assertion texts at `:92` and `:117-123` stay byte-identical.
- Existing tests that must stay green unedited: every other test in the suite (no shared code changes).

## 6. Gate (full CI of mnemonic-gui; the builder runs all of it on the final head)

    PATH=$HOME/.cargo/bin:$PATH sh -c 'cargo clippy --locked --all-targets -- -D warnings && cargo clippy --locked --no-default-features -- -D warnings && cargo build --locked -p mnemonic-gui --no-default-features && cargo +1.88.0 check --locked && cargo nextest run --locked --workspace --no-fail-fast && cargo test --locked --workspace --doc && MNEMONIC_BIN=mnemonic python3 scripts/secret-channels-mutations.py' > /scratch/cache/sec-secret-drift-gate-fail-closed.gate.log 2>&1

Not covered locally: the `snapshots`, `tutorial-snapshots` and `schema-mirror` real-binary jobs need the pinned
binaries; take them as green CI on the PR head. In the `schema-mirror` job's `cargo-test-full-suite` step the gate
now fails if the pinned `mnemonic` misbehaves; that step being green on the PR head is the evidence the pinned
binary still yields v5. Real-binary tests that fail locally on 922a0b38 because the local CLIs are not the pinned
ones are compared against base, not treated as regressions.

## 7. FOLLOWUPS

The repo line `FOLLOWUPS.md:81` (`secret-drift-gate-version-skip-silent`) is closed by a new appended line after
merge (FOLLOWUPS is append-only; no PR edits FOLLOWUPS). The closing line and any other proposed lines are in the
author report.

## 8. Out of scope (residual, reported, not closed here)

- `tests/schema_mirror.rs:418-436` (the `gui-schema` probe for the four CLIs) and `src/schema_check.rs:351-398`
  (`json_flag_names`, `run_gui_schema_json`) skip or fall back on spawn failure and non-zero exit in the same way.
  They gate flag names and defaults, not the secret projection; the author report proposes a line.
- `schema-mirror.yml:173` smoke asserts `version>=1`; with Rule F the full-suite step now fails on a pre-v5 pinned
  binary, so the smoke step is left as is (no `.github/` change).
