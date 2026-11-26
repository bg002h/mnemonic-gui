# Review: mnemonic-gui exit-code implementation (2026-11-26)

Independent adversarial review. Diff `ae3e4f8..HEAD` (691f9ce, 2545d0c) on
`fx/mnemonic-gui`, plus `a5fa42d` (the badge fix it replaces). Normative:
`SPEC_constellation_exit_codes.md` rev 5, §2-§5, R12, §7j, §9. I did not
write this code and did not edit source.

**What I ran:** `--lib` (exit_code, cli_version, runner signal), `tests/exit_badge.rs`
(10/10), `tests/exit_code_floor.rs` (6/6), `tests/g1_cli_version_gate.rs` (7/7),
`tests/gui_own_exit_codes.rs` (2/2), all passing. I also ran the built binary:
with no display it prints `mnemonic-gui: cannot start: winit EventLoopError ...`
and exits **32**; `mnemonic-gui --debug extra` exits **64**. Everything else
below comes from reading the code and checking the sibling repos' versions
(`git show origin/main:...`).

**Verdict: 1 Critical, 2 Important, 5 Minor, 3 Nit.** The decoding itself is
correct. The version floor that stops an old binary from being decoded is not.

---

## Critical

### C1. A pre-release is read as its release, and an old-code `mk` that exists today is decoded bit by bit

- **Where:** `src/cli_version.rs` `parse_version` (`core = t.split(['-','+'])`),
  used by `exit_scheme` (`if got < want`). The floor is `exit_bits_floor("mk") = "mk 0.18.0"`.
- **What I verified:** `mnemonic-key` `origin/main`, which does **not** have
  the renumber, has `crates/mk-cli/Cargo.toml` `version = "0.18.0-dev"`. clap
  prints `mk 0.18.0-dev`, which parses to `(0,18,0)`, which is `>=` the floor,
  so the scheme is `CodeScheme::Bits`. The renumbered worktree is also
  `0.18.0-dev`, so the two builds cannot be told apart by version.
- **Failure scenario:** a user installs `mk` from main today (pre-renumber)
  and runs it from this GUI.
  - Old `mk verify` mismatch exits 4 and shows amber "verify the result before
    trusting it". `a5fa42d` showed it red "MISMATCH". This commit regresses
    that fix.
  - Old `mk repair` exits 5 and shows red "5 = 1 + 4 — a check said no; verify ...".
  - Old invalid-card exit 1 shows "a check said no".
  - Old refusal or input-file exit 64 shows "bad command line, nothing ran".
  - §9 says "no stage may let an old binary and a new GUI meet without a
    version check". The check exists, but it passes the wrong binary.
- **The implementer saw this:** spec question 1 and the `exit_bits_floor`
  doc comment both note it. It is still shipped.
- **Fix, either or both:**
  - (a) Compare with semver precedence for the floor: `X.Y.Z-pre < X.Y.Z`.
    Then any `-dev` build at the floor's triple is undecoded. A post-renumber
    dev build is also undecoded until it is released, which is the safe
    direction.
  - (b) Raise mk's floor to a version mk-cli has never carried before the
    renumber (0.19.0), and get mk to commit to it.
- **Test to add:** `exit_scheme("mk", Ok("mk 0.18.0-dev"))` and
  `("md", "md 0.22.0-dev")` are `Undecoded`.

## Important

### I1. The floors are guesses that nothing ties to the CLIs' cutover releases (spec question 1: not accepted as is)

- **Where:** `src/cli_version.rs` `exit_bits_floor`. Floors are `mnemonic 0.108.0`,
  `md 0.22.0`, `ms 0.22.0`, `mk 0.18.0`. The renumbered sibling worktrees still
  carry `md-cli 0.21.1`, `ms-cli 0.21.0`, `mnemonic 0.107.1` and
  `mk-cli 0.18.0-dev`. No CLI has agreed to these numbers.
- **Failure scenario, unsafe direction:** md (or ms, or the toolkit) cuts
  0.22.0 (0.108.0) for an unrelated fix while the coordinated renumber is
  still waiting on `mr`, the long pole (decision 10). The GUI then decodes
  old codes from it. For example, old `md descriptor --verify-against` 5
  becomes "a check said no; verify the result before trusting it", and old
  `mnemonic` 4 (its whole VERIFY-ME class, mismatches included) becomes amber
  where it may be a "no".
- **Failure scenario, other direction:** the cutover ships as a patch
  (`md 0.21.2`), and the GUI never decodes md.
- **Fix:**
  - Bind each floor to the real cutover release: one `owed-to-build`
    FOLLOWUP line in each CLI repo, cross-cited.
  - Set the floors in the pin-bump PR, not before. Add a pin-coherence test
    that `exit_bits_floor(cli) <= pinned_version_for(cli)` once the pins move.
  - Stronger option: detect the scheme from what R11 makes normative. Every
    renumbered binary's root `--help` ends with the exact legend (mk already
    has `after_help = exit::HELP_LEGEND`). Decode only when
    `<bin> --help` contains `exit_code::HELP_LEGEND`, or when both the
    version floor and the legend agree. That ties detection to the behaviour,
    not to a number someone picked. `HELP_LEGEND` is already in
    `src/exit_code.rs` and is unused.

### I2. Until the pin bump, every CLI the GUI is pinned to is undecoded, and the warnings `a5fa42d` gave are lost

- **Where:**
  - `src/exit_badge.rs` `classify`, `CodeScheme::Undecoded` branch (always
    `BadgeKind::Plain`).
  - `pinned-upstream.toml` and `Schema::pinned_version` are unchanged
    (`md 0.21.0`, `ms 0.21.0`, `mk 0.17.0`, `mnemonic 0.107.1`), all below
    the floors.
- **Failure scenario:** if this branch merges (or the GUI releases) before
  the CLIs cut over, then for every user on the pinned CLIs:
  - old `ms repair` / `mnemonic repair --max-subst` exit 4 (a correction with
    no oracle) loses its amber VERIFY-ME;
  - `md descriptor --verify-against` 5 and `mk`/`ms verify` 4 lose the red
    `a5fa42d` gave them;
  - all of these show plain `exit: N (not decoded: ...)`.
  - The new tooltips (`md --verify-against` "1 = NOT spend-equal",
    `restore --cosigner` 1, `--bsms-token` 1, `--bsms-verify-strict` 1,
    `ms --out` 8) describe codes the pinned binaries do not emit.
  - `tests/exit_badge.rs::a_binary_below_the_floor_is_not_decoded` locks this
    behaviour in.
- **Fix, one of:**
  - (a) Make "land only in the same PR as the pin bump past every floor"
    (§9 step 4) an explicit merge condition. State it in the CHANGELOG entry
    and the impl report, and hold the tooltip changes for that PR.
  - (b) Keep `a5fa42d`'s per-command legacy classifier for a binary whose
    version is known to be old-scheme (`>= pinned` and `< floor`). It was
    correct for those versions. Use plain undecoded only for an unknown
    version.

## Minor

### M1. No test pins that the decision reaches the badge

- **Where:** `src/app_window.rs` `spawn_and_capture`, `Ok(mut result) => { result.exit_scheme = exit_scheme; ...`.
- **Gap:** `g1_spawn_and_capture_calls_the_gate_before_spawning` checks only
  that `gate_run(` comes before `run_plan(`. If the assignment is dropped,
  every badge silently shows "the binary's version was not checked". That is
  safe, but it disables decoding.
- **Fix:** extend that source-order test to require the assignment, or factor
  "gate, run, stamp" into an egui-free function and test it.

### M2. `--version` is probed on every run, synchronously on the UI thread, with no timeout

- **Where:** `src/cli_version.rs` `probe` calls `runner::run([bin, "--version"])`,
  with no timeout.
- **Failure scenario:** before this change only secret runs paid for the
  probe. Now every Run spawns twice, both blocking in the click frame. A
  `--version` that hangs (a wrapper script on PATH, a stalled network mount,
  antivirus scanning on Windows) freezes the window, and the 3 s signal
  fallback ends it with 130.
- **Fix:** put a short timeout on the probe (on timeout the scheme is
  undecoded, and a secret run is refused). Optionally cache per (resolved
  path, mtime) for non-secret runs only.

### M3. Exit 16 alone is green (spec question 5)

- **Where:** `src/exit_badge.rs` `bits_badge` (`Repaired`).
- **The spec's action for 16** is "the plate is damaged; re-engrave it".
  Green plus "a correction was applied" reads as plain success.
- **Fix:** keep green if you like, since the value is verified. But append
  the §3 action to the text (`— a correction was applied; the plate is
  damaged, re-engrave it`), or use a fourth, neutral-warning colour.
- Every other combination is correct: I checked by hand that green happens
  only at exactly 16, and that 17, 18, 24 and 48 are red and 20 is amber.

### M4. The `secret_channels_t2` baseline check is weaker

- **Where:** `tests/secret_channels_t2.rs`, baseline check (`base.stdout.trim().is_empty()`).
- **Before:** the baseline had to succeed (0 or 4).
- **Now:** a baseline that fails but prints something to stdout passes.
  Under R5 that includes `inspect` printing its whole report and setting 2.
- **Why only Minor:** cell fidelity still compares `(code, output)`, so a
  channel that misreads is still caught. But "measurement valid" can now
  cover a failing baseline.
- R12's rule is about how the GUI decides whether output exists. A test
  asking "did the baseline do its job" may still use the code.
- **Fix:** require non-empty stdout **and** no failure bit (old scheme: 0 or
  4; new: `code & (1|2|8|32) == 0`, `code < 64`). The same applies to the
  `valid` predicate in `table.py` and `table_build.py`.

### M5. The GUI's exit 32 is not tested

- **Where:** `tests/gui_own_exit_codes.rs`. It tests 64, `--help` and
  `--version`, not 32.
- I verified 32 by hand: no display gives `rc=32` and a "cannot start" line.
- **Fix:** add a test that runs the binary with `DISPLAY`, `WAYLAND_DISPLAY`
  and `WAYLAND_SOCKET` removed and HOME/XDG in a tempdir, gated on Linux, and
  asserts 32.

## Nit

- **N1.** `exit: 130 — stopped by the operator` is `Plain`, the same colour as
  success. Amber would stop it reading as "done".
- **N2.** 65..=125 (other than 101) show "ended abnormally". §3 calls any of
  them "a bug", so "ended abnormally (unexpected code; a bug)" would match the
  spec's wording.
- **N3.** `run_bytes.py` `classify` now counts a failing run with stdout as
  "ok". So a re-measurement may report `BOTH-OK-DIFFERENT` where it used to
  report fail-closed. That direction is safe; just note it when reading the
  re-measured table at the pin bump.

---

## Checked and found correct

- **Decoding:** `decode` and `classify` are correct for every code.
  - 0, 64, 101 and 130 are the only whole numbers.
  - 1..=63 is named bit by bit, with an exact name set for each code (the
    test checks all 63 codes against all 6 names; no false PASS: every name
    is unique).
  - 65..=125 other than 101, 126..=129, 131..=255, negative values and
    NTSTATUS values are all "ended abnormally".
  - A signal is taken from `ExitStatusExt::signal` before the scheme is
    considered.
  - `None`/`None` gives "ended abnormally (no exit status)".
- **Colours:** red for any of 1/2/8/32, 64, 101, abnormal ends and signals;
  amber for 4 or 20 without a failure bit; green only at exactly 16.
- **No decision on output from a code** remains in `src/`. `tree_form` is
  parse-keyed and `schema_check` uses `success()` on `gui-schema`. The badge
  never mentions stdout.
- **The secret refusal is not weakened.** `gate_run` passes the same single
  probe answer to `gate_secret_run`. An unknown binary, an older version,
  another program's name or a failed probe still refuse a secret run, and a
  run that is refused spawns nothing. `gate_secret_run` does not apply
  `cli_name` normalisation, so a secret run of a path `argv[0]` is refused as
  an unknown binary. That predates this change and fails safe.
- **The GUI's own codes:** `try_parse` gives 64 for errors and 0 for `--help`
  and `--version`. `run_native` failing gives 32 with a message. A panic is
  101: no `panic = "abort"` in `Cargo.toml`. The deliberate 130 is kept. A
  clean shutdown after a signal still exits 0; the spec allows that, and it
  was not changed.

## Spec questions

1. **Floors set to the next minor.** Not acceptable as they stand. See C1
   (pre-release handling) and I1 (bind each floor to the real cutover, or
   detect the R11 legend).
2. **Check versus refuse.** I agree with *check*. §7j says the check must
   "cover every run", and §9 says no stage may let old and new "meet without
   a version check". Neither asks for a refusal. Refusing every non-secret
   run on an old CLI would make the GUI unusable during the cutover and buys
   nothing once the status is shown undecoded. G1's secret refusal is
   correctly kept. The caveat is I2: "undecoded" should not throw away what
   the GUI correctly knew about the old scheme.
3. **R11 legend for the GUI.** Agree. R12 gives the GUIs their own code list,
   and R11 is for the eight binaries.
4. **N10.** Agree that it is deferred. Nothing prints N10 yet, and bit 8's
   text already names the network mismatch.
5. **16 green.** See M3. Acceptable if the action goes into the text.
6. **The fixpr merge.** `cli_version.rs` depends on it. This branch must land
   with or after fixpr; that is already stated in the report.
