# DESIGN — secrets over private channels, and the five unsurfaced forms

**Status:** design only, nothing implemented. **Fold 1** answers R0 (`mnemonic-engrave/design/agent-reports/gui-design-r0.md`, 1C/3I/7M/2N) and the operator's final F-687 ruling. The finding-by-finding map is in `mnemonic-engrave/design/agent-reports/gui-design-fold1.md`.
**Baseline:** mnemonic-gui `gui-followups`, fold 0 at `ec31aed` (on top of v0.62.0 `9f569e1`).
**Measured against the release binaries:** mnemonic 0.104.0, md 0.20.3, ms 0.19.1, mk 0.13.0, installed with `mnemonic-toolkit/scripts/install.sh --no-gui --no-man` (which pins ms 0.19.1). These binaries **predate** the F-687 CLI change (A3a).
**Follow-ups this covers:** `argv-secret-via-private-channels` (Part A), `md-ms-new-subcommands-unsurfaced` (Part B).

---

## Part A — every secret over a private channel

### A1. What is measured, and how to re-run it

Everything is in `design/measurements/secret-channels/`. Every table in Part A is **generated**; `check_design_tables.py` asserts this document still carries each generated block byte for byte.

```sh
export BIN_DIR=<dir holding mnemonic, md, ms, mk>
python3 run_all.py > channels.txt; python3 run2.py > channels2.txt; python3 run3.py > channels3.txt
python3 measure_groups.py && python3 table_build.py   # -> channel_table.json  (the data, §A4)
python3 table.py > table.md                           # -> §A2
python3 run_plans.py                                  # -> plans.json, plans.md (§A5, §A9 evidence)
python3 probe_missing.py                              # -> missing_sources.md (§A2b)
./c1_evidence.sh > c1_evidence.out                    # -> §A3a
python3 check_design_tables.py                        # this document == the generated blocks
```

- **Single-input rows** (`run_all.py`, `run2.py`, `run3.py`):
  - *Baseline:* run with the secret on argv plus `--allow-argv-secret`.
  - *Dependence control:* the same run with a different secret must change stdout (or the exit code). Otherwise the row is invalid.
  - *Channel variants:* each candidate channel, without the opt-in. **OK** = exit code and stdout equal the baseline.
  - Randomised splits are compared after a round trip through `combine`.
  - Fold 1 adds 16 rows (`run3.py`):
    - R0's M2 list: `convert --from bip38=`, `slip39`/`ms-shares split --from entropy=`, `xpub-search passphrase-of-xpub --ms1`, `ms hashlock <ms1>`;
    - the fold-0 "not measured" list: `bundle --slot @N.ms1=`/`@N.wif=`, `addresses --from seedqr=`/`electrum-phrase=`, `import-wallet --decrypt-password`;
    - **verify-bundle against a matching bundle** (R0 Q4; baseline `result: ok`): `--ms1`, `--passphrase`, and `--slot @N.{phrase,entropy,seedqr,ms1}=`.
- **Group row** (`measure_groups.py`): `ms combine` takes its N shares as one group, `-` (all on stdin, one per line) or `--in F`. It is measured against the argv baseline with a dependence control.
- **The channel table** (`table_build.py` → `channel_table.json`): for each input, the list of measured-OK channels, and nothing else. This file is what the Rust table must equal (T4).
- **Schema coverage** (`secret_sources.txt` from `enumerate_sources.rs`, `probe_missing.py`): the 124 secret sources the GUI's schema mirror can produce (definition in §A4.1), enumerated from the mirror. Each one with no table entry was run once to see whether the CLI accepts that input at all (§A2b).
- **Plans** (`plan.py`, `shapes.py`, `run_plans.py`):
  - `plan.py` is the rule, executable (§A4).
  - `shapes.py` lists every multi-secret shape the forms can produce for a measured input: 29 shapes, 60 sources.
  - `run_plans.py` plans each shape for Linux, macOS and Windows, then on Linux runs every plan **through a real runner** (pipes, env, stdin) and compares it with the argv baseline (§A9 T3′).

### A2. The measured single-input table

Legend:
- **OK:** accepted, and gives the baseline output.
- **WRONG (exit N, literal):** exit 0 or 4 with *different* output. The CLI took the spelling as a literal value. The exit code is printed (R0 N1).
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

#### A2b. Schema sources with no table entry

124 schema sources; 82 have a table entry, and 3 table entries are Part B's `ms hashlock`. The other 42 were run once each on argv:

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

The 32 "CLI rejects" rows cost nothing to refuse: the CLI would reject them anyway. The 10 **unmeasured** rows are verify-bundle's keyless-template completion (`--from`), which needs a fixture with a ≥5-byte `--expect-wallet-id` prefix (measured: a 4-byte prefix is refused, exit 4), plus `--slot @N.wif=`. Five of the nine `--from` nodes (`xprv`, `wif`, `bip38`, `electrum-phrase`, `minikey`) are outside the grammar `verify-bundle --help` states (`ms1=`/`phrase=`/`entropy=`/`seedqr=`). **They are refused until measured.** The owning phase is the implementation cycle, before the admission is deleted (§A4.6).

### A3. Findings that shape the design

1. **A generic rule is unsafe on today's binaries.** Some cells are literal, and they fall into two groups:
   - *`-` is the literal passphrase `-`* on every toolkit subcommand whose row is valid, and on `ms derive`. Exit 0, a different wallet.
   - *`@env:` is literal* on `silent-payment` and `ms derive` (again a different wallet), and on `verify-bundle --ms1 -` / `--passphrase -` (exit 4, a false "mismatch" against a matching bundle).

   The planner therefore uses only measured-OK cells. The operator's F-687 ruling makes these cells channels in the next ms and toolkit releases (§A3b).
2. **`ms` has no `@env:`.** `import-wallet --ms1` and `--slot` accept only `@env:`. `xpub-search --ms1` accepts only `--ms1-stdin`.
3. **One stdin per invocation**, enforced by the CLIs (fold 0 combos b2, e).
4. **The CLI refusal is not a safety net.** 0.104.0 runs `import-wallet --ms1`, `seed-xor`/`slip39`/`ms-shares combine --share` on argv with only a warning.
5. **Byte fidelity.** stdin strips exactly one trailing `\r?\n`, and `@env:` is verbatim. The GUI writes a value's exact bytes with no terminator.
6. **`ms combine`'s shares are one group:** one `-`, all shares on stdin one per line; or `--in F` (measured OK).

#### A3a. C1 — a secret field that holds `-` or `@env:VAR`

The operator's final ruling on F-687: `-` reads stdin and `@env:VAR` reads the environment, in both CLIs, on every command. So in a GUI secret field these spellings **mean that channel**. The GUI **never sends those characters as the secret bytes.** Concretely:

| the user typed | the GUI does | why |
|---|---|---|
| `@env:VAR` | Reads `VAR` from the GUI's **own** environment at Run time. The **bytes** go through the channel the plan assigns (§A4), exactly like a typed value, and are zeroized after the run. | Uniform and pin-independent (below). |
| `@env:VAR`, with `VAR` unset or empty | Refuse, naming the field and `$VAR`. | An empty passphrase is a different wallet. The user who wrote `@env:` meant a value. |
| `@env:MNEMONIC_GUI_…` | Refuse. | Those names are the planner's own (R0 C1's collision). |
| `-` | Refuse: "the GUI has no stdin of its own to forward; type the value, or use `@env:VAR`". | `-` means the user's own stdin, which does not exist in the GUI. Passing it through would read the GUI's pipe: the plan's other secret, or nothing. |

This applies to every **secret source** (§A4.1), including a node-valued Text such as `restore --from ms1=-` or `ms1=@env:SEED`, a slot row, a secret positional, and each element of a group.

**Resolution is unconditional.** It does not depend on whether the pinned CLI implements F-687. On today's binaries it is the only safe reading, because passing `@env:` through reaches the literal cells in §A3.1. After the pin bump it is still correct, and it keeps one code path. So **the pin bump is a table edit, not new code**:

- the re-measured `channel_table.json` turns the 15 WRONG cells in §A2 into OK `DashValue`/`EnvRef` cells;
- the planner picks them up as data;
- C1 does not change.

Measured (`c1_evidence.sh`: `restore --from phrase=<abandon…about> --template bip84`, intended passphrase `hunter2`):

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
```

"Reading 1" sends the typed text as the bytes, which gives the wrong wallet. "Reading 2" passes the spelling through, which takes it literally on today's binaries. The design's resolution matches the intended fingerprint. `run_plans.py` repeats this for **every source of every runnable shape**: typed as `@env:USER_SECRET_i`, resolved, planned and run, **56/56 equal the argv baseline**.

**Provenance is shown.** Preview and the confirm dialog list each secret's binding *and its source*, e.g. `--passphrase ← stdin (value of $MY_PW)` or `--from ms1= ← env MNEMONIC_GUI_S0 (typed)`.

**What C1 retires, in the implementing change:**
- the five help strings that teach pass-through: `src/schema/mnemonic.rs:600, 757, 1109, 4033, 4103`. Rewrite them to "type the value, or `@env:VAR` (read by the GUI)";
- kittest **Cell 8** (`tests/kittest_import_wallet_form.rs:326-366`), re-pinned: `@env:MNEMONIC_MS1_0` is resolved GUI-side, and argv carries only the planner's reference;
- `tests/f679_argv_secret_admission.rs` `restore_from_a_secret_node_is_admitted_but_a_private_channel_is_not` (private leg) and `private_channel_sentinels_in_secret_fields_need_no_opt_in`;
- `invocation::is_private_channel_value` and `masked_token_is_private_channel`;
- phase 1a's `restore_from_secret_node.rs::site1_private_channel_sentinel_is_not_masked`, which becomes a C1 test. `secrets::text_value_is_secret_node_token` splits into a node-only classifier (it decides the *source*) plus C1 (which handles the value).

A user whose real secret is literally `-` or begins with `@env:` cannot type it in the GUI; they get the refusal or the resolution. This is **documentation only**, stated in the field help. The CLI remains available to them.

#### A3b. F-687 on the pinned binaries

Until the GUI pins ms and toolkit releases that implement F-687, the CLI's own spelling of `-`/`@env:` is used **only** where the table measured it OK. That is already the planner's rule (§A4): the planner writes channel tokens itself, from OK cells only, and user text never becomes a channel token (§A3a). The bump procedure:
1. re-run §A1;
2. review the diff of `channel_table.json` (expected: the 15 WRONG cells flip to OK);
3. commit it with the regenerated §A2/§A5 blocks. `check_design_tables.py` and T4 fail until that is done.

### A4. The mechanism

#### A4.1. Secret source (R0 M1)

A **secret source** is any one of:
- a schema flag with `secret: true`, or in `SECRET_FLAG_NAMES`, whose value is Text;
- a `--slot` row whose subkey is in `SECRET_SLOT_SUBKEYS`;
- a `NodeValueComposite` value whose node is in `SECRET_NODE_TYPES_ARGV`;
- a plain Text value `<node>=<v>` whose node is in `SECRET_NODE_TYPES_ARGV`, **whatever `<v>` is** (C1 handles `-`/`@env:`);
- a secret positional, as one source or, for `ms combine`, one **group** source;
- the hand-marked `ms hashlock` inputs (Part B).

`secret_sources.txt` is this definition applied to today's mirror. T4 regenerates it in Rust.

#### A4.2. The plan

`form::channels::plan(schema, sub, state, user_env, platform) -> Result<RunPlan, ChannelRefusal>`:

```text
RunPlan { argv:  Vec<String>,                       // no secret byte, ever
          mask:  Vec<bool>,                         // unchanged meaning
          stdin: Option<Zeroizing<Vec<u8>>>,
          env:   Vec<(String, Zeroizing<String>)>,  // MNEMONIC_GUI_S<i>
          fds:   Vec<(RawFd, Zeroizing<Vec<u8>>)>,  // Linux: inherited pipe, argv says /dev/fd/<n>
          bindings: Vec<Binding> }                  // (source, channel, argv index, provenance)
```

`channel_table.rs` is data generated from `channel_table.json`: `(cli, subcommand, input) → [Channel]`, where `Channel ∈ {EnvRef, StdinMulti, StdinToggle(flag), DashValue, PosDash, FileFlag(flag), InFile}`, measured-OK cells only. **A missing entry is a refusal**, never a fallback to argv.

#### A4.3. The rule (authoritative; `plan.py` is its executable form, R0 I1)

0. **Resolve** (§A3a): `@env:VAR` becomes bytes; `-` and bad names are refused. Then drop the channels the platform lacks (fd outside Linux, §A6), and drop `EnvRef` for a value containing NUL, which an env var cannot carry. A source left with no channel is refused.
1. **Forced stdin.** Take the sources whose every remaining channel is a stdin channel. Two or more of them → refuse `two-stdin`. One → it takes stdin.
2. **Stdin toggle.** If stdin is still free, the first source in argv order that has a `--X-stdin` toggle takes stdin through it. These read raw bytes, NUL-preserving; this is the passphrase-class channel.
3. **The rest,** in argv order: `EnvRef` (a unique `MNEMONIC_GUI_S<i>`); else stdin, if still free; else a pipe fd; else refuse `no-channel-left`.

Within a class, the preference order is `StdinMulti > StdinToggle > DashValue > PosDash` and `FileFlag > InFile`. Step 1 comes first **regardless of source order** (R0 M5): `silent-payment` plans the same whichever of `--secret`/`--passphrase` comes first in the schema. T1 checks this by permuting the order.

#### A4.4. Env hygiene

Before spawning, the runner removes every inherited `MNEMONIC_GUI_S*`, then sets exactly the plan's variables. Names match `[A-Z0-9_]`, and the argv token carries the exact name.

#### A4.5. Stdin toggles and tree mode

The `*-stdin` toggles stay rendered disabled. Only the planner emits them.

`build-descriptor` has **no** secret flag (checked: `BUILD_DESCRIPTOR_FLAGS`). Tree-mode keys travel inside the spec JSON on stdin (`--spec -`), so tree mode has no argv secret source. `--spec -` is modelled as a pre-bound stdin, so a future stdin-only source there refuses.

#### A4.6. Deleting `--allow-argv-secret` admission

`admit_argv_secret_for_run` is deleted **per OS** (§A6), once the §A2b unmeasured rows are measured or accepted as refusals. The flag stays mirrored and hidden.

### A5. Plans, generated from the rule (R0 I1)

`plan.py` applied to every shape in `shapes.py`. Generated by `run_plans.py`; do not hand-edit. The Rust planner must produce these on every shape (T8).

| shape | Linux plan | macOS | Windows |
|---|---|---|---|
| addresses phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| restore phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| restore ms1+passphrase | --from ms1= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| derive-child phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| bundle slot+passphrase | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| bundle wsh-multi 2 slots | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| bundle wsh-multi 2 slots+passphrase | --slot @N.phrase= ← env MNEMONIC_GUI_S0; --slot @N.ms1= ← env MNEMONIC_GUI_S1; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| convert phrase+passphrase | --from phrase= ← env MNEMONIC_GUI_S0; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| convert wif+bip38-passphrase | --from wif= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin | same as Linux | same as Linux |
| convert bip38+bip38-passphrase | --from bip38= ← env MNEMONIC_GUI_S0; --bip38-passphrase ← stdin via --bip38-passphrase-stdin | same as Linux | same as Linux |
| xpub-search path-of-xpub phrase+passphrase | path-of-xpub --phrase ← stdin via --phrase-stdin; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search path-of-xpub ms1+passphrase | path-of-xpub --ms1 ← stdin via --ms1-stdin; path-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search passphrase-of-xpub phrase+passphrase | passphrase-of-xpub --phrase ← stdin via --phrase-stdin; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search passphrase-of-xpub ms1+passphrase | passphrase-of-xpub --ms1 ← stdin via --ms1-stdin; passphrase-of-xpub --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search account-of-descriptor phrase+passphrase | account-of-descriptor --phrase ← stdin via --phrase-stdin; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| xpub-search account-of-descriptor ms1+passphrase | account-of-descriptor --ms1 ← stdin via --ms1-stdin; account-of-descriptor --passphrase ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| silent-payment secret+passphrase | --secret ← pipe fd via --secret-file; --passphrase ← stdin via --passphrase-stdin | **refuse** (fd-not-on-platform) | **refuse** (fd-not-on-platform) |
| slip39 split phrase+passphrase | split --from phrase= ← env MNEMONIC_GUI_S0; split --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| slip39 combine 2 shares+passphrase | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1; combine --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| slip39 combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| seed-xor combine 2 shares | combine --share phrase= ← env MNEMONIC_GUI_S0; combine --share phrase= ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| ms-shares combine 2 shares | combine --share ← env MNEMONIC_GUI_S0; combine --share ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| import-wallet 2 cosigner ms1 | --ms1 ← env MNEMONIC_GUI_S0; --ms1 ← env MNEMONIC_GUI_S1 | same as Linux | same as Linux |
| verify-bundle ms1+slot+passphrase | --ms1 ← env MNEMONIC_GUI_S0; --slot @N.phrase= ← env MNEMONIC_GUI_S1; --passphrase ← stdin via --passphrase-stdin | same as Linux | same as Linux |
| ms combine share group | <shares> ← stdin via one `-` (all, one per line) | same as Linux | same as Linux |
| ms verify phrase+ms1 | --phrase ← stdin via `-`; <ms1> ← pipe fd via --in | **refuse** (fd-not-on-platform) | **refuse** (fd-not-on-platform) |
| ms derive ms1+passphrase | --passphrase ← stdin via --passphrase-stdin; <ms1> ← pipe fd via --in | **refuse** (fd-not-on-platform) | **refuse** (fd-not-on-platform) |
| ms derive phrase+passphrase | **refuse** (two-stdin) | same as Linux | same as Linux |
| ms derive hex+passphrase | **refuse** (two-stdin) | same as Linux | same as Linux |

The two refusals are `ms derive` with `--phrase`/`--hex` **and** `--passphrase`. ms has no second channel for a phrase or hex (`--in` reads only an ms1), so the refusal gives the CLI's own recipe: encode to an ms1 first, then derive from the card.

### A6. What runs where (R0 I3, Q3, M4, N2)

| channel | Linux | macOS | Windows |
|---|---|---|---|
| env, stdin | yes | yes, **after** its real-binary job is green; until then, today's admission path | same as macOS |
| pipe fd (`/dev/fd/N`) | yes (measured) | **refused** until a macOS real-binary job measures `ms --in /dev/fd/N` and `--secret-file /dev/fd/N` | **refused** (no `/dev/fd`). **No temp files**, so N2's sweep race and ACL claim go away |

The three fd shapes (`silent-payment` secret+passphrase, `ms verify` phrase+ms1, `ms derive` ms1+passphrase) refuse `fd-not-on-platform` outside Linux, with the CLI's own recipe. Follow-ups:
- a Windows named pipe (`\\.\pipe\…` opened by path), which might avoid disk;
- the macOS and Windows real-binary CI jobs (running T2, T3′ and T7), owned by the implementation cycle.

**Pipe mechanics** (R0 M4). The following apply per pipe:
- It is created `O_CLOEXEC` on both ends (`pipe2`).
- **The whole payload is written and the write end closed before spawn.** Payloads are refused above 4096 bytes, the POSIX minimum pipe capacity, so the write never blocks. Secrets are ≪ 4 KiB; a group goes over stdin, not fd.
- The read end is mapped into the child with `command-fds`, which handles the `dup2(n, n)` no-op that would otherwise leave CLOEXEC set.
- A leaked write end would stop the child seeing EOF. T7 covers this with a timeout, because the synchronous runner has no cancel button.

`run_plans.py` does exactly this, and every fd plan equals its baseline.

### A7. What the user sees

- **Preview and the confirm dialog** show the planned argv (no secret, no `--allow-argv-secret`) and one line per binding with its provenance (§A3a). Values are never shown.
  - **Q1 (adopted): the confirm dialog stays.** It is where a user catches a channel they did not expect. Its first sentence changes from "passes secret-bearing arguments to" to "sends these secrets privately to".
- **Copy command** emits the plan's spellings. It adds one comment line per binding:
  - POSIX: `# stdin: the ms1 card — paste it, then Ctrl-D`; `# read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1` (fish: `read -sx MNEMONIC_GUI_S1`).
  - A binding resolved from `$MY_PW` is written `# export MNEMONIC_GUI_S1="$MY_PW"`.
  - A pipe fd becomes `--in <FILE>` with `# FILE: a file holding the ms1 card`.
  - Windows: `REM` lines.

  The same comment lines go on the tree-mode `printf` pipeline if it ever gains an env-bound source (R0 M7; it has none today, §A4.5).
- **When the plan is refused** (R0 M3), Copy is **disabled** and its tooltip shows the refusal text. It never synthesises an unmeasured spelling.

### A8. Refusals (the GUI never falls back to argv)

| code | when |
|---|---|
| `C1-dash`, `C1-env-unset`, `C1-env-empty`, `C1-reserved-name` | §A3a |
| `no-table-entry` | a source with no measured channel (§A2b) |
| `no-channel-on-platform` / `fd-not-on-platform` | §A6 |
| `two-stdin`, `no-channel-left` | §A4.3 |
| `pipe-failed`, `payload-too-large` | pipe creation fails; a pipe payload over 4096 bytes |

### A9. Testing: every secret reaches exactly the flag the user filled

- **T1: plan property (pure).** For every shape in `shapes.py`, and for every combination of each subcommand's secret sources, filled with **distinct** sentinels:
  - no argv token contains a sentinel;
  - each binding's argv index is its own source's flag or slot;
  - each channel carries its own source's sentinel;
  - env names are unique, with at most one stdin;
  - a missing entry refuses.

  Two further legs:
  - **C1:** each source typed `-` refuses; typed `@env:X` with `X` set, it resolves to X's value and provenance `$X`.
  - **M5:** every shape with its sources permuted plans iff the original does.
- **T2: single-input real-binary equivalence** (gated on `*_BIN`). Every table cell: planned == argv baseline, **and** a different secret changes the output (without the dependence leg, the WRONG cells would pass).
- **T3′: multi-secret real-runner baseline** (R0 I2). Every runnable shape in `shapes.py` goes `plan()` → the real `runner` → exit and stdout **equal to the argv + `--allow-argv-secret` baseline**, with the effect line reported (fingerprint, address, xpub, verdict). Then, for every source pair, the values are **swapped** and the run must differ from the baseline, except for the listed symmetric pairs.
  - *How a swapping runner fails it:* a runner that writes source *i*'s bytes into source *j*'s channel produces exactly the swapped invocation. That is measured to differ from the baseline on **all 28 asymmetric pairs**, so T3′'s equality leg fails. The swap leg proves the equality leg can fail.
  - *Where a swap is output-invisible:* 4 pairs (the two shares of `slip39 combine` in both shapes, `seed-xor combine`, `ms-shares combine`). Combining shares is order-independent, so a swap there cannot change the result. The runner's wiring is still pinned for them by T1 (plan level) and T7 (runner level).
- **T7: runner echo test** (new; portable, no CLIs). The real runner executes each plan against a tiny helper binary. The helper prints, per binding, the bytes it received on the named env var, stdin or fd N. Each must equal its own source's sentinel. This catches any runner-level swap on every shape, symmetric ones included. It also catches a leaked write end, through its timeout.
- **T4: drift gate.**
  - `channel_table.rs` == `channel_table.json` regenerated at the pinned binaries.
  - The Rust source enumeration == `secret_sources.txt`.
  - Every source either has an entry or is listed in §A2b with its reason.
  - A pin bump that changes a cell fails T4 until the table is re-committed (§A3b).
- **T8: plan parity.** The Rust `plan()` gives `plans.json`'s bindings on every shape and platform.
- **T5: mutations,** each of which must turn a named test red:
  - swap two bindings in the runner → T3′, T7;
  - write stdin from the wrong binding → T3′, T7;
  - use `DashValue` for a passphrase on today's pins → T2 dependence;
  - drop the env scrub → T7 with a pre-set `MNEMONIC_GUI_S0`;
  - pass a user-typed `@env:` through instead of resolving it → T1 C1 leg and the re-pinned Cell 8;
  - remove step 1 → T1 M5 leg (`silent-payment` reversed refuses);
  - leave the pipe write end open → T7 timeout;
  - restore the admission → an assertion that it never appears in a planned argv.

  The fold-0 "Windows temp file" mutation is **removed**: there is no temp file, and it could never run (R0 I3).
- **T6: UI.** Preview, the confirm dialog and Copy contain no sentinel, and do contain each binding line with its provenance. The tutorial J1 modal re-pin is deliberate.
- **Where they run.**
  - T1, T4 (table half), T7, T8 and the unit legs of T5: plain `cargo test` on the Linux job.
  - T7 is also the first leg of the proposed macOS and Windows jobs.
  - T2 and T3′: the `schema-mirror` job, which installs the pinned binaries.

  `run_plans.py` is the prototype of T3′, and its result is:

| shape | baseline exit | planned exit | planned == baseline | effect (baseline) | source values swapped (i↔j) vs baseline | T1 | C1: `-` refused; `@env:VAR` resolved run == baseline |
|---|---|---|---|---|---|---|---|
| addresses phrase+passphrase | 0 | 0 | **yes** | `0  bc1qrm3qju2002wmwly8x2ee7ghdaunexsndwgedwv` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| restore phrase+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| restore ms1+passphrase | 0 | 0 | **yes** | `master fingerprint: 45fbfbe6  (passphrase: applied)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| derive-child phrase+passphrase | 0 | 0 | **yes** | `target biology midnight canal glass common include trophy glimpse north castle dove` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| bundle slot+passphrase | 0 | 0 | **yes** | `mk1qpd2y2pqqsqk4z99gdzlh7lxqvzg3vs7vs57ls3u2nlnjvzn90ffnjpcsauf2eggmpdquu02l9k7dpjhxhs3yaa` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| bundle wsh-multi 2 slots | 0 | 0 | **yes** | `mk1qpm6gzpqqspdayp9s00fqfvrw0za5zs8qjyty8kskx54hpzzjwjpds9j69su6hyzpkdq32t74e44wnhpg9dj4y5` | 0↔1: differs (exit 0) | ok | ok; 2/2 |
| bundle wsh-multi 2 slots+passphrase | 0 | 0 | **yes** | `mk1qpgaqcpqqspywsvg03r5rzrughalhes8qjyty83nr0wscquatny8cq3ctkcnc7w7lklfeq66dhl2ml4aacj89mq` | 0↔1: differs (exit 1); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | ok; 3/3 |
| convert phrase+passphrase | 0 | 0 | **yes** | `xpub: xpub6CPUCVp94gpNs3bS1eGiwSWwZMLNmfr2Uo1t5v8YtY4XVoxhUraBH7sRyVfgwSNCxRVpX1bDREtc5Kri` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| convert wif+bip38-passphrase | 0 | 0 | **yes** | `bip38: 6PYP8fdoVaE3ThLmEnYcGo3nJeBqd8PvB7CRvTz3TX5L9ojoPHKCg7QXG6` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| convert bip38+bip38-passphrase | 0 | 0 | **yes** | `wif: KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search path-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search path-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | ok; 2/2 |
| xpub-search passphrase-of-xpub phrase+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search passphrase-of-xpub ms1+passphrase | 0 | 0 | **yes** | `match: m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | ok; 2/2 |
| xpub-search account-of-descriptor phrase+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| xpub-search account-of-descriptor ms1+passphrase | 0 | 0 | **yes** | `match: cosigner @0  m/84'/0'/0'  (template=bip84, account=0)` | 0↔1: differs (exit 2) | ok | ok; 2/2 |
| silent-payment secret+passphrase | 0 | 0 | **yes** | `address:      sp1qq2d73kpx36h7r08gmawe6slzxkntu2tw0as7pkqe0hvv3k38mkvckqsp6dmrwxvd6mumfqj9` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| slip39 split phrase+passphrase | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| slip39 combine 2 shares+passphrase | 0 | 0 | **yes** | `8ab7aa39cd427130e225ed43c34f5b5a` | 0↔1: **same** (symmetric); 0↔2: differs (exit 1); 1↔2: differs (exit 1) | ok | ok; 3/3 |
| slip39 combine 2 shares | 0 | 0 | **yes** | `00000000000000000000000000000000` | 0↔1: **same** (symmetric) | ok | ok; 2/2 |
| seed-xor combine 2 shares | 0 | 0 | **yes** | `zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong` | 0↔1: **same** (symmetric) | ok | ok; 2/2 |
| ms-shares combine 2 shares | 0 | 0 | **yes** | `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ab` | 0↔1: **same** (symmetric) | ok | ok; 2/2 |
| import-wallet 2 cosigner ms1 | 0 | 0 | **yes** | `"descriptor": "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aS` | 0↔1: differs (exit 4) | ok | ok; 2/2 |
| verify-bundle ms1+slot+passphrase | 0 | 0 | **yes** | `result: ok` | 0↔1: differs (exit 2); 0↔2: differs (exit 2); 1↔2: differs (exit 1) | ok | ok; 3/3 |
| ms combine share group | 0 | 0 | **yes** | `entropy: 00000000000000000000000000000000` | n/a (one source) | ok | ok; 1/1 |
| ms verify phrase+ms1 | 0 | 0 | **yes** | `OK: round-trip valid (12 words, language=english)` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| ms derive ms1+passphrase | 0 | 0 | **yes** | `master_fingerprint:  45fbfbe6` | 0↔1: differs (exit 1) | ok | ok; 2/2 |
| ms derive phrase+passphrase | — | — | refused (expected: refuse) | — | — | — | ok; n/a |
| ms derive hex+passphrase | — | — | refused (expected: refuse) | — | — | — | ok; n/a |

### A10. Follow-ups (not this design)

- **Engrave F-687** (operator-ruled, being implemented in ms and the toolkit). Add this fold's evidence: `verify-bundle --ms1 -` / `--passphrase -` give exit 4 "mismatch" against a matching bundle.
- **Toolkit:** argv secrets not refused on `import-wallet --ms1`, `seed-xor`/`slip39`/`ms-shares combine --share`.
- **GUI:** the macOS and Windows real-binary jobs; a Windows named-pipe measurement; the §A2b unmeasured rows (verify-bundle template completion).

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
- **Byte-verbatim phrase** (measured: `"  pad  "` over `--hashlock-phrase-stdin` gives the same digest as on argv, and a different digest from `"pad"`). The widget must not trim. **T9**, added to A9: the real `ms`, `"  pad  "` ≠ `"pad"`, and the planned run == the argv baseline.
- **Q2 (adopted): build after Part A.** The phrase's only private channel is Part A's `StdinToggle` case.

### B6. Shared work for all five

- **Schema mirror.** Mirroring them brings them under the flag-name gate automatically (the gate walks only mirrored subcommands). The mirror hand-marks `--hashlock-phrase`, `--hex` and the `<ms1>` positional as secret.
- **Snapshots.** Five new form PNGs, and the gallery census goes from 61 to 66.
- **Manual.** The GUI manual gains five form pages. That is a toolkit-repo change (paired PR).

---

## Decisions (R0 recommendations adopted)

- **Q1:** keep the confirm dialog for fully private runs, reworded (§A7).
- **Q2:** `ms hashlock` after Part A (§B5).
- **Q3:** no Windows temp files; the three fd shapes refuse on Windows (and on macOS until measured) (§A6).
- **Q4:** verify-bundle is measured against a matching bundle (§A2, `run3.py`), and so is every M2 input. The remaining 10 rows (§A2b) are measured in the implementation cycle before the admission is deleted. Until then they refuse.
