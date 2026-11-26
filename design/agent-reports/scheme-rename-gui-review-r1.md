# Scheme / origin-path rename: GUI review, round 1

Diff reviewed: `origin/claude/project-thread-utm1aa..origin/claude/project-thread-0jg5n7` (one commit, 18da39a, 46 files). Reviewer did not edit source.

**Verdict: 0 Critical, 0 Important, 4 Minor.**

## Critical
None.

## Important
None.

## Minor
1. **Nothing tests the narrower threshold seed.** The scoping is inline at `src/app_window.rs:899-906`, and no test reaches it. `template_aware_seed.rs` only tests `template_defaults_for` and a copy of the hook, and no MnemonicGuiApp-driven test selects a build-descriptor scheme. Failure case: a later edit adds `build-descriptor` to `SCHEME_TEMPLATE_SUBCOMMANDS`, or drops the guard. Picking `decaying-multisig` would then push `--threshold 2` into build-descriptor, which has its own `--threshold` (`mnemonic.rs:4207`), and change its argv. No test would fail. Suggested fix: move the predicate into a function and test it, or assert that the set equals the mnemonic subcommands whose `--scheme` options come from `TEMPLATES`.
2. **Wrong old name in a comment.** `src/app_window.rs:897` says "`build-descriptor --scheme` (ex-`--scheme`)". It should say ex-`--archetype`.
3. **Comments disagree on the clap requirement name.** `src/schema/mnemonic.rs:4179` now says `requires = "scheme"`, but `mode_predicates.rs:19-24` and `conditional.rs` (build_descriptor doc) still say `requires = "archetype"`. Pick the name upstream uses.
4. **Migration is one-way.** After an autosave is migrated, a downgraded GUI sees `--scheme` as an unknown flag and falls back to `opts[0]`. This is acceptable, but the CHANGELOG could say so.

## Verified clean
- **md `--template` kept.** It stays on md verify, address and descriptor (`md.rs:319/523/878`). The conditionals keep it (`conditional.rs:873-899, 1180-1204`). `md:address` is absent from `LEGACY_FLAG_RENAMES`, and a test checks this.
- **Rename coverage is complete.** The base branch had `--template` on exactly 5 toolkit forms and `--archetype` on 1. Add md compose `--preset` and ms derive `--template`, and that is the 8 entries in the migration table. No old spelling is left in argv or conditional code (grepped `src/`).
- **Shared `--scheme` lookups are gated.** Every build-descriptor-only use checks `sub.name == "build-descriptor"`: `app_window.rs:579-597`, `render_emit.rs:231-235, 552-560`, `tree_form` selector and edit-as-tree. `template_is_in` is called only from per-form conditional functions.
- **`--origin-path` now names two flags.** ms derive's is a dropdown; mk's is text. The only lookup by that name is mk's own conditional (`conditional.rs:791`).
- **Secret masking is unaffected.** None of the renamed flags is secret, and `SECRET_FLAG_NAMES` has no entry for any of these names.
- **Migration behaves correctly.** It runs only from `load()`, before normalization, and does not bump `SCHEME_VERSION`. When both keys are present the new one wins. Running it twice changes nothing. The value is moved verbatim, and its type is the same before and after for all 8 entries.
- **The threshold seed is scoped correctly.** It is now limited to bundle, verify-bundle, convert, export-wallet and restore. Leaving out ms derive removes a seed that pushed values for flags that form does not have.
- **No test was weakened.** In a diff that ignores the spelling change, the only lines that differ are two snapshot rows re-aligned for the shorter flag name.

## Commands and counts
Run from `/home/user/mnemonic-gui` at 18da39a, with `PATH` and the four `*_BIN` variables pointing at `/home/user/scratch-gui/bin`. Logs are in `/home/user/scratch-gui-review/`.

- `CARGO_BUILD_JOBS=2 cargo test --locked --no-fail-fast` → `test.log`: 97 test binaries, 824 passed, 5 failed, 6 ignored. All 5 failures are the known unrelated ones:
  - `md_schema_flag_names_match_help_text`: `--allow-invalid-miniscript`
  - `t2_every_cell…`: wif/xprv baseline
  - `t6_shell_leg…`: no zsh
  - `wireshape_path_of_xpub_match`
  - `t2b_failed_write…`: read-only directory under root

  The mnemonic and ms `schema_mirror` checks pass against the branch binaries, which means the hidden aliases are not listed.
- `cargo clippy --locked --all-targets -- -D warnings` → `clippy.log`: exit 0, 0 warnings.
- Checked against the binaries: `mnemonic build-descriptor --scheme`, `ms derive --origin-path <NAME>` and `md compose --scheme`. md's `--json` key `preset` is unchanged.

## Fold

Folded by the implementer on `claude/project-thread-0jg5n7`, on top of 18da39a.

1. **Threshold-seed scoping is now tested.** The predicate moved from `app_window.rs` into `form::conditional::scheme_seed_value(cli, sub, state)`, and `app_window.rs` calls it. New test `tests/template_aware_seed.rs::scheme_seed_value_fires_for_exactly_the_five_toolkit_scheme_forms` checks:
   - `SCHEME_TEMPLATE_SUBCOMMANDS` holds exactly the five forms, and the hook fires on each of them.
   - It returns `None` for `build-descriptor` (with `decaying-multisig` selected), for `md compose` and for `ms derive`.
   - Every `mnemonic` form with a `--scheme` flag (six) is either one of the five or build-descriptor, so a new consumer has to be classified.
2. **Comment fixed:** it now says ex-`--archetype`.
3. **Requirement name aligned:** the `requires = "archetype"` comments in `mode_predicates.rs` and `conditional.rs` now say `requires = "scheme"`, matching upstream `build_descriptor.rs`.
4. **CHANGELOG:** one sentence says the saved-session migration is forward-only (an older GUI falls back to the dropdown's first option).

Checks:
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0.
- Tests touched or affected: lib, `template_aware_seed`, `conditional_visibility`, `archetype_form`, `tree_form`, `ui_harness_i2_conditional`, `gui_render_emit`, `build_descriptor_schema`. All pass: 229 passed, 0 failed.
- No new rustfmt drift in the edited files.
