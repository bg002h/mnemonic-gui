# DESIGN — secrets over private channels, and the five unsurfaced forms

**Status:** design only, not reviewed, nothing implemented.
**Baseline:** mnemonic-gui `gui-followups` at `e3a55a4` (on top of v0.62.0 `9f569e1`).
**Measured against the release binaries:** mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0, installed with `mnemonic-toolkit/scripts/install.sh --no-gui --no-man` (which pins ms 0.19.1).
**Follow-ups this covers:** `argv-secret-via-private-channels` (Part A), `md-ms-new-subcommands-unsurfaced` (Part B).

---

## Part A — every secret over a private channel

### A1. What was measured, and how

The harness is committed at `design/measurements/secret-channels/`. Re-run it with `BIN_DIR=<dir holding the four binaries> python3 run_all.py` (and `run2.py`, `combos.sh`, `refusals.py`, `table.py`). A re-run on this tree reproduced `channels.txt` exactly, apart from temp paths.

For each secret input the GUI can fill, the harness:

1. **Baseline.** Runs the command with the secret on argv plus `--allow-argv-secret`, and records the exit code and stdout.
2. **Dependence control.** Runs it again with a *different* secret. If stdout does not change, the row cannot show that a secret arrived, and it is marked invalid. Without this control, the literal-value hazard below looks like a pass.
3. **Channel variants.** Runs the command with the secret delivered through each candidate channel instead, without `--allow-argv-secret`: `-` (stdin), `@env:VAR`, a `--X-stdin` toggle, `--X-file F`, or `--in F`. A channel counts as **OK** only if the exit code and stdout match the baseline.
4. **Refusal probe.** Runs the secret on argv without the opt-in, to see whether the CLI refuses it.

Randomised splits (`slip39 split`, `ms-shares split`, `ms split`) are compared after a round trip through the matching `combine`. Each stdin channel was measured twice, once with a trailing newline and once without, and the results were identical.

### A2. The measured table

Legend: **OK** means the channel is accepted and gives the baseline output. **WRONG** means exit 0 (or 4) with *different* output: the CLI took the spelling as a literal value. **fails closed** means a non-zero exit. **—** means clap does not know the flag.

| input | argv w/o opt-in | `-` / positional `-` | `@env:VAR` | `--X-stdin` | file (`--X-file` / `--in`) | measurement valid |
|---|---|---|---|---|---|---|
| `mnemonic addresses --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic convert --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from minikey=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic convert --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic convert --bip38-passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic restore --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic derive-child --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic derive-child --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic nostr --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --passphrase` | refused | **WRONG (exit 0/4, literal)** | **WRONG (exit 0/4, literal)** | OK | — | yes |
| `mnemonic final-word --from phrase=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic seed-xor split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic seed-xor combine --share phrase=` | NOT refused (exit 0) | OK | OK | n/a | n/a | yes |
| `mnemonic seedqr encode --from phrase=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic seedqr decode --from seedqr=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic seedqr decode --digits` | refused | OK | fails closed | — | — | yes |
| `mnemonic repair --ms1` | refused | OK | fails closed | — | — | yes |
| `mnemonic inspect --ms1` | refused | OK | fails closed | — | — | yes |
| `ms encode --phrase` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `ms encode --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms inspect <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms decode <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms verify <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms derive <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `ms verify --phrase` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms derive --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms derive --phrase` | refused | OK | fails closed | fails closed | —/fails closed | yes |
| `ms derive --passphrase` | refused | **WRONG (exit 0/4, literal)** | **WRONG (exit 0/4, literal)** | OK | —/fails closed | yes |
| `ms repair --ms1` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `mnemonic derive-child --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from wif=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | NO (base exit 4, depends=False) |
| `mnemonic verify-bundle --passphrase` | refused | OK | OK | OK | — | NO (base exit 4, depends=False) |
| `mnemonic verify-bundle --ms1` | NOT refused (exit 1) | n/a | n/a | n/a | n/a/n/a | NO (base exit 1, depends=False) |
| `mnemonic import-wallet --ms1` | NOT refused (exit 0) | fails closed | OK | — | — | yes |
| `mnemonic import-wallet --slot @0.phrase=` | refused | fails closed | OK | n/a | n/a | yes |
| `mnemonic electrum-decrypt --decrypt-password` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic xpub-search path-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic slip39 split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic slip39 split --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic slip39 combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `mnemonic slip39 combine --passphrase` | refused | **WRONG (exit 0/4, literal)** | OK | OK | — | yes |
| `mnemonic ms-shares split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic ms-shares combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `ms split --phrase` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `ms split --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms combine <shares> (first)` | refused | OK | fails closed | n/a | fails closed | yes |
| `ms hashlock --hashlock-phrase` | refused | fails closed | fails closed | OK | —/fails closed | yes |
| `ms hashlock --hex` | refused | OK | fails closed | — | —/fails closed | yes |

(The table was generated by `table.py` from `channels.json` and `channels2.json`. Only the legend above was written by hand.)

Multi-secret combinations (`combos.sh`, output in `combos.out`). Every case was checked for byte-identical stdout against the argv baseline:

| # | invocation shape | result |
|---|---|---|
| a | `bundle --template wsh-multi --slot @0.phrase=@env:V1 --slot @1.phrase=@env:V2` | same as argv |
| a2 | the same with V1 and V2 **swapped** | output changes, so a swap is detectable |
| b | `bundle --slot @0.phrase=@env:V1 --passphrase-stdin` | same |
| b2 | `bundle --slot @0.phrase=- --passphrase-stdin` | refused: "single stdin per invocation" |
| c / c2 | `ms verify --phrase - --in F` (F = file, or a `/dev/fd/3` pipe) | same |
| d | `ms derive --in F --passphrase-stdin` | same |
| e | `ms derive --phrase - --passphrase-stdin` | refused: one stdin per invocation |
| f | `seed-xor combine --share phrase=- --share phrase=@env:V1` | same |
| g | `silent-payment --secret-file F --passphrase-stdin` | same |
| g2 | `silent-payment --secret-stdin --passphrase @env:V1` | **different: literal** |
| h | `convert --from bip38=- --bip38-passphrase @env:V1` | same |
| i | `addresses --from phrase=@env:V1 --passphrase-stdin` | same |
| j | passphrase `"pw "` via `--passphrase-stdin`, with or without a trailing newline | same as argv `"pw "`; `@env:` is verbatim too |

Also measured by hand: `ms combine --in F` and `ms combine -` both accept the whole share set (one share per line) and give the argv result.

**Not measured, so the design must refuse these until they are measured** (see A8):

- `bundle`: the `ms1` / `xprv` / `wif` slot subkeys.
- `addresses --from ms1=` / `seedqr=`.
- `verify-bundle`: all rows. My fixture's verify result did not depend on the secret, and `--ms1` had no invocation that got past clap.
- `import-wallet --decrypt-password`: needs a BIE1 fixture.
- `verify-bundle --from`.
- The `word-card --from` Text flag, which takes a public mk1/md1 but can be given a secret node.

### A3. Findings that shape the design

1. **No generic rule is safe.** `--passphrase -` is taken as the literal passphrase `-` on every `mnemonic` subcommand measured, and on `ms derive`. The run exits 0 with a different wallet. `--passphrase @env:VAR` works on most subcommands but is literal on `silent-payment` and `ms derive`. A rule like "secret flag → `<flag> -`" would therefore produce a **wrong wallet with exit 0**. The channel for each (CLI, subcommand, input) must come from a committed table of measured-OK cells. It is never inferred.
2. **`@env:` does not exist in `ms`.** It also fails on several toolkit inputs: `--from minikey=`, `final-word`, `seedqr`, `--ms1` on `repair`/`inspect`/`xpub-search`, and `nostr`/`silent-payment --secret`. In every one of those cases it fails closed.
3. **`import-wallet` `--ms1` and `--slot` accept only `@env:`.** Their `-` fails closed.
4. **One stdin per invocation**, enforced by the CLIs (b2, e). `ms` has no env channel. Its second channel is `--in FILE`, and that works for a pipe fd (c2), so nothing has to touch disk on Unix.
5. **The refusal is not a safety net.** 0.104.0 does not refuse argv secrets on `import-wallet --ms1`, `seed-xor combine --share`, `slip39 combine --share`, or `ms-shares combine --share`. They run with a stderr warning. The GUI must route them privately anyway, and must not use "the CLI would refuse" as its classifier.
6. **Byte fidelity.** stdin strips exactly one trailing `\r?\n`, and `@env:` is verbatim (j). The GUI writes the value's exact bytes with **no terminator**. Measured identical with and without one, but "no terminator" never adds or strips anything.

### A4. The mechanism

**A plan instead of an argv.** `assemble_argv_with_secret_mask` stays the Copy/Preview source of *shape*. A new `form::channels::plan(schema, sub, state) -> Result<RunPlan, ChannelRefusal>` produces:

```text
RunPlan { argv: Vec<String>,            // no secret bytes, ever
          mask: Vec<bool>,              // unchanged meaning (now marks channel tokens)
          stdin: Option<Zeroizing<Vec<u8>>>,
          env: Vec<(String, Zeroizing<String>)>,   // MNEMONIC_GUI_S<n>
          fds: Vec<Zeroizing<Vec<u8>>>,  // Unix: inherited pipe, argv says /dev/fd/<n>
          bindings: Vec<Binding> }       // (source, channel, argv index) — for Preview and tests
```

- **The channel table** is a committed data file, e.g. `src/form/channel_table.rs`. It maps `(cli, subcommand, input) → ordered [Channel]`, where `Channel ∈ {DashValue, EnvRef, StdinToggle(flag), FileFlag(flag), InFile, PosDash}` and each entry is a measured-OK cell from A2. **A missing entry is a refusal, never a fallback to argv.**
- **Assignment** processes secret sources in argv order:
  1. A source whose table entry has `EnvRef` gets a unique `MNEMONIC_GUI_S<n>` (n = source index).
  2. Stdin goes to the first source without `EnvRef` that has a stdin-class channel (`DashValue`, `PosDash`, or `StdinToggle`).
  3. Any other source without `EnvRef` takes `InFile` / `FileFlag` through a pipe fd (Unix) or a temp file (Windows, A6).
  4. If none is left, the plan refuses.

  Env goes first because it scales to N sources (multisig slots, share sets) and leaves stdin for the source that has nothing else. In a single-secret command whose entry lacks `EnvRef`, that source gets stdin.
- **Env hygiene.** Before spawning, the runner removes every inherited `MNEMONIC_GUI_S*` variable, so a stale variable cannot stand in for a missing binding. It then sets exactly the plan's variables. Names are only `[A-Z0-9_]`, and the `@env:` token carries the exact name.
- **The GUI-managed `--allow-argv-secret` goes away.** `admit_argv_secret_for_run` is deleted once every mirrored secret input has a table entry. The flag stays mirrored (schema parity) and hidden. Until then, a secret source with no entry is a **refusal** (A8), not an opt-in.
- **The `*-stdin` toggles** (`--passphrase-stdin` and the others) stay rendered disabled, as they are today. The planner emits them itself. A stale persisted toggle still never reaches argv (`boolean-stdin-secret-toggles-never-emit` still holds for form state).
- **Tree mode** (`build-descriptor --spec -`) already owns stdin. None of its secret sources are stdin-only today; the planner treats `--spec -` as a pre-bound stdin, so a future conflict refuses.

### A5. Multi-secret resolutions (from the table and combos)

| command shape | plan |
|---|---|
| `bundle` / `import-wallet` / `verify-bundle*`, N `--slot` secrets + `--passphrase` | every slot via `@env:` (distinct vars); passphrase via `--passphrase-stdin` |
| `convert --from <node>=` + `--passphrase` / `--bip38-passphrase` | node via `@env:` (stdin for `minikey`); passphrase via `-stdin` toggle, or `@env:` when the node took stdin (h) |
| `addresses` / `restore` / `derive-child`, `--from` + `--passphrase` | `--from` via `@env:`; passphrase via `--passphrase-stdin` (i) |
| `silent-payment --secret` + `--passphrase` | secret via `--secret-file` (fd/temp file); passphrase via `--passphrase-stdin` (g). **Never `@env:` for this passphrase** (g2) |
| `xpub-search`, `--phrase` + `--passphrase` | phrase `@env:`; passphrase `--passphrase-stdin` |
| `xpub-search`, `--ms1` + `--passphrase` | ms1 `--ms1-stdin` (its only channel); passphrase `@env:` (OK on all three modes) |
| `seed-xor combine`, N shares | all `@env:` (f shows one stdin + one env also works) |
| `slip39 combine`, N shares + passphrase | shares `@env:`; passphrase `--passphrase-stdin` |
| `ms-shares combine`, N shares | all `@env:` |
| `import-wallet`, N `--ms1` | all `@env:` (stdin fails closed) |
| `ms verify --phrase` + ms1 | phrase `--phrase -`; ms1 `--in /dev/fd/N` (c2) |
| `ms derive` ms1 + `--passphrase` | ms1 `--in /dev/fd/N`; passphrase `--passphrase-stdin` (d) |
| `ms derive --phrase` or `--hex` + `--passphrase` | **refuse.** ms has no second channel for a phrase or hex (`--in` reads only an ms1, e). The refusal gives the CLI's own recipe: encode to ms1 first, then derive from the card |
| `ms combine`, N shares | all on stdin via `-`, one per line (measured) |

`*` verify-bundle rows are unmeasured (A2), so the plan refuses them until they are measured.

### A6. Pipe fd first; a temp file only where there is no fd

- **Unix:** a second secret for `--in` / `--secret-file` goes through an inherited pipe. The GUI creates a pipe and writes the bytes; the pipe buffer is enough for these sizes, and a writer thread covers the general case. The child inherits the read end at a fixed fd via `CommandExt::pre_exec` + `dup2`, or the `command-fds` crate, and argv says `/dev/fd/<n>`. **Nothing reaches disk**, so a crash leaves nothing behind. Measured (c2).
- **Windows** has no `/dev/fd`. Use a temp file:
  - Create it with `tempfile::Builder` (already a dependency) in `%LOCALAPPDATA%\mnemonic-gui\run\`, a directory only the GUI uses. The file gets a random name and `O_EXCL` semantics, and inherits the user-profile ACL (owner-only in practice). Unix would use mode 0600 in a 0700 directory, but Unix does not need this path.
  - Write the bytes, close, pass the path.
  - A `NamedTempFile` guard removes it when the child exits. It is removed on every return path, including spawn failure.
  - **Crash or cancel:** the runner is synchronous (`spawn_and_capture` blocks the frame; there is no cancel button, by the tutorial's SAME-FRAME contract). "Cancel" therefore means the GUI is killed mid-run, and Drop does not run. At startup the GUI deletes every file in its `run\` directory, which only it writes.
  - Temp-file creation failure is a refusal (A8).
  - Only three shapes need this path: `ms verify` phrase+ms1, `ms derive` ms1+passphrase, and `silent-payment` secret+passphrase.
- The temp directory is never a world-shared `/tmp`.

### A7. What the user sees

- **Preview (masked) and the confirm modal** show the planned argv, e.g. `restore --from ms1=- --template bip84`, with a "Secrets go privately" list under it, one line per binding: ``--from ms1= ← stdin``, ``--passphrase ← environment variable MNEMONIC_GUI_S1``, ``ms1 ← pipe (/dev/fd/3)``. Values are never shown, and there is no `--allow-argv-secret`. The modal's first sentence stops saying "passes secret-bearing arguments to", because nothing is on argv any more. Whether the modal should appear at all for a fully private run is **open question Q1**: it is still a deliberate pause before a secret is used.
- **Copy command** produces text for a shell. It uses the same channel spellings, so it never contains a secret, and a pasted `-` would wait on the terminal. That is the "stdin looks like a hang" trap, so Copy prefixes one comment line per binding:
  - POSIX: `# stdin: the ms1 card — paste it, then Ctrl-D`, and `# export MNEMONIC_GUI_S1 first: read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1` (fish: `read -sx MNEMONIC_GUI_S1`).
  - Windows: `REM` lines.
  - A pipe fd becomes `--in <FILE>` with `# FILE: a file holding the ms1 card`, because a copied `/dev/fd/3` means nothing in a shell.
  - The tree-mode `printf` pipeline stays as it is (its stdin is public spec JSON).
- **Result pane:** unchanged. Output that carries a secret (e.g. `ms hashlock --json`) is out of scope here.

### A8. Refuse, never fall back

The plan returns `ChannelRefusal` and the Run button explains it. It never runs with the secret on argv. The cases:

1. A secret source has no table entry. This covers every "not measured" row in A2, and any input a future pin bump adds.
2. Two sources need stdin and the second has no other channel (the `ms derive` phrase/hex + passphrase case).
3. The temp file or pipe cannot be created.
4. A table channel is `EnvRef` but the value contains a NUL byte, which an environment variable cannot carry. Stdin can, so such a source is re-planned onto stdin if that is free; otherwise the plan refuses.
5. Tree-mode stdin is taken and a secret source is stdin-only.

The Copy command is still produced in every refusal case: it is correct shell text.

### A9. Testing — every secret reaches exactly the flag the user filled

- **T1 — plan property (pure, no binaries).** For every mirrored subcommand and every combination of its secret sources, filled with *distinct* sentinel values:
  - no argv token contains any sentinel;
  - every binding's argv index points at the token of *its* source's flag or slot;
  - the value delivered on that binding's channel equals *that* source's sentinel;
  - no two bindings share stdin, and env names are unique;
  - a source with no table entry yields `ChannelRefusal`.
  This covers the swap class structurally.
- **T2 — real-binary equivalence** (gated on `MNEMONIC_BIN`/`MS_BIN` etc. like `ui_harness_i4`). The A1 harness is promoted to `tests/channel_table_realcli.rs`. For every table entry, run `plan()`'s invocation through `runner` and the argv+opt-in baseline; assert identical exit and stdout, **and** that a different secret changes stdout (dependence). Without the dependence half, the WRONG cells in A2 would pass.
- **T3 — swap test** for every multi-secret shape in A5. Exchange the two sources' values and assert that stdout changes, or that the CLI refuses. Combined with T2, this shows each secret went to its own flag (the a/a2 pattern).
- **T4 — table ↔ measurement gate.** `channel_table.rs` must equal the harness's generated OK-cell set for the pinned binaries. A pin bump re-runs the harness. A cell that stops being OK fails the gate, and a new secret input with no cell fails T1's refusal leg.
- **T5 — mutation checks** each of these must turn a test red:
  - swap two env bindings (T3);
  - write the stdin bytes to the wrong source (T1/T3);
  - replace a `StdinToggle` with `DashValue` for a passphrase (T2's dependence leg, via the literal `-`);
  - drop the env scrub (a test that pre-sets `MNEMONIC_GUI_S0` in the parent env);
  - delete the temp file before spawn (Windows cell);
  - restore `--allow-argv-secret` admission (a test asserting it never appears in a planned argv).
- **T6 — UI.** Preview, the confirm modal and Copy contain no sentinel, and do contain each binding line (kittest + string asserts). The tutorial corpus will change (the J1 bundle modal shows the flag today), so it is regenerated as a deliberate, explained re-pin.

### A10. Out of scope, and follow-ups to file

- **Toolkit** (Rust-primary; file in `mnemonic-toolkit/design/FOLLOWUPS.md`):
  - `--passphrase -` is silently the literal `-` on the toolkit subcommands measured in A2, and `--passphrase @env:` is literal on `silent-payment`. Either makes a wrong wallet with exit 0. The CLI could refuse `-` and `@env:` on flags that do not implement them.
  - Argv secrets are not refused on `import-wallet --ms1`, `seed-xor combine --share`, `slip39 combine --share`, `ms-shares combine --share`.
- **ms:** `ms derive --passphrase -` / `@env:` are literal in the same way.

---

## Part B — the five unsurfaced forms

Source: each binary's `gui-schema` (md/ms schema v1 has no `secret` or `repeating` fields, so those columns come from `--help` and from running the binary). "GUI kind" is the existing `FlagKind`. All five fit the existing form machinery: **no new widget kind is needed.** New work is conditional-rule functions and one secret positional.

### B1. `md compose`, no secrets

| flag | GUI kind | secret | rule (measured) |
|---|---|---|---|
| `--wrapper` | Dropdown `tr, wsh, sh-wsh, sh` (JSON says text; the CLI refuses any other value, exit 1) | no | **required** (clap) |
| `--path` | Text, **repeating** (order is meaningful) | no | one of `--path` / `--preset` (clap: "cannot be used with") |
| `--preset` | Text (`<name>[,<k>of<n>]*[,<param>=<value>]*`) | no | one of, as above |
| `--unspendable` | Dropdown with a `(none)` sentinel + `nums, liana` | no | only with `--wrapper tr` (the CLI refuses `wsh`, exit 1); grey out otherwise |
| `--experimental`, `--json`, `--md-only` | Boolean | no | — |

Fits. It needs a `conditional` fn for path/preset exclusivity and for greying `--unspendable` unless `tr`. The spend-path mini-grammar stays free text in v1; a structured path builder is a later feature, not required.

### B2. `md shape-key`, no secrets

| input | GUI kind | secret | rule |
|---|---|---|---|
| `<PHRASES>…` | positional, repeating (md1 strings of one card) | no | one of `PHRASES` / `--descriptor` (clap refuses both) |
| `--descriptor` | Text (multipath BIP-380 descriptor) | no* | one of, as above |

Fits (same shape as `md address`'s positional + flag exclusivity).

### B3. `md descriptor`, no secrets

| flag | GUI kind | secret | rule (measured) |
|---|---|---|---|
| `<PHRASES>…` | positional, repeating | no | input mode A. `--key`/`--fingerprint` cannot be used with it (clap) |
| `--template` | Text | no | input mode B: **requires ≥1 `--key`** ("--key @i=<XPUB> required when --template is supplied", exit 2) |
| `--key` | Text, repeating (`@i=XPUB` / `@i=[fp/path]XPUB`) | no* | only with `--template` |
| `--fingerprint` | Text, repeating (`@i=HEX`) | no | only with `--template` |
| `--path` | Text | no | — |
| `--from-mk1` | Text, repeating, multi-value (already modelled for `md address`) | no | input mode C, with keyless md1 phrases |
| `--from-mk1-file` | Path | no | combines with `--from-mk1` |
| `--seat` | Text, repeating (`@i=<id>[#K]`) | no | with mode C |
| `--network` | Dropdown (default mainnet) | no | — |
| `--chain` | Number 0..1 | no | cannot be used with `--change` (clap) |
| `--change` | Boolean | no | cannot be used with `--chain` |
| `--emit` | Dropdown `(none)`, `md1` | no | `md1` only with mode C (CLI refuses otherwise, exit 2) |
| `--out` | Path (save) | no | meaningful with `--emit md1` |
| `--group-size` | Number (JSON says number, default 5) | no | with `--emit md1` |
| `--separator` | Dropdown `space` | no | with `--emit md1` |
| `--json`, `--experimental` | Boolean | no | — |
| `--verify-against` | Text (md1 or FILE) | no | any mode |

Fits. This is the largest conditional function (three input modes). The `--` before positionals from v0.62.0 already protects `--from-mk1`'s multi-value from swallowing a phrase.

### B4. `md decompose`, no secrets

| input | GUI kind | secret | rule (measured) |
|---|---|---|---|
| `<DESCRIPTORS>` | positional (exactly one; `-` = stdin) | no* | one of positional / `--in` (clap: "cannot be used with") |
| `--in` | Path | no | one of, as above |
| `--emit` | Dropdown `all, template, keys, fingerprints, descriptor, commands` (default `all`) | no | — |
| `--network` | Dropdown (default mainnet) | no | — |

Fits.

`*` **One divergence across B2–B4.** A user can paste a *private* descriptor (`xprv…`) into these public fields. md refuses it ("public keys must be 64, 66 or 130 characters", exit 1, measured on `decompose`), but by then the GUI has shown it in cleartext and persisted it. The existing `is_xprv_like` classifier (tree form) should mask and redact these Text/positional values the way Phase 1a does for `restore --from`. That is cheap and closes a plausible slip.

### B5. `ms hashlock`, **secret**

| input | GUI kind | secret | rule (measured) |
|---|---|---|---|
| `--hashlock-phrase` | Text → **SecretLineEdit** (`secret: true` in the hand mirror; ms's v1 schema carries no secret bit) | **yes** | exactly one source among phrase / phrase-stdin / `--hex` / ms1 / `--in` / `--random` ("exactly one source", exit 64) |
| `--hashlock-phrase-stdin` | Boolean, GUI-managed (the channel for the phrase; A2 row: the only working channel) | — | planner-emitted |
| `--hex` | Text → SecretLineEdit (a 32-byte preimage) | **yes** | one of the sources; channel `--hex -` |
| `<ms1>` | positional, **secret** (a preimage plate) | **yes** | one of the sources; channel `-` or `--in` |
| `--in` | Path (JSON says text) | no (a path) | one of the sources |
| `--random` | Boolean | no | **requires `--out`** ("--random needs --out FILE", exit 64) |
| `--kind` | Dropdown `(not specified), sha256, hash256, ripemd160, hash160` | no | lowercase only (the CLI rejects `SHA256`, exit 64). **No default materialised:** a silent `sha256` is the F-553 hazard for a ripemd160 wallet. The sentinel omits the flag, exactly as a shell user would |
| `--method` | Dropdown `hardened, sha256` (default hardened, suppressed at default) | no | phrase sources only |
| `--out` | Path (save) | no | the CLI writes it owner-only |
| `--json` | Boolean | no | **stdout then carries the secret.** Show a notice beside the box |
| `--emit-record` | Boolean | no | phrase source only ("--emit-record needs a phrase", exit 64) |
| `--no-engraving-card`, `--phrase-looks-like-digest-ok` | Boolean | no | — |
| `--group-size` | Number (default 5) | no | — |
| `--separator` | Dropdown `space, hyphen, comma` | no | — |
| `--allow-argv-secret` | GUI-managed, never rendered | — | — |

Fits. Secret Text flags and secret positionals both exist today. Two requirements:

- **The phrase is byte-verbatim** ("one trailing newline stripped"). Measured: `"  pad  "` through `--hashlock-phrase-stdin` gives the same digest as on argv, and a different digest from `"pad"`. The widget must not trim, and a test must pin that case against the real `ms`.
- **Its only private channel is `--hashlock-phrase-stdin`** (A2), so it is Part A's first `StdinToggle`-only case. Build this form after Part A, or ship it with `--allow-argv-secret` still admitted for just this form, which is what F-679 does everywhere today. That ordering is **open question Q2**.

### B6. Shared work for all five

- **Schema mirror.** Mirroring them brings them under the flag-name gate automatically (the gate walks only mirrored subcommands). The mirror sets `secret: true` on `--hashlock-phrase` and `--hex`, and marks the hashlock `ms1` positional secret, by hand, because ms's v1 `gui-schema` has no secret field.
- **Snapshots.** Five new form PNGs, and the gallery census goes from 61 to 66.
- **Manual.** The GUI manual gains five form pages. That is a toolkit-repo change (paired PR).

---

## Open questions for review

- **Q1.** Keep the run-confirm modal for runs whose secrets are all private (a deliberate pause), or drop it (its argv rationale is gone)?
- **Q2.** Build `ms hashlock` before or after Part A? After is cleaner. Before means one more form on the F-679 opt-in.
- **Q3.** Windows temp files (A6) versus refusing the three fd-only shapes on Windows.
- **Q4.** The unmeasured rows (A2): measure them in the implementation cycle (a fixture per row), or ship with those inputs refusing?
