# R0 round-2 architect review — SPEC_gui_v0_33_0_secret_flips_pin_bump (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). GUI 385d062 + toolkit 87c33c5. Verdict: GREEN (0 Critical / 0 Important / 2 new Minor — folded post-review before Phase 1). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

**M-NEW1 — §5's census name-list correction "5 → 7" is arithmetically wrong; the right post-flip count is 6.** Independently re-counted (script over `FlagSchema` blocks, cross-checked by grep): mnemonic.rs has exactly 4 distinct `secret: true` Boolean toggle NAMES today — `--passphrase-stdin` (×12), `--secret-stdin` (×2), `--decrypt-password-stdin` (×2), `--bip38-passphrase-stdin` (×1) = 17 sites; ms.rs adds the 18th SITE (`ms.rs:275` `--passphrase-stdin`, name-matched) but no new NAME; ms/mk/md have zero other secret Booleans (ms.rs's 8 `secret: true` sites are all Text — verified). The FOLLOWUPS.md:65 entry's body says "the 5 Boolean `secret: true` toggle names" while enumerating only 4 — the "5" is the entry's own pre-existing error, and round-1 M3's "5→7" anchored on it. Post-flip distinct names = 4 + `--phrase-stdin` + `--ms1-stdin` = **6**. The site count 18→24 is correct (17+1 → +6, re-verified). Fix §5: "name list → re-enumerate to the 6 names (the entry's current '5' already disagrees with its own 4-name enumeration)" — do not write "7" into the census.

**M-NEW2 — three cite/wording nits (substance verified correct in every case).** (a) §7 Phase 1 cites `tests/schema_mirror_secret_drift.rs:56-58` for the binary resolver; actual `fn resolve_mnemonic_bin` is **:54-56** (env read :55). The skip-block cite ":74-82" is acceptable (version<5 check :73-80; the silent-pass `return` is :85-90 if you want the exact skip site). (b) §3 cites ":104-112" for both halves of the gate-scope claim; the `schema::mnemonic::SCHEMA` walk is :105-112 ✓ but the `assert_eq!(cli, "mnemonic", …)` is at **:92**. (c) §2's redaction-union sentence self-contradicts: the union (`src/secrets.rs:323` `schema_secret_flag_names`, field-extracted over all 4 schemas) already CONTAINS `--phrase` (4 ms.rs `secret: true` sites) — as a name-set it does not "gain" it; the flips' new union members are `--phrase-stdin` and `--ms1-stdin` (verified absent today). Also cosmetic: §2's mode list "path-of-xpub / passphrase-of-xpub / account-of-descriptor" is not in the same order as the line triples (table order is path :2278 / account-of-descriptor :2440 / passphrase :2710); no pairing is claimed, so merely worth reordering for grep-followers.

## Fold-verification

**I1 — FOLDED-OK.** §7 Phase 1 now mandates `MNEMONIC_BIN=<locally-built v0.53.1 binary>` explicitly, states the gate has NO pinned-dep path (env-var else bare `mnemonic` on `$PATH` — verified :54-56), names the live false-green (`$PATH` binary re-confirmed `mnemonic 0.24.0` at `~/.cargo/bin/mnemonic` → pre-v5 → `fetch_v5_schema` returns `None` → silent pass-by-skip, verified :73-80 + :85-90), notes the Cargo dep feeds only byte-identical `secret_taxonomy` constants, and states the CI-only gate effect via `pinned-upstream.toml:22` → schema-mirror.yml `install-mnemonic-toolkit` (verified at :49-62, dynamic `TAG` from the pins step). The wrong "via the PINNED dep path" claim is gone. Modulo M-NEW2(a)'s 2-line cite slip, complete and correct.

**I2 — FOLDED-OK.** §8 risk (b) now says "three deterministic test breaks … not speculative", names all 3 cells with correct anchors (verified at 385d062: `cell_path_of_xpub_argv_assembles` — `--phrase` push :46, assert :75-76; `cell_account_of_descriptor_argv_assembles` — :117, assert :144; `cell_passphrase_of_xpub_argv_assembles` — :263, assert :290; all three push `--phrase` into `state.values` and assert emission), states the mechanism (`src/form/invocation.rs:255-273` verified: Text-secret branch reads ONLY `state.secret_widgets` then `continue`s → values-synthesized entry emits nothing), and prescribes the exact conversion with "do NOT delete the emission asserts" — seed via `SecretLineEdit::from_text` (verified `pub fn from_text` at `src/form/secret_widget.rs:55`; live pattern verified `tests/repeating_secret_rows.rs:210-218`). Survival-by-construction claims re-verified: `has_value` spans both maps (`src/schema/mod.rs:378-385` exact), `xpub_search_schema_mirror.rs:163-201` asserts only secret=**true** (both cells gate on per-flag presence — they go from vacuous-where-absent to live-and-passing post-flip, no break).

**M1 — FOLDED-OK.** §5 cites FOLLOWUPS.md:**17** for the audit-index line (verified: :17 is `secret-false-flags-render-cleartext-no-confirm`); §5 includes the toolkit-side ":81" → ":82" cross-cite fix at `mnemonic-toolkit/design/FOLLOWUPS.md:50` (verified: :50 carries `(:81)`; actual GUI header `ms-repair-ms1-not-secret-classified` is at :82).

**M2 — FOLDED-OK.** §8 now reads "paste-warn *eligibility* — the modal wiring is still dead code, see `paste-warn-modal-dead-code`", matching §2's wording.

**M3 — FOLDED-OK** (faithful to round-1's instruction: header "(18 sites)" → 24 AND name list update both present in §5; the 18→24 site arithmetic independently re-verified) — **but round-1's own "→7" was wrong; see M-NEW1.** The fold introduced no drift; the round-1 finding carried the error.

**M4 — FOLDED-OK.** §6 T2 hedge is gone; now correctly states NEW coverage with the round-1 rationale (`argv_assembler_visibility.rs:181-196` verified: `passphrase_typed_then_stdin_set_does_not_emit_typed_value` at :181 tests typed-value suppression UNDER a toggle, not the toggle's own no-emit) and the "pins the mechanism shared by all 24 census sites for the first time" framing.

## Verdict

**GREEN — 0 Critical / 0 Important** (2 new Minors: fix the census name-count to 6 and the three cite/wording nits before or during Phase 3's docs pass; neither blocks implementation start).

Re-measured baseline, all reproduced: `schema_mirror_secret_drift` vs the v0.53.1 binary (`MNEMONIC_BIN` set) is RED on **exactly the 9 expected pairs with `only_in_gui` empty**; `gui_schema_conditional_drift` (1 pass) + `archetype_schema_mirror` (5 pass) GREEN vs v0.53.1; `schema_mirror` 19/21 with only the `ms_schema`/`mk_schema` help cells red (the documented stale-`~/.cargo/bin` artifact — `ms 0.4.0` confirmed on PATH). Source re-verification all clean at GUI 385d062 / toolkit 87c33c5: the 9 flip sites (`--phrase` secret-lines :2286/:2448/:2718; `--phrase-stdin` blocks :2291/:2453/:2723; `--ms1-stdin` blocks :2312/:2474/:2744; address-of-xpub table :2612 correctly excluded), `ms.rs:321` `secret: false` on required-Text `--ms1`, the 7 mnemonic.rs `--ms1` `secret: true` twins (counted: exactly 7), `SECRET_FLAG_NAMES` 3-token legacy set at `secrets.rs:141-145` (§5's "needs NO additions" correction stands — the gate mirrors `FlagSchema.secret` per pair, not tokens), all 6 pin sites (Cargo.toml:42, pinned-upstream.toml:22, README.md:50 + :42 self-pin, mnemonic.rs:3949 + :1, tag commit `87c33c5` = `mnemonic-toolkit-v0.53.1^{commit}` confirmed), the NOT-pin transcription comments, schema-mirror.yml firing on master push AND `mnemonic-gui-v*` tags (Phase 3's claims hold), FOLLOWUPS headers :65/:73/:82, and §3's adjudication basis (c) verified live — `ms repair --help` itself says "secret material on stdout … ms1 is BIP-39 entropy and sensitive". The `envelope_v0_27_0.json` caveat is correctly hedged (sole consumer `tests/cli_envelope_smoke.rs` is input-only — include_str + form-filling, no mk1-byte comparison found). Proceed to implementation.
