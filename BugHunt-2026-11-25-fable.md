# Bug hunt 2026-11-25 (Fable): mnemonic-gui

Report-only review of `mnemonic-gui` at `f901421`, part of a constellation-wide hunt across all 11 bg002h repositories. Nothing in this repository was changed by the hunt; this file is the only addition.

**How it was done.** Eight Fable reviewers each audited an area (one of them the boundaries between repositories), reproducing what they could with locally built binaries or scratch tests that were not committed. Three further Fable verifiers then tried to disprove every High and Medium finding (and several Low ones) and re-graded them. Grades below are the verifiers' final grades where they ran.

**Grades.** Critical: can lose funds or leak a seed in ordinary use. High: wrong wallet, addresses or keys, or secret exposure, plausible in normal use, or recovery broken. Medium: needs an unusual but realistic input, or fails in a way a careful user might not notice. Low: fails closed, hardening, or a small exposure. Info: context, not a bug.

**Categories.** security (secret or key exposure, injection, integrity of what is trusted), funds (wrong address, amount, descriptor or recovery that could lose or strand coins), other.

**Status.** "Confirmed (reproduced)" means a command or test showed it. "Confirmed by reading" means the code path was traced but not executed. "Reviewer-confirmed, not re-verified" means the first reviewer reproduced or read it and no verifier re-checked it.

**Totals:** 1 Medium, 4 Low. No Critical findings.

## Summary

| ID | Grade | Category | Status | Finding |
|---|---|---|---|---|
| G1 | Medium | funds | Confirmed (reproduced) | No runtime check of the spawned CLI's version; an old CLI can take `@env:...` literally as the passphrase |
| G2 | Low | security | Confirmed by reading | Windows Copy quoting is for `CommandLineToArgvW` only; `cmd.exe` metacharacters in a field run as commands |
| G3 | Low | security | Confirmed (reproduced) | `state.json` written with default permissions |
| G4 | Low | other | Reviewer-confirmed, not re-verified | The paste warning says the secret goes on the command line; on Linux it no longer does |
| G5 | Low | other | Reviewer-confirmed, not re-verified | Pins and their comments drift (md-cli 0.21.0 vs 0.21.1, mk 0.17.0) |

IDs: `md#`, `mk#`, `ms#`, `toolkit#`, `G`, `refugium#`, `wallet#`, `E`, `T`, `SH#`, `M` are per-repository findings; `X` findings are at a boundary between repositories and appear in every report they touch.

## Findings

## G1: No runtime check of the spawned CLI's version; an old CLI can take `@env:...` literally as the passphrase

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `gui`)

### G1. No runtime check of the spawned CLI's version; `schema_check.rs` is not called by the app

- **Severity:** High if an older CLI is on `$PATH` (see cross.md X1 for the full chain)
- **Status:** SUSPECTED (GUI side read from code; old-CLI behaviour from the CLIs' CHANGELOGs, not executed)
- **Repos:** mnemonic-gui (with mnemonic-toolkit, mnemonic-secret)
- **Evidence:** `src/path_detect.rs:10-47` takes the first `mnemonic`/`md`/`ms`/`mk` on `$PATH`;
  `src/app_window.rs:491-492` prints the static label `Pinned: <version>`; `src/schema_check.rs:351,385` (the only
  code that could ask the binary) has no caller in `src/` outside its own file, and `:61-62` reads
  `pinned-upstream.toml` from `env!("CARGO_MANIFEST_DIR")`, a path that exists only on the build machine.
  `pinned-upstream.toml:5-6` and `:37-38` describe a "runtime soft-check" and its "banner" as existing.
- **Impact:** the planner sends `--passphrase @env:MNEMONIC_GUI_S0` (and `-`, `/dev/fd/N`) on the strength of
  measurements of the pinned binaries. A CLI from before `@env:` existed and before the argv guard takes that text
  as the secret, exit 0: a wallet under the passphrase `@env:MNEMONIC_GUI_S0`.
- **Fix direction:** probe `--version` once per binary and gate private-channel Runs on it; make the header of
  `pinned-upstream.toml` say what is true.

**Reviewer write-up** (source: `cross`)

### X1. mnemonic-gui has no runtime check of the CLI version it spawns; an older CLI takes `@env:MNEMONIC_GUI_S<i>` as the secret itself

- **Severity:** High (wrong wallet, exit 0) if the precondition holds
- **Status:** SUSPECTED. The GUI half is read from code; the old-CLI half is taken from the CLIs' own CHANGELOGs,
  not executed (no old binary available offline).
- **Repos:** mnemonic-gui, mnemonic-toolkit, mnemonic-secret (ms-cli), mnemonic-key, descriptor-mnemonic
- **Evidence:**
  - On Linux every secret goes over a private channel: `--passphrase @env:MNEMONIC_GUI_S0` and the like
    (`mnemonic-gui/src/form/channels/mod.rs:672-674`, step 3 of `plan_private`; policy `design/measurements/secret-channels/channel_policy.json:3`,
    `private_channels_on: ["linux"]`). The channel table was measured against the pinned binaries only
    (`a5.md:1`: "mnemonic 0.107.1 ... ms 0.21.0").
  - The binary is whatever `mnemonic`/`md`/`ms`/`mk` is first on `$PATH` (`src/path_detect.rs:10-47`, `src/app.rs:90-93`).
    Nothing runs `--version` or `gui-schema` at run time: `src/schema_check.rs` has no caller outside itself and the
    tests (`grep -rn "schema_check::\|json_flag_names" src` gives only `schema_check.rs`), and it reads
    `pinned-upstream.toml` from `env!("CARGO_MANIFEST_DIR")` (`schema_check.rs:61-62`), a build-machine path. The UI
    only prints the label `Pinned: mnemonic 0.107.1` (`src/app_window.rs:491-492`). `pinned-upstream.toml:5-6,37-38`
    still describe a "runtime soft-check" and a "banner".
  - mnemonic-secret `CHANGELOG.md:268-275` (ms-cli 0.20.0, F-687): before it, "`@env:VAR` was refused by the argv
    guard as material, or taken literally under `--allow-argv-secret`"; before the argv guard (ms-cli < 0.19.0) there
    was no refusal. The toolkit gained the same rule in 0.105.0 (`CHANGELOG.md` 0.105.0 section) and its argv guard in
    0.104.0 (`CHANGELOG.md:678`).
- **Impact:** a user who updates the GUI but keeps an older `mnemonic` or `ms` on `$PATH` (a distro package, an old
  `cargo install`) gets, for example, a bundle derived with the BIP-39 passphrase `@env:MNEMONIC_GUI_S0` instead of
  the passphrase typed, at exit 0. The GUI shows nothing but the "Pinned:" label. Restoring later with the real
  passphrase gives a different wallet.
- **Reproduction (to run where an old binary exists):** put toolkit <= 0.103 `mnemonic` first on `$PATH`, run the GUI,
  `convert` a phrase with a passphrase to an xpub; compare with `mnemonic convert ... --passphrase-stdin` of the same
  passphrase.
- **Fix direction:** before the first Run per binary, run `<bin> --version` (or `gui-schema`) and refuse to send a
  secret over `@env:`/`-` to a binary older than the version the channel table was measured against; correct the
  `pinned-upstream.toml` header.

**Verifier re-check** (source: `verify-2`)

### G1 / X1: mnemonic-gui never checks the version of the CLI it spawns

**Verdict: CONFIRMED-but-downgraded. Grade: Medium (claimed High). Category: funds.**

The GUI half holds. Binaries are the first `mnemonic`/`md`/`ms`/`mk` on `$PATH`
(`mnemonic-gui/src/path_detect.rs:19-47`, `src/app.rs:90-93`); nothing is bundled. No `--version` or
`gui-schema` probe runs in the app: `grep -rn "schema_check" src` outside `schema_check.rs` finds only
`src/lib.rs:20` (`pub mod`) and comments; the header shows the static string `sch.pinned_version`
(`src/app_window.rs:490-491`). The README documents installing the GUI alone (`README.md:91-94`), so a new GUI
over older CLIs is an ordinary state.

The old-CLI half is narrower than both write-ups say, and their suggested reproduction would not show it:

- **Toolkit `--passphrase` has resolved `@env:VAR` since 0.26.0** (`mnemonic-toolkit/CHANGELOG.md:3651`, under
  `[0.26.0] — 2026-05-18`), so "toolkit <= 0.103 takes it literally" is wrong for every release from 0.26.0 on.
  The exception the toolkit's own CHANGELOG records: on 0.104.0 "`@env:VAR` was literal on `silent-payment`"
  (`CHANGELOG.md:463-464`).
- **`convert` with a phrase and a passphrase does not send `@env:` for the passphrase at all.** The planner's
  step 2 gives stdin to the first source with a `--X-stdin` toggle (`src/form/channels/mod.rs:617-627`), and
  `mnemonic convert --passphrase` has one (`channel_table.json`: `DashValue, EnvRef, StdinToggle
  --passphrase-stdin`) while `--from phrase=` has none. So the passphrase goes by `--passphrase-stdin`, which
  old CLIs read correctly.
- **Where the literal reading is real:** (a) `ms derive <ms1> --passphrase`: the ms1 positional is stdin-only
  (`PosDash`), so the passphrase becomes `--passphrase @env:MNEMONIC_GUI_S<i>`. ms-cli 0.17.0 to 0.19.1 refuse
  that (argv guard, fails closed); ms-cli before 0.17.0 (2026-09-02) has no guard and takes the text as the
  passphrase (`mnemonic-secret/CHANGELOG.md:269-275`, guard dated at `:328`). (b) `mnemonic silent-payment`
  with `--secret` on stdin and a passphrase, against toolkit <= 0.104.0. (c) any toolkit before 0.26.0.
- On macOS and Windows the interim path sends `--allow-argv-secret`, which an old CLI rejects as an unknown
  flag, so those fail closed.

Result in the cases that remain: a master fingerprint/xpub, or a silent-payment address, for the passphrase
`@env:MNEMONIC_GUI_S<i>` at exit 0, with nothing on screen but "Pinned:". Not executed: no old binary and no
GUI build here. Medium because it needs a GUI upgraded without its CLIs plus one of three specific
verb/version pairs, and then fails unnoticed. The wrong wallet is recoverable by anyone who knows the literal
string, but the user would not know to try it.

## G2: Windows Copy quoting is for `CommandLineToArgvW` only; `cmd.exe` metacharacters in a field run as commands

**Grade:** Low · **Category:** security · **Status:** Confirmed by reading

**Reviewer write-up** (source: `gui`)

### G2. The Windows Copy button's quoting is for `CommandLineToArgvW` only; `cmd.exe` metacharacters in a field value run as commands

- **Severity:** Medium (needs a field value holding `"`; the user pastes the copied text into `cmd.exe`)
- **Status:** CONFIRMED for the string produced (transliteration of `cmd_quote` run in
  `work-x/cmdq.py`); SUSPECTED for execution (no Windows host here; the `cmd.exe` rule is the documented one in the
  article the function's own comment cites, whose second half, caret-escaping for `cmd`, is not implemented)
- **Repos:** mnemonic-gui
- **Evidence:** `src/form/invocation.rs:789-832` (`cmd_quote`), used by `render_copy_command` (`:701-714`) and
  `channels/copy.rs:303-313` for `ShellFlavor::WindowsCmd`. It escapes `"` as `\"` and nothing else.
  ```
  $ python3 work-x/cmdq.py
  'x" & calc.exe & rem "' -> "x\" & calc.exe & rem \""
  '%USERPROFILE%'         -> "%USERPROFILE%"
  ```
  `cmd.exe` does not know `\"`: the second `"` ends its quoted region, so ` & calc.exe & rem \` is parsed as
  commands. `%NAME%` is expanded inside double quotes, so a value such as a label or path holding `%...%` is
  changed before the CLI sees it. The lines are joined with `^\r\n`, which is `cmd` syntax, so `cmd.exe` is the
  intended target.
- **Impact:** a label, file name, descriptor comment or any text field that holds `"` (typed, or pasted from a
  message someone else sent) turns the copied command into one that runs extra commands when pasted; a value with
  `%` is silently altered, so the pasted command is not the command the GUI's Run executes.
- **Fix direction:** after the `CommandLineToArgvW` pass, caret-escape `( ) % ! ^ " < > & |` for `cmd`, or disable
  the Windows Copy for a token that holds any of them (the module already disables Copy in other unsafe cases).

**Verifier re-check** (source: `verify-2`)

### G2: Windows Copy quoting is `CommandLineToArgvW`-only

**Verdict: CONFIRMED-but-downgraded. Grade: Low (claimed Medium). Category: security.**

`cmd_quote` (`mnemonic-gui/src/form/invocation.rs:789-832`) escapes `"` as `\"` and nothing else, and the
Windows flavour joins tokens with `^\r\n` (`:708-712`, `channels/copy.rs:313-316`), which is `cmd.exe` syntax.
By `cmd.exe`'s quote-toggling rule the `"` in `\"` ends the quoted region, so `x" & calc.exe & rem "` becomes
`"x\" & calc.exe & rem \""` with ` & calc.exe & rem \` outside quotes. The string is confirmed from the code;
execution is not (no Windows host). Downgraded because it needs crafted text the user puts in a field, then
Copy, then a paste into `cmd.exe`. An innocent value holding `"` and `&` gives a command that breaks with
"not recognized", not one that runs something chosen by an attacker. The Run button is unaffected (no shell).

## G3: `state.json` written with default permissions

**Grade:** Low · **Category:** security · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `gui`)

### G3. `state.json` is written with default permissions

- **Severity:** Low (privacy)
- **Status:** CONFIRMED by reading
- **Repos:** mnemonic-gui
- **Evidence:** `src/persistence.rs:287-305`: `fs::write(&tmp, body)` then `rename`; no mode is set anywhere in
  `src/` (`grep -rn "set_permissions\|0o600" src` gives nothing). With the usual umask the file is 0644 in
  `~/.config/mnemonic-gui/`.
- **Impact:** secrets are stripped, but descriptors, xpubs, fingerprints, derivation paths and file paths the user
  typed stay, readable by other local users: enough to watch the wallet's addresses. The toolkit made the same
  change for `seedqr --json-out` (0600) on 2026-11-24.
- **Fix direction:** create the temporary file 0600 on Unix.

**Verifier re-check** (source: `verify-2`)

### G3: `state.json` written with default permissions

**Verdict: CONFIRMED. Grade: Low. Category: security (privacy).**

`write_atomic` (`mnemonic-gui/src/persistence.rs:287-304`) is `fs::create_dir_all`, `fs::write(&tmp)`,
`fs::rename`; `grep -rn "set_permissions\|0o600" src` finds nothing, and the only `PermissionsExt` use is the
executable test in `path_detect.rs:71`. With umask 022 the file is 0644. Secrets are stripped before the
write, so what other local users can read is descriptors, xpubs, fingerprints and paths: address privacy, no
spending power.

## G4: The paste warning says the secret goes on the command line; on Linux it no longer does

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `gui`)

### G4. The paste warning says the secret goes on the command line; on Linux it does not

- **Severity:** Low (wrong text shown to the user)
- **Status:** CONFIRMED by reading
- **Repos:** mnemonic-gui
- **Evidence:** `src/secrets.rs:225-237` (`PASTE_WARN_MODAL_TEXT`), shown at `src/app_window.rs:1238`: "This GUI
  runs the binary with the value you pasted as a command-line argument. The OS may log this argument in process
  tables and shell history files." Since the secret-channels work the Run path on Linux uses `@env:`, stdin or a
  pipe (`channel_policy.json:3`); argv is used only on the interim path (macOS, Windows). The separate run-confirm
  text has both variants (`RUN_CONFIRM_MODAL_PREFIX` / `RUN_CONFIRM_PRIVATE_PREFIX`); this one has not.
- **Fix direction:** pick the text by the same `private_channels_on` test the run-confirm dialog uses.

## G5: Pins and their comments drift (md-cli 0.21.0 vs 0.21.1, mk 0.17.0)

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `gui`)

### G5. Pins and their comments

- **Severity:** Low. **Status:** CONFIRMED by reading.
- `[md].tag = descriptor-mnemonic-md-cli-v0.21.0` (`pinned-upstream.toml:42`) while md-cli 0.21.1 is released with
  four fixes (duplicate `--key`/`--fingerprint`, `md encode` minting a card `md decode` refuses, `md descriptor
  --change` printing the receive descriptor for a wallet with no change branch, a wrong coordinator note). The GUI
  surfaces `md descriptor` (`secrets.rs:207-211` names its fields).
- `[mk].tag = mk-cli-v0.17.0` is before the unreleased bh-mk-C1 fix (`mk address`/`mk derive` on a toolkit WIF
  card); see cross.md X4.
- The comment above `[md]` (`:32-41`) describes v0.20.3 and v0.11.0, not the pinned v0.21.0.
- `Cargo.lock` links the toolkit library at tag `v0.107.1` with md-codec 0.47.0, mk-codec 0.4.1, ms-codec 0.7.0.
  The GUI uses only `secret_taxonomy` from it (`src/secrets.rs:1-3`), so the old codecs are linked but not called.
  The toolkit's HEAD is still versioned 0.107.1 with 131 commits after the tag; a `mnemonic` built from HEAD and
  one built from the tag both print `mnemonic 0.107.1` apart from the commit suffix.

---

_Generated by [Claude Code](https://claude.ai/code) on 2026-11-25. Findings were not fixed; each fix needs its own review under this repository's gates._
