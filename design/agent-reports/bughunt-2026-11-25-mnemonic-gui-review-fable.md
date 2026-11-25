# mnemonic-gui bug-hunt fix branch: final pre-PR review (Fable)

**Scope:** `git diff origin/master...claude/project-thread-utm1aa` in /home/user/mnemonic-gui (8 commits, c3d91f0..b8135e1; G1-G5, CHANGELOG, two review reports, FOLLOWUPS). Nothing tracked was edited, committed or pushed.

**Verdict: 0 Critical, 0 Important, 4 Minor, 5 Nit.** The branch can go to PR. Three of the Minor items are cheap enough that I would fold them into this PR rather than log them (see "What I would fold before the PR").

## What I ran and checked

- **Targeted tests, all green** (`CARGO_INCREMENTAL=0 cargo test --locked -j 2 --test ...`): copy_command 20, g1_cli_version_gate 5, persistence 17, pin_coherence 3, secrets 24. Also `--lib cli_version`: 6 passed (the module's own unit tests, which the earlier reviews' `--test` runs did not execute).
- **Clippy, CI's flags, clean** on the changed surface: `cargo clippy --locked --lib --bins --test <the five> -- -D warnings`, and `cargo clippy --locked --no-default-features -- -D warnings` (`cli_version` is not behind the `gui` feature, so that build matters). The full `--all-targets` clippy and the full suite were not run (disk).
- **Not run:** anything needing the pinned CLIs (tutorial snapshots, secret_channels T2/T7, schema_mirror); no Windows or macOS host. `cargo fmt --check` is not a gate in this repo (master already differs in about 40 files).
- **Threshold provenance, confirmed sound.** The gate's threshold is hand-kept `Schema::pinned_version`, but the chain to the measurement is closed by tests: `measured_with.json.pinned_tag == pinned-upstream tag` (tests/secret_channels_t1.rs:716-724) and `tag version == pinned_version` (the new G5 test). So "the version the channel table was measured against" is what is enforced.
- **`--version` shape at the pinned tags.** `descriptor-mnemonic-md-cli-v0.21.0` has `#[command(name = "md", version, ...)]` and `mnemonic-toolkit-v0.107.1` has `name = "mnemonic"`, both read from the tags. The `ms`/`mk` tags are not in the local clones; their HEAD sources have `name = "ms"` / `name = "mk"`.
- **No secret source escapes the gate.** `has_secrets()` is "bindings non-empty", and a secret source with no channel-table entry is refused (`no-table-entry`, src/form/channels/mod.rs:454), so it cannot become a binding-less plan. The two direct spawns in tree_form.rs (236, 509) carry no secret source; `build-descriptor` has no entry in the channel table.
- **G2 continuation lines.** cmd.exe escapes the first character of the line after a trailing `^`. The separator is `" ^\r\n  "`, so that character is a space, not the token's opening quote. The admitted set (`& | < > ( ) ^` inside quotes) really is inert.
- **G4 private-path text is true where it is shown.** The paste warning is raised only by secret widgets and secret-node values (src/form/widget.rs:659, secret_widget.rs:223), all of which get a private channel or a refusal on Linux. The md "private key in a public field" case, which does ride argv, does not raise this modal.
- **G3.** Remove-stale, then `create_new` with mode 0600, then rename is correct; error paths remove the temp; compiles on non-Unix.

## Is `src/cli_version.rs` the right shape, and is the gate in the right place?

Yes on both, with reservations that are Minor.

- **Shape.** Pure parsing and judging (`parse_version`, `parse_version_line`, `check_reported`), one impure `probe`, and a gate with the probe injected. That is the right split and makes the module testable without binaries.
- **Place.** `spawn_and_capture` is the single route for a secret-bearing Run (plain and confirmed). Probing at spawn time, every time, with no cache, is the safe choice: nothing can go stale between probe and run.
- **Alternative considered.** Putting the gate in `runner::run_plan` would make it impossible to bypass, but it would turn the runner into a policy holder and break the real-binary tests that call it directly. I would keep it where it is.
- **Reservations.** M2 (where the refusal surfaces), M3 (nothing in CI runs the gate against a real pinned binary) and N2 (name-to-schema lookup duplicated).

## Findings

### Minor

**M1. The Copy button still hands out the exact invocation the gate just refused, and nothing says so.** Challenge to r2-m1, which I think is under-weighted. `src/cli_version.rs:69-76` (refusal text), `src/form/channels/copy.rs:195-215` (Copy emits `@env:` references).
- **Scenario:**
  1. Linux, `ms 0.16.x` first on PATH. The user fills `ms derive` with a passphrase and clicks Run.
  2. The GUI answers "Run refused ... older than the pinned ms 0.21.0".
  3. Copy command (POSIX) sits next to Run and is still enabled. It yields `ms derive ... --passphrase @env:MNEMONIC_GUI_S0` with a `read -rs` comment.
  4. Pasted into the user's terminal (same PATH, same old `ms`), it derives the wallet for the literal passphrase `@env:MNEMONIC_GUI_S0` at exit 0. That is G1's Medium funds outcome, one click away from the refusal.
- **Why it is worse than r2 graded:**
  - The Copy plan is private by construction on every OS (`Policy::for_copy`), so this also holds on macOS and Windows, where Run itself is not exposed.
  - r2's own analysis of m1 lists "the GUI's Copy button still emits the command" as a way around the refusal. That advice is unsafe.
- **Grade:** still Minor, because it predates the branch and the user has just read a warning. But it means G1 is closed for Run only.
- **Ask (two lines):** add a sentence to the refusal ("the copied command is not safe with this binary either"), and add the comment line r2 proposed to both Copy flavours.

**M2. The refusal appears after the user has confirmed, labelled as a subprocess error.** `src/app_window.rs:1395-1401`, displayed at `src/app_window.rs:444-446`.
- **What the user sees:** every secret-bearing Run opens the confirm modal first (argv, "Secrets:" bindings, Run/Cancel). Only after the user approves does the gate run. The result is rendered in the output pane as `subprocess error: Run refused — the \`ms\` on $PATH is version 0.16.0, ...`: one red label of about 450 characters, prefixed "subprocess error" although no subprocess of the user's ran.
- **Inconsistency:** every other refusal (DESIGN §A8) disables Run and shows a red "Run refused — ..." beside the button before any modal (`src/app_window.rs:1111-1113`). The new one borrows the phrase and behaves differently.
- **Ask:**
  - Drop the "subprocess error:" prefix for this case.
  - Longer term, probe once at launch next to `detect_all`, show the found version beside "Pinned:", and pre-disable Run with the reason. Keep the spawn-time probe as the authority.

**M3. Nothing in CI runs the gate against a real pinned binary, so "a correct install passes" is an unchecked fact for md, ms and mk.** `tests/g1_cli_version_gate.rs` (every test injects the probe), `src/cli_version.rs:182-189` (the only real spawn is `true`).
- **Current coverage:** the tutorial job compares real `mnemonic --version` to `pinned_version` exactly (`tests/gui_tutorial_snapshots.rs:1612`, `:1677-1679`), which covers `mnemonic` only. The Linux `schema-mirror.yml` job installs all four pinned binaries, but its real-binary tests call `runner::run_plan` directly and never touch the gate.
- **Scenario:** a future CLI release changes its `--version` first line (a banner line, a crate-style name, a version that is not `X.Y.Z`). The pin bump passes pin_coherence and every CI job. The shipped GUI then refuses every secret-bearing Run of that CLI for users with the correct install.
- **Grade:** fails closed, hence Minor. This is the kind of fact that should have a test of its own, and r1 established it for today's pins only by reading sibling sources at HEAD.
- **Ask:** in the job that already has the binaries, add one test: for each of the four CLIs, `check_reported(pinned_version_for(cli), &probe(cli)?)` is `Ok`.

**M4. The cross-repo and documentation obligations of the new behaviour are recorded nowhere durable.**
- **GUI manual.** r1 n7 says "the PR body already lists the manual-gui follow-ups", but there is no PR yet. Neither this repo's FOLLOWUPS.md nor the toolkit's `design/FOLLOWUPS.md` has an entry. This repo's CLAUDE.md asks for mirrored entries in both repos. What is owed:
  - a troubleshooting row for "Run refused ... older than the pinned";
  - the Windows Copy that is now disabled for some values;
  - the Linux paste-warning text;
  - the `state.json` mode.
- **A false remedy in the manual.** Toolkit `docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md:13` tells users to set `$MNEMONIC_BIN` when the binary is outside `$PATH`. The app never reads it: `CliTab::bin_env_var` (`src/app.rs:51`) has no caller, and only the tests' `schema_check.rs` honours `*_BIN`. This predates the branch, but it is now the first thing a refused user will try, and it will do nothing.
- **README and CHANGELOG.** README "Install" (lines 91-105) documents a GUI-only install and does not say that secret-bearing Runs now need the pinned CLI versions or newer. The CHANGELOG G1 entry does not say it either. r2 logged this as a Nit (r2-n2). I grade it Minor: it is the only user-facing statement of a behaviour that will meet every GUI-only upgrader, and it costs one sentence.
- **DESIGN §A8.** The refusal table (`design/DESIGN_secret_channels_and_new_forms.md:668`) is the record of every refusal. The version gate is a new one, applies on every path, and is absent from it.

### Nit

- **N1. Stale comment contradicts the new G5 test.** `src/schema/mod.rs:20-23` still gives the example "mnemonic-toolkit-v0.8.1: `mnemonic 0.8.0` (the v0.8.1 git tag did NOT bump the crate package version)". The G5 test (`tests/pin_coherence.rs:46-61`) now requires tag version to equal `pinned_version`, and the gate requires the binary to report at least that. A tag of that kind can no longer be pinned without refusing every secret run. Delete the example or say so.
- **N2. A fifth hand-written name-to-schema table.** `pinned_version_for` (`src/cli_version.rs:51-61`) repeats the match already at `src/app_window.rs:309`, `src/form/render_emit.rs:47`, `src/persistence.rs:198` and `CliTab::from_bin_name` (`src/app.rs:39`). The unit test hardcodes the four names again. Adding a CLI fails closed ("unknown binary") rather than loudly at compile time. A `schema::by_cli_name`, or passing `&Schema` to the gate, would remove the copy.
- **N3. The tree-mode Windows Copy gate has no test, and the contract is a comment.** `src/app_window.rs:1008-1016` is inline UI code. `render_copy_command(.., WindowsCmd)` stays public and unguarded, with "every Copy button checks that first" as its only protection (`src/form/invocation.rs:702-704`). Returning `Result` or `Option` from the Windows rendering would make the check impossible to skip.
- **N4. G4 text and test residue.**
  - On Linux the modal now reads "... so it is not shown in process tables. macOS and Windows now suppress OS screenshot APIs ... Linux remains unmitigated". The last sentence is about screenshots, but directly after a Linux process-table claim it reads as a contradiction. A paragraph break would fix it.
  - `tests/paste_warn_wiring_v0_40_0.rs:84-87` still says "The modal renders the documented constant" and checks only `PASTE_WARN_MODAL_TEXT`, which Linux no longer shows.
  - `src/secrets.rs:265-266` has a stray double blank line.
- **N5. Refusal wording.** `src/cli_version.rs:74`: "Install ms 0.21.0 or newer first on $PATH" names neither the package (`ms-cli`) nor the tag, and says `$PATH` on Windows. The README's install lines are the remedy; pointing at them would help.

## Logged items: do I agree with the grades?

- **m1 (threshold is the pin): agree Minor**, with one caveat. On the interim path (macOS, Windows) the plan carries no channel reference, and an old CLI without `--allow-argv-secret` already fails in clap. There the gate is pure cost and its message is false (m5). Decide deliberately between skipping the probe when `plan.interim` and keeping it uniform with corrected text. The current state is the worst of both.
- **m5 (refusal text wrong off Linux): Minor is right, but it should not ship.** It is a new instance of the defect class G4 fixes in this same branch: text that tells the user something untrue about how the secret travels. The fix is to select the wording by `plan.interim`.
- **r2-m1 (Copy ungated):** see M1. Minor, but it is the open half of G1.
- **r2-n2 (CHANGELOG):** raised to Minor inside M4.
- **m2, m3, m4 and the nits:** agree as graded. On m2, PSReadLine parsing the whole paste before running anything is what keeps it Minor.

## What I would fold before the PR

All text-only or a few lines each; none changes a plan or a spawn.

1. **Refusal text** (`src/cli_version.rs:69-76`): pick the wording by `plan.interim` (m5), and add the sentence that the copied command is not safe with this binary either (M1).
2. **One sentence in CHANGELOG G1 and README "Install":** secret-bearing Runs need the pinned CLI versions or newer (M4, m1).
3. **FOLLOWUPS entries** (here, plus the toolkit companion) for the manual-gui items and the false `$MNEMONIC_BIN` advice (M4), so they do not depend on a PR body.

M2, M3 and the nits can be logged.

## Bottom line

The five fixes are correct. The gate is fail-closed and placed at the one choke point. The Windows Copy block is sound for cmd.exe, including the caret-continuation case. The `state.json` write is right, and the paste text is accurate where it appears. I found nothing Critical or Important. The remaining weak point is M1: the refused invocation is still available through Copy.
