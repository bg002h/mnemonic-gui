# FIX sec-gui-allowlist-shape: a persisted tree key must have the shape of a public key (Rules S and E)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Track: small. Base: mnemonic-gui `origin/master` 7e1b498.
Closes coordinator secret rows `gui-persist-allowlist-concat-residue` (I, `status/secret-criticals.md:123`) and
`fixdoc-secguibracket-r1-concat-residue` (I, `:404`, a duplicate of 123: same defect, same fix-doc finding).
Rulings: coordinator `agent-reports/architect-secret-ci-decisions-1124.md` Q1 (:7-17; Rule S, the 10-digit suffix bound,
trailing whitespace, the 5 marker texts ride this unit); `status/brian-rulings-2026-11-24.md` last line (Brian
2026-11-24: keep blank-on-save, with Rule S AND Rule E); BRIAN-Q-1 (Brian 2026-11-22, autosave never blanks a key
being typed); marker texts and layout `agent-reports/architect-gui-q1-typing-prefixes.md` Q2; Rule E definition and
its probe (0 of 222000 honest prefixes blanked) `architect-gui-q1-typing-prefixes.md` Q3.

## 1. Defect (at 7e1b498)

- `src/form/tree_model.rs:721-744` `is_extended_public_like`, the persist allowlist for tree `key`/`keys[i]`, ends at
  `XPUB_PREFIXES.iter().any(|p| b[..n] == p[..n])` (`:743`): it checks the first 4 bytes of the body and nothing after.
- So a value that starts with a public prefix and carries a WIF or raw hex after a separator (`xpub…|WIF`,
  `xpub…/HEX64`, `[origin]tpub…/WIF`), a secret concatenated with no separator (`xpub…WIF`, `tpub…tprv…`), a long
  decimal run (`tpub…/<77 digits>`) or a whole WIF / 64 hex typed inside an incomplete key (`tpub…[..40]WIF`) is
  kept by `blank_non_extended_public_keys` (`:754`), reached from `TreeState::redacted_for_persistence` (`:180-191`)
  and `persistence.rs:163`, and written to `state.json` in cleartext on autosave and on exit.
- README.md:142@7e1b498 promises "Secret material NEVER persists". FIX_sec-gui-allowlist-bracket §8 records the residue.

## 2. Spec delta (the rules the build implements)

Persistence stays blank-on-save (Brian 2026-11-24): only the persisted copy (`redacted_for_persistence`, which
clones) is blanked. **The live, on-screen field is never changed**; test `k2_live_field_is_never_changed` pins it.

Rule A (FIX_sec-gui-allowlist-bracket, as folded for BRIAN-Q-1) stays, with two changes and two new rules.
`is_extended_public_like` (or a wrapper the persist walk and `key_marker` both call; the build chooses) is true iff:

- T. Leading ASCII whitespace AND (new) trailing ASCII whitespace are judged as absent (the ruling: "Trailing ASCII
  whitespace counts as absent", mirroring Rule T). The value judged below is the value with both ends trimmed of
  ASCII whitespace; the stored value keeps its whitespace.
- P. The whole value is not `is_xprv_like` (unchanged).
- A.1 / A.1'. Origin strip and origin grammar, unclosed origin persists if it matches the grammar (unchanged).
- A.2. No `[` or `]` in the body (unchanged; now implied by S).
- A.3 / A.3'. The body starts with one of the 10 literals or is a proper prefix of one (unchanged).
- **S (new).** Let the run be the longest prefix of the body made of base58 characters (ASCII alphanumerics except
  `0 O I l`; the literal counts in the run). The run is at most 111 characters (every extended key in all 10 versions
  is 111 base58 characters). Every character after the run is one of `0-9 / h H ' < > ; *`, and **every maximal run
  of ASCII digits after the run is at most 10 digits** (a u32 index; the bound `is_origin_like` uses, `:696`).
- **E (new).** Neither holds: (E.1) the value contains 64 consecutive ASCII hex digits (`0-9 a-f A-F`); (E.2) some
  window of 51 or 52 consecutive characters of the run base58check-decodes (`bitcoin::base58::decode_check`) to
  bytes whose first byte is `0x80` or `0xef` and whose length is 33 or 34 (a WIF: version byte, 32-byte key,
  optional `0x01`; bitcoin 0.32.9 `src/crypto/key.rs:477-487` `PrivateKey::from_wif`).

E applies to `key`/`keys[i]` only. The `hex` and `w` arms (`is_xprv_like` deny-list, `:770-775`) do not change: a
hashlock digest is legitimately 64 hex.

Decision table (each row is a test cell in section 4; `TPUB`, `XPUB`, `FWIF`, `FTPRV`, `HEX64` = `"a1"×32`,
`DEC77` = 77 digits are the fake vectors of `tests/persist_typing_prefixes.rs`):

| value | persists? 7e1b498 | persists? after | rule |
|---|---|---|---|
| `XPUB\|FWIF`, `TPUB FWIF`, `TPUB,FWIF` | yes (leak) | no | S (character after run) |
| `XPUB/HEX64` | yes (leak) | no | S (`a` after run) |
| `XPUBFWIF`, `TPUBFTPRV`, `tpubFTPRV` | yes (leak) | no | S (run over 111) |
| `[73c5da0a/84h/1h/0h]TPUB/FWIF` | yes (leak) | no | S |
| `TPUB/12345678901`, `TPUB/0/12345678901h`, `[origin]TPUB/<0;12345678901>/*`, `TPUB/DEC77` | yes (leak) | no | S (digit run over 10) |
| `TPUB[..40]FWIF`, `tpubFWIF`, `[origin]TPUB[..40]FWIF`, `TPUB[..40]FWIF ` | yes (leak) | no | E.2 |
| `TPUB[..20]HEX64`, `tpub` + `"A1"×32`, `[origin]TPUB[..20]HEX64` | yes (leak) | no | E.1 |
| `t`, `TPUB[..40]`, `[73c5da0a/84h`, `[73c5/84h]TPUB`, `[]TPUB`, `TPUB/2147483647`, `TPUB/<0;1>/*`, `TPUB `, `TPUB/0/* \t`, `TPUB[..40] ` | yes | yes | honest typing (BRIAN-Q-1) |
| `tp `, `[73c5da0a/84h ` | no | yes | T (trailing whitespace absent) |
| every prefix of the 12 honest shapes (1458 values) | yes | yes | BRIAN-Q-1 |

The marker (architect-gui-q1-typing-prefixes Q2, ruled to ride this unit; texts unchanged by the 11-24 ruling): one
line directly under each tree `key` and `keys[i]` field, in the slot `xprv_hint` uses (`tree_form.rs:689`, `:718`),
which become `key_hint(ui, …)`; a field shows at most one line. The xprv hint (`tree_form.rs:780-788`) keeps priority
and its text is unchanged. Otherwise the new pure `pub fn tree_model::key_marker(&str) -> Option<(String, bool)>`
(`bool` = amber) gives the first that applies, judged on the value trimmed as in Rule T:

0. the value trimmed as in Rule T is empty, or the value is is_xprv_like: no line (key_hint shows the xprv hint for the latter).
1. the value is not empty and fails the persist rule above (A, S or E): amber `⚠ not an extended public key: this value will not be saved`
2. `[` with no `]`: grey `incomplete: the origin needs a closing ]`
3. a closed origin whose fingerprint is not 8 characters: grey `incomplete: the fingerprint needs 8 hex digits`
4. a run shorter than 111: grey `incomplete: {n} of 111 key characters` (`{n}` = the run length, literal included)
5. a run of 111 that fails `bitcoin::base58::decode_check`: amber `⚠ checksum fails: a character is wrong or swapped`
6. otherwise no line.

Grey is `ui.weak(text)`; amber is `ui.colored_label(AMBER, text)` (`tree_form.rs:55`). `bitcoin` becomes a direct
dependency: `bitcoin = { version = "0.32", default-features = false, features = ["std"] }` in `[dependencies]`
(already locked at 0.32.9 through mnemonic-toolkit; the only `Cargo.lock` change is the line ` "bitcoin",` in the
`mnemonic-gui` package's dependency list, so the build runs `cargo update -p mnemonic-gui --offline` once, then every
command is `--locked`). WORDING: mnemonic-gui has no `design/WORDING_REVIEW.md`; the 5 texts are logged in the
coordinator's WORDING_REVIEW (architect Q2), the coordinator's step, not the build's.

Doc-comment changes the build makes (code comments, not holder text):
- `tree_model.rs:705-720` (`is_extended_public_like` doc): add Rules T (both ends), S and E in one bullet each, citing
  this FIX and the row ids `gui-persist-allowlist-concat-residue` / `fixdoc-secguibracket-r1-concat-residue`.
- `persistence.rs:155-158`: the comment still says keys are blanked "when xprv-matching (the toolkit gate.rs heuristic —
  strip `[origin]` via rsplit(']')…"; replace it with "blanked unless they have the shape of an extended public key
  (`tree_model::is_extended_public_like`, Rules A, S and E)".

SPEC amendment (the build appends one sentence). Anchor, occurs once in
`design/SPEC_gui_v0_34_0_persist_redaction_hardening.md` (line 32): `(BRIAN-Q-1: no prefix of an honest key being typed is blanked).`
Append after it, on the same line:

     Amended 2026-11-24 by `design/FIX_sec-gui-allowlist-shape.md` (secret rows `gui-persist-allowlist-concat-residue`, `fixdoc-secguibracket-r1-concat-residue`): trailing ASCII whitespace is also judged as absent; the body's leading base58 run is at most 111 characters and may be followed only by `0-9 / h H ' < > ; *` with no digit run longer than 10; and the key blanks if it holds 64 consecutive hex digits or a 51- or 52-character window of the run that base58check-decodes to a WIF. Only the saved copy is blanked; the field on screen keeps what was typed. Under each tree key field one line shows the first of: the xprv hint; `⚠ not an extended public key: this value will not be saved`; `incomplete: the origin needs a closing ]`; `incomplete: the fingerprint needs 8 hex digits`; `incomplete: {n} of 111 key characters`; `⚠ checksum fails: a character is wrong or swapped`.

Printed texts and exit codes: no CLI text or exit code changes. The 5 marker lines are new GUI texts (above).
Toolkit manual lockstep: none. `docs/manual/src/40-cli-reference/` is untouched (no flag, no printed CLI text);
`docs/manual-gui/src` has no text on the tree key fields or the persist allowlist (`git grep -i "will not be saved\|extended PRIVATE key\|incomplete:\|keys\[\|autosave"` at toolkit origin/master 7ff7de83: only unrelated hits);
its promise "The GUI does not write secrets to its session-state JSON" (`14-secret-handling.md:482`) becomes truer.

## 3. Production change (what, not how)

`src/form/tree_model.rs`: `is_extended_public_like` implements Rules T, S and E as above; new `pub fn key_marker`;
the doc comments of section 2. `src/form/tree_form.rs`: `key_hint` replaces the two `xprv_hint` calls (`:689`,
`:718`); `xprv_hint` itself is unchanged. `src/persistence.rs`: the comment only. `Cargo.toml` + `Cargo.lock`: the
`bitcoin` line. `secrets.rs:219` stays byte-identical (mutation anchor `B2-B4`, `scripts/secret-channels-mutations.py`).

Author's feasibility check (not part of the FIX, not committed; diff kept at `/scratch/cache/gui-shape-proto.diff`,
91 added and 12 removed lines): Rules T, S, E, `key_marker` and `key_hint` as written above, on 7e1b498 plus the RED
patch, made all 9 section-4 rows pass; full `cargo nextest run --locked --workspace --no-fail-fast`:
`800 tests run: 794 passed, 6 failed, 6 skipped`, the 6 being local real-binary tests (`part_b_forms`
`b_real_every_new_form_runs_at_exit_zero`, `secret_channels_t2` x2, `t3prime`, `t6`, `t9`), the set FIX
sec-gui-allowlist-bracket §3 records as failing on an unpatched base when the local CLIs are not the pinned ones;
`cargo clippy --locked --all-targets -- -D warnings`: exit 0. The `Cargo.lock` change was the one line above
(`cargo update -p mnemonic-gui --offline`: "Locking 0 packages").

## 4. RED rows

The tests ship as a patch next to this file: `design/FIX_sec-gui-allowlist-shape.red.patch`. It edits
`tests/persist_typing_prefixes.rs` (adds 6 tests and a helper, adds one cell to an existing test, deletes the
residual pin) and adds `tests/tree_key_marker.rs` (1 test). Fake signet/testnet-shaped vectors only. Run on the
branch head, from the worktree root:

    git apply design/FIX_sec-gui-allowlist-shape.red.patch
    PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked --no-fail-fast --test persist_typing_prefixes

Expected on 7e1b498 (measured 2026-11-24, cargo 1.97.0-nightly eb9b60f1f, the default toolchain here; CI runs `@stable`), exit 100:

    Summary [...] 8 tests run: 2 passed, 6 failed, 0 skipped

| row (test name, `persist_typing_prefixes`) | 7e1b498 | after the fix | what it shows |
|---|---|---|---|
| `s1_rule_s_blanks_separator_and_overlength_secrets` | FAIL, `secret survived persist for: ["xpub\|wif", "xpub/hex64", "xpubwif", "[origin]tpub/wif", "tpub wif", "tpub,wif", "tpubtprv", "tpub+tprv"]` | PASS | recon (a)-(d): a secret after a separator or concatenated past 111 blanks (Rule S) |
| `s2_rule_s_blanks_an_eleven_digit_suffix_run` | FAIL, `secret survived persist for: ["tpub/11 digits", "tpub/0/11 digits h", "[origin]tpub/<0;11 digits>", "tpub/77 digits"]` | PASS | a suffix digit run of 11 or more blanks (Rule S bound) |
| `e1_rule_e_blanks_a_whole_wif_inside_an_incomplete_key` | FAIL, `secret survived persist for: ["tpub[..40]+wif", "tpub+wif", "[origin]tpub[..40]+wif", "tpub[..40]+wif+space"]` | PASS | a whole signet WIF inside a key Rule S keeps blanks (Rule E.2) |
| `e2_rule_e_blanks_64_hex_inside_an_incomplete_key` | FAIL, `secret survived persist for: ["tpub[..20]+hex64", "tpub+HEX64", "[origin]tpub[..20]+hex64"]` | PASS | 64 hex digits inside a key Rule S keeps blank (Rule E.1) |
| `q1_secret_rows_still_blank_with_prefix_closure` (edited) | FAIL, `secret survived persist for: ["tpub/77 digits"]` | PASS | the architect's 77-digit cell; the 8 earlier cells still pass |
| `k1_typing_in_progress_public_keys_persist` | FAIL, `honest typing blanked: ["tp ", "[73c5da0a/84h "]` | PASS | honest half-typed keys persist (the architect's examples, the 10-digit index, trailing whitespace) |
| `k2_live_field_is_never_changed` | PASS | PASS | guard: `redacted_for_persistence` leaves the live tree equal to its clone; only the saved copy blanks |
| `q1_every_prefix_of_an_honest_key_persists` (unedited) | PASS | PASS | guard: BRIAN-Q-1, 1458 prefixes, none blanked |

    PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked --no-fail-fast --test tree_key_marker

Expected on 7e1b498 (measured 2026-11-24), exit 101, the compile failure (no `key_marker` yet):

    error[E0432]: unresolved import `mnemonic_gui::form::tree_model::key_marker`
     --> tests/tree_key_marker.rs:5:5

| row (test name, `tree_key_marker`) | 7e1b498 | after the fix | what it shows |
|---|---|---|---|
| `m1_key_marker_texts` | does not compile (E0432) | PASS | the 5 marker texts, their colour, and no line for an empty, whole, origin+whole, suffixed, trailing-space or xprv value (16 cells) |

The build applies the patch as its RED commit, sees both results above (`--no-fail-fast` keeps the first independent of
the job count), then makes the section-3 change and sees `8 tests run: 8 passed` and `1 test run: 1 passed`.

## 5. Tests that change

- Edited: `persist_typing_prefixes::q1_secret_rows_still_blank_with_prefix_closure` gains the cell
  `tpub/77 digits` (architect Q1: "adds it to the prefix test's secret list").
- Deleted: `persist_typing_prefixes::q1_known_residual_concat_inside_incomplete_key_still_persists` (it pinned the
  open row, "flip it when Q3 lands"; Rule E is Q3). Its value `TPUB[..40]FWIF` is the first cell of
  `e1_rule_e_blanks_a_whole_wif_inside_an_incomplete_key`.
- Added: the 7 tests of section 4.
- Must stay green unedited: `q1_every_prefix_of_an_honest_key_persists` (BRIAN-Q-1, 1458 prefixes),
  `persist_redaction_v0_34_0::t5*`, `t6*`, `t7*`, `t7b*`, `t7c*`, `form::tree_model::tests::*`,
  `widget_secret_mask_cycle15g` `t7_tree_key*` (4), `ui_harness_i3_secret_nopersist`, `secrets`.

## 6. Gate (full CI of mnemonic-gui; the builder runs all of it on the final head)

    PATH=$HOME/.cargo/bin:$PATH sh -c 'cargo clippy --locked --all-targets -- -D warnings && cargo clippy --locked --no-default-features -- -D warnings && cargo build --locked -p mnemonic-gui --no-default-features && cargo +1.88.0 check --locked && cargo nextest run --locked --workspace --no-fail-fast && cargo test --locked --workspace --doc && MNEMONIC_BIN=mnemonic python3 scripts/secret-channels-mutations.py' > /scratch/cache/sec-gui-allowlist-shape.gate.log 2>&1

Not covered locally: the `snapshots`, `tutorial-snapshots` (lavapipe) and `schema-mirror` real-binary jobs, and the
PARTB rows of `secret-channels-mutations.py`, need the pinned binaries; take them as green CI on the PR head.
`key_hint` changes the tree form's rendering under a key field, no snapshot shows a non-empty tree key field, so no image is expected to change; if CI shows a diff, that is a finding.

## 7. Review class

`scripts/review-class.sh` (coordinator) on the feasibility diff over 7e1b498 prints `B funds path (1 files): Cargo.lock`
and `D tests/docs/evidence (5 files)`: its path lists are mr/rw paths, so it sees mnemonic-gui `src/` as D and the
lock line as B. Override upward (the script says "never downward"): **class A** for the 5 new holder-visible marker
texts in `tree_model.rs`/`tree_form.rs`, on a secret-persistence path (the recon graded the unit A), with the
script's B kept for the lock line (no funds path is touched). Review: s1 Opus executed advice + design slice (run the
GUI or the unit rows over every marker line and every decision-table row), s2 Sonnet tests-can-fail against the
mutation-gate survivors.

## 8. Rows closing on GREEN, and residual

On GREEN: `gui-persist-allowlist-concat-residue` (123) closes; `fixdoc-secguibracket-r1-concat-residue` (404) closes
as its duplicate. Residual, not closed (filed as Minor secret rows by the FIX review r1): digit runs of at most 10 separated by `/`, `h` or the other suffix characters, in the suffix or in an origin path (word indices or a split decimal); a fragment of a WIF, tprv or hex shorter than the whole (including a whole WIF with one wrong character) inside an incomplete key; a BIP-39 phrase typed with no separators after a public prefix when no word has an `l`. A decimal secret of 64 or more digits blanks under E.1 (digits are hex digits). Closes no repo
FOLLOWUPS line; proposed lines, if any, are in the author report.
