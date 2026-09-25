# DESIGN — secrets over private channels, and the five unsurfaced forms

**Status:** design only, nothing implemented.
- **Fold 2** answers R1 (`mnemonic-engrave/design/agent-reports/gui-design-r1.md`, 0C/2I/7M/4N); the map is `gui-design-fold2.md`.
- **Fold 1** answered R0 and the operator's F-687 ruling; its map is `gui-design-fold1.md`.

**Baseline:** mnemonic-gui `gui-followups`, fold 1 at `2d244d2`.
**Measured against the release binaries:** mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0, installed with `mnemonic-toolkit/scripts/install.sh --no-gui --no-man`. These binaries **predate** the F-687 CLI change (§A3b).
**Follow-ups this covers:** `argv-secret-via-private-channels` (Part A), `md-ms-new-subcommands-unsurfaced` (Part B).

---

## Part A — every secret over a private channel

### A1. What is measured, and how to re-run it

Everything is in `design/measurements/secret-channels/`. Every table in Part A is **generated**. `check_design_tables.py` needs no binaries. It runs three checks:
- it regenerates §A5 from the planner;
- it runs the pure tests;
- it asserts this document carries each measured block byte for byte.

**Data** (the planner reads only these):

| file | what it holds |
|---|---|
| `channel_table.json` | Generated. For each input: its measured-OK channels and each channel's measured **terminator** (§A3c). |
| `channel_policy.json` | Hand-maintained, **one entry per decision**: `env_value_rule` (§A3c), `private_channels_on` and `fd_channel_on` (§A6), the reserved env prefix, the env-name rule, `pipe_payload_max`. |

**Re-run, in dependency order:**

```sh
export BIN_DIR=<dir holding mnemonic, md, ms, mk>
python3 run_all.py > channels.txt; python3 run2.py > channels2.txt; python3 run3.py > channels3.txt
python3 measure_groups.py                  # ms combine's share group
python3 run_bytes.py                       # byte fidelity per channel -> bytes.json, bytes.md (§A3c)
python3 table_build.py                     # -> channel_table.json
python3 table.py > table.md                # -> §A2
python3 gen_plans.py                       # PURE: plan.py on every shape x OS -> plans_pure.json (§A5)
python3 run_plans.py                       # MEASURED: every plan through a real runner -> plans.json, t3.md (§A9)
python3 probe_missing.py                   # -> missing_sources.md (§A2b)
./c1_evidence.sh > c1_evidence.out         # -> §A3a
bash combos.sh > combos.out; python3 refusals.py > refusals.txt   # fold-0 combos and refusal probe
python3 test_plan.py                       # PURE refusal / per-OS / permutation legs
python3 mutations.py                       # PURE: 12 planner mutations, each must go red
python3 check_design_tables.py             # the gate
```

**What each script measures:**

- **Single-input rows** (`run_all.py`, `run2.py`, `run3.py`). Each row compares a channel against a baseline:
  - *Baseline:* the secret on argv plus `--allow-argv-secret`.
  - *Dependence control:* a different secret must change the output.
  - *Channel variants:* each channel without the opt-in. **OK** = exit and stdout equal the baseline.
- **Byte fidelity** (`run_bytes.py`). Every OK channel of every input is run with seven value endings: none, `\n`, `\r\n`, `\r`, two trailing spaces, space+`\n`, and an interior `\nX`. The result must equal argv carrying **the exact same bytes**. The script finds the terminator that makes each channel exact (§A3c).
- **Schema coverage** (`secret_sources.txt` from `enumerate_sources.rs`, then `probe_missing.py`): the 124 secret sources the mirror can produce (§A4.1).
- **Plans** (`plan.py`, `gen_plans.py`, `shapes.py`, `run_plans.py`, `test_plan.py`, `mutations.py`):
  - `plan.py` is the rule, executable.
  - `shapes.py` lists 29 multi-secret shapes (60 sources).
  - `gen_plans.py` plans them for Linux, macOS and Windows with no binaries.
  - `run_plans.py` runs the Linux plans through a real runner.
  - `test_plan.py` pins every refusal.
  - `mutations.py` shows each of those tests can fail.

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
| `mnemonic addresses --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic bundle --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic convert --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from minikey=` | refused | OK | fails closed | n/a | n/a | yes |
| `mnemonic convert --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic convert --bip38-passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic restore --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --from ms1=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic restore --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic derive-child --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic derive-child --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic nostr --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --secret` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic silent-payment --passphrase` | refused | **WRONG (exit 0, literal)** | **WRONG (exit 0, literal)** | OK | — | yes |
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
| `ms derive --passphrase` | refused | **WRONG (exit 0, literal)** | **WRONG (exit 0, literal)** | OK | —/fails closed | yes |
| `ms repair --ms1` | refused | OK | fails closed | — | OK (`--in`) | yes |
| `mnemonic derive-child --from xprv=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic convert --from wif=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --passphrase` | refused | **WRONG (exit 4, literal)** | OK | OK | — | yes |
| `mnemonic verify-bundle --ms1` | refused | **WRONG (exit 4, literal)** | OK | — | — | yes |
| `mnemonic import-wallet --ms1` | NOT refused (exit 0) | fails closed | OK | — | — | yes |
| `mnemonic import-wallet --slot @0.phrase=` | refused | fails closed | OK | n/a | n/a | yes |
| `mnemonic electrum-decrypt --decrypt-password` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic xpub-search path-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search path-of-xpub --passphrase` | refused | **WRONG (exit 4, literal)** | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --ms1` | refused | fails closed | fails closed | OK | — | yes |
| `mnemonic xpub-search account-of-descriptor --passphrase` | refused | **WRONG (exit 4, literal)** | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --passphrase` | refused | **WRONG (exit 4, literal)** | OK | OK | — | yes |
| `mnemonic xpub-search passphrase-of-xpub --phrase` | refused | fails closed | OK | OK | — | yes |
| `mnemonic slip39 split --from phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic slip39 split --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
| `mnemonic slip39 combine --share` | NOT refused (exit 0) | OK | OK | — | — | yes |
| `mnemonic slip39 combine --passphrase` | refused | **WRONG (exit 0, literal)** | OK | OK | — | yes |
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
| `mnemonic import-wallet --decrypt-password` | refused | fails closed | fails closed | OK | OK (`-file`) | yes |
| `mnemonic addresses --from electrum-phrase=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.entropy=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.seedqr=` | refused | OK | OK | n/a | n/a | yes |
| `mnemonic verify-bundle --slot @0.ms1=` | refused | OK | OK | n/a | n/a | yes |

**17 WRONG cells, in 15 rows.** They split into three groups:
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
| `@env:VAR` | Reads `VAR` from the GUI's own environment at Run time. The **target bytes** are `env_value_rule(raw)` (§A3c), byte-identical to what the pinned CLI's own `@env:VAR` would use. They are delivered through the planned channel, then zeroized. |
| `@env:VAR`, `VAR` unset | Refuse `C1-env-unset`, naming the field and `$VAR`. |
| `@env:VAR`, `VAR` empty | Refuse `C1-env-empty`. The CLI would take it as *no passphrase* (measured: fingerprint `73c5da0a`, exit 0), a different wallet. |
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
reading 2, sentinel passed through: --passphrase -  (stdin=hunter2): 66d564d1
  = the literal passphrase '-' on argv                              : 66d564d1
DESIGN (GUI resolves $MY_PW itself), bytes via @env:MNEMONIC_GUI_S1 : ca2c62d2
DESIGN (GUI resolves $MY_PW itself), bytes via --passphrase-stdin   : ca2c62d2
reading 2 on silent-payment: --passphrase @env:MY_PW (MY_PW=hunter2) vs argv hunter2:
  DIFFERENT: the @env: text is taken literally
Copy spelling for a stdin-bound value from $MY_PW (R1 Nm7), MY_PW=$'hunter2\n' (trailing newline):
  argv with the exact bytes 'hunter2\n'                          : 762fff19
  bash  printf '%s\r\n' "$MY_PW" | … --passphrase-stdin         : 762fff19
  zsh   printf '%s\r\n' "$MY_PW" | … --passphrase-stdin         : 762fff19
  fish  printf '%s\r\n' "$MY_PW" | … --passphrase-stdin         : 762fff19
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

#### A3b. F-687 on the pinned binaries, and the pin bump

Until the GUI pins ms and toolkit releases that implement F-687, the CLI's own `-`/`@env:` spellings are written only where the table measured them OK. The planner guarantees this: user text never becomes a channel token (§A3a).

**The bump is a data edit.** No planner code changes:
1. Re-run §A1 against the new binaries.
2. If the CLIs' `@env:` value rule changed, edit the **single entry** `env_value_rule` in `channel_policy.json` (expected under F-687: `strip-one-trailing-newline`). `run_bytes.py` then re-derives every channel's terminator, `EnvRef` included, against the new target.
3. Review the `channel_table.json` diff. Expected: the WRONG cells F-687 covers flip to OK. `--bip38-passphrase -` and `verify-bundle --ms1 -` flip only if F-687/F-689 widen to them.
4. Commit it with the regenerated blocks; `check_design_tables.py` and T4 are red until then.
5. The Copy gate in `test_plan.py` (§A7) goes red if the new rule leaves a stdin-only input without an exact Copy spelling.

#### A3c. Byte-exact delivery (R1 NI1)

**The target.** The bytes the CLI must end up with are:
- for a typed value, the typed text;
- for `@env:VAR`, `env_value_rule(raw)`, where `raw` is the variable's content.

`env_value_rule` is **one entry** in `channel_policy.json`. Today it is `verbatim`, measured: the CLI's own `@env:` equals argv-exact on every `EnvRef` cell, for all seven endings. It tracks the CLIs' F-687 ruling.

**The terminator.** Each channel cell in `channel_table.json` carries a measured `terminator`, which the GUI appends after the target. The channel's own strip then removes exactly that terminator:
- `\r\n` for the stripping stdin channels. The GUI sends `target + "\r\n"`, and the CLI strips one `\r?\n`, so a target that itself ends in `\n`, `\r\n` or `\r` survives intact.
- `\n` for `--decrypt-password-file`.
- `""` for `EnvRef` today.

**Lenient cells.** A channel that trims whitespace where argv would not has terminator `null`. It may carry only a *clean* value: no CR/LF and no edge whitespace. For a non-clean value the planner drops that channel, and refuses `value-not-byte-exact` if nothing is left. Every lenient mismatch is "argv fails, channel succeeds"; none gives both sides OK with different output.

Endings: '', '\n', '\r\n', '\r', '  ', ' \n', '\nX'; 146 channel cells.

| channel kind | measured terminator | cells |
|---|---|---|
| DashValue | '\r\n' | 42 |
| DashValue | lenient (null) | 9 |
| EnvRef | '' | 53 |
| FileFlag | '\n' | 2 |
| FileFlag | '\r\n' | 2 |
| InFile | '\r\n' | 8 |
| PosDash | '\r\n' | 5 |
| StdinToggle | '\r\n' | 22 |
| StdinToggle | lenient (null) | 3 |

Lenient cells (terminator null; every mismatch is argv-fails/channel-ok, 0 are both-ok-different): `mnemonic convert --from entropy=` DashValue; `mnemonic convert --from xprv=` DashValue; `mnemonic convert --from minikey=` DashValue; `mnemonic inspect --ms1` DashValue; `mnemonic derive-child --from xprv=` DashValue; `mnemonic convert --from wif=` DashValue; `mnemonic xpub-search path-of-xpub --ms1` StdinToggle(--ms1-stdin); `mnemonic xpub-search account-of-descriptor --ms1` StdinToggle(--ms1-stdin); `mnemonic convert --from bip38=` DashValue; `mnemonic slip39 split --from entropy=` DashValue; `mnemonic ms-shares split --from entropy=` DashValue; `mnemonic xpub-search passphrase-of-xpub --ms1` StdinToggle(--ms1-stdin).

With NO terminator (fold 1's delivery), a wrong output at exit 0/4 on 14 cells: `mnemonic addresses --passphrase` StdinToggle(--passphrase-stdin); `mnemonic bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --bip38-passphrase` StdinToggle(--bip38-passphrase-stdin); `mnemonic restore --passphrase` StdinToggle(--passphrase-stdin); `mnemonic derive-child --passphrase` StdinToggle(--passphrase-stdin); `mnemonic silent-payment --passphrase` StdinToggle(--passphrase-stdin); `ms derive --passphrase` StdinToggle(--passphrase-stdin); `mnemonic verify-bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search path-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search account-of-descriptor --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search passphrase-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 split --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 combine --passphrase` StdinToggle(--passphrase-stdin).

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

`channel_table.rs` and `channel_policy.rs` are generated from the two JSON files. A missing table entry is a refusal, on every OS.

#### A4.3. The rule (`plan.py` is its executable form)

0. **Resolve C1** (§A3a), on every OS. Then:
   - an unmeasured source refuses `no-table-entry`;
   - on an OS **not in `private_channels_on`**, stop here and return the **interim plan** (§A6);
   - otherwise drop the channels this source cannot use exactly: fd outside `fd_channel_on`; `EnvRef` for a value containing NUL; lenient channels for a non-clean value (§A3c).
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

Policy: private channels on ['linux'], fd channel on ['linux'], env_value_rule `verbatim` (channel_policy.json).

| shape | Linux plan | macOS | Windows | macOS/Windows once their flag flips |
|---|---|---|---|---|
| addresses phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| restore phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| restore ms1+passphrase | --from ms1= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from ms1= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| derive-child phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| bundle slot+passphrase | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --slot @N.phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| bundle wsh-multi 2 slots | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1 | --slot @N.phrase= ← argv + --allow-argv-secret (interim); --slot @N.phrase= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| bundle wsh-multi 2 slots+passphrase | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.ms1= ← env MNEMONIC_GUI_S1; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --slot @N.phrase= ← argv + --allow-argv-secret (interim); --slot @N.ms1= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| convert phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --from phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| convert wif+bip38-passphrase | --from wif= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin + '\r\n' | --from wif= ← argv + --allow-argv-secret (interim); --bip38-passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| convert bip38+bip38-passphrase | --from bip38= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin + '\r\n' | --from bip38= ← argv + --allow-argv-secret (interim); --bip38-passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search path-of-xpub phrase+passphrase | path-of-xpub --phrase ← stdin via --phrase-stdin + '\r\n'; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | path-of-xpub --phrase ← argv + --allow-argv-secret (interim); path-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search path-of-xpub ms1+passphrase | path-of-xpub --ms1 ← stdin via --ms1-stdin; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | path-of-xpub --ms1 ← argv + --allow-argv-secret (interim); path-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search passphrase-of-xpub phrase+passphrase | passphrase-of-xpub --phrase ← stdin via --phrase-stdin + '\r\n'; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | passphrase-of-xpub --phrase ← argv + --allow-argv-secret (interim); passphrase-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search passphrase-of-xpub ms1+passphrase | passphrase-of-xpub --ms1 ← stdin via --ms1-stdin; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | passphrase-of-xpub --ms1 ← argv + --allow-argv-secret (interim); passphrase-of-xpub --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search account-of-descriptor phrase+passphrase | account-of-descriptor --phrase ← stdin via --phrase-stdin + '\r\n'; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | account-of-descriptor --phrase ← argv + --allow-argv-secret (interim); account-of-descriptor --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| xpub-search account-of-descriptor ms1+passphrase | account-of-descriptor --ms1 ← stdin via --ms1-stdin; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | account-of-descriptor --ms1 ← argv + --allow-argv-secret (interim); account-of-descriptor --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| silent-payment secret+passphrase | --secret ← pipe fd via --secret-file + '\r\n'; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --secret ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | **refuse** (fd-not-on-platform) |
| slip39 split phrase+passphrase | split --from phrase= ← env MNEMONIC_GUI_S0; split --passphrase ← stdin via --passphrase-stdin + '\r\n' | split --from phrase= ← argv + --allow-argv-secret (interim); split --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| slip39 combine 2 shares+passphrase | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1; combine --passphrase ← stdin via --passphrase-stdin + '\r\n' | combine --share ← argv + --allow-argv-secret (interim); combine --share ← argv + --allow-argv-secret (interim); combine --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| slip39 combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | combine --share ← argv + --allow-argv-secret (interim); combine --share ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| seed-xor combine 2 shares | combine --share phrase= ← env MNEMONIC_GUI_S0; combine --share phrase= ← env MNEMONIC_GUI_S1 | combine --share phrase= ← argv + --allow-argv-secret (interim); combine --share phrase= ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms-shares combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | combine --share ← argv + --allow-argv-secret (interim); combine --share ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| import-wallet 2 cosigner ms1 | --ms1 ← env MNEMONIC_GUI_S0; --ms1 ← env MNEMONIC_GUI_S1 | --ms1 ← argv + --allow-argv-secret (interim); --ms1 ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| verify-bundle ms1+slot+passphrase | --ms1 ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1; --passphrase ← stdin via --passphrase-stdin + '\r\n' | --ms1 ← argv + --allow-argv-secret (interim); --slot @N.phrase= ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms combine share group | <shares> ← stdin via one `-` (all, one per line) | <shares> ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms verify phrase+ms1 | --phrase ← stdin via `-` + '\r\n'; <ms1> ← pipe fd via --in + '\r\n' | --phrase ← argv + --allow-argv-secret (interim); <ms1> ← argv + --allow-argv-secret (interim) | same as macOS | **refuse** (fd-not-on-platform) |
| ms derive ms1+passphrase | --passphrase ← stdin via --passphrase-stdin + '\r\n'; <ms1> ← pipe fd via --in + '\r\n' | --passphrase ← argv + --allow-argv-secret (interim); <ms1> ← argv + --allow-argv-secret (interim) | same as macOS | **refuse** (fd-not-on-platform) |
| ms derive phrase+passphrase | **refuse** (two-stdin) | --phrase ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |
| ms derive hex+passphrase | **refuse** (two-stdin) | --hex ← argv + --allow-argv-secret (interim); --passphrase ← argv + --allow-argv-secret (interim) | same as macOS | same as Linux |

The two Linux refusals are `ms derive` with `--phrase`/`--hex` plus `--passphrase`. ms has no second channel for a phrase or hex, so the refusal gives the CLI's own recipe: encode to an ms1 first, then derive from the card.

### A6. What runs where: data, with a gate (R1 NI2)

**Which path each OS runs is data:** `private_channels_on` in `channel_policy.json`, today `["linux"]`.

**On an OS in the list**, the private-channel planner runs (§A4.3).

**On every other OS, the interim path runs:**
1. C1 resolution (§A3a);
2. `no-table-entry` refusal for any unmeasured source;
3. the target bytes of every source on argv, with `--allow-argv-secret`.

That is exactly today's admission path, minus its pass-through of `-` and `@env:`. Nothing typed as `-` or `@env:`, and no unmeasured source, reaches argv literally on any OS. `test_plan.py` checks the interim path's C1 and `no-table-entry` legs on macOS and Windows. The interim invocation *is* the argv baseline, so T3′ holds for it by construction.

**Flipping an OS is one list edit, and it is gated.** `test_plan.py`'s **OS gate** fails unless every OS in `private_channels_on` has a CI job running on that OS with the pinned binaries (`MNEMONIC_BIN` set). The job must run T2, T3′ and T7. Linux passes, through the `schema-mirror` job. A macOS or Windows job is added in the same change that adds the OS to the list.

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
| `EnvRef` | typed | `--passphrase @env:MNEMONIC_GUI_S1` + `# read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1` (fish: `read -sx MNEMONIC_GUI_S1`) |
| `EnvRef` | `$MY_PW` | the user's own `--passphrase @env:MY_PW`. The cell's `EnvRef` is measured exact, so the CLI's own rule yields the target by definition. |
| stdin | typed | the plan's spelling (`--passphrase-stdin`) + `# stdin: type it, then Enter, then Ctrl-D`. The CLI strips the Enter. |
| stdin | `$MY_PW` | `printf '%s<T>' "$MY_PW" \| <command>`, where `<T>` is the cell's terminator as a printf escape (`\r\n`). Measured equal to argv-exact in bash, zsh and fish with a value ending in `\n` (§A3a block). |
| pipe fd | any | `--in <FILE>` + `# FILE: a file holding the ms1 card` |

- **Copy gate:** this `printf` form reproduces the target only while `env_value_rule` is `verbatim`. `test_plan.py` fails if the rule changes while any input lacks an exact `EnvRef` cell.
- **Windows Copy (cmd):** env-bound bindings use `@env:` with `REM set …` lines. A stdin-bound binding disables the Windows Copy, with the tooltip "needs a pipe; use the POSIX copy".
- **When the plan is refused**, Copy is disabled and its tooltip shows the refusal (R0 M3).

### A8. Refusals (the GUI never falls back to argv)

| code | when |
|---|---|
| `C1-dash`, `C1-env-unset`, `C1-env-empty`, `C1-bad-name`, `C1-reserved-name` | §A3a (reserved: any field) |
| `no-table-entry` | an unmeasured source (§A2b), on every OS |
| `value-not-byte-exact` | a non-clean value on an input whose channels are all lenient (§A3c) |
| `no-channel-on-platform` / `fd-not-on-platform` | §A6 |
| `two-stdin`, `no-channel-left` | §A4.3 |
| `pipe-failed`, `payload-too-large` | pipe creation fails; a pipe payload over 4096 bytes |

### A9. Testing: every secret reaches exactly the flag the user filled, as the exact bytes

- **T1: plan property (pure).** Prototype: `test_plan.py`, 0 failures.
  - For each shape, and each combination of a subcommand's secret sources filled with **distinct** sentinels:
    - no argv token contains a sentinel;
    - each binding sits at its own source's flag or slot;
    - each channel carries its own source's target plus terminator;
    - env names are unique, with at most one stdin.
  - **C1 legs:** all seven refusal spellings on every source on **all three OSes** (1260 legs).
  - **Resolution:** under both `env_value_rule` values.
  - **Pass-through guard.**
  - **Interim:** macOS and Windows plans are all `Argv`.
  - **M5:** permutations.
  - **Lenient:** the non-clean refusal.
  - **Bounds:** `payload-too-large`.
- **T2: single-input real-binary equivalence.** Every table cell equals the argv baseline, **with dependence**.
- **T2b: byte fidelity** (`run_bytes.py`). Every channel cell, with the seven endings and its terminator, equals argv-exact (§A3c).
- **T3′: multi-secret real-runner baseline** (`run_plans.py`). Every runnable shape through the real runner equals the argv baseline, with its effect line. Two further legs:
  - **Swap leg:** every source pair swapped must differ from the baseline, except the 4 symmetric pairs.
  - **NI1 leg:** each source typed as `@env:USER_SECRET` with endings `\n`, `\r\n`, `\nX` and trailing spaces must equal argv-exact, **and** the CLI's own `@env:USER_SECRET` wherever that cell's `EnvRef` is OK. Fingerprints and addresses are in the effect lines.
- **T7: runner echo test** (portable, no CLIs). The real runner executes each plan against a helper that **parses its own argv** for `@env:NAME`, `/dev/fd/N` and `-`/`--X-stdin` (R1 Nit), and prints the bytes it received on each. They must equal each source's target plus terminator. This catches runner swaps (symmetric shapes included), an argv/env name mismatch, and a leaked write end (timeout).
- **T4: drift gate.**
  - `channel_table.rs` == the regenerated `channel_table.json` at the pinned binaries, terminators included.
  - The Rust source enumeration == `secret_sources.txt`.
  - Every source has an entry or is in §A2b.
- **T8: plan parity.** The Rust `plan()` == `plans_pure.json` on every shape × OS. `check_design_tables.py` regenerates that file from `plan.py` on every run, so a planner edit that is not re-generated goes red (R1 Nm3).
- **T9: `ms hashlock` phrase fidelity.** With the real `ms`, `"  pad  "` ≠ `"pad"`, and the planned run (`--hashlock-phrase-stdin` + `\r\n`) == argv-exact for `"  pad  "`, `"pad\n"` and `"pad\r\n"`.
- **T10: OS gate** (`test_plan.py`). Every OS in `private_channels_on` has a real-binary CI job on that OS.
- **T5: mutations.** Prototype: `mutations.py` applies 12 planner mutations, and **all 12 are killed**. They are: `-` not refused; reserved, bad-name, unset and empty names allowed; `env_value_rule` ignored; pass-through guard off; interim path off; lenient filter off; step 1 removed; payload bound off; `no-table-entry` off. The runner-side mutations go to T3′ and T7: swap bindings; write stdin from the wrong binding; drop the terminator (NI1, which the T3′ NI1 leg kills); drop the env scrub; leave the write end open; restore the admission.
- **T6: UI.** Preview, the confirm dialog and Copy contain no sentinel, and do contain each binding with its provenance. Copy spellings match §A7. The tutorial J1 modal re-pin is deliberate.
- **Where they run.**
  - T1, T8, T10, the pure T5 half and T7: plain `cargo test`, Linux job. T7 is also the first leg of any macOS or Windows job.
  - T2, T2b, T3′, T9: the `schema-mirror` job, which installs the pinned binaries.

`run_plans.py` is T3′'s prototype. Its result:

| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | source values swapped (i↔j) vs baseline | T1 | NI1: `@env:` + endings == argv-exact / == CLI's own `@env:` |
|---|---|---|---|---|---|---|---|
| addresses phrase+passphrase | 0 | 0 | **yes** | `0  bc1qrm3qju2002wmwly8x2ee7ghdaunexsndwgedwv` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| restore phrase+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| restore ms1+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| derive-child phrase+passphrase | 0 | 0 | **yes** | `target biology midnight canal glass common include trophy glimpse north castle dove` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| bundle slot+passphrase | 0 | 0 | **yes** | `mk1qpd2y2pqqsqk4z99gdzlh7lxqvzg3vs7vs57ls3u2nlnjvzn90ffnjpcsauf2eggmpdquu02l9k7dpjhxhs3yaa` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| bundle wsh-multi 2 slots | 0 | 0 | **yes** | `mk1qpm6gzpqqspdayp9s00fqfvrw0za5zs8qjyty8kskx54hpzzjwjpds9j69su6hyzpkdq32t74e44wnhpg9dj4y5` | 0↔1: differs (exit 0) | ok | 10/10; 10/10 |
| bundle wsh-multi 2 slots+passphrase | 0 | 0 | **yes** | `mk1qpgaqcpqqspywsvg03r5rzrughalhes8qjyty83nr0wscquatny8cq3ctkcnc7w7lklfeq66dhl2ml4aacj89mq` | 0↔1: differs (exit 1); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | 15/15; 15/15 |
| convert phrase+passphrase | 0 | 0 | **yes** | `xpub: xpub6CPUCVp94gpNs3bS1eGiwSWwZMLNmfr2Uo1t5v8YtY4XVoxhUraBH7sRyVfgwSNCxRVpX1bDREtc5Kri` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| convert wif+bip38-passphrase | 0 | 0 | **yes** | `bip38: 6PYP8fdoVaE3ThLmEnYcGo3nJeBqd8PvB7CRvTz3TX5L9ojoPHKCg7QXG6` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| convert bip38+bip38-passphrase | 0 | 0 | **yes** | `wif: KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| xpub-search path-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| xpub-search path-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | 6/6; 5/5 |
| xpub-search passphrase-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| xpub-search passphrase-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | 6/6; 5/5 |
| xpub-search account-of-descriptor phrase+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| xpub-search account-of-descriptor ms1+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | 6/6; 5/5 |
| silent-payment secret+passphrase | 0 | 0 | **yes** | `address:      sp1qq2d73kpx36h7r08gmawe6slzxkntu2tw0as7pkqe0hvv3k38mkvckqsp6dmrwxvd6mumfqj9` | 0↔1: differs (exit 1) | ok | 10/10; 0/0 |
| slip39 split phrase+passphrase | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: differs (exit 1) | ok | 10/10; 10/10 |
| slip39 combine 2 shares+passphrase | 0 | 0 | **yes** | `8ab7aa39cd427130e225ed43c34f5b5a` | 0↔1: **same** (symmetric); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | 15/15; 15/15 |
| slip39 combine 2 shares | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: **same** (symmetric) | ok | 10/10; 10/10 |
| seed-xor combine 2 shares | 0 | 0 | **yes** | `zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong` | 0↔1: **same** (symmetric) | ok | 10/10; 10/10 |
| ms-shares combine 2 shares | 0 | 0 | **yes** | `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ab` | 0↔1: **same** (symmetric) | ok | 10/10; 10/10 |
| import-wallet 2 cosigner ms1 | 0 | 0 | **yes** | `"descriptor": "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aS` | 0↔1: differs (exit 4) | ok | 10/10; 10/10 |
| verify-bundle ms1+slot+passphrase | 0 | 0 | **yes** | `result: ok` | 0↔1: differs (exit 2); 0↔2: differs (exit 2); 1↔2: differs (exit 1) | ok | 15/15; 15/15 |
| ms combine share group | 0 | 0 | **yes** | `entropy: 00000000000000000000000000000000` | n/a (one source) | ok | 0/0; 0/0 |
| ms verify phrase+ms1 | 0 | 0 | **yes** | `OK: round-trip valid (12 words, language=english)` | 0↔1: differs (exit 1) | ok | 10/10; 0/0 |
| ms derive ms1+passphrase | 0 | 0 | **yes** | `master_fingerprint:  45fbfbe6` | 0↔1: differs (exit 1) | ok | 10/10; 0/0 |
| ms derive phrase+passphrase | — | — | refused (expected: refuse) | — | — | — | — |
| ms derive hex+passphrase | — | — | refused (expected: refuse) | — | — | — | — |

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
