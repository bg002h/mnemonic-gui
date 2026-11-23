# FIX sec-gui-allowlist-bracket: the tree persist allowlist and the xprv mask see past `]`

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: small. Class: other (not a funds path).
Base: mnemonic-gui `origin/master` 336e5443. Closes secret row `gui-persist-allowlist-bracket-bypass` (I),
coordinator `status/secret-criticals.md:121`; grader evidence `agent-reports/sweep-1122/grade-gui.md:12,26,43-45`.

## 1. Defect (at 336e544)

- `src/form/tree_model.rs:679-682` `is_xprv_like` and `:699-707` `is_extended_public_like` both judge only the text
  after the LAST `]` (`key.rsplit(']').next()`).
- So a tree `key`/`keys[i]` value of the form `<private>]<public>` or `<private>[<origin>]<public>` is judged public:
  `blank_non_extended_public_keys` (`:718-742`, called from `TreeState::redacted_for_persistence` `:182`, reached
  from `persistence.rs:160-163`) keeps the whole string, and it is written to `state.json` in cleartext on autosave
  and on exit.
- The same string is not masked on screen (`tree_form.rs:686`, `:709` gate `.password(..)` on `is_xprv_like`) and
  gets no amber hint (`tree_form.rs:781`).
- A key that starts with a public prefix and carries an xprv after a non-bracket separator (`tpub… tprv…`) is also
  kept today: the rsplit returns the whole string, which starts `tpub`.
- The code comment at `tree_model.rs:696-698` and SPEC v0.34.0 `design/SPEC_gui_v0_34_0_persist_redaction_hardening.md:32`
  record the `]` case as "tolerated". README.md:139 promises "Secret material NEVER persists".

## 2. Spec delta (the rules the build implements)

Rule P (`is_xprv_like`, the deny-list used by the mask, the hint, the `hex`/`w` persist arm and
`secrets::text_holds_private_key`): true iff ANY token of the value, splitting on every character that is not ASCII
alphanumeric, has the bytes `prv` at offsets 1..4. This is the token split `secrets.rs:217-220`
(`text_holds_private_key`) already uses. No `rsplit`.

Rule A (`is_extended_public_like`, the persist allowlist for `key`/`keys`): true iff all three hold:
1. Origin strip: if the value starts with `[`, the body is the text after the FIRST `]`; if it starts with `[` and
   has no `]`, the result is false. A value that does not start with `[` is its own body (no strip).
2. The body contains no `[` and no `]`.
3. The body starts with one of the 10 four-byte literals already listed (`xpub tpub ypub Ypub zpub Zpub upub Upub
   vpub Vpub`; unchanged), AND Rule P is false for the whole value.

Unchanged: the 10-prefix list, the "do not tighten to {xpub,tpub}" note, the hex/w deny-list arm (it now inherits
Rule P), the walk's recursion, every holder-visible text. The hint text at `tree_form.rs:784-785`
("this value will not be saved and the gate will refuse it") becomes exact for key fields: every value Rule P flags in
`key`/`keys` now fails Rule A and blanks.

Decision table (each row is a test cell in section 4; `TPRV`, `TPUB`, `WIF` are the fake vectors in the patch):

| value | Rule P (mask, hint) | persists? 336e544 | persists? after |
|---|---|---|---|
| `TPRV]TPUB` | false -> true | yes (leak) | no |
| `TPRV[73c5da0a/84h/1h/0h]TPUB` | false -> true | yes (leak) | no |
| `[73c5da0a/84h/1h/0h]TPRV]TPUB` | false -> true | yes (leak) | no |
| `WIF]TPUB` | false | yes (leak) | no (Rule A.1: no leading `[`, body starts `c`) |
| `[73c5da0a/84h/1h/0h]TPUB]WIF` | false | no | no (Rule A.2; a first-`]` cut alone would regress this row) |
| `TPUB TPRV` | false -> true | yes (leak) | no (Rule A.3) |
| `[73c5da0a/84h/1h/0h]TPUB` | false | yes | yes |
| `[73c5da0a/84'/1'/0']TPUB/<0;1>/*` | false | yes | yes |
| `TPUB/0/*` | false | yes | yes |
| `[]TPUB` | false | yes | yes |
| `sv` (in `w`) | false | yes | yes |

Doc-comment changes the build makes (code comments, not holder text):
- `tree_model.rs:668-678` (`is_xprv_like` doc): replace the "strip an optional `[origin]` prefix via `rsplit(']')`"
  clause with Rule P in one sentence, and cite this FIX.
- `tree_model.rs:696-698`: replace the "Inherited `rsplit(']')` caveat ... tolerated" bullet with Rule A in one bullet
  citing this FIX and the row id `gui-persist-allowlist-bracket-bypass`.

SPEC amendment (the build replaces one line). Anchor, occurs once in
`design/SPEC_gui_v0_34_0_persist_redaction_hardening.md`: `` - `rsplit(']')` caveat (R0-r1 M4, pre-existing in `is_xprv_like`) ``.
Replace that whole line (line 32) with:

    - Origin handling (amended 2026-11-22 by `design/FIX_sec-gui-allowlist-bracket.md`, closing R0-r1 M4 and secret row `gui-persist-allowlist-bracket-bypass`): an origin is stripped only when the key starts with `[`, up to the FIRST `]`; the body must contain no `[` or `]`, must start with one of the 10 prefixes, and the whole key must not be `is_xprv_like`, which now tests every token split on non-ASCII-alphanumeric characters. `xprv…]xpub…`, `WIF]xpub…` and `xpub… xprv…` therefore blank at persist, and an xprv anywhere in the value masks the field.

Printed texts and exit codes: none change (no CLI, no holder-visible string). Toolkit manual lockstep: none
(`docs/manual/src/40-cli-reference/` untouched: no flag or printed text; `docs/manual-gui/src` has no text on the
tree persist allowlist, checked by `git grep` at toolkit origin/master).

## 3. Production change (what, not how)

`src/form/tree_model.rs` only: `is_xprv_like` implements Rule P; `is_extended_public_like` implements Rule A
(it may call `is_xprv_like`). No other file's code changes. `secrets.rs:219`
`.any(crate::form::tree_model::is_xprv_like)` stays byte-identical: it is the anchor of mutation `B2-B4` in
`scripts/secret-channels-mutations.py:130-131`.

Author's feasibility check (not part of the FIX, not committed): a 14-line prototype of Rules P and A on 336e544 made
all three section-4 tests pass, kept the full suite at its base result (776 run, 769 passed, the same 7 local
real-binary failures as base: `part_b_forms`, `schema_mirror` mk, `secret_channels_t2` x2, `t3prime`, `t6`, `t9`,
which fail identically on unpatched 336e544 here because the local CLI binaries are not the pinned ones), and passed
`cargo clippy --locked --all-targets -- -D warnings`.

## 4. RED rows

The tests ship as a patch next to this file: `design/FIX_sec-gui-allowlist-bracket.red.patch` (adds one unit test to
`src/form/tree_model.rs` and two integration tests plus a helper to `tests/persist_redaction_v0_34_0.rs`; fake
signet-shaped vectors only). Run on the branch head, from the worktree root:

    git apply design/FIX_sec-gui-allowlist-bracket.red.patch
    PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked -E 'test(/is_xprv_like_sees_private_key_past_any_bracket|t7_tree_persist_blanks_private_key_around_brackets|t7b_tree_persist_keeps_origin_and_suffix_public_keys/)'

Expected on 336e544 (measured 2026-11-22; the output contains `Compiling mnemonic-gui`), exit 100:

    Summary [...] 3 tests run: 1 passed, 2 failed, 779 skipped

| row (test name) | 336e544 | after the fix | what it shows |
|---|---|---|---|
| `is_xprv_like_sees_private_key_past_any_bracket` (unit, `form::tree_model::tests`) | FAIL, first message `assertion failed: is_xprv_like(&format!("{TPRV}]{TPUB}"))` | PASS | mask and hint stay off for a private key before `]` (Rule P) |
| `t7_tree_persist_blanks_private_key_around_brackets` (`persist_redaction_v0_34_0`) | FAIL, `secret text survived redact_for_persistence for: ["tprv]tpub", "tprv[origin]tpub", "[origin]tprv]tpub", "wif]tpub", "tpub tprv"]` | PASS | secret text reaches the serialized `FormState` (Rule A); the sixth cell `[origin]tpub]wif` is the regression guard and passes on both |
| `t7b_tree_persist_keeps_origin_and_suffix_public_keys` (`persist_redaction_v0_34_0`) | PASS | PASS | guard: origin, suffix and empty-origin public keys still persist |

The build applies the patch as its RED commit (the tests become part of the tree), sees the summary above, then makes
the section-3 change and sees `3 tests run: 3 passed`.

## 5. Tests that change

- Added (the patch): the three tests above.
- Existing tests that must stay green unedited: `form::tree_model::tests::is_xprv_like_matches_gate_heuristic`,
  `persist_redaction_v0_34_0::t5_tree_persist_keeps_only_extended_public_keys`, `::t6_tree_persist_blanks_xprv_in_hex_and_w`,
  `widget_secret_mask_cycle15g` `t7_tree_key*` (4), `ui_harness_i3_secret_nopersist`, `secrets` (token-split callers).
  No existing test is edited.

## 6. Gate (full CI of mnemonic-gui; the builder runs all of it on the final head)

    PATH=$HOME/.cargo/bin:$PATH sh -c 'cargo clippy --locked --all-targets -- -D warnings && cargo clippy --locked --no-default-features -- -D warnings && cargo build --locked -p mnemonic-gui --no-default-features && cargo +1.88.0 check --locked && cargo nextest run --locked --workspace --no-fail-fast && cargo test --locked --workspace --doc && MNEMONIC_BIN=mnemonic python3 scripts/secret-channels-mutations.py' > /scratch/cache/sec-gui-allowlist-bracket.gate.log 2>&1

Not covered locally: the `snapshots`, `tutorial-snapshots` (lavapipe) and `schema-mirror` real-binary jobs need the
pinned binaries, and so does `scripts/secret-channels-mutations.py` (its PARTB rows run `part_b_forms` and
`secret_channels_t9`); take them as green CI on the PR head. Seven real-binary tests fail locally on 336e544 when the local
CLIs are not the pinned ones (section 3); compare that set against base, do not treat it as a regression.

## 7. FOLLOWUPS

Closes no existing repo FOLLOWUPS line (the defect was untracked). Proposed lines (direct-pushed after merge, not in
the PR) are listed in the author report.

## 8. Out of scope (residual, reported, not closed here)

Rule A still keeps a value that starts with a public prefix and carries a NON-xprv secret (WIF, raw hex) after a
non-bracket separator, or any secret concatenated with no separator (`TPUB/WIF`, `TPUB WIF`, `TPUBWIF`, `TPUBTPRV`
with no separator). Only a full shape check (body = prefix + exact base58 length + derivation-suffix grammar) closes
that, and it would blank half-typed public keys at autosave: a product point, reported as a STOP for a separate
row, not decided here.
