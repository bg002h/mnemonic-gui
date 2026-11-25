# mnemonic-gui fix branch: fold review, round 2

**Scope:** fold commit `587e1c4` on `claude/project-thread-utm1aa` (/home/user/mnemonic-gui), plus a fresh-eyes regrade of r1's Minor findings. I read the code. I did not edit, commit or push anything, and I ran no tests: every question here could be answered from source.

**Verdict: 0 Critical, 0 Important, 1 new Minor, 2 Nit.** The fold is accurate and touches nothing else. None of r1's Minors rises to Important. The new Minor (r2-m1) predates the branch: the Copy path carries the same old-CLI hazard that G1 closes for Run.

## Q1. Does the fold log every r1 Minor and Nit accurately?

**Yes.**

- **Report copy:** `design/agent-reports/bughunt-2026-11-25-mnemonic-gui-review-r1.md` is byte-identical to the scratchpad r1 (`diff` printed nothing).
- **Minors:** the `FOLLOWUPS.md:1194-1203` entry has one bullet each for m1–m5. Every bullet keeps r1's grade, file:line and substance. The "pre-existing" tags on m2 and m3 match r1's "predates the branch". m1 keeps "Fails closed".
- **Nits:** all seven are in the single Nits bullet:
  - n1: `-dev`
  - n2: no timeout
  - n3: TOCTOU
  - n4: TAB/control characters
  - n5: fsync
  - n6: the three weak-test points
  - n7: wrapped-string test and the lost "Byte-exact per SPEC §9"
- **Small omissions:** none changes meaning.
  - m4 does not cite `persistence.rs:305` for stale temp files.
  - n1 leaves out the repeated-`v` strip.
  - n7 leaves out "the PR body lists the manual-gui follow-ups".
- **Format:** the heading style (`## \`slug\` — title`) and the Surfaced/Status/Tier bullets match the neighbouring tail entries (`FOLLOWUPS.md:1156-1192`).

**Nit r2-n1** (`FOLLOWUPS.md:1194`):
- The file's original body nests entries as `###` under `## Active` (`:8-10`). The tail section, which this entry joins, uses `##`. This is pre-existing drift; the fold follows its nearest neighbours.
- `windows` is a new tier name, used nowhere else in the file. Harmless.

## Q2. Did the fold change anything else?

**No.** `git diff 587e1c4~1 587e1c4 --stat` shows two files: `FOLLOWUPS.md` (+11, appended at EOF, no other lines touched) and the new report (+123). The commit message is accurate and carries the attribution lines.

## Q3. Is any r1 Minor actually Important for wallet software holding real funds?

### m1: G1 refuses every passphrase run when the CLI is older than the pin. It stays Minor.

- **Where:** `src/cli_version.rs:114-126` and `src/app_window.rs:1391-1400`.
- **Funds stay reachable:**
  - The refusal spawns nothing and changes no state. It applies only to `plan.has_secrets()` runs (`channels/mod.rs:830`, "bindings non-empty").
  - The message names the fix: install `{pinned}` or newer, or move the old binary out of the way.
  - The pinned version is always a released tag, kept in lockstep by `pin_coherence`.
  - Three other routes stay open: the user runs the CLI directly; the GUI's Copy button still emits the command (Copy is not gated, see r2-m1); or the user keeps the previous GUI.
  - Nothing is written, deleted or migrated, so a restore is delayed and never prevented.
- **It is the safe direction:** the alternative G1 prevents is exit 0 with a *different wallet* from an old CLI that reads `@env:MNEMONIC_GUI_S0` literally. That is the funds-relevant failure; a loud refusal is not.
- **One aggravating detail, still Minor:**
  - On the interim argv path (macOS/Windows, `mod.rs:878`) there is no channel reference for an old CLI to misread.
  - A CLI older than the argv guard would reject the unknown `--allow-argv-secret` through clap, which also fails closed.
  - So on those OSes the gate refuses CLIs at or above the argv-guard version but below the pin, and gains nothing in safety. That is over-refusal, not a hazard, and it strengthens r1's m1 ask (a per-CLI minimum version, or skip the probe when `plan.interim`) together with m5's wording fix.

### m2: PowerShell paste of the Windows Copy. It stays Minor.

- **Where:** `src/form/invocation.rs:749` and `:780`; label at `src/app_window.rs:1063`.
- **The usual paste parses as a whole and fails:** in PowerShell with PSReadLine, the default since Windows 10 1809, a multi-line paste is parsed as one script before anything runs.
  - Every `"…" ^` line is a parse error ("Unexpected token '^'").
  - So the whole buffer is rejected and nothing runs, including any `$(…)`.
- **Injection needs line-by-line feeding:** the console must feed lines one at a time (legacy conhost or no PSReadLine). Even then only the **last** line evaluates, because each earlier line is a parse error. Three things must all hold:
  - (a) attacker text in the last argv token;
  - (b) the user chooses the Windows Copy;
  - (c) the user pastes into a line-feeding PowerShell.
- **Who controls field values:**
  - Values come only from what the user types or pastes.
  - `state.json` sits in the user's own config directory. Anyone who can write it can already edit shell profiles or PATH, so it is not an extra attack surface.
  - Tree-mode/spec JSON goes over stdin, not the cmd Copy.
  - So an attacker must persuade the user to paste a crafted string into a field. Someone with that leverage could just as well hand over a command.
- **Secrets are not exposed:** secret values never appear in the Copy; they are `@env:` references or placeholders.
- **The ask is cheap:** add "cmd.exe" to the label and block `$` and the backtick. Grade unchanged.

### m3: bare `<FILE>` placeholders on Windows. It stays Minor.

- **Where:** `src/form/channels/copy.rs:284-295`; the overstated doc claim is at `src/form/invocation.rs:789-791`.
- **Pasted unchanged:** this is harmless. In cmd, `<FILE>` redirects from a nonexistent `FILE` and the command errors. In PowerShell `<` is reserved and the paste fails to parse.
- **To clobber a file:** the user must edit the placeholder into `<path>`, and the token right after it must be a path the user cares about. Usually it is a quoted flag name, so the damage is a junk file such as `--network` in the current directory.
- **No attacker control:** the placeholder text is fixed. An attacker controls neither it nor the user's edit, and no secret leaves through it; the card is read *from* the file.
- **Remaining issue:** user-error-dependent clobbering plus an inaccurate doc comment. Grade unchanged.

### m4 and m5

These were not asked about. With fresh eyes they still read as Minor:
- **m4:** a 0644 `.bak` holds the public material G3 meant to tighten, not secrets.
- **m5:** the message is wrong but fails closed.

## New finding

**r2-m1 (Minor, pre-existing, outside G1's Run-only scope): the Copy path reproduces G1's hazard and is not gated.** Location: `src/form/channels/copy.rs:100-125` and `:195-252`.
- **What the Copy emits:** the command carries `@env:VAR` references, with the comment "read by the CLI from $VAR".
- **What it lacks:** it has no version check and gives no "requires `{pinned}` or newer" caveat.
- **What happens:** the user pastes it into their own shell with an older `ms`/`mnemonic` first on PATH, the same setup G1 guards against. That CLI can take `@env:VAR` literally as the passphrase and exit 0 with a different wallet.
- **Why only Minor:**
  - The user runs the command outside the GUI, so the GUI cannot probe that binary.
  - The wrong passphrase is the deterministic literal string, so such a wallet can be recovered once someone diagnoses it.
  - The behaviour predates the branch.
- **Ask:** add one comment line to both Copy flavours: `# requires <pinned_version> or newer; older CLIs may read @env: literally`.

## Nits

- **r2-n1:** heading-level drift and the new tier name; see Q1.
- **r2-n2:** the CHANGELOG's G1 entry (`CHANGELOG.md:10`) does not tell users that a GUI upgrade now requires CLI upgrades before secret runs. This is r1 m1's "Ask", logged but not done; recording it so the release notes do not miss it.

## Bottom line

- The fold is correct and complete, and it is limited to logging.
- No r1 grade needs to change.
- r2-m1 and r2-n1/r2-n2 can be appended to the same `bh1125-gui-review-r1-residue` FOLLOWUPS section, or a sibling r2 section.
- Nothing blocks the PR under the "Critical/Important fixed" rule.
