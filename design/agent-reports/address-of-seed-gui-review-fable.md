# Final review (fable): mnemonic-gui `xpub-search address-of-seed` mirror, `96ea304..edd9ff9`

Reviewer: independent adversarial review, software-developer lens, after the r1 fold.
Scope: worktree `/home/user/wt-gui-aos`, branch `address-of-seed-gui`, commits `3f94ece`,
`9ca4bb3`, `d948737`, `edd9ff9`. Toolkit PR #230 binary
`/home/user/wt-address-search/target/debug/mnemonic`; pinned tag binary
`/home/user/bins/mnemonic-v0.107.1`. No source was modified; this file is the only write.

**Counts: 0 Critical, 2 Important, 4 Minor, 4 Nit.**

Both Important findings are the same class r1 I1 opened (a secret typed into a public
key field of this form). The fold closed the exact-match cases; these are the near
misses it left, and one of them is pinned as intended behaviour by a new test.

## What was checked and holds

- **The r1 fold holds for the cases it names.** A valid xprv, tprv or WIF (all four WIF
  forms) in `--cosigner` or `--descriptor` is masked in the argv, makes Copy say it
  reveals a secret, opens the run-confirm dialog with the "passes secret-bearing
  arguments" wording (`src/app_window.rs:980-983`, `:1186`), and is dropped at save.
  The same holds on `account-of-descriptor --descriptor`, `address-of-xpub --xpub`,
  `path-of-xpub`/`passphrase-of-xpub --target-xpub`.
- **The masking fix changes no other verb.** I enumerated every `(cli, subcommand, flag)`
  for which `field_masks_private_key_content` is now true: the six xpub-search fields
  plus `md shape-key --descriptor` and `md descriptor --key` (and the `md decompose`
  positional). No md, ms or mk subcommand name starts with a nested-parent prefix, so
  the new canonicalisation cannot create a match that was not listed.
- **Seed, ms1, entropy, SeedQR and passphrase typed into their own fields** do not
  persist, are masked, and Run/Copy refuse (`no-table-entry` or `C1-*`) per source and
  spelling on all three OS names. `--descriptor-from ms1=<card>` and
  `--descriptor-from phrase=<seed>` are caught by the node-prefix rule (masked, not
  saved, Run refused).
- **Toolkit stdout carries no seed.** The printed `bundle` command repeats the source
  spelling (`@0.phrase=@env:VAR` or `@0.phrase=-`), never the phrase; `--json` likewise.
  The intake refusal for a private key says "the value is not repeated" and does not
  repeat it.
- **WIF detector false positives: none found** on xpub, tpub, bech32/bech32m/legacy/P2SH
  addresses (mainnet and testnet), a 33-byte hex pubkey, md1/mk1 strings, a `@N`
  template, a base58check string of WIF length with another version byte. A 4-byte
  checksum makes an accidental hit negligible.
- **Schema entry equals `gui-schema` of the #230 binary**: 25 flags, same names,
  `required` only on `--target-address`, `secret` on the same six, same defaults and
  dropdown values, the two globals. `schema_mirror` (mnemonic cells),
  `schema_mirror_secret_drift`, `schema_mirror_defaults_drift` and
  `xpub_search_schema_mirror` pass against it.
- **CI's T4 coverage half passes at this pin.** Reproduced the three parts that do not
  need the release binaries of md/ms/mk:
  - `enumerate_sources.rs` built against this tree prints exactly `secret_sources.txt`
    (134 lines, byte-equal);
  - `probe_missing.py` run against v0.107.1 regenerates `missing_sources.txt` and `.md`
    byte-equal (seven `exit 64 … unrecognized subcommand` rows, verdict **unmeasured**);
  - every source has a channel-table entry or an §A2b row;
    `check_design_tables.py` prints `stale blocks: none | test_plan.py: 0 failures`.
  Not run: the derived-file pipeline and `--plans` (they need the four release binaries
  whose sha256 `measured_with.json` records; the local v0.107.1 is a source build).
  The diff touches none of their inputs.
- **FOLLOWUPS line citations are right**: `cases.py:81-87`, `cases3.py:22`,
  `shapes.py:76`, `refusals.py:51-52`; "17 unmeasured rows" and "32 CLI rejects" match
  the table.

## Test results

`CARGO_INCREMENTAL=0 MNEMONIC_BIN=<#230 binary> cargo test --locked --no-fail-fast`, two
selective batches (logs `run1.log`, `run2.log` in the session scratchpad).

| Target | Result |
|---|---|
| lib | 35 passed |
| `xpub_search_address_of_seed` | 8 passed |
| `xpub_search_schema_mirror` | 10 passed |
| `schema_mirror_secret_drift` | 11 passed |
| `schema_mirror_defaults_drift` | 3 passed |
| `schema_mirror` | 18 passed, 3 failed (`md`/`ms`/`mk` not on PATH; environmental) |
| `secret_flips_v0_33_0`, `f5_nested_argv`, `f679_argv_secret_admission` | 3, 3, 16 passed |
| `persist_redaction_v0_34_0`, `persistence`, `persistence_wiring_v0_35_0` | 12, 17, 10 passed |
| `secret_mask_preview_v0_39_0`, `secrets`, `copy_command`, `widget_secret` | 11, 24, 21, 1 passed |
| `ui_harness_i3_secret_nopersist`, `ui_harness_sweep` | 7 passed; 2 passed (3 ignored) |
| `restore_from_secret_node`, `paste_warn_wiring_v0_40_0`, `h3_minikey_secret_surfaces` | 12, 4, 7 passed |
| `xpub_search_widgets`, `default_form_state`, `secret_taxonomy_pin` | 8, 3, 9 passed |
| `secret_channels_t1`, `_t6`, `_t8` | 18, 9, 2 passed |
| `gui_render_faithfulness`, `gui_form_snapshots` (no `GUI_SNAPSHOTS`), `manual_anchor_coverage` | 2, 2, 5 passed (1 ignored) |

Not run: the rest of the suite (disk), the snapshot job (no wgpu adapter), and the three
drift tests against v0.107.1 (r1 verified that they fail there, as the FOLLOWUP says).
`cargo clippy --locked --all-targets -- -D warnings`: see the end of this file.

The findings below were reproduced with a scratch crate outside the worktree that
depends on `mnemonic-gui` by path (the way `regen_check.py` builds
`enumerate_sources.rs`) and calls `assemble_argv_with_secret_mask`,
`redact_for_persistence` and `channels::plan`. Its output is quoted per finding.

## Important

### I1: a WIF with one wrong character is shown, saved and run without a prompt, and a test pins that

**Where.** `src/secrets.rs:250-258` (`is_wif`), used by `text_holds_private_key`
(`:239-243`); `tests/xpub_search_address_of_seed.rs:285-286` ("A WIF with its checksum
broken" listed under values that are *not* a private key).

**What happens.** `is_wif` requires a valid base58check checksum. The toolkit classifies
by shape (`address_of_seed.rs:389-394`: `[5KLc9]` then 50-51 base58 characters between
non-base58 boundaries) and refuses a WIF whose checksum is wrong as "private-key
material". So for a WIF with a typing or transcription error, the CLI says "this is a
private key" and the GUI has already treated it as public. One wrong character leaves
the key recoverable in a few thousand tries.

Measured (`--cosigner @1=<v>` and `--descriptor wpkh(<v>)` on address-of-seed):

```
valid WIF                 --cosigner    masked=true   persisted=false
WIF, last char mistyped   --cosigner    masked=false  persisted=true   RUNS, value on argv
WIF, last char mistyped   --descriptor  masked=false  persisted=true   RUNS, value on argv
WIF, first 50 of 52 chars --cosigner    masked=false  persisted=true
WIF glued to '0'          --cosigner    masked=false  persisted=true
two WIFs glued            --cosigner    masked=false  persisted=true
```

Toolkit, same value: `error: xpub-search address-of-seed: --cosigner holds private-key
material (an xprv/tprv-style key or WIF) …`, exit 2.

Unmasked means all of: clear in the field, clear in the Preview and the last-run
`argv:` line, Copy `Ready` with no "reveals secret" label, no run-confirm dialog, and
written to `state.json`. The "first 50 of 52" row is also what the autosave sees while
a WIF is typed by hand: every prefix is saved until the last character lands.

The same detector serves the five sibling fields and the name-keyed persistence belt, so
the gap is on all of them.

**Fix.**
1. Classify by shape, as the toolkit does: a token of 51-52 base58 characters starting
   with `5`, `K`, `L`, `c` or `9` is a WIF whether or not the checksum verifies. No
   public value this form accepts has that shape (legacy addresses stop at 35
   characters, xpubs are 111, bech32 strings and hex contain `0` or `l`). Keep the
   decode only if a narrower rule is wanted for some other caller.
2. Match the toolkit's boundaries (non-base58 characters, so `0`, `O`, `I`, `l` end a
   token) or simply port its regex, so the two cannot disagree.
3. Flip the test line: a broken-checksum WIF must be a private key. Add the prefix case
   if you take the next point.
4. For the typed-by-hand case, either treat a base58 token of 36-52 characters starting
   with `[5KLc9]` as a WIF in progress, or, better, give `--cosigner`, `--xpub` and
   `--target-xpub` the positive rule the tree form already uses at persist
   (`tree_model::is_extended_public_like`, plus `mk1…` and `@N=`): save and show in
   clear only what looks like a public key. That also closes I2.

### I2: an ms1 card pasted into `--cosigner` (the field that takes mk1 cards) is shown, saved and run

**Where.** `src/secrets.rs:201-211` and `:239-243`; `src/persistence.rs:108-124`.

**What happens.** `--cosigner` accepts an mk1 card. The form also has an `--ms1` field,
and the two cards look alike on a plate. An ms1 card in `--cosigner` is not `?prv` and
not a WIF, so nothing fires:

```
bare ms1 card   --cosigner        masked=false persisted=true  RUNS, value on argv
@1=ms1 card     --cosigner        masked=false persisted=true  RUNS, value on argv
```

The card is the seed's entropy. It lands in `state.json` in clear, shows in the Preview,
copies without a label, and goes to the child on argv with no confirm dialog. The
toolkit rejects it with `error: base58check decode: decode` (it does not echo it), so
the user sees an unhelpful error and no hint that a secret was just saved.

r1 I1 rated the xprv/WIF form of this mistake Important; an ms1 card is at least as
damaging and is the likelier slip on this form.

**Fix.** In `text_holds_private_key`, also treat a token that starts with `ms1`
(case-insensitive) as secret material; `ms1` is not a prefix of any public value these
fields take. Add the cell next to the WIF cells. The positive rule in I1 step 4 covers
it as well. The toolkit could refuse an `ms1` HRP in `--cosigner`/`--descriptor` by
name, the way it refuses private keys; that belongs in the toolkit entry.

## Minor

### M1: a value the GUI has just classified as a private key is still sent to the child on argv

**Where.** `src/form/channels/mod.rs:1144-1148` (a form with no planner source is a
plain run); `src/form/invocation.rs:582-587`.

A masked xprv or WIF in `--cosigner`/`--descriptor` is not a planner source, so Run
proceeds (after the confirm dialog) with the key on the child's command line: measured
`RUNS argv_has_value=true` on Linux, the OS where the GUI otherwise keeps secrets off
argv. The toolkit refuses every such run at intake (exit 2), so the exposure buys
nothing. The dialog wording is honest, and md's fields set the precedent, so this is
Minor.

**Fix.** Refuse the plan (for example code `private-key-in-public-field`, "this field
takes public keys only") when a listed field holds private-key content. Run and Copy
are then disabled with a message that is more useful than the CLI's.

### M2: the form's other free-text fields have no content check

**Where.** `src/schema/mnemonic.rs` `--target-address`, `--add-path`,
`--descriptor-from`; `src/secrets.rs:201-211`.

```
xprv            --target-address   masked=false persisted=true
WIF             --target-address   masked=false persisted=true
bare ms1 card   --target-address   masked=false persisted=true
WIF             --add-path         masked=false persisted=true
md1=xprv…       --descriptor-from  masked=false persisted=true
```

Less likely slips than I1/I2, but `--target-address` is the first field of the form.
Two of these the toolkit then echoes in its error, which the GUI shows verbatim:
`error: --target-address <value>: legacy address base58 string` and
`--add-path \`<value>\` is not a derivation path`.

**Fix.** List the three fields in `PRIVATE_KEY_CONTENT_FIELDS` (none takes a private
key), or record them in `private-key-content-fields-other-verbs`. Tell the toolkit side
about the two echoing errors.

### M3: `private-key-content-fields-other-verbs` understates what is still unmasked

**Where.** `FOLLOWUPS.md:21-24`.

The entry lists `bundle`, `restore`, `verify-bundle`, `export-wallet` and
`compare-cost`. The name-keyed persistence belt also covers, and the screen mask also
misses: `mnemonic build-descriptor --key`, `md encode --key`, `md verify --key`,
`md address --key`, `mk encode --xpub`, `mk verify --xpub`. The fold changed behaviour
on the two mk verbs (an xprv in `--xpub` was saved before and is dropped now); that is
fail-closed and fine, but the entry's title says "mnemonic verbs" and its list is the
record a later fixer will work from.

**Fix.** Add the six fields to the entry.

### M4: no rendered-widget cell for the bug the fold says it fixed

**Where.** `tests/xpub_search_address_of_seed.rs:300-308`; `src/form/widget.rs:468-472`.

The fold note says the on-screen mask "never matched a nested verb before" because the
widget passes the schema name. The new test checks only that
`field_masks_private_key_content` accepts both spellings. Nothing renders the form and
asserts that the `--cosigner` field is in password mode with an xprv in it, so a later
change to what the widget passes would regress silently.

**Fix.** One kittest cell (the harness in `tests/ui_harness*` already drives Text
fields): type an xprv into `--cosigner`, assert the field is masked and the reveal
toggle is present.

## Nit

- **N1.** `src/form/widget.rs:599-601`: the comment names `seed-xor-combine --from`; the
  repeating composite on that verb is `--share`.
- **N2.** `is_xprv_like` is looser than the toolkit's pattern (`?prv` and nothing more).
  A public descriptor whose checksum happens to be `?prv…` (measured with `#qprv2k3l`),
  or a BIP-388 policy JSON whose name contains a word like `xprv`, is masked, labelled
  as revealing a secret and silently not saved. About 1 in 32,768 per checksummed
  descriptor. Requiring 20 or more base58 characters after `prv`, as the toolkit does,
  removes it.
- **N3.** For the pin bump: once the verb is measured, the toolkit's "to mint this
  wallet, run:" line will name whatever channel the GUI used
  (`--slot @0.phrase=@env:MNEMONIC_GUI_S0` or `=-`). Harmless, but a user cannot run it
  as printed. Worth a line in `address-of-seed-gui-lockstep`.
- **N4.** `tests/xpub_search_schema_mirror.rs` now skips a subcommand the binary lacks,
  so at v0.107.1 the address-of-seed cell passes without checking anything. The main
  `schema_mirror` test still fails there, so the draft gate is intact; the skip message
  is the only trace.

## Clippy

`CARGO_INCREMENTAL=0 cargo clippy --locked --all-targets -- -D warnings` finished clean (exit 0,
no warnings). The `--no-default-features` clippy job was not run. `cargo fmt --check` reports
differences, but in files this range does not touch as well (for example `src/app_window.rs`),
and no workflow gates on it.

`target/` in the worktree was deleted after the runs.
