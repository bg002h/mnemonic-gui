# DESIGN — secrets over private channels, and the five unsurfaced forms

**Status:** design only, nothing implemented.
- **Fold 5** answers R4 (`mnemonic-engrave/design/agent-reports/gui-design-r4.md`, 0C/2I/3M/3N). **The GUI no longer depends on CLI behaviour outside the probe set** (§A0.1): two broad refusals are decisions that make delivery safe whatever a CLI does with channel-like or newline-ending values. The map is `gui-design-fold5.md`.
- **Fold 4** answered R3 (`mnemonic-engrave/design/agent-reports/gui-design-r3.md`, 0C/3I/4M/1N) by **changing the shape** (§A0): every behavioural fact is derived by measurement and regenerated in CI, never hand-kept, never its own oracle. The map is `gui-design-fold4.md`.
- **Fold 3** answered R2 (`mnemonic-engrave/design/agent-reports/gui-design-r2.md`, 1C/2I/2M/2N) and the F-687 CLI implementation now under review (ms `e534917`, toolkit `9846a784`); the map is `gui-design-fold3.md`.
- **Fold 2** answered R1 (`mnemonic-engrave/design/agent-reports/gui-design-r1.md`, 0C/2I/7M/4N); the map is `gui-design-fold2.md`.
- **Fold 1** answered R0 and the operator's F-687 ruling; its map is `gui-design-fold1.md`.

**Baseline:** mnemonic-gui `gui-followups`, fold 4 at `462c648`.
**Measured against the release binaries:** mnemonic 0.105.1, md 0.20.3, ms 0.20.1, mk 0.13.0, installed with `mnemonic-toolkit/scripts/install.sh --no-gui --no-man` (F-694 re-pin, 2026-09-25; the generated blocks below are from these). Fold 1–5 measured mnemonic 0.104.0 and ms 0.19.1, which **predate** the F-687 CLI change (§A3b); prose that quotes those releases says so. The F-687 **master** builds (toolkit `9da32e2f`, ms `d1ab447`; sha256 `88cb9fbf…`/`1d13f0f8…`) report **the same version strings**, so binaries are identified by **pinned tag + sha256** (§A0).
**Follow-ups this covers:** `argv-secret-via-private-channels` (Part A), `md-ms-new-subcommands-unsurfaced` (Part B).

---

## Part A — every secret over a private channel

### A0. The shape: decisions are kept, behaviour is derived (R3)

Four review rounds each found the same defect one level deeper:
- R1 NI1: the newline rule;
- R2 NC1: argv re-reads;
- R3 NI5: a stale re-read row;
- R3 NI7: a per-flag rule that the design had made global.

Each time, a policy row described CLI behaviour, a human kept it in sync, and the tests took their expected answer **from that same row**. A stale row therefore passed every gate. Fold 4 removes the class rather than the instance:

1. **Behaviour is data derived by measurement, never hand-kept.** Every fact about what a CLI does is produced by the measurement pipeline against the pinned binaries:
   - `channel_table.json`: channels, terminators, the **per-input** `@env:` value rule, and whether `--flag=VALUE` is exact;
   - `reinterpret.json`: which argv values each CLI re-reads;
   - `measured_with.json`: pinned tag, version and **sha256** of each binary.

   The committed JSON is a **cache**. `channel_policy.json` now holds **decisions only**: per-OS switches, the reserved prefix, the name rule, bounds, and the CI test targets.
2. **CI regenerates the cache and diffs it** (`regen_check.py`, T4). It installs the binaries for the tags in `pinned-upstream.toml` and checks their **sha256** against `measured_with.json`. It re-derives every behavioural file in a scratch copy of the repo layout, and any difference is red. A same-version rebuild with different behaviour (exactly the F-687 builds) fails on identity and on the diff.
3. **Expected answers come from independent oracles, never from the data.** The real-runner legs (T3′) compare the GUI's run with one of two things:
   - the CLI's **own `@env:VAR`** run, which by definition is what `@env:VAR` means;
   - where the input has no working CLI `@env:`, **known bytes**: argv-exact, or, for bytes argv would re-read, a non-lenient `--X-stdin` toggle.

   The NC1 leg **executes** every interim plan it admits and requires "never `$OTHER`'s wallet". It no longer derives "should refuse" from the row.

**R3's NI5 scenario, re-run against the F-687 master builds** (`demo_ni5.sh`):
- **(A)** the committed cache against the F-687 binaries: red on identity and on the diff;
- **(C)** a complete, correct re-derivation: **GREEN**, so the bump is data-only;
- **(B)** everything re-derived, but ms's re-read row left at `[-]`: red on the diff.

Since fold 5, (A) and (B) **cannot produce `$OTHER`'s wallet at runtime** (in fold 4 they did). The planner's safety no longer reads the re-read data (§A0.1).

```
F-687 binaries: mnemonic 0.104.0, ms 0.19.1

=== (A) committed cache vs F-687 binaries: BIN_DIR=F687 regen_check.py --plans
RED  identity: mnemonic binary sha256 88cb9fbf420f7ded… (mnemonic 0.104.0) != measured c894bee53193cfe6… (same version string: True)
RED  identity: ms binary sha256 1d13f0f84cdf1a05… (ms 0.19.1) != measured a6d748119b0ee25a… (same version string: True)
RED  derived: channel_table.json differs from a regeneration. 13 inputs differ, e.g. ['mnemonic addresses --passphrase', 'mnemonic bundle --passphrase', 'mnemonic convert --passphrase', 'mnemonic derive-child --passphrase']
RED  derived: reinterpret.json differs from a regeneration. ms: committed ['-'] vs measured ['-', '@env:{V}']
RED  derived: measured_with.json differs from a regeneration. 
RED  derived: bytes.json differs from a regeneration. 
regen_check: 6 red
exit 1

=== (B) re-derived on F-687, ms row left at [-]: regen_check.py --data-from STALE --plans

=== (C) the bump done right: everything re-derived on F-687: regen_check.py --data-from FRESH --plans
regen_check: GREEN
exit 0

=== (B) continued: the stale variant
  F-687 measured ms re-reads: ['-', '@env:{V}']; the stale bump leaves it at ['-']
RED  derived: reinterpret.json differs from a regeneration. ms: committed ['-'] vs measured ['-', '@env:{V}']
regen_check: 1 red
exit 1
```

### A0.1. What the GUI no longer depends on (R4)

**The problem.** R4 showed that the derived shape holds against every edit, skipped step and partial bump. It also showed the class no finite probe set can close: CLI behaviour **outside the probes**. Two wrappers around the F-687 master builds re-derive **byte-identically** and pass every gate in fold 4, yet give a different wallet at exit 0:
- **W2** re-reads a padded `" @env:X"`;
- **W1** strips *every* trailing newline from `@env:`.

**The answer is to stop depending on that behaviour, not to probe more.** Two decisions live in `channel_policy.json`, independent of any measurement:

1. **Channel lookalikes are refused (R4 NI8).** Normalize a resolved or typed secret: NFKC, drop Unicode categories Cc and Cf (controls, NUL, zero-width), strip Unicode whitespace, case-fold. If it then equals `-` or starts with `@env`, it is refused on **every path**, private and interim, on every OS: `value-looks-like-a-channel`, naming the field. The operator ruled that nobody wants those as secrets. Examples refused: `" @env:X"`, `"@ENV:X"`, `"\u200b@env:X"`, `"\uff20env:X"`, `"- "`, `"\u00a0-"`, `"@envelope"`. Examples not refused: `"--"`, `"-leading"`, `"pass@env:X"`. This is safe whatever F-691 (ms and the toolkit disagree on `"- "`) decides.
2. **A target ending in CR or LF is refused (R4 NI9).** Its last byte being `\r` or `\n` means `value-ends-in-newline`, on every path. Delivery then **never depends on how many trailing newlines a CLI strips**: strip-one, strip-all and verbatim deliver the same bytes for every target the GUI sends. Nobody's passphrase ends in a newline.

   **This is the more robust option and the one chosen.** The shared corpus and single rule implementation below are kept as *detection*: they turn an unmodelled rule red (`UNKNOWN`). Safety does not rest on them.

**Still measured, but no longer load-bearing:**
- `reinterpret.json` now probes the whole lookalike corpus (`corpus.CHANNEL_VARIANTS`, 21 spellings). It feeds the oracle legs and a **consistency** test: every spelling a pinned CLI re-reads must be inside the broad predicate. If a CLI starts re-reading something odd, that test goes red.
- **One corpus and one rule (R4 NI9).** `corpus.py` is shared by `run_bytes.py`, `run_plans.py` and `measure_reinterpret.py`. It has 17 suffixes (every `\r`/`\n` tail up to `\r\n\r\n` and `\n\n\n`, whitespace before a newline, a tab, an interior newline) and 4 prefixes (leading `\n`, `\r\n`, space, tab). The rule functions exist **once**, as `plan.RULES`, which `run_bytes.py` imports.

**R4's wrappers, through a full correct re-derivation and the CI gate** (`demo_r4_wrappers.sh`):
- **W2:** the measurement now *records* the padded re-reads and the gate is GREEN, correctly, because the planner refuses every one of them anyway.
- **W1:** strip-all is measured `UNKNOWN` on the passphrase inputs and the gate goes red. The trailing-newline decision already made it harmless.

```
=== wrappers behave as R4 describes (OTHER = hunter2-passphrase):
  W2 ms derive --passphrase ' @env:OTHER' (argv) : master_fingerprint:  45fbfbe6   (OTHER's wallet is 45fbfbe6)
  real   ms derive --passphrase ' @env:OTHER' (argv) : master_fingerprint:  8d31199c
  W1 restore --passphrase @env:X, X='hunter2-passphrase\n\n' : 45fbfbe6
  real  restore --passphrase @env:X, same X                  : 5d5b0e1a

=== w2: full correct re-derivation, then regen_check --data-from it --plans
  derived: rules {'verbatim': 42, 'strip-one-trailing-newline': 13, 'None': 30} | ms re-reads ['\t-', '\t@env:{V}', '\n@env:{V}', ' -', ' @env:{V}', '-', '-\n', '-\r\n', '- ', '@env:{V}', '@env:{V}\n', '@env:{V}\r\n', '@env:{V} ']
regen_check: GREEN
  exit 0

=== w1: full correct re-derivation, then regen_check --data-from it --plans
  derived: rules {'verbatim': 42, 'UNKNOWN': 12, 'None': 30, 'strip-one-trailing-newline': 1} | ms re-reads ['-', '@env:{V}']
RED  test_plan.py on this data:   FAIL C1 convert phrase+passphrase src1 '@env:EMPTY' macos: got C1-env-rule-unknown, want C1-env-empty — FAIL C1 addresses phrase+passphrase src1 '@env:EMPTY' linux: got C1-env-rule-unknown, want C1-env-empty; FAIL C1 addresses phrase+passphrase src1 '@env:EMPTY' macos: got C1-env-rule-unknown, want C1-env-empty; FAIL C1 addresses phrase+passphrase src1 '@env:EMPTY
RED  T3' run_plans.py: FAILURES: ['addresses phrase+passphrase', 'restore phrase+passphrase', 'restore ms1+passphrase', 'derive-child phrase+passphrase', 'bundle slot+passphrase', 'bundle wsh-multi 2 slots+passphrase', 'convert phrase+passphrase', 'xpub-search path-of-xpub phrase+passphrase', 'xpub-search path-of-xpub ms1+passphrase', 'xpub-search passphrase-of-xpub phrase+passphrase', 'xpub-searc
regen_check: 2 red
  exit 1

=== the planner on R4's cases, fold 5 (pure; any OS):
  ms derive ms1+passphrase, variable = ' @env:OTHER', linux: refused value-looks-like-a-channel
  ms derive ms1+passphrase, variable = ' @env:OTHER', macos: refused value-looks-like-a-channel
  restore phrase+passphrase, variable = 'hunter2-passphrase\n\n', linux: refused value-ends-in-newline
  restore phrase+passphrase, variable = 'hunter2-passphrase\n\n', macos: refused value-ends-in-newline
```

### A1. What is measured, and how to re-run it

Everything is in `design/measurements/secret-channels/`. Every table in Part A is **generated**. `check_design_tables.py` needs no binaries. It runs three checks:
- it regenerates §A5 from the planner;
- it runs the pure tests;
- it asserts this document carries each measured block byte for byte.

**Data** (the planner reads only these):

| file | kind | what it holds |
|---|---|---|
| `channel_table.json` | **derived** | For each input: its measured-OK channels; each channel's **terminator** (§A3c); the input's **`cli_env_rule`**, i.e. what the CLI's own `@env:` does to the variable (`verbatim`, `strip-one-trailing-newline`, or `null` for no working CLI `@env:`; R3 NI7); and **`argv_eq_exact`**, whether `--flag=VALUE` is byte-identical to `--flag VALUE` (R3 Nm13). |
| `reinterpret.json` | **derived** | Per CLI: the spellings it re-reads when they arrive as an argv value (R2 NC1). |
| `measured_with.json` | **derived** | Per CLI: pinned tag, `--version`, and **sha256** of the binary measured. |
| `channel_policy.json` | **decided** | `private_channels_on`, `fd_channel_on` (§A6); **`channel_lookalike`** and **`refuse_trailing_cr_lf`** (§A0.1); `real_binary_test_targets` (T10); the reserved env prefix; the GUI's own env-name rule; `pipe_payload_max`. Nothing about CLI behaviour. |
| `corpus.py` | shared | The one value corpus (suffixes, prefixes, channel lookalikes) that every measuring and oracle script uses (R4 NI9). |

**Re-run, in dependency order:**

```sh
export BIN_DIR=<dir holding mnemonic, md, ms, mk>
python3 run_all.py > channels.txt; python3 run2.py > channels2.txt; python3 run3.py > channels3.txt
python3 measure_groups.py                  # ms combine's share group
python3 run_bytes.py                       # byte fidelity per channel -> bytes.json, bytes.md (§A3c)
python3 table_build.py                     # -> channel_table.json (+ cli_env_rule, argv_eq_exact), measured_with.json (tag+sha256)
python3 measure_reinterpret.py             # -> reinterpret.json (§A6, R2 NC1)
python3 table.py > table.md                # -> §A2
python3 gen_plans.py                       # PURE: plan.py on every shape x OS -> plans_pure.json (§A5)
python3 run_plans.py                       # MEASURED: every plan through a real runner -> plans.json, t3.md (§A9)
python3 probe_missing.py                   # -> missing_sources.md (§A2b)
./c1_evidence.sh > c1_evidence.out         # -> §A3a
bash combos.sh > combos.out; python3 refusals.py > refusals.txt   # fold-0 combos and refusal probe
python3 copy_evidence.py                   # Copy recipes in bash, zsh, fish -> copy_evidence.md (§A7)
python3 test_plan.py                       # PURE: refusal, interim-value, NC1, per-input rule, per-OS, pin, OS-gate legs
python3 mutations.py                       # 20 planner mutations killed BY ASSERTION + a no-op control that must survive
python3 regen_check.py [--plans]           # T4 (CI): identity by sha256, every derived file re-derived and diffed, schema coverage
F687_BIN_DIR=... ./demo_ni5.sh             # §A0: the NI5 scenario against the F-687 master builds -> ni5_demo.out
F687_BIN_DIR=... ./demo_r4_wrappers.sh     # §A0.1: R4's out-of-probe wrappers -> r4_wrappers_demo.out
./demo_t4_coverage.sh                      # §A9 T4: a new secret source with no entry goes red -> t4_coverage_demo.out
python3 check_design_tables.py             # the document gate
```

**What each script measures:**

- **Single-input rows** (`run_all.py`, `run2.py`, `run3.py`). Each row compares a channel against a baseline:
  - *Baseline:* the secret on argv plus `--allow-argv-secret`.
  - *Dependence control:* a different secret must change the output.
  - *Channel variants:* each channel without the opt-in. **OK** = exit and stdout equal the baseline.
- **Byte fidelity** (`run_bytes.py`). Every OK channel of every input is run with seven value endings: none, `\n`, `\r\n`, `\r`, two trailing spaces, space+`\n`, and an interior `\nX`. The result must equal argv carrying **the exact same bytes**. The script derives three things:
  - the terminator that makes each channel exact;
  - **per input**, the CLI's own `@env:` rule, by testing whether `@env:VAR` equals argv-exact of `f(raw)` for each candidate rule `f`;
  - whether `--flag=VALUE` is exact.
- **Schema coverage** (`secret_sources.txt` from `enumerate_sources.rs`, then `probe_missing.py`): the 124 secret sources the mirror can produce (§A4.1).
- **Plans** (`plan.py`, `gen_plans.py`, `shapes.py`, `run_plans.py`, `test_plan.py`, `mutations.py`):
  - `plan.py` is the rule, executable.
  - `shapes.py` lists 29 multi-secret shapes (60 sources).
  - `gen_plans.py` plans them for Linux, macOS and Windows with no binaries.
  - `run_plans.py` runs the Linux plans through a real runner.
  - `test_plan.py` pins every refusal.
  - `mutations.py` shows each of those tests can fail.
- **Argv re-interpretation** (`measure_reinterpret.py`): for every input, is a value of `@env:VAR` or `-` on argv resolved a second time? mnemonic 0.105.1: `-` and `@env:`; ms 0.20.1: `-` and `@env:` (ms 0.19.1 re-read only `-`).
- **Copy recipes** (`copy_evidence.py`): the §A7 recipes run in bash, zsh and fish, compared against argv-exact.
- **T10** (`os_gate.py`, `fixtures/ci/`): the OS gate, pinned against 10 decoy and 3 genuine workflow fixtures.
- **T4** (`regen_check.py`): identity and re-derivation (§A0). `demo_ni5.sh` runs it against the F-687 builds.

### A2. The measured single-input table

Legend:
- **OK:** accepted, and gives the baseline output.
- **WRONG (exit N, literal):** exit 0 or 4 with *different* output. The CLI took the spelling as a literal value.
- **fails closed:** non-zero exit.
- **—:** clap does not know the flag.

| input | argv w/o opt-in | `-` / positional `-` | `@env:VAR` | `--X-stdin` | file (`--X-file` / `--in`) | measurement valid |
|---|---|---|---|---|---|---|
| `mnemonic addresses --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic convert --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from minikey=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic convert --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic convert --bip38-passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic restore --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic derive-child --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic derive-child --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic nostr --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --passphrase` | refused | OK | OK | OK | — | yes |
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
| `ms derive --passphrase` | refused | OK | OK | OK | —/fails closed | yes |
| `ms repair --ms1` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `mnemonic derive-child --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from wif=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic verify-bundle --ms1` | refused | OK | OK | — | — | yes |
| `mnemonic import-wallet --ms1` | NOT refused (exit 0) | fails closed | OK | — | — | yes |
| `mnemonic import-wallet --slot @0.phrase=` | refused | fails closed | OK | n/a | n/a | yes |
| `mnemonic electrum-decrypt --decrypt-password` | refused | OK | OK | OK | OK (`-file`) | yes |
| `mnemonic xpub-search path-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic slip39 split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic slip39 split --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic slip39 combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `mnemonic slip39 combine --passphrase` | refused | OK | OK | OK | — | yes |
| `mnemonic ms-shares split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic ms-shares combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `ms split --phrase` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `ms split --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `ms hashlock --hashlock-phrase` | refused | fails closed | fails closed | OK | —/fails closed | yes |
| `ms hashlock --hex` | refused | OK | fails closed | — | —/fails closed | yes |
| `mnemonic convert --from bip38=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic slip39 split --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic ms-shares split --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic xpub-search passphrase-of-xpub --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `ms hashlock <ms1>` | refused | OK | fails closed | n/a | OK | yes |
| `mnemonic bundle --slot @0.ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.wif=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic addresses --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic import-wallet --decrypt-password` | refused | OK | OK | OK | OK (`-file`) | yes |
| `mnemonic addresses --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.ms1=` | refused | OK | OK | n/a | n/a | yes |

**0 WRONG cells since the F-694 re-pin** (mnemonic 0.105.1, ms 0.20.1): each of the 17 below is now OK, and the `-`/`@env:` cells of `electrum-decrypt` and `import-wallet --decrypt-password` went from fails-closed to OK. On mnemonic 0.104.0 and ms 0.19.1 there were **17 WRONG cells, in 15 rows**, in three groups:
- **`--passphrase -` (13 rows):** literal on every toolkit subcommand whose row is valid, and on `ms derive`. Exit 0, a different wallet (exit 4 on verify-bundle).
- **`--passphrase @env:` (2 rows):** literal on `silent-payment` and `ms derive`.
- **Two further cells:** `--bip38-passphrase -`, and `verify-bundle --ms1 -` (exit 4, a false mismatch; filed as engrave **F-689**).

#### A2b. Schema sources with no table entry

124 schema sources; 82 have an entry, and 3 table entries are Part B's `ms hashlock`. The other 42 were run once each on argv:

| input with no table entry | argv run (+opt-in) | verdict |
|---|---|---|
| `mnemonic addresses --from xprv=` | exit 1: error: --from Xprv is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from wif=` | exit 1: error: --from Wif is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from ms1=` | exit 1: error: --from Ms1 is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from bip38=` | exit 1: error: --from Bip38 is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic addresses --from minikey=` | exit 1: error: --from MiniKey is not supported by `addresses` (use xpub/phrase/entropy/seedqr/electrum-phrase) | CLI rejects this input itself |
| `mnemonic bundle --slot @N.xprv=` | exit 1: error: --slot @0.xprv not supported in v0.4.2; deferred to v0.5+ pending ms-codec XPRV-tag extension. See FOLLOWUP `unified-slot-xprv-resolution-needs | CLI rejects this input itself |
| `mnemonic verify-bundle --from phrase=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from entropy=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from xprv=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from wif=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from ms1=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from bip38=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from electrum-phrase=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from seedqr=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --from minikey=` | exit 1: error: slot count 0 out of range 1..=16 | **unmeasured** |
| `mnemonic verify-bundle --slot @N.xprv=` | exit 1: error: --slot @0.xprv not supported in v0.4.2; deferred to v0.5+ pending ms-codec XPRV-tag extension. See FOLLOWUP `unified-slot-xprv-resolution-needs | CLI rejects this input itself |
| `mnemonic verify-bundle --slot @N.wif=` | exit 4: (no error line) | **unmeasured** |
| `mnemonic export-wallet --slot @N.phrase=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.seedqr=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.entropy=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.ms1=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.xprv=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic export-wallet --slot @N.wif=` | exit 2: error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret materi | CLI rejects this input itself |
| `mnemonic restore --from xprv=` | exit 1: error: --from xprv is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from wif=` | exit 1: error: --from wif is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from bip38=` | exit 1: error: --from bip38 is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from electrum-phrase=` | exit 1: error: --from electrum-phrase is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic restore --from minikey=` | exit 1: error: --from minikey is not a seed source for restore (use ms1/phrase/entropy/seedqr) | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.seedqr=` | exit 1: error: import-wallet: --slot @0.seedqr=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.entropy=` | exit 1: error: import-wallet: --slot @0.entropy=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.ms1=` | exit 1: error: import-wallet: --slot @0.ms1=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.xprv=` | exit 1: error: import-wallet: --slot @0.xprv=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic import-wallet --slot @N.wif=` | exit 1: error: import-wallet: --slot @0.wif=: only the `phrase` subkey is supported by import-wallet | CLI rejects this input itself |
| `mnemonic word-card --from phrase=` | exit 2: error: positional argument 'phrase=aband…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from entropy=` | exit 2: error: positional argument 'entropy=0000…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from xprv=` | exit 2: error: positional argument 'xprv=xprv9s2…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from wif=` | exit 2: error: positional argument 'wif=KyZpNDKn…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from ms1=` | exit 2: error: positional argument 'ms1=ms10entr…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from bip38=` | exit 2: error: positional argument 'bip38=6PYP8f…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from electrum-phrase=` | exit 2: error: positional argument 'electrum-phr…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from seedqr=` | exit 2: error: positional argument 'seedqr=00000…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |
| `mnemonic word-card --from minikey=` | exit 2: error: positional argument 'minikey=S6c5…' does not begin with a recognized HRP prefix (expected one of: mk1, md1) | CLI rejects this input itself |

- **The 32 "CLI rejects" rows** cost nothing to refuse.
- **The 10 unmeasured rows** are verify-bundle's keyless-template completion (`--from`, which needs a fixture with a ≥5-byte `--expect-wallet-id`) and `--slot @N.wif=`. **They refuse on every OS until measured** (§A6: the interim path refuses `no-table-entry` too). The owning phase is the implementation cycle.

### A3. Findings that shape the design

1. **Use measured cells only.** The WRONG cells (§A2) mean a generic rule such as "a secret goes as `<flag> -`" gives a wrong wallet on today's binaries. The planner therefore writes channel tokens only from measured-OK cells.
2. **`ms` has no `@env:`.** `import-wallet --ms1` and `--slot` accept only `@env:`. `xpub-search --ms1` accepts only `--ms1-stdin`.
3. **One stdin per invocation**, enforced by the CLIs.
4. **The CLI refusal is not a safety net.** 0.104.0 runs `import-wallet --ms1` and `seed-xor`/`slip39`/`ms-shares combine --share` on argv with only a warning.
5. **Stdin is not byte-transparent.** Every stdin channel strips one trailing `\r\n` or `\n`; `--X-file` strips one `\n`; `@env:` is verbatim. So "send the value's exact bytes" is wrong on a stripping channel. §A3c specifies the fix, and `run_bytes.py` measures it.
6. **`ms combine`'s shares are one group:** one `-` with all shares on stdin, one per line, or `--in F`.

#### A3a. C1 — a secret field that holds `-` or `@env:VAR`

The operator's final ruling on F-687 is that `-` reads stdin and `@env:VAR` reads the environment, in both CLIs, on every command. In a GUI secret field these spellings **mean that channel**, and the GUI **never sends those characters as the secret**. This runs on **every OS, before either Run path** (the private-channel planner or the interim argv path, §A6):

| the user typed | the GUI does |
|---|---|
| `@env:VAR` | Reads `VAR` from the GUI's own environment at Run time. The **target bytes** are `env_value_rule(raw, this input's cli_env_rule)` (§A3c), byte-identical to what the pinned CLI's own `@env:VAR` would use **on this input**. They are delivered through the planned channel, then zeroized. |
| `@env:VAR`, `VAR` unset | Refuse `C1-env-unset`, naming the field and `$VAR`. |
| `@env:VAR`, empty **after** `env_value_rule` | Refuse `C1-env-empty`. The CLI would take it as *no passphrase* (measured: fingerprint `73c5da0a`, exit 0), a different wallet. It is judged on the target, so a variable holding only `\n` refuses under the F-687 rule too (R2 Nit 1). |
| any value holding NUL | Refuse `nul-in-value` on every OS. argv and env cannot carry NUL, so there is one message everywhere (R2 Nit 1). |
| `@env:name` that fails `[A-Z_][A-Z0-9_]*` | Refuse `C1-bad-name`. This is the CLI's own rule (`env_sentinel.rs`). |
| `@env:MNEMONIC_GUI_…` | Refuse `C1-reserved-name`. It applies in **every** field, pass-through (non-secret) ones included (R1 Nm1). The CLI resolves `@env:` on some non-secret inputs, and the planner's variables live in the child's environment. |
| `-` | Refuse `C1-dash`: "the GUI has no stdin of its own to forward; type the value, or use `@env:VAR`". |

A **secret source** (§A4.1) includes a node-valued Text such as `restore --from ms1=-` or `ms1=@env:SEED`, a slot row, a secret positional, and each element of a group.

**Measured** (`c1_evidence.sh`: `restore --from phrase=<abandon…about>`, intended passphrase `hunter2`; plus the Copy spelling of §A7):

```
intended (argv 'hunter2')                                      : ca2c62d2
no passphrase                                                  : 73c5da0a
today's pass-through: --passphrase @env:MY_PW (MY_PW=hunter2)    : ca2c62d2
reading 1, typed text is the bytes: S1='@env:MY_PW' via @env:S1   : fee4dc32
reading 1, typed text is the bytes: '@env:MY_PW' via -stdin       : fee4dc32
reading 2, sentinel passed through: --passphrase -  (stdin=hunter2): ca2c62d2
  = the literal passphrase '-' on argv                              : 73c5da0a
DESIGN (GUI resolves $MY_PW itself), bytes via @env:MNEMONIC_GUI_S1 : ca2c62d2
DESIGN (GUI resolves $MY_PW itself), bytes via --passphrase-stdin   : ca2c62d2
reading 2 on silent-payment: --passphrase @env:MY_PW (MY_PW=hunter2) vs argv hunter2:
  same
Copy spelling for a stdin-bound value from $MY_PW (R1 Nm7), MY_PW=$'hunter2\n' (trailing newline):
  argv with the exact bytes 'hunter2\n'                          : 762fff19
  bash  printf '%s\r\n' "$MY_PW" | … --passphrase-stdin         : 762fff19
  zsh   printf '%s\r\n' "$MY_PW" | … --passphrase-stdin         : 762fff19
  fish  printf '%s\r\n' "$MY_PW" | … --passphrase-stdin         : 762fff19
R2 NC1: MY_PW='@env:OTHER', OTHER=hunter2 ($OTHER's wallet is ca2c62d2):
  CLI's own --passphrase @env:MY_PW (resolves once)                 : a1c20c4f
  GUI Linux plan: resolved bytes over --passphrase-stdin + \r\n      : a1c20c4f
  UNGUARDED interim: resolved bytes on argv (re-resolved by the CLI) : ca2c62d2
  GUI, fold 5: refused value-looks-like-a-channel on EVERY path (plan.py; test_plan.py NI8 legs)
```

`run_plans.py` also types **every source of every runnable shape** as `@env:USER_SECRET`, with the five endings `""`, `\n`, `\r\n`, `\nX` and two trailing spaces. Each run is compared against argv-exact **and** against the CLI's own `@env:USER_SECRET` wherever that cell's `EnvRef` is measured OK. See the last column of the §A9 table. `test_plan.py` checks the seven refusal spellings on every source of every shape, on **all three OSes**: 1260 legs.

**Provenance is shown** in Preview and the confirm dialog, e.g. `--passphrase ← stdin (value of $MY_PW)` or `--from ms1= ← env MNEMONIC_GUI_S0 (typed)`.

**What C1 retires, in the implementing change:**
- **Help strings.** 25 of them teach `-` or `@env:` on a secret source. Rewrite them to "type the value, or `@env:VAR` (read by the GUI)":
  - `src/schema/mnemonic.rs`: `:600` restore `--from`, `:757` restore `--passphrase`, `:1109` verify-bundle `--from`, `:1346` convert `--from`, `:1766` derive-child `--from`, `:1871` slip39 split `--from`, `:1960` slip39 combine `--share`, `:2062` ms-shares split `--from`, `:2154` ms-shares combine `--share`, `:2204` seed-xor split `--from`, `:2261` seed-xor combine `--share`, `:2316` seedqr encode `--from`, `:2368` seedqr decode `--from`, `:2392` seedqr decode `--digits`, `:2493` final-word `--from`, `:2543` repair `--ms1`, `:2647` inspect `--ms1`, `:4033` addresses `--from`, `:4103` addresses `--passphrase`;
  - `src/schema/ms.rs`: `:85`, `:249`, `:309`, `:466` (the ms1 positionals of inspect, decode, verify and derive), `:479` repair `--ms1`, `:570` split `--phrase`.

  Help on *public* inputs (`--mk1`, `--md1`, `--blob`, `--output`, `--ciphertext`, …) keeps its `-`.
- kittest **Cell 8** (`tests/kittest_import_wallet_form.rs:326-366`) is re-pinned: `@env:MNEMONIC_MS1_0` (a valid name, not under the reserved `MNEMONIC_GUI_` prefix) is **resolved GUI-side**, and argv carries only the planner's reference. A new cell pins `@env:MNEMONIC_GUI_S0` refused.
- `tests/f679_argv_secret_admission.rs`: `restore_from_a_secret_node_is_admitted_but_a_private_channel_is_not` (private leg) and `private_channel_sentinels_in_secret_fields_need_no_opt_in`.
- `invocation::is_private_channel_value` and `masked_token_is_private_channel`.
- phase 1a's `site1_private_channel_sentinel_is_not_masked` becomes a C1 test. `secrets::text_value_is_secret_node_token` splits into a node-only source classifier plus C1.

A user whose real secret is literally `-` or begins with `@env:` cannot type it in the GUI. This is documentation only, stated in the field help.

**A resolved value that is itself a channel spelling (R2 NC1; broadened in fold 5, §A0.1).** `$MY_PW` may *contain* `@env:OTHER`, `-`, or a padded or case-varied form of either. Since fold 5 every such value is refused on **every** path (`value-looks-like-a-channel`), whatever any CLI does; the history below explains why.
- **On the private path** these are just bytes. The CLI resolves the planner's `@env:MNEMONIC_GUI_S<i>` **once** and does not look inside, and stdin/fd content is never resolved. This is measured: Linux equals the CLI's own `@env:MY_PW` on every shape and never gives `$OTHER`'s wallet (§A9 NC1 column).
- **On the interim argv path** the CLI would resolve them a second time. This is measured below: an unguarded interim run gives `$OTHER`'s wallet, `ca2c62d2`. Fold 3 refused them on the interim path from the measured data (`value-is-a-channel-spelling`). Fold 5 refuses them, and every padded or case-varied form, on **every** path, by decision (`value-looks-like-a-channel`, §A0.1).
- Which spellings each CLI re-interprets is **derived** (`reinterpret.json`, by `measure_reinterpret.py`), not a policy row. Today (F-694 re-pin): mnemonic 0.105.1 `-` and `@env:`; ms 0.20.1 `-` and `@env:`. On ms 0.19.1 it was `-` only; F-687 added `@env:`. CI re-derives the file and diffs it (§A0). The oracle leg catches a stale file independently, by executing the admitted plan.
- The operator ruled that nobody wants these as passphrases.

#### A3b. F-687 on the pinned binaries, and the pin bump

Until the GUI pins ms and toolkit releases that implement F-687, the CLI's own `-`/`@env:` spellings are written only where the table measured them OK. The planner guarantees this: user text never becomes a channel token (§A3a).

**Identity is checked by content.**
- `test_plan.py` checks that `measured_with.json`'s pinned tags equal `pinned-upstream.toml`'s, and that a sha256 is recorded.
- `regen_check.py` (CI) checks the installed binaries' **sha256** against those recorded.

A bump that changes a tag, or a rebuild that changes a binary, is red until the cache is re-derived.

**What F-687 changes, measured on F-687 master** (ms `d1ab447`, toolkit `9da32e2f`; fold 4 measured the earlier `e534917`/`9846a784`):
- `-`, `--passphrase-stdin` and `@env:` strip exactly one trailing `\n` or `\r\n` **on `--passphrase`**;
- an argv literal is verbatim;
- ms gains `@env:` on `--passphrase`;
- F-687 fold 1 stopped ms trimming the `=` form, so **`ms derive --passphrase=VALUE` is now exact** (`argv_eq_exact` flips to `True`). `ms hashlock --hashlock-phrase` stays not exact.

Re-derived per input, the `@env:` rule is `strip-one-trailing-newline` on the 13 `--passphrase` inputs and **stays `verbatim` on `--bip38-passphrase`** and the other `@env:` inputs. The operator may extend F-687 to `--bip38-passphrase` and `--decrypt-password` (pending). Either outcome is a re-derivation with no code change: the rule is per input and measured (R3 NI7).

**The bump procedure**, all data, all enforced:
1. Bump the tags.
2. Re-run the §A1 derive steps against the new binaries and commit the cache.
3. `regen_check.py` must be green. It is red if any step was skipped or hand-edited, on identity, on the diff, or on the oracle legs (§A0 (B)).
4. Review the cache diff: new OK cells, new re-reads, rule changes, and any shape whose plan changed in §A5.
5. The Copy gate (§A7) and the UNKNOWN-rule guard (`test_plan.py`) are red if a newly measured rule has no exact Copy spelling or matches no known rule.

`demo_ni5.sh` (C) is this procedure against the F-687 builds, and it is green.

#### A3c. Byte-exact delivery (R1 NI1)

**The target.** The bytes the CLI must end up with are:
- for a typed value, the typed text;
- for `@env:VAR`, `env_value_rule(raw, rule)`, where `raw` is the variable's content and `rule` is **this input's** derived `cli_env_rule` (R3 NI7).

`cli_env_rule` is `null` where the input has no working CLI `@env:`; the variable's bytes are then taken as typed.
- **Today:** `verbatim` on 53 inputs and `null` on the rest.
- **On the F-687 builds:** `strip-one-trailing-newline` on the 13 `--passphrase` inputs, `verbatim` elsewhere (§A3b).
- A rule that matches no known candidate is recorded `UNKNOWN`: `@env:` on that input refuses (`C1-env-rule-unknown`) and `test_plan.py` is red.

**The terminator.** Each channel cell in `channel_table.json` carries a measured `terminator`, which the GUI appends after the target. The channel's own strip then removes exactly that terminator:
- `\r\n` for the stripping stdin channels. The GUI sends `target + "\r\n"`, and the CLI strips one `\r?\n`, so a target that itself ends in `\n`, `\r\n` or `\r` survives intact.
- `\n` for `--decrypt-password-file`.
- `""` for `EnvRef` today.

**Lenient cells.** A channel that trims whitespace where argv would not has terminator `null`. It may carry only a *clean* value: no CR/LF and no edge whitespace. For a non-clean value the planner drops that channel, and refuses `value-not-byte-exact` if nothing is left. Every lenient mismatch is "argv fails, channel succeeds"; none gives both sides OK with different output.

Value variants (corpus.py, shared): suffixes '', '\n', '\r\n', '\r', '\n\n', '\r\n\r\n', '\n\r\n', '\r\r\n', '\n\r', '\r\r', '\n\n\n', '  ', ' \n', '  \r\n', '\t\n', '\t', '\nX'; prefixes '\n', '\r\n', ' ', '\t'; 167 channel cells.

| channel kind | measured terminator | cells |
|---|---|---|
| DashValue | '\r\n' | 58 |
| DashValue | lenient (null) | 10 |
| EnvRef | '' | 41 |
| EnvRef | lenient (null) | 16 |
| FileFlag | '\n' | 2 |
| FileFlag | '\r\n' | 2 |
| InFile | '\r\n' | 8 |
| PosDash | '\r\n' | 5 |
| StdinToggle | '\r\n' | 22 |
| StdinToggle | lenient (null) | 3 |

Lenient cells (terminator null; every mismatch is argv-fails/channel-ok, 108 are both-ok-different): `mnemonic addresses --passphrase` EnvRef; `mnemonic bundle --passphrase` EnvRef; `mnemonic convert --from entropy=` DashValue; `mnemonic convert --from xprv=` DashValue; `mnemonic convert --from minikey=` DashValue; `mnemonic convert --passphrase` EnvRef; `mnemonic convert --bip38-passphrase` EnvRef; `mnemonic restore --passphrase` EnvRef; `mnemonic derive-child --passphrase` EnvRef; `mnemonic silent-payment --passphrase` EnvRef; `mnemonic repair --ms1` DashValue; `mnemonic inspect --ms1` DashValue; `ms derive --passphrase` EnvRef; `mnemonic derive-child --from xprv=` DashValue; `mnemonic convert --from wif=` DashValue; `mnemonic verify-bundle --passphrase` EnvRef; `mnemonic electrum-decrypt --decrypt-password` EnvRef; `mnemonic xpub-search path-of-xpub --ms1` StdinToggle(--ms1-stdin); `mnemonic xpub-search path-of-xpub --passphrase` EnvRef; `mnemonic xpub-search account-of-descriptor --ms1` StdinToggle(--ms1-stdin); `mnemonic xpub-search account-of-descriptor --passphrase` EnvRef; `mnemonic xpub-search passphrase-of-xpub --passphrase` EnvRef; `mnemonic slip39 split --passphrase` EnvRef; `mnemonic slip39 combine --passphrase` EnvRef; `mnemonic convert --from bip38=` DashValue; `mnemonic slip39 split --from entropy=` DashValue; `mnemonic ms-shares split --from entropy=` DashValue; `mnemonic xpub-search passphrase-of-xpub --ms1` StdinToggle(--ms1-stdin); `mnemonic import-wallet --decrypt-password` EnvRef.

`--flag=VALUE` byte-identical to `--flag VALUE` (R3 Nm13; None = not a value-form input): False 1, None 43, True 40; not exact: `ms hashlock --hashlock-phrase`.

Per-input CLI `@env:` value rule (R3 NI7; None = no working CLI `@env:`, the GUI treats the bytes as typed): None 27, strip-one-trailing-newline 16, verbatim 41.

With NO terminator (fold 1's delivery), a wrong output at exit 0/4 on 28 cells: `mnemonic addresses --passphrase` DashValue; `mnemonic addresses --passphrase` StdinToggle(--passphrase-stdin); `mnemonic bundle --passphrase` DashValue; `mnemonic bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --passphrase` DashValue; `mnemonic convert --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --bip38-passphrase` DashValue; `mnemonic convert --bip38-passphrase` StdinToggle(--bip38-passphrase-stdin); `mnemonic restore --passphrase` DashValue; `mnemonic restore --passphrase` StdinToggle(--passphrase-stdin); `mnemonic derive-child --passphrase` DashValue; `mnemonic derive-child --passphrase` StdinToggle(--passphrase-stdin); `mnemonic silent-payment --passphrase` DashValue; `mnemonic silent-payment --passphrase` StdinToggle(--passphrase-stdin); `ms derive --passphrase` DashValue; `ms derive --passphrase` StdinToggle(--passphrase-stdin); `mnemonic verify-bundle --passphrase` DashValue; `mnemonic verify-bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search path-of-xpub --passphrase` DashValue; `mnemonic xpub-search path-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search account-of-descriptor --passphrase` DashValue; `mnemonic xpub-search account-of-descriptor --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search passphrase-of-xpub --passphrase` DashValue; `mnemonic xpub-search passphrase-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 split --passphrase` DashValue; `mnemonic slip39 split --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 combine --passphrase` DashValue; `mnemonic slip39 combine --passphrase` StdinToggle(--passphrase-stdin).

The last line is R1's NI1, reproduced. Fold 1 delivered with no terminator, and on those 14 passphrase toggles a value ending in `\n` gave a different wallet at exit 0. With the measured terminators, **every non-lenient channel cell matches argv-exact on all seven endings** (lenient cells match on clean values), and every shape's `@env:` ending run matches both argv-exact and the CLI's own `@env:` (§A9).

### A4. The mechanism

#### A4.1. Secret source

A **secret source** is any one of:
- a schema `secret: true` Text flag, or one in `SECRET_FLAG_NAMES`;
- a `--slot` row whose subkey is in `SECRET_SLOT_SUBKEYS`;
- a `NodeValueComposite` value whose node is in `SECRET_NODE_TYPES_ARGV`;
- a plain Text value `<node>=<v>` whose node is in `SECRET_NODE_TYPES_ARGV`, whatever `<v>` is;
- a secret positional, as one source or as `ms combine`'s **group**;
- the hand-marked `ms hashlock` inputs.

`secret_sources.txt` is this definition applied to today's mirror.

#### A4.2. The plan

`form::channels::plan(schema, sub, state, user_env, os) -> Result<RunPlan, ChannelRefusal>`:

```text
RunPlan { argv:  Vec<String>,                       // no secret byte (private path)
          mask:  Vec<bool>,
          stdin: Option<Zeroizing<Vec<u8>>>,        // target + terminator
          env:   Vec<(String, Zeroizing<String>)>,  // MNEMONIC_GUI_S<i>
          fds:   Vec<(RawFd, Zeroizing<Vec<u8>>)>,  // Linux: inherited pipe, argv says /dev/fd/<n>
          bindings: Vec<Binding> }                  // (source, channel, argv index, terminator, provenance)
```

`channel_table.rs`, `reinterpret.rs` and `channel_policy.rs` are generated from the JSON files; the first two are derived caches (§A0). A missing table entry is a refusal, on every OS.

#### A4.3. The rule (`plan.py` is its executable form)

0. **Resolve C1** (§A3a), on every OS. Then:
   - an unmeasured source refuses `no-table-entry`;
   - a value holding NUL refuses `nul-in-value`;
   - on **every** path, a value that looks like a channel refuses `value-looks-like-a-channel`, and a value ending in CR or LF refuses `value-ends-in-newline` (§A0.1);
   - on an OS **not in `private_channels_on`**, return the **interim plan** (§A6), refusing:
     - `value-starts-with-dash` for a leading-dash value where `--flag=VALUE` did not measure exact;
   - otherwise drop the channels this source cannot use exactly: fd outside `fd_channel_on`; lenient channels for a non-clean value (§A3c).
1. **Forced stdin.** Sources whose every remaining channel is a stdin channel: two or more → refuse `two-stdin`; exactly one → it takes stdin.
2. **Stdin toggle.** If stdin is free, the first source in argv order that has a `--X-stdin` toggle takes it.
3. **The rest,** in argv order: `EnvRef` (a unique `MNEMONIC_GUI_S<i>`); else stdin, if free; else a pipe fd (payload ≤ `pipe_payload_max`, or refuse `payload-too-large`); else refuse `no-channel-left`.

Within a class the order is `StdinMulti > StdinToggle > DashValue > PosDash` and `FileFlag > InFile`. Every binding carries its channel's terminator. Step 1 runs regardless of source order: `test_plan.py` permutes every shape (M5).

#### A4.4. Env hygiene

Before spawning, the runner removes every inherited `MNEMONIC_GUI_*` variable, then sets exactly the plan's variables.

#### A4.5. Stdin toggles and tree mode

The `*-stdin` toggles stay rendered disabled; only the planner emits them. `build-descriptor` has no secret flag. Tree-mode keys travel inside the spec on stdin, and `--spec -` is modelled as a pre-bound stdin.

### A5. Plans, generated from the rule and the policy

`gen_plans.py` applies `plan.py` to every shape in `shapes.py`, using `channel_table.json` and `channel_policy.json`. `check_design_tables.py` regenerates this block on every run (R1 Nm3). The Rust planner must produce these on every shape and OS (T8). The last column shows what macOS and Windows get the day their `private_channels_on` entry flips; fd stays Linux-only.

Policy (decided): private channels on ['linux'], fd channel on ['linux']. Derived (measured, CI-regenerated): per-input CLI `@env:` rule None ×28, strip-one-trailing-newline ×16, verbatim ×41; measured argv re-reads (informational; safety is the broad predicate) mnemonic 0.105.1: `-` `@env:{V}`, ms 0.20.1: `-` `@env:{V}`.

| shape | Linux plan | macOS | Windows | macOS/Windows once their flag flips |
|---|---|---|---|---|
| addresses phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| restore phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| restore ms1+passphrase | --from ms1= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from ms1= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| derive-child phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| bundle slot+passphrase | --passphrase ← stdin via --passphrase-stdin + '\r\n'; --slot @N.phrase= ← env MNEMONIC_GUI_S1 | --passphrase ← argv + --allow-argv-secret (interim); --slot @N.phrase= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| bundle wsh-multi 2 slots | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1 | --slot @N.phrase= ← argv + --allow-argv-secret (interim); --slot @N.phrase= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| bundle wsh-multi 2 slots+passphrase | --passphrase ← stdin via --passphrase-stdin + '\r\n'; --slot @N.phrase= ← env MNEMONIC_GUI_S1; --slot @N.ms1= ← env MNEMONIC_GUI_S2 | --passphrase ← argv + --allow-argv-secret (interim); --slot @N.phrase= ← argv + --allow-argv-secret (interim); --slot @N.ms1= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| convert phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| convert wif+bip38-passphrase | --from wif= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin + '\r\n' | --from wif= ← argv + --allow-argv-secret (interim); --bip38-passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| convert bip38+bip38-passphrase | --from bip38= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin + '\r\n' | --from bip38= ← argv + --allow-argv-secret (interim); --bip38-passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search path-of-xpub phrase+passphrase | path-of-xpub --phrase ← stdin via --phrase-stdin + '\r\n'; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | path-of-xpub --phrase ← argv + --allow-argv-secret (interim); path-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search path-of-xpub ms1+passphrase | path-of-xpub --ms1 ← stdin via --ms1-stdin; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | path-of-xpub --ms1 ← argv + --allow-argv-secret (interim); path-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search passphrase-of-xpub phrase+passphrase | passphrase-of-xpub --phrase ← stdin via --phrase-stdin + '\r\n'; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | passphrase-of-xpub --phrase ← argv + --allow-argv-secret (interim); passphrase-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search passphrase-of-xpub ms1+passphrase | passphrase-of-xpub --ms1 ← stdin via --ms1-stdin; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | passphrase-of-xpub --ms1 ← argv + --allow-argv-secret (interim); passphrase-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search account-of-descriptor phrase+passphrase | account-of-descriptor --phrase ← stdin via --phrase-stdin + '\r\n'; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | account-of-descriptor --phrase ← argv + --allow-argv-secret (interim); account-of-descriptor --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search account-of-descriptor ms1+passphrase | account-of-descriptor --ms1 ← stdin via --ms1-stdin; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | account-of-descriptor --ms1 ← argv + --allow-argv-secret (interim); account-of-descriptor --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| silent-payment secret+passphrase | --passphrase ← stdin via --passphrase-stdin + '\r\n'; --secret ← pipe fd via --secret-file + '\r\n' | --passphrase ← argv + --allow-argv-secret (interim); --secret ← argv + --allow-argv-secret (interim) | same as macOS | --passphrase ← env MNEMONIC_GUI_S0; --secret ← stdin via --secret-stdin + '\r\n' |
| slip39 split phrase+passphrase | split --from phrase= ← env MNEMONIC_GUI_S0; split --passphrase ← stdin via --passphrase-stdin + '\r\n' | split --from phrase= ← argv + --allow-argv-secret (interim); split --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| slip39 combine 2 shares+passphrase | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1; combine --passphrase ← stdin via --passphrase-stdin + '\r\n' | combine --share ← argv + --allow-argv-secret (interim); combine --share ← argv + --allow-argv-secret (interim); combine --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| slip39 combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | combine --share ← argv + --allow-argv-secret (interim); combine --share ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| seed-xor combine 2 shares | combine --share phrase= ← env MNEMONIC_GUI_S0; combine --share phrase= ← env MNEMONIC_GUI_S1 | combine --share phrase= ← argv + --allow-argv-secret (interim); combine --share phrase= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms-shares combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | combine --share ← argv + --allow-argv-secret (interim); combine --share ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| import-wallet 2 cosigner ms1 | --ms1 ← env MNEMONIC_GUI_S0; --ms1 ← env MNEMONIC_GUI_S1 | --ms1 ← argv + --allow-argv-secret (interim); --ms1 ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| verify-bundle ms1+slot+passphrase | --passphrase ← stdin via --passphrase-stdin + '\r\n'; --ms1 ← env MNEMONIC_GUI_S1; --slot @N.phrase= ← env MNEMONIC_GUI_S2 | --passphrase ← argv + --allow-argv-secret (interim); --ms1 ← argv + --allow-argv-secret (interim); --slot @N.phrase= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms combine share group | <shares> ← stdin via one `-` (all, one per line) | <shares> ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms verify phrase+ms1 | --phrase ← stdin via `-` + '\r\n'; <ms1> ← pipe fd via --in + '\r\n' | --phrase ← argv + --allow-argv-secret (interim); <ms1> ← argv + --allow-argv-secret (interim) | same as macOS | **refuse** (fd-not-on-platform) |
| ms derive ms1+passphrase | --passphrase ← stdin via --passphrase-stdin + '\r\n'; <ms1> ← pipe fd via --in + '\r\n' | --passphrase ← argv + --allow-argv-secret (interim); <ms1> ← argv + --allow-argv-secret (interim) | same as macOS | --passphrase ← env MNEMONIC_GUI_S0; <ms1> ← stdin via positional `-` + '\r\n' |
| ms derive phrase+passphrase | --phrase ← stdin via `-` + '\r\n'; --passphrase ← env MNEMONIC_GUI_S1 | --phrase ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms derive hex+passphrase | --hex ← stdin via `-` + '\r\n'; --passphrase ← env MNEMONIC_GUI_S1 | --hex ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |

The two Linux refusals are `ms derive` with `--phrase`/`--hex` plus `--passphrase`. ms has no second channel for a phrase or hex, so the refusal gives the CLI's own recipe: encode to an ms1 first, then derive from the card.

### A6. What runs where: data, with a gate (R1 NI2)

**Which path each OS runs is data:** `private_channels_on` in `channel_policy.json`, today `["linux"]`.

**On an OS in the list**, the private-channel planner runs (§A4.3).

**On every other OS, the interim path runs:**
1. C1 resolution (§A3a);
2. `no-table-entry` and `nul-in-value` refusals;
3. the channel-lookalike and trailing-newline refusals (§A0.1), which apply on every path and subsume fold 3's `value-is-a-channel-spelling`. That check read the measured `reinterpret.json`; the broad predicate does not;
4. a leading-dash value (R3 Nm13) goes as `--flag=VALUE` where that form measured byte-exact (`argv_eq_exact`); otherwise it refuses `value-starts-with-dash`. Measured: `--flag=VALUE` is exact on 39 inputs and **not** on `ms derive --passphrase` and `ms hashlock --hashlock-phrase`, where ms trims the `=` form (`--passphrase=pw\n` derives `pw`'s wallet);
5. the target bytes of every other source on argv as separate words, with `--allow-argv-secret`.

That is today's admission path minus its pass-through of `-` and `@env:`, and minus its double resolution. Nothing typed as `-` or `@env:`, no unmeasured source, and no value the CLI would re-read reaches argv on any OS.

argv is byte-transparent **only** for values the CLI does not re-interpret. That is why step 3 exists, and why fold 2's "the interim invocation *is* the argv baseline by construction" was wrong (R2 NC1).

**Testing the interim path (R2 NI3; oracles per §A0):**
- **Pure** (`test_plan.py`): for every source of every shape on macOS and Windows, typed as `@env:USER_SECRET`, the interim plan carries `env_value_rule(raw)` and never the typed text. Also covered: the NC1 refusal follows the derived data, and the leading-dash rule follows `argv_eq_exact`.
- **Real runner** (`run_plans.py`), with the interim plan **executed on Linux by forcing the OS**:
  - with endings `""`, `\n`, `\r\n` and trailing spaces, **compared with the oracle**: the CLI's own `@env:` run, or known bytes;
  - with the variable holding `@env:OTHER` or `-`, executed whenever admitted: it must equal the oracle and **never** be `$OTHER`'s wallet;
  - with typed `-leading`, `--help` and `-lead  ` values against the oracle.
- **Mutations:** R2's mutation (the interim plan sends the typed text), the NC1 refusal off, and the Nm13 refusal off are each killed **by assertion** in `run_plans.py` as well as in `test_plan.py`.
- **T8's** `plans_pure.json` pins **values**: the exact bytes each binding delivers, per OS.

**Flipping an OS is one list edit, and it is gated (T10, hardened for R3 Nm10).** An OS may be in `private_channels_on` only if some workflow **triggered on `push` or `pull_request`** has a job that meets all of these (`os_gate.py`, parsed YAML):
- **no guards:** it has no `if:` and no `continue-on-error`, on the job or the qualifying step;
- **the right OS:** it runs on that OS, with a literal `runs-on` or `${{ matrix.<k> }}` resolved through `strategy.matrix` (`include` added, **`exclude` removed**);
- **a real test command:** a step whose `run:` contains a **shell command line**, not a comment or an `echo`. The program must be `cargo` and the subcommand `test` or `nextest run`, naming **every** target in `real_binary_test_targets` (`secret_channels_t2`, `_t3prime`, `_t7`). Leading `VAR=value` assignments are allowed, and so is a trailing `&& …`. **Rejected (R4 Nm14):** `||`, `|`, `;` or `&` on the line; `--no-run`; filter or skip options (`-E`, `--skip`, `--exact`, …); and any bare word that would be a test-name filter, whether before `--` or as a non-flag word after it;
- **no skipped dependency (R4 Nm14):** its `needs:` chain reaches no job with `if:` and no missing job;
- **the binaries:** a **non-empty** `MNEMONIC_BIN` in the step's, the job's or the **workflow's** `env:`.

It is pinned against `fixtures/ci/`:
- **rejected:** 14 decoys:
  - a comment-only mention, `if: false`, and a missing target;
  - R3's seven: `continue-on-error`, `echo cargo test`, `cargo build --test`, a shell-commented line, `workflow_dispatch` only, a matrix `exclude`, and an empty `MNEMONIC_BIN`;
  - R4's four: `|| true`, `--no-run`, `needs:` on an `if: false` job, and a filter that matches nothing;
- **accepted:** a matrix job, the Linux job, a workflow-level `env:` (R3's false red), and `-- --nocapture && echo done`.

**Why not a signed-artifact scheme:** flipping an OS is a reviewed one-line edit and the decoys are accidents, so a signing key would add key management for no threat we have.

**CI wiring:** the implementing change adds T4 (`regen_check.py --plans`) and T10 (`secret_channels_t10`) to the Linux `schema-mirror.yml` job, which installs the pinned binaries. Until the T2/T3′/T7 targets exist, the repo's own workflows cannot pass T10, so the prototype reports `linux: pending`. The acceptance criterion is in §A9 T10.

**`fd_channel_on`** (today `["linux"]`) is separate. Even after macOS's flip, the three fd shapes refuse there (`fd-not-on-platform`) until a macOS measurement adds it. Windows has no `/dev/fd`, and there are **no temp files** (Q3).

**Pipe mechanics** (R0 M4):
- create the pipe with `pipe2(O_CLOEXEC)`;
- write the whole payload (target + terminator) and close the write end **before spawn**;
- payloads over `pipe_payload_max` = 4096 are refused;
- map the read end in with `command-fds`, which handles the `dup2(n,n)` no-op;
- a leaked write end is caught by T7's timeout.

### A7. What the user sees

**Preview and the confirm dialog** show the planned argv and one line per binding, with its provenance. The interim path shows its resolved argv masked, as today. **Q1:** the dialog stays; its first sentence becomes "sends these secrets privately to".

**Copy command** never contains a secret value and never synthesises an unmeasured spelling. It is generated from the **private-channel plan on every OS**, so a pasted command behaves like Linux's Run. One comment line accompanies each binding:

| binding | provenance | POSIX Copy |
|---|---|---|
| `EnvRef` | typed | `--passphrase @env:MNEMONIC_GUI_S1` + `# IFS= read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1` (fish: `read -s -x --delimiter \n MNEMONIC_GUI_S1`). `IFS=` keeps edge whitespace; fold 2's `read -rs` trimmed it (R2 NI4). |
| `EnvRef` | `$MY_PW` | the user's own `--passphrase @env:MY_PW`. The cell's `EnvRef` is measured exact, so the CLI's own rule yields the target by definition. |
| stdin | typed | the plan's spelling (`--passphrase-stdin`) + `# stdin: type it, then Enter, then Ctrl-D`. The CLI strips the Enter. |
| stdin | `$MY_PW` | `printf '%s<T>' "$MY_PW" \| <command>`, where `<T>` is the cell's terminator as a printf escape (`\r\n`). Measured equal to argv-exact in bash, zsh and fish with a value ending in `\n` (§A3a block). |
| pipe fd | any | `--in <FILE>` + `# FILE: a file holding the ms1 card` |
| `StdinMulti` (`ms combine` shares) | typed | `ms combine -- -` + `# stdin: paste each share on its own line, then Ctrl-D` (R2 Nm9) |
| `StdinMulti` | `$S1`, `$S2`, … | `printf '%s\n' "$S1" "$S2" \| ms combine -- -` (R2 Nm9) |

Measured by `copy_evidence.py` against argv-exact, in bash, zsh and fish (the typed values exclude `-`/`@env:…`, which C1 handles before any typed recipe):

| typed passphrase | argv-exact | bash (§A7 recipe) | zsh (§A7 recipe) | fish (§A7 recipe) | bash (fold-2 recipe) | zsh (fold-2 recipe) | fish (fold-2 recipe) |
|---|---|---|---|---|---|---|---|
| `'pad'` | 8728f37c | 8728f37c | 8728f37c | 8728f37c | 8728f37c | 8728f37c | 8728f37c |
| `'  pad  '` | bc403662 | bc403662 | bc403662 | bc403662 | **8728f37c** | **8728f37c** | bc403662 |
| `'\tpad'` | 6e09a416 | 6e09a416 | 6e09a416 | 6e09a416 | **8728f37c** | **8728f37c** | 6e09a416 |
| `'pad\t'` | 665b5e3e | 665b5e3e | 665b5e3e | 665b5e3e | **8728f37c** | **8728f37c** | 665b5e3e |
| `' a  b '` | 4fecc1c4 | 4fecc1c4 | 4fecc1c4 | 4fecc1c4 | **e3ef7859** | **e3ef7859** | 4fecc1c4 |
| `'back\\slash'` | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 |
| `'tab\tin'` | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d |
| `'a\'b"c'` | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 |
| `'*'` | 05469108 | 05469108 | 05469108 | 05469108 | 05469108 | 05469108 | 05469108 |
| `'$HOME'` | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 |
| `'%s%d'` | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 |

§A7 recipe mismatches: 0 of 33. Typed stdin row (`--passphrase-stdin`, value + Enter): 0 mismatches of 11. Share group, `printf '%s\n' "$S1" "$S2" | ms combine -- -`: bash == argv, zsh == argv, fish == argv; typed (one share per line, Ctrl-D): == argv.

Multi-line typed values (R3 Nm11): `'mid\nline'`: argv-exact 74db797b, `read` recipe bash **bdb4bfbc**, zsh **bdb4bfbc**, fish **bdb4bfbc**, typed stdin row ==; `'a\r\nb'`: argv-exact d5e2d0db, `read` recipe bash **e19f2f8f**, zsh **e19f2f8f**, fish **e19f2f8f**, typed stdin row ==. So Copy is disabled for a typed EnvRef-bound value holding CR or LF; the stdin row stays.

- **Copy gate (per input, R3 NI7):** the `printf` form delivers the raw bytes, so it is exact only where the input's `cli_env_rule` is `verbatim` or `null`. `test_plan.py` fails if any input with another rule lacks an OK `EnvRef` cell, which is where Copy spells `@env:VAR` and lets the CLI apply its own rule. On the F-687 builds every stripping input has one (§A0 (C) is green).
- **Multi-line typed values (R3 Nm11):** a shell `read` takes one line, which is measured below: `mid\nline` through the `read` recipe gives `mid`'s wallet. So **Copy is disabled** for a typed EnvRef-bound value holding CR or LF, with the tooltip "this value spans lines; use Run, or paste it into the command's own stdin prompt". The typed stdin row (value, Enter, Ctrl-D) keeps every line and stays.
- **Windows Copy (cmd):** env-bound bindings use `@env:` with `REM set …` lines. A stdin-bound binding disables the Windows Copy, with the tooltip "needs a pipe; use the POSIX copy".
- **When the plan is refused**, Copy is disabled and its tooltip shows the refusal (R0 M3).

### A8. Refusals (the GUI never falls back to argv)

| code | when |
|---|---|
| `C1-dash`, `C1-env-unset`, `C1-env-empty`, `C1-bad-name`, `C1-reserved-name` | §A3a (reserved: any field; empty: judged on the target) |
| `nul-in-value` | any value holding NUL, on every OS |
| `value-looks-like-a-channel` | every path: under NFKC, Cc/Cf removal, whitespace strip and case-fold, the value equals `-` or starts with `@env` (R4 NI8; replaces fold 3's `value-is-a-channel-spelling`) |
| `value-ends-in-newline` | every path: the target's last byte is CR or LF (R4 NI9) |
| `value-starts-with-dash` | interim path only: a leading-dash value where `--flag=VALUE` did not measure exact (R3 Nm13) |
| `C1-env-rule-unknown` | `@env:` on an input whose CLI rule matched no known rule (R3 NI7) |
| `no-table-entry` | an unmeasured source (§A2b), on every OS |
| `value-not-byte-exact` | a non-clean value on an input whose channels are all lenient (§A3c) |
| `no-channel-on-platform` / `fd-not-on-platform` | §A6 |
| `two-stdin`, `no-channel-left` | §A4.3 |
| `pipe-failed`, `payload-too-large` | pipe creation fails; a pipe payload over 4096 bytes |

### A9. Testing: every secret reaches exactly the flag the user filled, as the exact bytes

**Oracle rule (§A0).** No test takes its expected answer from the data it is testing.
- **Pure tests (T1, T8)** check that the planner follows the data.
- **Real-binary tests (T2, T2b, T3′, T9)** check the data and the planner against an **independent oracle**: the CLI's own `@env:` run, argv-exact, or known bytes over a non-lenient stdin toggle.
- **T4** checks that the data is the measurement.

- **T1: plan property (pure).** Prototype: `test_plan.py`, 0 failures.
  - For each shape, and each combination of a subcommand's secret sources filled with **distinct** sentinels:
    - no argv token contains a sentinel;
    - each binding sits at its own source's flag or slot;
    - each channel carries its own source's target plus terminator;
    - env names are unique, with at most one stdin.
  - **C1 legs:** all seven refusal spellings on every source on **all three OSes** (1260 legs). A crash under a mutation is an assertion failure here (`refusal()` catches it), never an uncaught exception.
  - **Resolution:** under both rules, and **per input** with the derived `cli_env_rule` (R3 NI7). No input may be `UNKNOWN`.
  - **Pass-through guard.**
  - **Interim:** macOS and Windows plans are all `Argv`, carry the resolved target and never the typed text. A leading-dash value uses `--flag=` exactly where `argv_eq_exact` (R3 Nm13).
  - **NI8 (R4):** every one of the 21 lookalike spellings, held by the user's variable, is refused on every path and every OS (3717 legs). Every re-read spelling in the measured `reinterpret.json` must be inside the predicate (consistency). Ordinary secrets (`--`, `-leading`, `pass@env:X`, …) are not refused.
  - **NI9 (R4):** a target ending in CR or LF is refused on every path under both rules. `pw\n` under strip-one plans.
  - **Refusals:** empty-after-rule, and `nul-in-value` on every OS.
  - **Identity:** `measured_with.json`'s pinned tags equal `pinned-upstream.toml`, with a sha256 recorded (checked against the binaries in T4).
  - **M5:** permutations.
  - **Lenient:** the non-clean refusal.
  - **Bounds:** `payload-too-large`.
  - **Copy gate:** per input.
- **T2: single-input real-binary equivalence.** Every table cell equals the argv baseline, **with dependence**.
- **T2b: byte fidelity** (`run_bytes.py`). Every channel cell, with the seven endings and its terminator, equals argv-exact. The same pass derives `cli_env_rule` and `argv_eq_exact`.
- **T3′: multi-secret real-runner legs** (`run_plans.py`). Every leg compares against the **oracle**:
  - **Baseline:** every plannable shape through the real runner equals the argv baseline, with its effect line. A shape's `expect` field is informational; a plan must pass, and a refusal is always safe.
  - **Swap leg:** every source pair swapped must differ from the baseline, except the 4 symmetric pairs.
  - **NI1 leg:** each source typed as `@env:USER_SECRET`, run through the **whole shared corpus** (21 variants), must equal the oracle. Variants the decisions refuse are counted as refused, and a refusal is always safe.
  - **Interim leg (R2 NI3):** the interim plan, executed on Linux over the same corpus, must equal the oracle.
  - **NC1 leg (R2, R3 NI5, R4 NI8):** the variable holds each of the 21 lookalike spellings. **Both** paths are executed whenever they plan. Each must equal the oracle, must **never** be `$OTHER`'s wallet, and must be refused by the lookalike predicate; any other outcome is red.
  - **Nm13 leg:** typed `-leading`, `--help` and `-lead  ` on both paths must equal the oracle. Where there is no oracle, the run must **fail closed or equal the `--flag=VALUE` argv run** (R4 N5).
- **T4: derived-data gate** (`regen_check.py`, CI; R3 shape change):
  - the installed binaries for the pinned tags must match `measured_with.json` by **sha256**;
  - every derived file must equal a re-derivation against them (`channel_table.json`, `reinterpret.json`, `measured_with.json`, `groups.json`, `bytes.json`);
  - `--plans` also runs T1 and T3′ on that data;
  - **schema coverage (R4 Nm15), in the prototype:**
    - the GUI's secret sources are **re-enumerated from the schema mirror** (`enumerate_sources.rs` built against the repo) and must equal `secret_sources.txt`;
    - §A2b's verdicts (`probe_missing.py`) are re-derived and diffed;
    - every source must have a table entry or a §A2b row;
  - a missing cache file is a named red line, not a traceback (R4 N4).

  **Shown red** on the F-687 builds for the stale cache and for R3's NI5 scenario, and **green** for the complete re-derivation (§A0). Adding a secret source with no entry is **shown red** both ways, a mirror change and a cache edit, and a missing file gives a clean red:

```
=== (1) mirror change: ms derive --account marked secret (scratch copy of the repo)
RED  coverage: the schema mirror's secret sources differ from secret_sources.txt; in the mirror only: ['ms derive --account']; in the file only: []
regen_check: 1 red
exit 1

=== (2) cache-only change: a source appended to secret_sources.txt, no table entry
RED  coverage: the schema mirror's secret sources differ from secret_sources.txt; in the mirror only: []; in the file only: ['ms derive --account']
RED  coverage: secret source 'ms derive --account' has no channel-table entry and no §A2b row
regen_check: 2 red
exit 1

=== (3) R4 N4: a missing cache file is a named RED line, not a traceback
RED  derived: reinterpret.json missing from <path>
regen_check: 1 red
exit 1
```
- **T7: runner echo test** (portable, no CLIs). The real runner executes each plan against a helper that **parses its own argv** for `@env:NAME`, `/dev/fd/N` and `-`/`--X-stdin`, and prints the bytes it received on each. They must equal each source's target plus terminator.
- **T8: plan parity.** The Rust `plan()` == `plans_pure.json` (bindings and payload values) on every shape × OS. `check_design_tables.py` regenerates it from `plan.py`.
- **T9: `ms hashlock` phrase fidelity.** With the real `ms`, `"  pad  "` ≠ `"pad"`, and the planned run == the oracle for `"  pad  "`, `"pad\n"` and `"pad\r\n"`.
- **T10: OS gate** (`os_gate.py`; in CI as `secret_channels_t10`, in the same Linux job as T4). Hardened as §A6 describes, and pinned against 14 decoys and 4 genuine fixtures.

  **Acceptance criterion (R3 Nit 3):** the change that implements Part A must make T10 **pass against the repo's own workflows** for every OS in `private_channels_on`, which means it adds the Linux job running `secret_channels_t2`, `_t3prime` and `_t7` with the pinned binaries. From that change on, T10's "pending" report becomes a **hard failure**. That change is not complete while T10 reports pending.
- **T5: mutations** (`mutations.py`; R3 NI6):
  - Each mutation runs in a scratch copy that **keeps the repo layout** (pin file, workflows, design doc). A mutation counts as killed **only by an assertion failure**: `test_plan.py` printing `N failures` with N > 0, a changed §A5, or `run_plans.py` printing `FAILURES: [...]`. A crash counts as **survived**.
  - A **no-op control** must survive, or the harness fails.
  - Result: **20/20 killed by assertion; control survived.**
    - Fold 5 adds: the lookalike refusal off; normalization without whitespace strip; normalization without case-fold; the trailing-newline refusal off; and **the one shared strip-one rule turned into strip-all**. That last one also exercises `run_bytes.py`, which now imports the same function.
    - Fold 4's harness found one gap on its first run: turning off the Nm13 refusal survived, because `-leading` has no whitespace for ms's `=` form to trim. The `-lead  ` value and a pure leg were added.
  - Runner-side mutations go to T3′ and T7: swap bindings; write stdin from the wrong binding; drop the terminator; drop the env scrub; leave the write end open; restore the admission.
- **T6: UI.** Preview, the confirm dialog and Copy contain no sentinel, and do contain each binding with its provenance. Copy spellings match §A7.
  - **Shell leg:** the printed recipes in bash, zsh and fish must equal argv-exact (0 of 33 mismatches). Copy is disabled for multi-line typed values.
  - The tutorial J1 modal re-pin is deliberate.
- **Where they run.**
  - T1, T8, the pure T5 half and T7: plain `cargo test`, Linux job.
  - T2, T2b, T3′, T4, T9, T10 and T6's shell leg: **one** Linux job (`schema-mirror.yml`), which installs the pinned binaries (R3 Nit 3: one job, not two).
  - T7 is also the first leg of any macOS or Windows job.

`run_plans.py` is T3′'s prototype. Its result:

| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | source values swapped (i↔j) vs baseline | T1 | NI1: `@env:` + endings == oracle | interim (NI3) == oracle | NI8 corpus in the variable: OTHER's wallet; == oracle; refused Linux/interim | Nm13 leading dash == oracle (Linux, interim) |
|---|---|---|---|---|---|---|---|---|---|---|
| addresses phrase+passphrase | 0 | 0 | **yes** | `0  bc1qrm3qju2002wmwly8x2ee7ghdaunexsndwgedwv` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| restore phrase+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| restore ms1+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| derive-child phrase+passphrase | 0 | 0 | **yes** | `target biology midnight canal glass common include trophy glimpse north castle dove` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| bundle slot+passphrase | 0 | 0 | **yes** | `mk1qpd2y2pqqsqk4z99gdzlh7lxqvzg3vs7vs57ls3u2nlnjvzn90ffnjpcsauf2eggmpdquu02l9k7dpjhxhs3yaa` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| bundle wsh-multi 2 slots | 0 | 0 | **yes** | `mk1qpm6gzpqqspdayp9s00fqfvrw0za5zs8qjyty8kskx54hpzzjwjpds9j69su6hyzpkdq32t74e44wnhpg9dj4y5` | 0↔1: differs (exit 0) | ok | 16/16 (+26 refused) | 16/16 (+26 refused) | 0 OTHER; 0/0 eq; 42/42 refused | n/a |
| bundle wsh-multi 2 slots+passphrase | 0 | 0 | **yes** | `mk1qpgaqcpqqspywsvg03r5rzrughalhes8qjyty83nr0wscquatny8cq3ctkcnc7w7lklfeq66dhl2ml4aacj89mq` | 0↔1: differs (exit 1); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | 29/29 (+34 refused) | 29/29 (+34 refused) | 0 OTHER; 0/0 eq; 63/63 refused | 3/3, 3/3 |
| convert phrase+passphrase | 0 | 0 | **yes** | `xpub: xpub6CPUCVp94gpNs3bS1eGiwSWwZMLNmfr2Uo1t5v8YtY4XVoxhUraBH7sRyVfgwSNCxRVpX1bDREtc5Kri` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| convert wif+bip38-passphrase | 0 | 0 | **yes** | `bip38: 6PYP8fdoVaE3ThLmEnYcGo3nJeBqd8PvB7CRvTz3TX5L9ojoPHKCg7QXG6` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| convert bip38+bip38-passphrase | 0 | 0 | **yes** | `wif: KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| xpub-search path-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| xpub-search path-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | 4/4 (+38 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 2/2 (+1 refused), 3/3 |
| xpub-search passphrase-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| xpub-search passphrase-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | 4/4 (+38 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 2/2 (+1 refused), 3/3 |
| xpub-search account-of-descriptor phrase+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| xpub-search account-of-descriptor ms1+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | 4/4 (+38 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 2/2 (+1 refused), 3/3 |
| silent-payment secret+passphrase | 0 | 0 | **yes** | `address:      sp1qq2d73kpx36h7r08gmawe6slzxkntu2tw0as7pkqe0hvv3k38mkvckqsp6dmrwxvd6mumfqj9` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| slip39 split phrase+passphrase | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| slip39 combine 2 shares+passphrase | 0 | 0 | **yes** | `8ab7aa39cd427130e225ed43c34f5b5a` | 0↔1: **same** (symmetric); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | 29/29 (+34 refused) | 29/29 (+34 refused) | 0 OTHER; 0/0 eq; 63/63 refused | 9/9, 9/9 |
| slip39 combine 2 shares | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: **same** (symmetric) | ok | 16/16 (+26 refused) | 16/16 (+26 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| seed-xor combine 2 shares | 0 | 0 | **yes** | `zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong` | 0↔1: **same** (symmetric) | ok | 16/16 (+26 refused) | 16/16 (+26 refused) | 0 OTHER; 0/0 eq; 42/42 refused | n/a |
| ms-shares combine 2 shares | 0 | 0 | **yes** | `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ab` | 0↔1: **same** (symmetric) | ok | 16/16 (+26 refused) | 16/16 (+26 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| import-wallet 2 cosigner ms1 | 0 | 0 | **yes** | `"descriptor": "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aS` | 0↔1: differs (exit 4) | ok | 16/16 (+26 refused) | 16/16 (+26 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 6/6, 6/6 |
| verify-bundle ms1+slot+passphrase | 0 | 0 | **yes** | `result: ok` | 0↔1: differs (exit 2); 0↔2: differs (exit 1); 1↔2: differs (exit 2) | ok | 29/29 (+34 refused) | 29/29 (+34 refused) | 0 OTHER; 0/0 eq; 63/63 refused | 6/6, 6/6 |
| ms combine share group | 0 | 0 | **yes** | `entropy: 00000000000000000000000000000000` | n/a (one source) | ok | 0/0 (+0 refused) | 0/0 (+0 refused) | 0 OTHER; 0/0 eq; 0/0 refused | n/a |
| ms verify phrase+ms1 | 0 | 0 | **yes** | `OK: round-trip valid (12 words, language=english)` | 0↔1: differs (exit 1) | ok | 16/16 (+26 refused) | 16/16 (+26 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 0/0, 0/0 |
| ms derive ms1+passphrase | 0 | 0 | **yes** | `master_fingerprint:  45fbfbe6` | 0↔1: differs (exit 1) | ok | 21/21 (+21 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 3/3, 3/3 |
| ms derive phrase+passphrase | 0 | 0 | **yes** | `master_fingerprint:  45fbfbe6` | 0↔1: differs (exit 1) | ok | 11/11 (+31 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 2/2 (+1 refused), 3/3 |
| ms derive hex+passphrase | 0 | 0 | **yes** | `master_fingerprint:  45fbfbe6` | 0↔1: differs (exit 1) | ok | 11/11 (+31 refused) | 21/21 (+21 refused) | 0 OTHER; 0/0 eq; 42/42 refused | 2/2 (+1 refused), 3/3 |

### A10. Follow-ups (not this design)

- **Engrave F-687** (being implemented in ms and the toolkit) and **F-689** (`verify-bundle --ms1 -`).
- **Toolkit:** argv secrets not refused on `import-wallet --ms1` and `seed-xor`/`slip39`/`ms-shares combine --share`.
- **GUI:**
  - the macOS and Windows real-binary jobs (each flips its `private_channels_on` entry);
  - a macOS `/dev/fd` measurement (flips `fd_channel_on`);
  - a Windows named-pipe measurement;
  - the §A2b unmeasured rows.

---

## Part B — the five unsurfaced forms

Source: each binary's `gui-schema` (md/ms schema v1 has no `secret` or `repeating` fields, so those columns come from `--help` and from running the binary). "GUI kind" is the existing `FlagKind`. All five fit the existing form machinery: **no new widget kind is needed.** The new work is conditional-rule functions and one secret positional. R0 re-ran every conditional below; all match.

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
| `--hashlock-phrase` | Text → **SecretLineEdit** (hand-marked secret; ms's v1 schema has no secret bit) | **yes** | exactly one **source** (below) |
| `--hashlock-phrase-stdin` | Boolean, GUI-managed: the phrase's only private channel (A2) | — | planner-emitted |
| `--hex` | Text → SecretLineEdit (a 32-byte preimage) | **yes** | a source; channel `--hex -` |
| `<ms1>` | positional, **secret** (a preimage plate) | **yes** | a source; channel `-` or `--in` (A2 row `ms hashlock <ms1>`, measured in fold 1) |
| `--in` | Path (JSON says text) | no (a path) | a source |
| `--random` | Boolean | no | a source; **requires `--out`** ("--random needs --out FILE", exit 64) |
| `--kind` | Dropdown: `(choose)`, `sha256`, `hash256`, `ripemd160`, `hash160`, `all kinds — lookup only` | no | lowercase only (the CLI rejects `SHA256`, exit 64). See below |
| `--method` | Dropdown `hardened, sha256` (default hardened, suppressed at default) | no | phrase sources only |
| `--out` | Path (save) | no | the CLI writes it owner-only |
| `--json` | Boolean | no | **stdout then carries the secret.** Show a notice beside the box |
| `--emit-record` | Boolean | no | phrase source only ("--emit-record needs a phrase", exit 64) |
| `--no-engraving-card`, `--phrase-looks-like-digest-ok` | Boolean | no | — |
| `--group-size` | Number (default 5) | no | — |
| `--separator` | Dropdown `space, hyphen, comma` | no | — |
| `--allow-argv-secret` | GUI-managed, never rendered | — | — |

- **Source exclusivity (R0 M6).** Phrase, `--hex`, `<ms1>`, `--in` and `--random` get a conditional: the first one filled wins, and the others grey out (the v0.62.0 `ms` input-source pattern). Otherwise phrase + hex would reach A8's `two-stdin` refusal instead of "exactly one source".
- **`--kind` (R0 M6).** The dropdown starts at `(choose)`, and **Run is disabled until a kind is chosen.** Omitting `--kind` puts a sha256 record on stdout (measured: "no --kind given; stdout carries the sha256 record"), which is the F-553 pipe hazard.
  - `all kinds — lookup only` omits the flag deliberately, and shows a banner beside the result: "stdout is the sha256 record; your wallet's kind may differ".
  - Tests: the default state emits no `--kind` token and has Run disabled; `all kinds` emits none and shows the banner.
- **Byte-verbatim phrase** (measured: `"  pad  "` over `--hashlock-phrase-stdin` gives the same digest as on argv, and a different digest from `"pad"`). The widget must not trim; pinned by **T9** (§A9).
- **Q2 (adopted): build after Part A.** The phrase's only private channel is Part A's `StdinToggle` case.

### B6. Shared work for all five

- **Schema mirror.** Mirroring them brings them under the flag-name gate automatically (the gate walks only mirrored subcommands). The mirror hand-marks `--hashlock-phrase`, `--hex` and the `<ms1>` positional as secret.
- **Snapshots.** Five new form PNGs, and the gallery census goes from 61 to 66.
- **Manual.** The GUI manual gains five form pages. That is a toolkit-repo change (paired PR).

---

## Decisions (R0 recommendations adopted)

- **Q1:** keep the confirm dialog for fully private runs, reworded (§A7).
- **Q2:** `ms hashlock` after Part A (§B5).
- **Q3:** no Windows temp files; the three fd shapes refuse wherever `fd_channel_on` excludes the OS (§A6).
- **Q4:** verify-bundle is measured against a matching bundle (§A2, `run3.py`), and so is every M2 input. The remaining 10 rows (§A2b) are measured in the implementation cycle before the admission is deleted. Until then they refuse.
