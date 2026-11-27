# Review r1: address-of-seed secret-channel measurement (mnemonic-gui `0e3d5dc..ae16266`)

Reviewer: independent adversarial review (Claude Opus 5.5), 2026-11-27.
Scope: `git diff 0e3d5dc..ae16266` on branch `address-of-seed-gui` (draft PR bg002h/mnemonic-gui#61).
Binaries: `/home/user/bins/pr230/` (mnemonic = local build of toolkit PR #230; md/ms/mk as found there).

## What I ran

- `BIN_DIR=/home/user/bins/pr230 python3 regen_check.py --no-enumerate --plans`: **GREEN** (whole pipeline re-derived and diffed against the committed caches, plus `test_plan.py` and `run_plans.py`).
- `python3 check_design_tables.py`: stale blocks none, `test_plan.py` 0 failures.
- `cargo test --locked --test xpub_search_address_of_seed --test secret_channels_t1 --test secret_channels_t2 --test secret_channels_t3prime` with the four `*_BIN` set to the pr230 dir: 18 + 2 + 1 + 13 passed, 0 failed.
- I ran the verb by hand with each spelling and contrast value, ran the 2-seed shape in both seed orders, ran `bundle --slot @0.wif=` under a template and under a descriptor, ran `derive-child` with the old depth-3 xprv and the new master xprvs, and downloaded the md 0.21.0 release assets (REST API) to hash the `md` binaries inside them.

## Critical

None.

## Important

### I1. `measured_with.json` now records the wrong md 0.21.0 asset, and the FOLLOWUP's explanation of this is false

`design/measurements/secret-channels/measured_with.json` (md `sha256` 52c462aa… -> ef04e916…); `FOLLOWUPS.md` (new "Still owed at the release tag" bullet: "`md` 0.21.0's release asset sha256 (`ef04e916…`) differs from the one `measured_with.json` records (`52c462aa…`) at the same version string; that identity check is red on the current base too").

The descriptor-mnemonic release `descriptor-mnemonic-md-cli-v0.21.0` ships two x86_64 Linux md builds, and both hashes are genuine:

- `md-0.21.0-linux-amd64.tar.gz` (glibc) contains `md` with sha256 `52c462aa64da9f61…`, the value the base recorded;
- `md-0.21.0-x86_64-linux-musl.tar.gz` contains `md` with sha256 `ef04e9161939c203…`, the binary in `/home/user/bins/pr230/md` and the value this diff records.

CI's `install-pinned-release-binaries` step runs the toolkit installer pinned at `5da5c780…`. Its `asset_for` maps md on `linux-x86_64-gnu`, which is what ubuntu-latest is, to `md-$v-linux-amd64.tar.gz`. Its comment says "x86_64 Linux ships only a glibc build (md-<v>-linux-amd64), so a musl x86_64 host builds md from source." So CI installs the 52c462aa binary. The base's record was right for CI. It was red only on this cloud box, because the box has the musl asset.

Failure scenario: the pin bump follows the FOLLOWUP's own recipe ("re-run `BIN_DIR=<pinned> regen_check.py --plans`") with this box's bin dir. measured_with.json keeps ef04e916, and CI's T4 identity check (`secret-channels-regen-check`) then fails on md even after the toolkit tag is right. The FOLLOWUP text also tells the next person that this red is a pre-existing upstream anomaly, which invites them to bless whatever hash they happen to have.

Fix:
- Restore md's sha256 to `52c462aa…`.
- Correct the FOLLOWUP sentence: two assets exist and the installer picks the glibc one on CI.
- Say that the cloud box's `md` is the musl asset, so a local `regen_check` here is red on md identity by construction. Alternatively, measure with the asset that CI installs.

## Minor

### M1. The new tests never check the resolved value of an `@env:` source

`tests/xpub_search_address_of_seed.rs`, `aos_each_source_alone_plans_or_refuses_dash` (the env closure gives `AOS_SET` = `"value"`) and `assert_private` / `SECRETS` (`const SECRETS = [PHRASE, PHRASE2, MS1, ENTROPY, SEEDQR, PASSPHRASE]`).

For the seven `@env:AOS_SET` states, the secret the GUI resolves and delivers is `"value"`, and `"value"` is not in `SECRETS`. This makes two of the assertions vacuous for every env-sourced row:

- On Linux, "no secret byte on argv" passes even if the resolved bytes went onto argv.
- On macOS/Windows, the interim-path check (`if SECRETS…contains → assert masked`) passes even if the resolved bytes are on argv unmasked.

Failure scenario: a regression that puts a user-env-resolved value on argv unmasked would still pass this file. It would not reach a user, because the generic T1/T3' legs cover env provenance per table entry. But this file's doc comment claims the property for "every source … in each of its spellings".

Fix: use a distinctive env value such as `"aos-env-secret-xyz"` and add it to the checked set for those rows.

### M2. The Windows Copy text is never checked

`tests/xpub_search_address_of_seed.rs`: `aos_run_and_copy_plan_private_channels` (`let (posix, _windows) = …`) and the non-dash branch of `aos_each_source_alone_plans_or_refuses_dash`.

Only the POSIX recipe is checked for secrets. The Windows recipe (different quoting and renderer) is dropped, so a leak in the Windows Copy rendering for this verb would pass. Fix: apply the same `text()` and `!SECRETS…` assertion to `windows`.

### M3. Wrong comments about the contrast values

- `design/measurements/secret-channels/cases.py` ("the contrast value finds no wallet (exit 4)").
- `cases3.py` ("the contrast seed finds no wallet").

The `--ms1` and `--from ms1=` contrast (`…q34v7g`) does not reach a wallet. It fails the codex32 checksum ("invalid short checksum", exit 1); it is not auto-repaired into the same wallet, which I checked. The `--from seedqr=` contrast `000100…0003` exits 1 with "not a valid SeedQR". Only phrase, entropy and passphrase reach exit 4.

`depends=True` still holds for every row, and the OK verdicts still compare the full masked stdout, which carries the 73c5da0a fingerprint, so the measurement is sound. Only the comments are false. Fix: correct the comments. Optionally, use a valid other-seed ms1 or SeedQR so the contrast is "different wallet" rather than "parse error", as the phrase rows do.

### M4. The FOLLOWUP's older bullet still describes the pre-measurement state

`FOLLOWUPS.md`, bullet "Owed at the pin bump", which this diff leaves unchanged. It still says:

- every secret-bearing Run and Copy "refuses (`no-table-entry`, fail closed, pinned … by `aos_run_and_copy_fail_closed_until_measured` and `aos_each_source_alone_fails_closed`)";
- the seven sources sit in `missing_sources.txt`/`.md` "as **unmeasured**".

Both tests were renamed or replaced, and the rows left `missing_sources`. The new bullets are correct, but the old one reads as current state.

Fix: mark that bullet as history, or trim it to what is still owed.

### M5. The FOLLOWUP understates what is red in CI until the tag

The new "Still owed at the release tag" bullet names only the T4 identity check as red until the bump. At this commit, CI (pinned v0.107.1 release binaries) also fails on:

- `wireshape_path_of_xpub_match` (161 vs 140);
- T2 and T3' (`secret_channels_t2` / `t3prime`): the seven address-of-seed table keys give exit 64 at v0.107.1, so the dependence checks fail;
- T4's re-derivation diff (the address-of-seed keys disappear, and `bundle --slot @N.wif=` comes back as measurable at v0.107.1).

This is expected for a draft, but the list of what must turn green at the bump should be complete.

## Nit

- **N1.** `missing_sources.md` labels `bundle --slot @N.wif=` "**unmeasured**". That is because `probe_missing.py`'s REJECTS patterns do not match "is refused"; it is really a CLI refusal. The §A2b prose then says "no channel to measure … They refuse on every OS until measured", which is self-contradictory for that row. Fail-closed either way: the GUI refuses with `no-table-entry` on every OS before the interim branch (`src/form/channels/mod.rs:455`), and Copy is disabled. The cost is UX only: the user sees "input not measured" instead of the toolkit's reason. I checked that the toolkit refuses wif under `--template` and under `--descriptor` (`wpkh(@0/<0;1>/*)`: "not supported in descriptor mode").
- **N2.** The `shapes.py` comment and the FOLLOWUP say the 2-seed shape's "second `--from` takes `@env:`". `plans.json` shows that on Linux both sources bind `EnvRef` (`MNEMONIC_GUI_S0`, `S1`); the first does not take stdin. The statement is true but implies something else.
- **N3.** `assert_private` zips `p.argv` with `p.mask`. If `mask` were ever shorter than `argv`, the tail would go unchecked. An `assert_eq!(p.argv.len(), p.mask.len())` would close that.

## Answers to the five questions

1. **Masking cannot make a wrong-secret or wrong-wallet run compare equal. The Python and Rust masks are equivalent.**
   - What is masked: only the token after `--slot @N.<kind>=` and after `--passphrase ` (followed by a space), and only on lines whose trimmed form starts with `mnemonic bundle `.
   - What stays visible: the `<kind>`, the descriptor with its key-origin fingerprints (`[73c5da0a/84h/0h/0h]`), the `slots:` / `slot @N:` lines with fingerprints, `match:`, the path and the exit code.
   - What the toolkit prints in the masked position: it never prints the secret there. A literal on argv prints as `-` or `@env:MNEMONIC_PASSPHRASE`, and the other channels print their own spelling, which I checked for each.
   - Wrong passphrase: it changes the fingerprint (45fbfbe6 vs 73c5da0a).
   - Swapped seeds in the 2-of-2 shape: the descriptor and slot lines change (I ran both orders), and `symmetric: []` makes `run_plans` expect the difference.
   - Effect on the NC1 and re-read detectors: masking makes them more sensitive, not less.
   - Python `str.strip`/`splitlines` vs Rust `trim`/`lines`: they differ only on exotic control characters (`\x1c`-`\x1f`, `\x0b`, `\x85`, U+2028), which are absent from this output.
2. **`depends=True` holds for all seven rows (`channels.json`, `channels3.json`), checked against the binary.** The ms1 contrast is a checksum failure (exit 1), not a silent auto-repair, so no row is meaningless. Only the comments describing the contrasts are wrong (M3).
3. **The derive-child fixture change is correct.**
   - The depth-3 `XPRV` is refused: "the xprv/tprv is at depth 3, not a master key".
   - `XPRV_M` is the abandon seed's master: `derive-child` from it equals `derive-child` from the phrase ("prosper short ramp …").
   - `XPRV_M2` (zoo) derives a different child.

   **Dropping `bundle --slot @N.wif=` is correct and fails closed in the GUI.** The planner refuses `no-table-entry` on every OS before the interim branch, and Copy is disabled. No GUI path treats the row as runnable.
4. **The flipped tests assert the core properties, with gaps.** The tests check: Linux not interim and no literal secret on argv; interim tokens holding a literal secret are masked; `-` refuses with exactly `C1-dash`; POSIX Copy text holds no literal secret. Gaps: env-resolved values are never checked (M1), and the Windows Copy text is never checked (M2). I found no secret from the form reachable on argv or in Copy text unmasked in practice: the tests pass, and so do `regen_check --plans` and T1, T2 and T3'.
5. **False or stale text:** the md sha256 explanation (I1), the stale "Owed at the pin bump" bullet (M4), the incomplete red-until-tag list (M5), and the contrast comments (M3). These claims check out:
   - leg counts 1260 -> 1386 and 3717 -> 4095 (T1 passes);
   - the per-source channel verdicts;
   - the derive-child refusal;
   - toolkit `f89d7167` exists ("xpub-search searches the paths bundle now mints");
   - the "17 -> 11 unmeasured rows" arithmetic;
   - N3, the printed `@env:MNEMONIC_GUI_S<n>` line the user cannot run.

**Verdict:** 0 Critical, 1 Important (I1), 5 Minor, 3 Nit.
