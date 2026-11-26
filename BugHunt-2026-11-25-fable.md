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

---

## Addendum: the 2026-11-25 bug hunt compared with the 2026-11-19 bug hunt

_Written 2026-11-26 for the coordinator that holds the 2026-11-19 fix queue. Read-only comparison: nothing here was merged or pushed to a default branch._

_This is the per-repo copy. The full addendum, with every repo's 11-19 table, is appended to `BugHunt-2026-11-25-fable.md` in mnemonic-refugium (report PR #247, branch `claude/project-thread-ly86x4`)._

### Why this exists, and how to use it

Two hunts ran a week apart:

- **2026-11-19** (OpenAI Codex, gpt-6). It covered 7 repos: descriptor-mnemonic (md), mnemonic-engrave (me), mnemonic-key (mk), mnemonic-secret (ms), mnemonic-toolkit (tk), mnemonic-transaction (mt) and seedhammer (sh). The reports are `design/agent-reports/bug-hunt-2026-11-19.md` on each default branch. The keys are `bh-<repo>-<id>`.
- **2026-11-25** (Fable). It covered all 11 repos. Each repo has a `BugHunt-2026-11-25-fable.md` report in an open, add-only PR on branch `claude/project-thread-ly86x4`. The fixes are in a second set of open PRs on branch `claude/project-thread-utm1aa`, one per repo.

Before you run any queued 11-19 item:

1. Look the key up in the per-repo tables below. Each table says where the fix is (the merged PR on the default branch), whether the 11-25 hunt found that bug again or a related one, and what is still left to do.
2. If a row's last column says "nothing", the item is done. Don't redo it.
3. Every 11-25 fix branch starts from the current default branch (checked 2026-11-26: 0 commits behind in all 10 repos). So every 11-19 fix merged so far is already underneath the 11-25 fix PRs, and the 11-25 PRs do not undo any of them.

#### The 11-25 PRs

| Repo | Report PR (11-25 findings) | Fix PR (11-25 fixes) |
|---|---|---|
| descriptor-mnemonic | #75 | #76 |
| mnemonic-engrave | #67 | #68 |
| mnemonic-key | #42 | #43 |
| mnemonic-secret | #47 | #48 |
| mnemonic-toolkit | #226 | #227 |
| mnemonic-transaction | #31 | #32 |
| seedhammer | #28 | #29 |
| mnemonic-refugium | #247 | #248 |
| mnemonic-gui | #57 | #58 |
| refugium-wallet | #228 | #229 |
| rust-miniscript | #1 | none (it has only Info findings) |

All of these PRs were open and unmerged when this addendum was written.

#### How the 11-19 fix status was worked out (and its limits)

- The starting point is the closure record `coordinator/agent-reports/recon-bughunt-1119-closure.md` in mnemonic-refugium (2026-11-24). It maps 125 Critical/Important keys to 58 FIX rows and found 27 of them open and 6 unrouted.
- After that record was written, more PRs merged on 11-24 and 11-25. I matched them to keys by the key names their commit titles cite: md#70, #71, #74; mk#37, #38, #39; ms#41, #42; tk#211, #212, #214, #220; mt#26, #27, #29; me#64; sh#25. With those, **every 11-19 Critical/Important key that had a FIX row is now merged.** What is still open are keys that were graded Minor and filed as FOLLOWUPS lines.
- "Merged" means the PR that names the key is on the default branch. I did not re-run each key's reproduction against the merged code.
- The closure record lists ms "I5" as the stdin-alias bug, but that bug is ms M5 in the source report. Source I5 is "verify calls a Japanese Mnem card v0.1 entr". Both are open FOLLOWUPS lines (see the ms table).

### Summary of the overlap

The 11-25 hunt has 50 distinct findings. Compared with the 11-19 hunt:

- **6 are the same bug as an 11-19 finding.** In every case the 11-19 fix merged, but only in one place: one repo, or Rust and not the Go port. The 11-25 hunt found the same bug somewhere else.
- **5 are related, and I'm not sure whether they count as the same bug** (marked UNCERTAIN).
- **About 39 are new.** That includes all 16 findings from the four repos the 11-19 hunt did not cover: mnemonic-gui, mnemonic-refugium, refugium-wallet and rust-miniscript.
- **None of the 11-19 bugs that are still open was found again on 11-25.**

| 11-25 finding | 11-19 key | Match | 11-19 fix (merged) | Where the bug was still present on 11-25 | 11-25 fix PR (open) |
|---|---|---|---|---|---|
| X7 (md-codec pin before 0.48.6 can mint a card the decoder refuses) | bh-md-C2 | SAME | md#59, released in md-codec 0.48.6/0.48.7 | consumers' md-codec pins (engrave, refugium) | engrave#68 `fc5f7c5`, refugium#248 `f8a0d9b`: both pin descriptor-mnemonic `c8cde23` |
| E5 (engrave's mt-codec pin) | bh-mt-I18 | SAME | mt#21 | engrave's mt-codec pin `eec7910b` | engrave#68 `67c3f1c`: pins mt `9b094e0b` |
| X4 (WIF slot writes a depth-0, zero-chain-code xpub onto cards) | bh-mk-C1 | SAME | mk#30 (mk-cli; not released yet) | toolkit mints these cards; md and the Go port derive from them; GUI and installer pin mk-cli 0.17.0 | tk#227 `59057e10` (refuses a WIF slot in template mode), md#76 `59c158e` (md refuses it). Still to do: release mk-cli and move the GUI/installer pins (gui#58 records that no release exists yet) |
| SH#6 (Go: change list equals receive list) | bh-md-I6 | SAME, Go port | md#64 (Rust) | seedhammer `address/`, `gui/` | seedhammer#29 `c2326a3` |
| SH#4 (Go `mk.Encode` lacks the origin refusals) | bh-mk-I9 (same function as bh-sh-I10) | SAME, Go port | mk#29 (Rust, mk-codec 0.6.0); sh#13 fixed only the network check | seedhammer `mk/encode.go` | seedhammer#29 `19b6075` |
| md#3 (placeholder with no wildcard minted as `/*`) | bh-md-C1 | SAME root (same lexer line, `parse/template.rs:345`) | md#60 fixed only the `md decompose` route | md template lexer (encode/address/descriptor) | md#76 `59c158e` and `6604c88` (bare `@N` refused at mint) |
| toolkit#3 (Word Card parity budget below what is stated) | bh-tk-I6 | UNCERTAIN: same weakness (one wrong word loses the card), different positions (header and checkpoint words today; the stop-sign word on 11-19) | tk#212 (stop-sign recovery) | header and checkpoint words | tk#227 `e6fd50c8`, `573dc14f`: the help and manual now state the budget the decoder really has. The decoder itself is unchanged |
| md#4 (`c:` over a non-key child mints a card md cannot restore) | bh-md-I5 | UNCERTAIN: same class (the encoder accepts what restore refuses); md#62 added the mint-time self-decode, but that check does not type-check | md#62 | md-codec mint check | md#76 `f8b4a67`, `ca48a05`, `5ad5a9b`, `fcefa83` (type rule at mint; address/descriptor refuse invalid miniscript unless `--allow-invalid-miniscript`) |
| SH#3 (Go `md.Decode` accepts four shapes Rust refuses) | bh-md-C2 | UNCERTAIN: one of the four shapes (an override keyed on baseline `@0`) is the card C2 used to mint | md#59 (Rust encoder) | Go decoder | seedhammer#29 `216a8af` |
| mk#2 (`mk verify --from-md1` passes a non-cosigner) | bh-mk-I2, I15 | RELATED: encode got the membership check, verify did not | mk#31 | `mk verify` | mk#43 `015d106` |
| toolkit#6 (`--account` >= 2^31 panics in six commands) | toolkit FOLLOWUP `xpub-search-account-window-2p31-panic` (listed as "previously filed" in the 11-19 tk report) | RELATED: same panic class; the earlier fix covered xpub-search only | tk#214 | bundle, verify-bundle, export-wallet, addresses, restore, convert | tk#227 `2d0057e8` |

Not an 11-19 match, but worth knowing:

- X6 (non-ASCII characters read as codex32 symbols by older ms/mk codec pins) repeats a bug from the **October 31** hunt that was fixed in the codecs. mk#43 `34c69b8` covers mk-cli's own md1 input. The consumer pins are tracked by the 11-25 X6/X8 entries.
- toolkit#7 (legacy `sh(sortedmulti)` defaults to BIP-48 `1'`) sits next to the 11-19 "previously filed" toolkit line `xpub-search-sh-wsh-default-origin-notice`, but it is a different wrapper and a different bug. It is fixed by tk#227 `6c99456c` plus the BIP-defaults follow-ups.
- engrave E3 (`--as md1` accepts a contradictory origin) is the md1 route of the ruling in `FIX_sec_canonical_keeps_xpub_depth` (me#54, 11-23). That ruling was not an 11-19 key. engrave#68 `094979e` fixes it.

### Work that the 11-25 PRs create for later

These items follow from the 11-25 fixes. Check the coordinator's queue against them so nothing is done twice:

- **md-codec 0.49.0 re-pins.** md#76 changes minting (explicit origins on keyed cards, the bare `@N` refusal, the type rule, thresh "any k of n"). After it merges, mnemonic-refugium, mnemonic-engrave (their md-codec pins must stay equal), mnemonic-toolkit, refugium-wallet and the seedhammer port each need a re-pin or a check. engrave#68 and refugium#248 pin `c8cde23` (0.48.7), not 0.49.0.
- **mk-cli release.** The bh-mk-C1 fix (mk#30) is still unreleased. gui#58 and the toolkit installer still pin `mk-cli-v0.17.0` (11-25 X4, G5).
- **seedhammer provenance pins.** seedhammer#29 `dde21fe` re-pins md, mk and mt to what the ports converge to (SH#5). Re-check them after md#76 merges.

### Repos the 11-19 hunt did not cover

mnemonic-gui, mnemonic-refugium, refugium-wallet and rust-miniscript have no 11-19 report, so their 11-25 findings are new by definition. Cross-repo entries that appear in these reports are covered above: X7 in refugium (= bh-md-C2, refugium#248 `f8a0d9b`), and X4 / G5 in gui (= bh-mk-C1; the mk-cli release is still owed).

