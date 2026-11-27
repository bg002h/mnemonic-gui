# Review r1: mnemonic-gui `xpub-search address-of-seed` mirror (3f94ece on 96ea304)

Reviewer: independent adversarial review (secrets focus). Scope: the whole diff
`96ea304..3f94ece` on branch `address-of-seed-gui` (worktree `/home/user/wt-gui-aos`),
against toolkit PR #230 (`/home/user/wt-address-search`, head `eb8658b9`, SPEC
`design/SPEC_xpub_search_address_of_seed.md` §2, §6-§8) and the pinned toolkit v0.107.1
(`/home/user/wt-tk-pin`). No source was modified.

**Counts: 0 Critical, 2 Important, 3 Minor, 4 Nit.**

## What was verified and holds

- **Schema vs `gui-schema` of the #230 binary: exact.** 25 flags on both sides with identical
  names. `required` only on `--target-address`. `secret: true` on exactly the six flags
  gui-schema marks (`--phrase`, `--phrase-stdin`, `--ms1`, `--ms1-stdin`, `--passphrase`,
  `--passphrase-stdin`). Defaults match (`--language english`, `--network mainnet`,
  `--min-account 0`, `--number-of-accounts 20`, `--gap-limit 20`; none on `--max-account`).
  `--no-auto-repair` and `--allow-argv-secret` are `global: true`. The dropdown values match
  `LANGUAGES` and `NETWORKS` in order. Repeatable flags match the clap `Vec<String>` fields
  (`--target-address`, `--from`, `--descriptor-from`, `--cosigner`, `--add-path`;
  `address_of_seed.rs:63-140`). The `--gap-limit` max of 2^31 matches §2. `--descriptor-file`
  without a stdio sentinel matches `read_to_string(f)`. Leaving the positional `MS1` out
  follows the sibling verbs, which also have an upstream repeating positional and
  `NO_POSITIONALS`.
- **Run and Copy fail closed.** `plan_sources` (`src/form/channels/mod.rs:454-457`) refuses
  `no-table-entry` before the interim/private branch, so the refusal holds on every OS.
  `channel_table.json` has no `mnemonic xpub-search address-of-seed …` key. Two kinds of
  value are refused before that check, and both refusals are also fail-closed: a value of `-`
  (`C1-dash`) and a bad or unset `@env:` name. Run is disabled while the plan is refused
  (`src/app_window.rs:1094-1096`). Copy uses the same planner and returns `Disabled`
  (`copy.rs:122-137`). The Preview of a refused plan uses the masked form argv
  (`app_window.rs:993-996`).
- **The plan sees every seed and passphrase source.** The planner treats a form with no
  sources as a plain run (`mod.rs:1137-1141`), so the refusal is only as complete as source
  detection. Detection is complete for this form:
  - Secret Text values (`--phrase`, `--ms1`, `--passphrase`) come only from `secret_widgets`,
    and each produces a site (`invocation.rs:391-417`).
  - Every `--from` composite row produces a site and is masked, because all four nodes are
    in `SECRET_NODE_TYPES_ARGV` (`invocation.rs:636-665`).
  - The `*-stdin` toggles never emit.
  - A form holding only `--phrase-stdin` runs with nothing secret in it, and the child gets
    `Stdio::null()` (`runner.rs:194-196`).
- **The `--from` composite is masked on screen, warns on paste, asks to confirm a run and is
  dropped at save.** These key on the node: `widget.rs:629`, `secrets.rs:339`, and
  `persistence.rs:101-105`.
- **Is gui-schema's `--from secret: false` a toolkit bug?** No, it is deliberate.
  `mnemonic_toolkit::secrets::flag_is_secret` looks only at the flag name, not the
  subcommand, and its module doc (`crates/mnemonic-toolkit/src/secrets.rs:48-51`) excludes
  `--from`/`--to` on purpose: whether they are secret depends on the node, and the GUI
  classifies by node. That predicate cannot express "every node of this verb's `--from` is
  secret" without a per-subcommand table. Marking it secret in the GUI alone would break
  `schema_mirror_secret_drift`. The GUI's node-keyed handling covers the gap. There is no
  toolkit issue to file; an optional enhancement is noted in M3.
- **The draft-until-release claim is true at v0.107.1.**
  `MNEMONIC_BIN=/home/user/wt-tk-pin/target/debug/mnemonic cargo test --locked --test schema_mirror_secret_drift`
  fails `secret_drift_gate_mnemonic_v5_schema_matches_gui_handcode`. The #230 binary still
  reports `mnemonic 0.107.1`, as the FOLLOWUP says.

## Test run

`MNEMONIC_BIN=/home/user/wt-address-search/target/debug/mnemonic cargo test --locked`
(output in `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/gui-aos-review.log`).

The disk is effectively about 70 GB and this worktree's `target/` held 33 GB. The first
attempt died in the linker with a bus error (ENOSPC); that log is saved as
`gui-aos-review.attempt1-enospc.log` in the same folder. To finish, I deleted
`target/debug/incremental` and ran each test target once: `--lib --bins`, then each
`--test <file>`, then `--doc`. After each binary ran I deleted its executable. Every test
target ran exactly once.

Result: lib+bins ok, doc ok, 91 targets `ok`, and 11 failing tests in 7 binaries:

| Test | Cause |
|---|---|
| `schema_mirror::{md,ms,mk}_schema_flag_names_match_help_text` | `md`/`ms`/`mk` not on PATH |
| `part_b_forms::b_real_every_new_form_runs_at_exit_zero` | spawn NotFound |
| `secret_channels_t2` ×2, `secret_channels_t3prime` ×1, `secret_channels_t9` ×1 | spawn NotFound (pinned binaries / oracle absent) |
| `persistence_autosave_v0_36_0::t2b_failed_write_does_not_poison_the_cache` | running as root, so a 0o555 directory is writable |
| `wire_shape_snapshot::wireshape_path_of_xpub_match` | `searched_count` 161 vs golden 140: a #230 toolkit change, as the FOLLOWUP records |

All new and changed cells pass: `xpub_search_address_of_seed` (4/4), `xpub_search_schema_mirror`
(10/10), `schema_mirror_secret_drift` (11/11 against #230), `schema_mirror_defaults_drift`,
`f5_nested_argv`, `ui_harness_i3_secret_nopersist` (7/7), the census tests, and the
mnemonic `schema_mirror` cells. See N4 on the baseline count.

## Important

### I1: a private key in `--cosigner` or `--descriptor` is shown, copied without warning and (for `--cosigner`) persisted

**Where.** `src/schema/mnemonic.rs:3705` (`--descriptor`) and `:3741` (`--cosigner`), with
`src/secrets.rs:199-203` (`PRIVATE_KEY_CONTENT_FIELDS` lists md fields only).

**What the toolkit says.** SPEC §2 and §7 name this input class: "Private keys (xprv, tprv,
WIF) are refused at intake (exit 2) in --descriptor, --descriptor-file and --cosigner … The
argv guard covers neither flag." The toolkit only catches them at intake
(`address_of_seed.rs:499-512`, `refuse_private`), so by then the GUI has already acted on
the value.

**What the GUI does today.** `field_masks_private_key_content("mnemonic xpub-search address-of-seed", …)`
is false for both flags. So a pasted `xprv…` or WIF in either field:
- is shown in clear in the Preview and in the last-run `argv:` line;
- leaves Copy `Ready` with the key in it, labelled without "reveals secret" because the mask
  is false (`copy.rs:70-89`);
- goes on the child's argv;
- for `--cosigner`, is written to `state.json`. `redact_for_persistence`
  (`persistence.rs:113-122`) drops a `?prv` value only under the names `--descriptor` and
  `--key`. `--descriptor` is saved from that by name, but only for `?prv`; a WIF in either
  field persists, because `text_holds_private_key` (`secrets.rs:215`) checks only `?prv`.

This is the DESIGN §B2–B4 leak class that the GUI already closed for md fields ("by then it
would have shown and persisted").

**Reproduction.** A test cell:

```rust
let mut s = FormState::default();
s.values.push(("--target-address".into(), FlagValue::Text(TARGET.into())));
s.values.push(("--descriptor".into(), FlagValue::Text("wpkh(@0/<0;1>/*)".into())));
s.values.push(("--cosigner".into(), FlagValue::Text("@1=xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi".into())));
let (argv, mask) = assemble_argv_with_secret_mask(&SCHEMA, sub(), &s);
// the xprv token is unmasked
let saved = redact_for_persistence(&s);
// saved.values still holds --cosigner with the xprv
```

**Fix.**
1. Add `("mnemonic xpub-search address-of-seed", "--descriptor")` and
   `("mnemonic xpub-search address-of-seed", "--cosigner")` to `PRIVATE_KEY_CONTENT_FIELDS`.
   The persistence belt keys on the name alone, so this also redacts `--cosigner` values that
   hold an xprv on every verb, which is fail-closed.
2. Widen `text_holds_private_key` to the toolkit's WIF shape (`[5KLc9]` base58 of 51-52
   chars; the regex is at `address_of_seed.rs:391`), or file that as a follow-up.
3. Add one cell: masked, Copy labelled as revealing a secret, and not persisted.
4. `account-of-descriptor --descriptor` has the same pre-existing gap. Log it in FOLLOWUPS
   rather than widening this PR.

### I2: the owed channel measurement is under-specified, and T4's coverage gate goes red when this merges

**Where.** `FOLLOWUPS.md:10-16` (`address-of-seed-gui-lockstep`), owed-at-pin-bump list.

**Problem 1: re-running `regen_check.py` does not measure the verb.** The FOLLOWUP says to
"re-derive the secret-channel cache (`regen_check.py` against the new binaries) so
`channel_table.json` measures … address-of-seed". But the channel measurements come from
hand-written case lists: `cases.py:86-87`, `cases3.py:22`, `shapes.py:76` and
`refusals.py:51` list the xpub-search verbs by hand. With no `address-of-seed` rows there,
`run_all.py`/`table_build.py` produce no keys for it. The table stays without them, and the
"flips with the regen" story in `aos_run_and_copy_fail_closed_until_measured` never happens.

**Problem 2: `secret_sources.txt` is stale from this commit on.** `regen_check.py:103-129`
(T4 schema coverage) re-enumerates the mirror's sources with `enumerate_sources.rs` and
diffs them against `secret_sources.txt`. This commit adds seven sources (`--phrase`,
`--ms1`, `--passphrase`, `--from {phrase,ms1,entropy,seedqr}=`) without updating the file.
It also requires each source to have a channel-table entry or a §A2b row.
`.github/workflows/schema-mirror.yml:151` runs it on push to `master` and on release tags
(not on PRs). So merging this mirror before the measurement turns master red.

The draft status prevents that today, but only if the FOLLOWUP's owed list is complete.

**Fix.** In the FOLLOWUP's owed list, name:
- the case rows to add for every key: `--phrase`, `--ms1`, `--passphrase` (including `-`
  and `@env:`), and `--from <node>=` for each of the four nodes. Include the one-stdin-reader
  rule (§2), because the planner will need a second channel for `--from` rows beyond the
  first;
- regenerating `secret_sources.txt` (and `missing_sources.*` via `probe_missing.py`);
- running `regen_check.py --plans` and `check_design_tables.py` green.

Optionally commit the regenerated `secret_sources.txt` now so the enumeration diff is
reviewed with the mirror.

## Minor

### M1: the fail-closed test only proves the first source refuses

**Where.** `tests/xpub_search_address_of_seed.rs:163-198`.

`plan_sources` returns at the first source without a table entry. Both states tested have
`--phrase` or `--from phrase=` first, so the refusals for `--ms1`, `--passphrase` and
`--from ms1=/entropy=/seedqr=` alone are never exercised. A partial measurement at the pin
bump (for example `--phrase` measured and `--from seedqr=` not) would still be caught by the
code, but not pinned by this test, which claims "every secret-bearing run".

**Fix.** Loop over single-source states, one per key, plus an `@env:SET_VAR` variant (with
a `user_env` that returns a value) and a `-` variant (`C1-dash`). Assert that the refusal
code is `no-table-entry` or `C1-*` and that Copy is disabled for each.

### M2: GUI help text describes the CLI's channels, not the GUI's

**Where.** `src/schema/mnemonic.rs:3620-3630` (`--phrase`) and `:3674-3684` (`--passphrase`).

- `--passphrase` help says "`-` reads stdin; `@env:VAR` reads an environment variable". In
  the GUI, `-` is refused (`C1-dash`) and `@env:VAR` is read by the GUI.
- `--phrase` help says "Inline is refused by the argv guard unless --allow-argv-secret". In
  the GUI the value goes over a private channel, or the run is refused.

**Fix.** Use the wording `--from` already uses and the other secret fields share ("Type the
value, or `@env:VAR` (read by the GUI; a value that is `-` or starts with `@env` is
refused)").

### M3: the companion FOLLOWUP and SPEC §8 disagree with the implementation on two points

**Where.** Toolkit `design/FOLLOWUPS.md:6943` (`address-of-seed-gui-lockstep`).

The toolkit entry lists positional `MS1 (secret)` as a GUI flag and says "Companion: owed in
mnemonic-gui `FOLLOWUPS.md`". SPEC §8 says to mark "the repeatable `--from <node>=`
composite" secret. The GUI deliberately omits the positional (`NO_POSITIONALS`) and keeps
`--from secret: false`, with every node secret. Both are correct choices.

**Fix.** Update the toolkit entry, and its companion line, to say so and to point at the GUI
entry; the GUI side cannot do this. Optional enhancement for the toolkit: a per-subcommand
"every node secret" bit in gui-schema, so the drift gate could see `--from` here.

## Nit

- **N1.** `tests/secret_flips_v0_33_0.rs:86`: `t3_phrase_is_secret_on_all_three_xpub_search_modes`
  now iterates four modes. The name and doc ("all three modes") are stale.
- **N2.** `src/form/widget.rs:600`: the comment "no NodeValueComposite flag repeats today" is
  stale (`seed-xor-combine --share`, now `--from`). The row salt it guards is present, so this
  is wording only.
- **N3.** A `--from` composite secret lives in `state.values` as a plain `String`, not
  `SecretLineEdit`/`Zeroizing`. This is inherited from `convert --from`, and the widget
  comment acknowledges it. It is now the main way to enter 2+ seeds here, so it is worth a
  line in the FOLLOWUP (residue class `gui-secret-buffer-allocator-residue`).
- **N4.** Baseline: the implementer counts 9 environmental failures on base. I observed 10
  environmental failures plus the expected `wireshape_path_of_xpub_match` (11 tests, 7
  binaries; the table above). The difference is probably counting units, or the root-only
  `t2b`. I did not run base (disk); state the base list by test name in the PR so the
  comparison is mechanical.
