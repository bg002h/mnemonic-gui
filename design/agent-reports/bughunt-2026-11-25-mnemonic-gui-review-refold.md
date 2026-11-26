# mnemonic-gui PR #58 — review of the F-M1 / F-M4 fold (`85a0396..4025f58`)

Reviewer: Claude (read-only; no commits, pushes or GitHub comments). HEAD `4025f58`.
Commits: `9b905ae` (code, tests, README, CHANGELOG), `4025f58` (FOLLOWUPS status).

**Result: 0 Critical / 0 Important / 0 Minor / 5 Nit.**

## Tests run

`CARGO_INCREMENTAL=0 cargo test --locked -j 2` (debug profile, no `--release`):

| target | result | log |
|---|---|---|
| `--lib cli_version` | 7 passed | `scratchpad/refold-lib.log` |
| `--test copy_command` | 21 passed | `scratchpad/refold-int.log` |
| `--test g1_cli_version_gate` | 6 passed | `scratchpad/refold-int.log` |
| `--test readme_pin_coherence` | 2 passed | `scratchpad/refold-int.log` |

## Does each fix do what was asked?

- **F-M1 / m5 (refusal text).** `refusal_text` (`src/cli_version.rs:95-117`) picks the wording by `plan.interim`. `plan.interim` is true exactly when a binding is `ChannelKind::Argv`, and the interim planner turns *every* binding into Argv (`src/form/channels/mod.rs:488-520`), so "on its command line with --allow-argv-secret" is true whenever the flag is set, and "private channels (@env:, stdin or a pipe)" is true otherwise. The text adds "The copied command (Copy buttons) is not safe with this binary either", which is what F-M1 asked. `$PATH` is replaced by "the search path", which is accurate on Windows. Rust's `Command` searches the exe's own directory and the system directories as well as PATH, and a GUI started from Finder on macOS gets launchd's PATH. Both wrap sites (probe failure and check failure) get the context. The "unknown binary" refusal stays bare, which is correct: it has no pinned version to cite. **Done.**
- **F-M1 / r2-m1 (Copy comment).** `render` (`src/form/channels/copy.rs:348-357`) inserts `copy_version_warning(c, pinned)` as the first comment in both flavours. It does this only on the secret-bearing path, because the `!plan.has_secrets()` early return at `:182-190` skips it. Copy is not disabled, which matches the owner's choice. **Done.**
- **F-M4 / r2-n2 (README / CHANGELOG).** README "Install" (`README.md:108-113`) and the CHANGELOG G1 bullet now say that a GUI-only upgrade is not enough for secret-bearing runs. The pinned strings (`mnemonic 0.107.1`, `md 0.21.0`, `ms 0.21.0`, `mk 0.17.0`) match the README install tags and `pinned-upstream.toml`. **Done.** The other F-M4 items (toolkit manual-gui rows, `$MNEMONIC_BIN`, DESIGN §A8 table) are correctly left open in FOLLOWUPS.

## Can the comment line break a pasted command?

The new line is `# Needs <cli> <ver> or newer: an older <cli> can take an @env:/stdin reference below literally as the secret.` (POSIX, joined with `\n`), or the same text after `REM ` (Windows, joined with `\r\n`). It contains no `% ! " < > | & ^ ` $ * ? [ ( )` and no backtick. The pinned strings contain only letters, digits, dots and a space.

- **bash:** interactive shells have `interactive_comments` on by default, so the line is a comment. The command, or the `printf … |` pipe prefix, follows on its own line. No effect.
- **fish:** `#` is always a comment. No effect.
- **zsh:** `INTERACTIVE_COMMENTS` is **off** by default in plain zsh (Oh My Zsh and most frameworks turn it on). With it off, the line runs `#` as a command and prints `command not found: #`. The line has no glob or metacharacters, so nothing else happens and the next line still runs. This is **not new**: every secret-bearing POSIX Copy already began with at least one `#` line before this fold, because every binding kind pushes one. See Nit 5 for the pre-existing, worse case.
- **cmd.exe:** the Windows flavour is designed for cmd (`ShellFlavor::WindowsCmd`, `^\r\n` continuations, the G2 cmd blocker, tooltips naming cmd.exe). It already emitted `REM` lines for EnvRef and file bindings (`copy.rs:228`, `:313`). `REM Needs …` is a valid comment there. Its first token is not `/?`, and it holds no `%` to expand. No effect.
- **PowerShell:** `REM` is not a PowerShell comment or alias, so the line errors with "The term 'REM' is not recognized". This is the **pre-existing** m2 hazard and is not made worse. The Windows Copy was already PowerShell-incompatible before the fold: it uses `^` continuations (PowerShell uses a backtick), and it has pre-existing `REM --from: set NAME=<the value>` lines, whose `<` is a reserved-operator parse error under PSReadLine's whole-buffer paste. The fold adds no new failure class for PowerShell. The comment's own `@env:/stdin` text sits on a line that already fails.

## Could it leak a secret, or change argv or plans?

No. The line interpolates only `plan.argv[0]`, through the fixed four-name lookup in `pinned_version_for`, and a `&'static` pinned string. `render` takes `&RunPlan` and only builds the output string. `toks` and the argv are untouched, and the warning is inserted after the G2 blocker has examined the tokens. `refusal_text` interpolates the probe or check reason, which already existed and is the binary's first `--version` line, plus static text. No secret value reaches either string.

## Do the tests assert the behaviour?

Mostly yes. Mutation reasoning:
- Removing the `comments.insert(0, …)` fails `fm1_secret_copy_starts_with_the_version_warning`. So does emitting `#` for Windows, because the test compares against `copy_version_warning(c, …)` with `c` per flavour.
- Passing `false` instead of `plan.interim` in `gate_secret_run` fails `g1_refusal_text_matches_the_platform_path_and_warns_about_copy`. That test uses a real macOS-planned interim plan, not a hand-set flag, which is good.
- Inverting the `if interim` arms fails the lib test `refusal_text_is_platform_accurate_and_warns_about_copy`.
- The README and CHANGELOG sentences are pinned by `readme_and_changelog_say_secret_runs_need_the_pinned_cli_versions`.

Gaps are listed as nits below.

## Findings

### Nit 1 — the probe-failure wrap is not asserted
`src/cli_version.rs:145`. Scenario: a later edit reverts `probe(bin).map_err(|e| refusal_text(…))?` to `probe(bin)?`. A user whose `ms` exits non-zero on `--version`, or is missing, then gets the bare "`ms --version` failed (exit Some(2)); cannot confirm…" with no remedy and no Copy warning. The only test of this branch, `tests/g1_cli_version_gate.rs:55`, checks `is_err()`. Add one `contains("Install ")` assertion on that error.

### Nit 2 — the Copy warning names "@env:/stdin" even when the only secret binding is a file
`src/form/channels/copy.rs:44-50`. Scenario: a Windows Copy whose only secret binding is a pipe-fd input renders as `--in <FILE>`. The first line then says "an older ms can take an @env:/stdin reference below literally as the secret", but the command below holds no such reference. The text is wrong but on the safe side, since the version requirement still stands. Wording such as "a secret reference below" would cover both cases.

### Nit 3 — the negative case is checked for the POSIX flavour only
`tests/copy_command.rs:438-441`. The no-secret Windows path returns before the insertion at `copy.rs:182-190`, so it is safe today. The test would not catch a refactor that moves the insertion into the shared tail for the Windows flavour only. Also check `windows.text()`.

### Nit 4 — the README test pins a hard line wrap, and the README wording follows the Linux Run
`tests/readme_pin_coherence.rs:107` asserts `"pinned version above\nor newer"`, so rewrapping the paragraph fails the test without any change in meaning. Separately, `README.md:110-112` says the GUI's `@env:`/stdin reference can be misread. That holds for Linux Run and for Copy on every OS, but on macOS and Windows the Run sends values on argv. The refusal text gets this right; the README sentence is merely Linux-centric.

### Nit 5 (pre-existing, out of diff) — the zsh default with the existing `read` hint line
`src/form/channels/copy.rs:222-226`. In plain zsh with `INTERACTIVE_COMMENTS` off, the existing line `# --from: IFS= read -rs NAME; export NAME   (fish: …)` does not behave as a comment. `#` fails as a command, then `export NAME (fish: …` is parsed and fails at `(`. The new version line does not make this worse. It is recorded only because the question asked about zsh.

## Is the refusal text true on each platform?

- **Linux (private plan):** true. Secrets go as `@env:MNEMONIC_GUI_S<i>`, stdin or `/dev/fd/N`, and an older CLI can read the reference literally.
- **macOS/Windows (interim plan):** true. Every binding is Argv with `--allow-argv-secret`, and "may refuse or misread" is accurate, because an older CLI may reject the opt-in flag. The Copy sentence is true on every OS, because Copy is always generated from the private plan (`Policy::for_copy`).
- **All platforms:** "the search path" is correct where `$PATH` was not, on Windows and in a GUI launched from Finder on macOS.
