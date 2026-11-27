# Final review (fable): address-of-seed secret-channel measurement (mnemonic-gui `0e3d5dc..38f67fb`)

Reviewer: independent adversarial review (Claude Fable 5.2), 2026-11-27.
Scope: `git diff 0e3d5dc..38f67fb` (three commits: `ed26078`, `ae16266`, `38f67fb`) in the worktree `/home/user/wt-gui-aos`.
Binaries: `/home/user/bins/pr230/` (mnemonic = local build of toolkit PR #230; md/ms/mk = pinned releases, md the musl asset). For old-versus-new comparisons: `/home/user/bins/rel/mnemonic` (release 0.107.1, sha256 `6b13825b…`, the value the base recorded).
Prior review: `design/agent-reports/address-of-seed-gui-measure-review-r1.md` (0C/1I/5M/3N). I do not re-raise what it settled.

**Verdict: 0 Critical, 0 Important, 4 Minor, 6 Nit.** r1's fixes held. Nothing in the diff lets the GUI deliver a secret to the wrong input or put one on the private path's argv.

## What I ran

- `BIN_DIR=/home/user/bins/pr230 python3 regen_check.py --no-enumerate --plans`: exactly 2 red, both by construction after r1's I1 fold (see N5). Every other derived cache, `test_plan.py` and `run_plans.py` are green. The run left the tree clean.
- `cargo test --locked --no-fail-fast` (whole suite, once, the four `*_BIN` on pr230): 838 passed, 1 failed, 6 ignored. The one failure is `t2b_failed_write_does_not_poison_the_cache`. This matches the FOLLOWUP's "839 tests, 1 failure".
- A temporary, untracked probe test (deleted afterwards; `git status` is clean). It built 21 form states for this verb, planned each through `mnemonic_gui::form::channels::plan` for `linux` and `macos`, and executed the resulting argv/env/stdin against the pr230 binary. A second probe printed `copy_commands` for five states.
- By hand against pr230: the verb's happy paths, error paths and stdin-conflict combinations; and against the 0.107.1 release where a comparison was needed.
- A script comparing every committed JSON cache old versus new, outside the new keys.

## Critical

None.

## Important

None.

## Minor

### M1. The toolkit echoes the whole value of a mis-typed `--ms1` / `--ms1-stdin`, including over the private stdin channel

Not introduced by this diff, but newly reachable on this verb because the diff turns its Run from fail-closed to runnable. Nothing records it.

Reproduction (GUI plan for a form with the abandon phrase pasted into the `--ms1` field, Linux):

```
argv: xpub-search address-of-seed --target-address … --ms1-stdin      stdin: "abandon … about"
exit 2
error: --ms1-stdin expects a value with HRP 'ms', got 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' (HRP is not BCH-protected; re-type the prefix)
```

- The interim path (`--allow-argv-secret --ms1 <value>`) prints the same line.
- The GUI shows stderr verbatim; I found no output scrubbing in `src/`.
- The echoed text is everything before the last `1` in the value, lower-cased, or the whole value when it has no `1`. A BIP-39 phrase has none, so it is printed in full.
- `--from ms1=<junk>` does not echo ("ms1 string length N not in v0.1 set"). The other error paths I tried do not echo either: bad word ("at position 11"), bad checksum, entropy ("the value is not repeated"), SeedQR, the clap exclusivity errors.
- The same echo exists on `path-of-xpub --ms1-stdin` in the 0.107.1 release, so the three sibling verbs already carry it.

Failure scenario: a user pastes a seed phrase into the ms1 field and presses Run. The phrase appears in clear in the output pane, although the GUI delivered it privately.

Fix: file a toolkit FOLLOWUP with a GUI companion (withhold the value, as the checksum error already does: "input withheld"). Optionally have the GUI refuse a `--ms1` value that does not start with `ms1` before sending.

### M2. A third measured row moved under #227 and the FOLLOWUP does not say so

`FOLLOWUPS.md:15` says "Two rows moved because the toolkit stack under #230 (#227) changed behaviour" (derive-child fixture, bundle wif). The caches show a third: both `bundle wsh-multi` shapes now exit 4 instead of 0 (`t3.md:8-9`, `DESIGN_secret_channels_and_new_forms.md:788-789`, `plans.json` every leg).

Cause, checked on both binaries with the same argv: at #227 `bundle --template wsh-multi` without `--multisig-path-family` prints

```
warning: --multisig-path-family was not given, so the toolkit chose BIP-87 (m/87'/0'/0') for @0,@1 and the cards record it. …
warning: to state this choice explicitly, run:
  mnemonic bundle … --slot @0.phrase=@env:A --slot @1.phrase=@env:B --multisig-path-family bip87
```

and exits 4. The cards on stdout are unchanged (20 lines either way), so the channel measurement is sound.

What it means for the GUI at the pin bump:
- A multisig bundle Run without the path family shows the amber VERIFY-ME badge (`src/app_window.rs:1318`) where it showed `exit: 0`.
- Its stderr carries a second "run this" line with the GUI's `@env:MNEMONIC_GUI_S<n>` spelling. This is r1's N3 class, now on `bundle` as well.

Fix: name the third row in the "Prepared" bullet, and add the bundle badge and printed line to the "Still owed at the release tag" list and the manual note.

### M3. A public `--descriptor-file /dev/stdin` takes the planner's stdin payload; the toolkit's one-stdin guard does not see it

`src/schema/mnemonic.rs:3719` (`--descriptor-file`, a public Path field); the planner (`src/form/channels/mod.rs`, `plan_private`) counts only secret sources as stdin readers.

- `--descriptor-from md1=-` together with a stdin-bound secret is refused by the CLI ("… all read stdin; only one input can come from stdin per invocation", exit 2). With the secret on env it reads the GUI's empty stdin ("md1 chunk set empty"). Both are safe.
- `--descriptor-file /dev/stdin` is not caught. The descriptor reader consumes the secret's bytes and the secret's own reader then sees an empty stdin. By hand, with stdin holding a valid descriptor:
  `… --from phrase=@env:S --passphrase-stdin --descriptor-file /dev/stdin` exits 0 with fingerprint `73c5da0a`, the wallet of an **empty** passphrase.

Through the GUI the stdin bytes are always the secret itself, which is not a descriptor. Every case I ran ended in an error with no echo ("--descriptor has neither @N placeholders nor public keys", "BIP-39 word count 0", "ms1 string length 0"). So I found no wrong-wallet result and no leak; this is a guard gap, not a reachable failure.

Fix: toolkit FOLLOWUP (count `--descriptor-file` paths that resolve to fd 0 as a stdin reader). Cheap GUI defence: refuse `/dev/stdin`, `/dev/fd/0`, `/proc/self/fd/0` in a path field when the plan binds stdin. The other verbs that have `--descriptor-file` are in the same class.

### M4. The flipped tests plan a form the CLI can never run, and no test executes a form-built plan for this verb

`tests/xpub_search_address_of_seed.rs:66` (`filled_form`) and `:191` (`aos_run_and_copy_plan_private_channels`).

`filled_form` carries `--phrase`, `--ms1` and four `--from` rows at once. The planner returns a plan on every OS:

```
linux: --phrase @env:MNEMONIC_GUI_S0 --ms1-stdin --from phrase=@env:…S2 … --passphrase @env:…
```

The CLI refuses it at clap (exit 64, "the argument '--phrase <PHRASE>' cannot be used with: …"). The message names flags only, on both paths, so nothing leaks. But the test's claim "a secret-bearing Run plans on every OS" is shown only on a combination that cannot run, and nothing in the suite runs a plan assembled from this verb's schema against the binary. T3′ runs `shapes.py` shapes through `plan_sources`, not the schema assembler.

My probe did run them, and they work:
- phrase+passphrase, ms1+passphrase, one `--from` + passphrase, two and three `--from` rows: exit 0 or the expected 4/2.
- `--allow-argv-secret` sits right after the two subcommand tokens and is accepted.
- `--passphrase=-dashpw`, `--passphrase=--json`, `--phrase=-…` and `--ms1=-…` parse as values on the interim path.

Fix: add one runner test per intake (phrase, ms1, `--from`×2) built from `FormState`, asserting the fingerprint line. Either split `filled_form` into runnable forms or say in its comment that it is deliberately unrunnable.

## Nit

- **N1. Stale FOLLOWUP wording.**
  - `FOLLOWUPS.md:13` ("Done on branch") still lists "Run/Copy fail closed" as what the test file covers.
  - `FOLLOWUPS.md:15` still says the 2-seed shape's "second `--from` takes `@env:`". This is r1's N2, fixed in `shapes.py` only; both rows bind `@env:`.
- **N2. Copy text names a version that lacks the verb.** The recipe for this verb begins "# Needs mnemonic 0.107.1 or newer" (`src/form/channels/copy.rs:47`, from the pinned version). 0.107.1 has no such verb. A real 0.107.1 answers exit 64 before reading anything, so it is fail-safe, and it corrects itself at the bump.
- **N3. Orphan doc line.** `tests/xpub_search_address_of_seed.rs:164`: "/// Every secret the given state carries, as typed." is a leftover first line now glued to `ENV_VALUE`'s doc comment.
- **N4. `aos_mask` / `AOS_CHECK` cover the text report only.** Under `--json` the same spelling sits in `wallets[].bundle_command` and `redaction_note`, not on a line starting `mnemonic bundle `. No measured case uses `--json`, so nothing is wrong today. The `--json` wire-shape cells owed at the tag (`tests/wire_shape_snapshot.rs`) must not pin those two fields across channels.
- **N5. `regen_check` is red on this box by construction after the I1 fold.** Two reds: md identity (`ef04e916…` here vs the recorded glibc `52c462aa…`) and "measured_with.json differs from a regeneration". The FOLLOWUP explains the md half. Two things it does not say:
  - `measured_with.json` now records a pair that never ran together (local mnemonic `5e744fef…` + glibc md).
  - `mnemonic --version` on the PR build prints only `0.107.1`, so nobody can tie `5e744fef…` to head `eb8658b9`. I confirmed `eb8658b9` is the tip of `origin/claude/address-of-seed` and that `f89d7167` exists.
- **N6. Mutually exclusive intakes are not modelled in the form.** The GUI lets `--phrase`, `--ms1` and `--from` be filled together and hands every secret to a child that then exits 64. Harmless (see M4); a conditional or a pre-run message would be kinder.

## The four lenses

**(a) Planner semantics: no misdelivery found.**
- At most one stdin reader in every plan produced.
- `--from` rows always bind `@env:` (both their channels are byte-exact, so they never need stdin). The passphrase takes `--passphrase-stdin` when no `--phrase`/`--ms1` is present; with a phrase or ms1 on stdin it takes `@env:`.
- ms1 plus a passphrase with edge whitespace refuses `two-stdin` on Linux (Run and Copy), which is correct. On macOS the same form runs on the interim path.
- Ticked `--phrase-stdin` / `--ms1-stdin` / `--passphrase-stdin` booleans never emit, with or without a typed value.
- `@env:USERV` in a secret field is resolved and delivered as bytes. `-` refuses `C1-dash`. An empty passphrase emits nothing.
- The only way I found for the CLI to read differently than planned is M3.

**(b) Interim path: the argv guard accepts it exactly as for the siblings; no echo on the paths that work.**
- Without `--allow-argv-secret` each source is refused at exit 2 with "Nothing about it is printed here" (`refusals.json`, re-checked by hand).
- With it, stdout and stderr hold the fingerprint, the path and a `bundle` line whose secret positions read `-` / `@env:MNEMONIC_PASSPHRASE`, never the value. `--json` is the same.
- The one echo is M1, and it is not specific to the interim path.

**(c) Previously measured verbs: no GUI behaviour changes beyond what r1 settled.**
- `channel_table.json`: the seven new keys, minus `bundle --slot @N.wif=`. Nothing else.
- `reinterpret.json`: the same key movements only.
- `bytes.json`: 84 → 90 rows, only the wif row removed, no surviving row changed.
- `channels.json`: only `searched=140 → 161` in two `base_err` strings and temp-file names.
- `channels3.json`: only the wif row.
- `plans.json`: only the two bundle wsh-multi exits (M2).
- `plans_pure.json`: additions only.
- The derive-child row's verdicts are unchanged with the master-xprv fixture.

**(d) FOLLOWUPS claims: true against the files, except M2 and N1.**
- Channel verdicts per source: match `channel_table.json`.
- 1386 and 4095 legs: +126 = 3 shapes × 2 sources × 7 spellings × 3 OSes; +378 = 6 × 21 × 3; T1 passes.
- 11 unmeasured and 32 "CLI rejects" rows: counted in `missing_sources.md`.
- Test names, the 161 golden, the suite count and the head commit: all check out.
- The two md asset hashes: I could confirm only the musl one (the binary on this box); the glibc one rests on r1's download.

## r1's fixes

All held.
- I1: `measured_with.json` md back to `52c462aa…`, FOLLOWUP corrected.
- M1: `ENV_VALUE` is distinctive and in `SECRETS`.
- M2: the Windows Copy text is checked.
- M3: comments corrected.
- M4, M5: the old bullet is marked superseded and the red-until-tag list is complete.
- N3: `assert_eq!` on the mask length.
- N2 was folded only in `shapes.py` (N1 above).
