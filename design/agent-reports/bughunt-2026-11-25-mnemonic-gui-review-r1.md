# mnemonic-gui fix branch: adversarial whole-diff review, round 1

**Scope:** `git diff origin/master...claude/project-thread-utm1aa` in /home/user/mnemonic-gui. Six commits, c3d91f0..efc6263 (G1–G5 plus the CHANGELOG). Nothing tracked was edited. One scratch test file was created, run, and then deleted along with its binary.

**Verdict: 0 Critical, 0 Important, 5 Minor, 7 Nit.** The fixes do what they claim. No regression was found that blocks a merge. The Minor findings are residual gaps, and some of them date from before this branch.

## Evidence gathered

- **Targeted tests, all green.** Command: `CARGO_INCREMENTAL=0 cargo test --locked -j 2 --test g1_cli_version_gate --test copy_command --test persistence --test pin_coherence --test secrets`.
  - copy_command: 20 passed
  - g1_cli_version_gate: 5 passed
  - persistence: 17 passed
  - pin_coherence: 3 passed
  - secrets: 24 passed
- **End-to-end G1 check (scratch test, now deleted).** A fake `mnemonic` shell script was put first on PATH, and the real planner built a Linux `restore --from ms1=@env:SEED` plan. That plan was run through `gate_secret_run(&p, probe)`:
  - `mnemonic 0.107.1` → Ok
  - `mnemonic 0.107.1-dev (abc)` → Ok
  - `mnemonic 0.200.0` after a blank line → Ok
  - `mnemonic 0.106.9` → refused
  - version printed on stderr only → refused (stdout empty)
  - exit 2 → refused
  - `mnemonic-toolkit 0.107.1` → refused (name mismatch)
  - `parse_version_line("mr 0.1.0-dev (ee692025)")` returns `("mr",(0,1,0))`.
  - A macOS plan is `interim=true`, `has_secrets=true` and `argv0="mnemonic"`, so the gate also fires on the argv path.
- **Clap program names checked against the sibling sources.** Each CLI sets an explicit clap `name`:
  - md-cli: `name = "md"`, main.rs:100
  - ms-cli: `name = "ms"`, main.rs:65
  - mk-cli: `name = "mk"`, main.rs:29
  - toolkit: `name = "mnemonic"`, cli.rs:40

  So `--version` prints `<bin> X.Y.Z` on stdout on every OS, Windows `.exe` included, and the crate names (`md-cli`, `ms-cli`, `mk-cli`, `mnemonic-toolkit`) never appear. A correct install therefore passes the name check. The tutorial harness already requires `--version` to equal the pin exactly (tests/tutorial/mod.rs:488), so it is unaffected.
- **Bypass search.** Every spawn in `src/` was checked: `runner::run*`, `Command::new`.
  - The secret-bearing Run has one route. The plain Run (app_window.rs:1146) and the confirm-modal Run (app_window.rs:1223) both go through `spawn_and_capture`, and the gate there (app_window.rs:1395) sits after `detect` and before `runner::run_plan`.
  - The other spawns do not use `@env:`/`-` channels:
    - tree_form.rs:236 (Edit as tree) and :509 (Validate pipe spec JSON) straight to `mnemonic`.
    - schema_check.rs is test-only.
    - The probe itself.
  - The plan with a masked token but no bindings (Review M2: a private key pasted into a public md field) is not probed. It carries the literal value on argv, so an old CLI cannot misread a channel reference there. That is consistent with G1's threat.
- **Plan unchanged on a passing run.** `gate_secret_run` takes `&RunPlan` and `spawn_and_capture` passes the same `plan` to `run_plan`. The probe runs through `runner::run` with Stdio::null, so none of the plan's env, stdin or fds reach it.

## Findings

### Minor

**m1 — G1: the threshold is the pinned version, not the first version that is safe.** `src/cli_version.rs:114-126`; the documentation note is at `src/schema/mnemonic.rs:4841-4846`.
- **What happens:** any secret-bearing Run is refused unless the CLI is at or above `pinned_version`. Each GUI pin bump therefore blocks every secret-bearing Run, on Linux and on the interim argv path, for users whose CLIs are older but would still work. Examples are `mnemonic` 0.106.x or 0.105.x and `ms` 0.20.x, all of which have handled `@env:` correctly since toolkit 0.26.0 and ms 0.20.0.
- **Scenario:** a user upgrades the GUI from the release page (the README documents GUI-only installs) and keeps `mnemonic 0.105.1`. Every passphrase-bearing `convert` or `restore` now says "Run refused". The run before the upgrade was correct.
- **Why only Minor:** the gate fails closed, the message names the remedy, and pin_coherence keeps the pin and the tag in lockstep.
- **Ask:** state in the CHANGELOG or README that a GUI upgrade now requires CLI upgrades for secret runs. Optionally, gate on a per-CLI minimum channel-safe version instead.

**m2 — G2: the Windows Copy is cmd syntax, but PowerShell's metacharacters are not considered.** `src/form/invocation.rs:780`.
- **The gap:** the button is labelled "Copy command (Windows)" (app_window.rs:1063), and Windows 11's default terminal profile is PowerShell. Inside `"…"`, PowerShell expands `$var` and `$(…)`, and a backtick is its escape character. None of these are in `CMD_UNSAFE_CHARS`.
- **Scenario:**
  1. The last argv token holds `x$(Start-Process calc)`, for example a label or output path pasted from a message.
  2. The user pastes the copied text into PowerShell.
  3. If the console feeds the text line by line, each `… ^` line fails to parse.
  4. The last line, a bare `"x$(Start-Process calc)"`, has no `^` and evaluates, so calc runs.
- **Not executed:** there is no Windows host here.
- **Scope:** this predates the branch and is outside G2's stated scope (cmd.exe).
- **Ask:** at least say "cmd.exe" in the button label or tooltip. Better, also block `$` and the backtick.

**m3 — G2: the `<FILE>` placeholders are emitted bare, which is cmd.exe redirection syntax. The new doc claim overstates the guarantee.** `src/form/channels/copy.rs:295`; the doc claim is at `src/form/invocation.rs:789-791`.
- **The overstated claim:** the comment says "`None` means every token survives `cmd.exe` verbatim". However, `render` deliberately passes only the *quoted* tokens to the blocker, and the pipe-fd placeholder `<FILE>`/`<FILEn>` reaches the Windows Copy unquoted. FileFlag/InFile bindings are not disabled on Windows.
- **Scenario:**
  1. The copy says `REM FILE: a file holding the ms1 card`.
  2. The user replaces only the word inside the brackets: `<C:\card.txt> ^` followed by `"--next-token"`.
  3. cmd reads `<C:\card.txt>` as an input redirect from the card file plus an output redirect to the next token.
  4. A file named after the next token is created or truncated, and the CLI's stdout goes there.
  5. If the next token is a user path, that file is clobbered.
- **Pasted unchanged:** with no file named `FILE` present, the command only errors.
- **Scope:** this predates the branch.
- **Ask:** quote the placeholder on Windows, or include placeholder tokens in the blocker and the documentation.

**m4 — G3: existing world-readable copies are not tightened.**
- **`.bak` files:** `load()` renames a mismatched or malformed `state.json` to `state.json.bak` (`src/persistence.rs:364`, `:370`). The rename keeps the old mode, so a 0644 file from before the fix stays 0644 as `.bak` indefinitely.
- **Stranded temp files:** `*.tmp` files left by earlier crashed PIDs, written 0644 before the fix, are removed only when the PID matches (`:305`).
- **Scenario:** a user's first launch of the fixed GUI after a schema bump leaves `state.json.bak` at 0644, holding the descriptors and xpubs G3 meant to protect.
- **Ask:** `set_permissions(0o600)` on the `.bak` file after the rename (Unix).

**m5 — G1: the refusal text is wrong on the interim path.** `src/cli_version.rs:69-77`.
- **The problem:** the message always says secrets go "over private channels (@env:, stdin, pipes)". On macOS and Windows the gate also fires (confirmed: macOS plan `interim=true`), and there the secrets ride argv with `--allow-argv-secret`. A user would be told something untrue about how the secret travels, and G4 was a finding of exactly this kind.
- **Ask:** pick the wording by `plan.interim`.

### Nit

- **n1 — pre-release ordering** (`cli_version.rs:26-27`). `0.107.1-dev` is accepted as ≥ 0.107.1 although semver orders it lower. `trim_start_matches('v')` also strips repeated `v`s. This is harmless today because `@env:` predates the current pins, and the PR body documents it.
- **n2 — no timeout** (`cli_version.rs:99-110`). The probe runs on the UI thread with no timeout, so a binary that hangs on `--version` freezes the GUI. This matches the existing synchronous-Run contract, which the harness requires.
- **n3 — check-then-run gap.** Both the probe and the run resolve `bin` through PATH at spawn time. Replacing the binary between the two calls needs write access to a PATH directory, which already gives arbitrary code execution. Not actionable.
- **n4 — G2 control characters.** A TAB triggers cmd's filename completion on paste, which alters the argument. Other control characters are not blocked either. This changes an argument; it does not inject a command.
- **n5 — G3 durability.** There is no fsync before the rename. This is a documented non-goal (persistence.rs:286), and `load()` has a malformed→`.bak` path.
- **n6 — test strength:**
  - `g1_pinned_or_newer_cli_passes_and_the_plan_is_untouched` (`tests/g1_cli_version_gate.rs:59-83`): the "untouched" half is tautological, because the gate only borrows `&RunPlan`.
  - `g1_spawn_and_capture_calls_the_gate_before_spawning` is a source-text ordering grep. It does not prove the `Err` arm returns before the spawn.
  - No committed test drives `probe` against a fake binary that reports an older version; the only one uses `true`. My scratch test shows the real path behaves; consider committing one (Unix, tempdir on PATH).
- **n7 — G4 text:**
  - `tests/secrets.rs:472` asserts the wrapped string `"not as a command-line\nargument"`, so rewrapping the text breaks the test.
  - The interim `PASTE_WARN_MODAL_TEXT` was reflowed ("for this\nwindow" became "for\nthis window"), and its "Byte-exact per SPEC §9" doc line was dropped. If SPEC §9 is still meant to be byte-exact, either the SPEC or the text has drifted.
  - The PR body already lists the manual-gui follow-ups for G1, G2, G3 and G4.

## Answers to the brief's questions

1. **G1 parsing and gating:**
   - **Parsing:** robust for clap output, including pre-release and build suffixes such as `mr 0.1.0-dev (ee692025)`.
   - **Binary names:** `md` versus `md-cli` is not a problem, since every CLI's clap name equals its binary name.
   - **stderr, non-zero exit:** version text printed only on stderr is refused, which is correct for clap, since clap prints `--version` on stdout. A non-zero exit is refused.
   - **Timeout:** there is none (n2).
   - **Check-then-run gap:** not exploitable (n3).
   - **Bypass:** none found.
   - **Correct installs:** these pass. Only older-but-safe CLIs are refused (m1).
   - **Plan:** a passing run spawns exactly the same plan.
2. **G2 cmd.exe set:** the set is sufficient for cmd.exe for the quoted tokens. Every token from `render_copy_command` and every quoted Copy token goes through `cmd_quote`, which always wraps. Once `"` is excluded, cmd's quote state stays balanced, so `& | < > ( ) ^` stay inert, and `%`, `!`, CR and LF are blocked. Two residual gaps remain:
   - The unquoted `<FILE>` placeholders (m3).
   - PowerShell as the paste target (m2).

   The REM comment lines carry only schema labels and `@env:` names validated as `[A-Z_][A-Z0-9_]*` (channels/mod.rs:345).
3. **G3 permissions and the temp file:**
   - **Temp file:** same-directory, per-PID name; stale-temp removal first, then `create_new` (O_EXCL).
   - **Symlinks:** a planted symlink cannot be followed. A race causes an EEXIST failure (denial of service only), and the attacker would need write access to `~/.config/mnemonic-gui`.
   - **Windows:** the mode code is `cfg(unix)`. Windows builds compile and rely on the profile ACL.
   - **Errors:** if the write or the rename fails, the temp is removed and the error propagates.
   - **fsync:** none (n5).
   - **Gap:** the `.bak` and old `.tmp` files (m4).
4. **Tests:** they assert what they claim, with the weak spots in n6 and n7. The G2 cmd model is a fair model of cmd's phase-2 parsing for this purpose, and the G3 test checks both `save` and `save_if_changed` and that no temp file is left behind.
