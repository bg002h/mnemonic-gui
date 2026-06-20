# Changelog

All notable changes to `mnemonic-gui` are recorded here. Follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## mnemonic-gui [0.43.0] — 2026-06-19

**SemVer-MINOR — schema-mirror lockstep for `mnemonic-toolkit-v0.59.0`'s `#28` keyless single-sig template flags.** The toolkit added three new clap flags across `bundle` / `restore` / `verify-bundle` for keyless, fingerprint-stripped, canonical-origin-elided single-sig template md1 cards; this release mirrors them into the GUI clap-flag schema and bumps the toolkit pin.

- **`bundle --md1-form <policy|template>`** added to `BUNDLE_FLAGS` (`src/schema/mnemonic.rs`) as a **value-enum dropdown** (new `MD1_FORMS` const = `["policy", "template"]`, mirroring `Md1Form::Policy`/`Template`), `default_value: Some("policy")`. `policy` (default) = the pre-#28 full wallet-policy md1; `template` = the keyless single-sig template (requires `descriptor.n == 1` + a single-leaf shape; account/origin supplied at restore). `schema_mirror` gates flag NAMES only — the new dropdown VALUE list is the paired-PR discipline (measured clean vs the v0.59.0 `gui-schema`), same precedent as `--network`/`--template`/`--archetype`.
- **`restore --origin <ORIGIN>` + `restore --expect-wallet-id <PREFIX>`** added to `RESTORE_FLAGS` (both `FlagKind::Text`, non-secret): `--origin` supplies the derivation path the template elided for keyless single-sig template restore; `--expect-wallet-id` recomputes the `WalletPolicyId` from the completed fully-keyed explicit-origin descriptor and refuses on prefix mismatch (skipped when `--origin` overrides the canonical account path).
- **`verify-bundle --origin <ORIGIN>` + `verify-bundle --expect-wallet-id <PREFIX>`** added to `VERIFY_BUNDLE_FLAGS` (both `FlagKind::Text`, non-secret) — the verify/recompose mirror of the `restore` pair for a keyless template bundle.
- **Pin bump:** `mnemonic-toolkit-v0.58.0` → `v0.59.0` (Cargo.toml dep tag + Cargo.lock rev `d72856f1` + `pinned-upstream.toml` `[mnemonic].tag` + README install line + schema module-doc header + `pinned_version` string). The intervening toolkit releases v0.58.1 (convert mk1 SLIP-0132 hint) + v0.58.2 (faithful non-taproot per-cosigner use-site restore) added no clap flag NAMES, so the pin jump v0.58.0→v0.59.0 surfaces exactly the three #28 flags — no accumulated lagging-gate drift. The toolkit's transitive `md-codec` dep moved 0.36.0 → 0.37.0 in `Cargo.lock` as a consequence of the pin bump; the `secrets` `secret_taxonomy` compile-time guard recompiled clean.
- **Gates:** `schema_mirror` (4 CLIs) GREEN against the v0.59.0 binary (`MNEMONIC_BIN`) + the unchanged md/ms/mk pins; `pin_coherence` + `readme_pin_coherence` green; full suite green.

## mnemonic-gui [0.42.0] — 2026-06-17

**SemVer-MINOR — schema-mirror catch-up for `import-wallet --format descriptor` (toolkit v0.58.0, Tier-2 C5).** The toolkit added a new `descriptor` value to `import-wallet --format` (a generic commented-descriptor intake — reads a watch-only descriptor from text, tolerating `#`-comments; singlesig + multisig; explicit-only); this release mirrors it into the GUI dropdown and bumps the toolkit pin.

- **`IMPORT_WALLET_FORMATS` gains `"descriptor"`** (`src/schema/mnemonic.rs`, alphabetically after `coldcard-multisig`). The GUI import-wallet `--format` dropdown now offers all 9 toolkit formats. Note: `schema_mirror` gates flag-NAMES only (not dropdown VALUES), so this value addition is the **paired-PR discipline**, not a gate-caught drift — measured clean against the v0.58.0 `gui-schema`.
- **Pin bump:** `mnemonic-toolkit-v0.56.0` → `v0.58.0` (Cargo.toml dep tag + Cargo.lock rev + `pinned-upstream.toml` + README install line + schema module-doc + `pinned_version` string). The intervening toolkit releases v0.57.0 (verify-bundle BIP-388 intake) + v0.57.1 (unrestorable-shape advisory) added no clap flag NAMES, so the pin jump v0.56.0→v0.58.0 surfaces only the new `descriptor` value — `schema_mirror` (flag-name parity) stays green. The v0.56.0→v0.58.0 toolkit jump recompiled the `secrets` `secret_taxonomy` compile-time guard clean.

## mnemonic-gui [0.41.0] — 2026-06-15

**SemVer-MINOR — schema-mirror catch-up for the cross-repo mstring display-grouping cycle.** The four constellation CLIs gained uniform display-grouping flags (`--group-size <u16>`, `--separator <space|hyphen|comma>`) on their emit subcommands; this release mirrors them into the GUI clap-flag schema and bumps all four upstream pins to the grouping-enabled releases (final phase, P5).

- **Flags added to `src/schema/{mnemonic,md,ms,mk}.rs`** on exactly the subcommands that gained them upstream: `mnemonic` {bundle, convert, ms-shares-split, ms-shares-combine}; `md` {encode}; `ms` {encode, split}; `mk` {encode}. `--group-size` is a `Number{0..=65535}` (default 5; 0 = unbroken); `--separator` is an **I7 keyword dropdown** (`space|hyphen|comma`, default space). The delta was measured empirically against the v0.56.0 / 0.7.0 / 0.8.0 / 0.9.0 binaries via the `gui-schema` JSON path — purely additive, with no accumulated lagging-gate drift on any other declared subcommand.
- **I7 (SPEC §I7):** the toolkit reports `--separator` as kind `text` (its `value_parser` accepts keyword OR literal). The GUI deliberately narrows it to a keyword `Dropdown` so the GUI→argv path never emits a literal space (whitespace ambiguity). `schema_mirror` gates flag NAMES only, so the Dropdown-vs-text kind divergence is invariant-safe (same precedent as the archetype/build-format dropdowns). `default_value: Some("space")`/`Some("5")` suppress the flags from argv when left at default (`is_at_default`), so no `""` sentinel is needed.
- **Pin bumps (six edit categories, lockstep):** `mnemonic-toolkit-v0.53.1`→`v0.56.0` (Cargo.toml dep tag + Cargo.lock rev `a1dcff82` + `pinned-upstream.toml` + README), `descriptor-mnemonic-md-cli-v0.6.2`→`v0.7.0`, `ms-cli-v0.7.0`→`v0.8.0`, `mk-cli-v0.7.0`→`v0.9.0`; module-doc + `pinned_version` strings updated to each binary's `--version`. The v0.53.1→v0.56.0 toolkit jump recompiled the `secrets` `secret_taxonomy` compile-time guard clean (grouping touched no secret taxonomy).
- **Gates:** `schema_mirror` (4 CLIs) GREEN against the new binaries; `pin_coherence`, `readme_pin_coherence`, `secret_taxonomy_pin`, `schema_mirror_secret_drift`, `gui_schema_conditional_drift`, `archetype_schema_mirror` all green; `clippy -D warnings` clean. R0 plan-review GREEN (0C/0I); review persisted to `design/agent-reports/`.

## mnemonic-gui [0.40.0] — 2026-06-11

**SemVer-MINOR — the paste-warn modal is WIRED: pasting secret-length material into a secret field now fires an informational warning.**

- `should_warn_on_paste` + `PASTE_WARN_MODAL_TEXT` + `PASTE_WARN_THRESHOLD` had been DEAD code since v0.2 (no `src/` caller; `SecretLineEdit::show` did no paste detection) — the documented mitigation did not exist. Now `SecretLineEdit::show` detects an over-threshold (`>= 8` chars) `egui::Event::Paste` into the FOCUSED field and raises a signal; `update()` reads it once per frame and shows an informational, non-blocking modal (`PASTE_WARN_MODAL_TEXT` — clipboard-manager / paste-history caveats) with a Dismiss button. Resolves `paste-warn-modal-dead-code` + `paste-warn-live-wiring-untested`. Last item of the GUI secret-exposure cluster (Item 2 = v0.38.0, Item 1 = v0.39.0).
- **Signal routing = an egui Context-data bus, NOT a per-call-site return.** `show` sets a `ctx.data_mut` temp flag (`secret_widget::paste_warn_id()`); `update()` `remove_temp`s it (read-AND-clear, so no leak into the next frame). Because the flag is set INSIDE `show`, `show` is the single chokepoint covering all 3 direct call sites (`widget.rs` scalar + repeating, `main.rs` positional) AND every transitive archetype-form path — no signature change, no forgotten site. `Event::Paste` stays in `input.events` after the TextEdit reads it (egui 0.31 `filtered_events` clones), confirmed from source.
- **Attribution via `response.changed()`:** a paste only changes the focused field's buffer, so only the recipient field raises the flag — no false-positive from a paste into a non-secret field, no multi-row double-trigger. Benign false-negative: pasting text identical to the field's current content doesn't `changed()` → no warn.
- Tests: `tests/paste_warn_wiring_v0_40_0.rs` (4 — T-B2 live kittest `Event::Paste` injection ≥/< threshold [RED-proven via scratch-revert], T-B3 read-once+clear no-leak, T-B4 3-direct-site chokepoint pin). SPEC + R0 ×2 GREEN: `design/SPEC_gui_v0_40_0_paste_warn_wiring.md`, `design/agent-reports/gui-v0-40-0-paste-warn-r0-round{1,2}-review.md`.
- **Scope honesty:** the warn fires on **`SecretLineEdit`** paste events. Secret `NodeValueComposite` value fields (`--from phrase=…`) and secret SLOT fields (`@N.phrase=…`) use different widgets and are NOT yet covered — tracked as `composite-paste-warn-parity` + `slot-field-paste-warn-uncovered`. No clap flag / secret-bit / schema_mirror / manual / toolkit-pin change.

## mnemonic-gui [0.39.0] — 2026-06-11

**SemVer-MINOR — secret values are masked in every on-screen command display (preview / run-confirm modal / last-run argv); copying the real runnable command is a deliberate, labeled action.**

- `assemble_argv` pushed real secret VALUES into argv, and `render_copy_command` shell-quoted them verbatim — so the always-on-screen `Preview:` label, the run-confirm modal's token list (the modal that exists *because* a secret is present printed it cleartext), and the last-run `argv:` line (under "show command-line") all leaked secrets. Now a parallel secret-mask, computed at argv-assembly time, drives a masked display: every secret value token renders as a fixed `••••` sentinel (un-quoted; never run). Resolves `run-confirm-and-preview-show-secrets-cleartext` (user decision (d): mask the display, reveal on deliberate copy).
- **Four secret-value sources covered** — the same four `should_confirm_run` classifies: (1) secret Text flag values (`--passphrase`), (2) secret-bearing slot value tokens (`@N.phrase=…`), (3) secret positionals (`ms combine <shares>`), (4) value-dependent `NodeValueComposite` values — both the secret flag `--share` and the secrecy-by-node `--from phrase=<seed>` (while `--from xpub=…` is correctly NOT masked). The mask is correct-by-construction: every `argv.push` pairs exactly one `mask.push` (`mask.len() == argv.len()`, debug-asserted), set `true` iff the token is a secret value.
- **The copy buttons still copy the REAL runnable command** (the masked display is never copied or run) — when a secret is present the button relabels `Copy command (POSIX/Windows) — reveals secret` so the reveal is an informed click. Run / the run-confirm "Run" spawn the real argv unchanged.
- New: `assemble_argv_with_secret_mask`, `render_copy_command_masked`, `SECRET_MASK`, `SlotState::to_slot_argv_masked` (the old `to_slot_argv` now thin-delegates); `RunResult` gains a mask field (the runner layer stays mask-oblivious — `spawn_and_capture` attaches the assembly-time mask). No clap flag / secret-bit / schema_mirror / manual / toolkit-pin change.
- **Anti-split-brain pin (T-A3):** every state where a token is masked also gates the run-confirm modal (`mask.any() ⟹ should_confirm_run`) — a secret can never render masked while the run skips the confirm. The safe asymmetry (a greyed `*-stdin` toggle confirms but emits no token) is documented, not falsely asserted.
- Tests: `tests/secret_mask_preview_v0_39_0.rs` (11; per-source mask correctness, masked-render-hides / real-render-reveals, dangerous-direction, preview string — composite + slot RED-proven via scratch-revert). Second of the GUI secret-exposure cluster; Item 3 (paste-warn wiring) is v0.40.0. SPEC + R0 ×3 + impl review: `design/SPEC_gui_v0_39_0_mask_preview_paste_warn.md`, `design/agent-reports/gui-v0-39-0-mask-{paste-warn-r0-round1,r0-round2,r0-round3,impl}-review.md`.

## mnemonic-gui [0.38.0] — 2026-06-10

**SemVer-MINOR — secret-bearing slot values render masked, and zeroize on row removal.**

- The SlotEditor's value field rendered every subkey in plaintext (`slot_editor.rs` branched only on the Path hint). Secret-bearing subkeys (Phrase / Seedqr / Entropy / Ms1 / Wif / Xprv) now render as masked password fields (`.password(true)`, gated on `is_secret_bearing()` FIRST — Path is never secret, so masking never combines with the path hint). Watch-only subkeys (Xpub / MasterXpub / Fingerprint / Path) are unchanged.
- Removing a slot row now zeroizes a secret-bearing value first (`SlotRow::zeroize_if_secret` + a free `remove_row` seam) — previously the plain `String` was dropped without scrubbing. Best-effort (current heap buffer only; the allocator-residue limit is the same one `SecretLineEdit` documents). Secret slot values already never persisted (`redact_for_persistence` drops secret-subkey rows) — this is the render-side + in-memory residue half.
- **Split-brain pin:** the new render/zeroize gate (`is_secret_bearing()`) and the persistence gate (the toolkit-imported `SECRET_SLOT_SUBKEYS`) are now asserted EQUAL for all 10 `SlotSubkey` variants (T3) — no prior test pinned them across all variants (the closest omitted Seedqr+Ms1), so a future variant could have rendered plaintext while being persist-redacted.
- Tests: `tests/slot_secret_mask_v0_38_0.rs` (T1 zeroize-on-remove, T2 kittest `Role::PasswordInput` positive+negative — verified RED without the mask, T3 the split-brain pin). First of the GUI secret-exposure cluster (preview/copy/modal masking + paste-warn wiring follow). Resolves `slot-secret-values-rendered-unmasked`. SPEC + R0 ×2: `design/SPEC_gui_v0_38_0_slot_secret_mask.md`, `design/agent-reports/gui-v0-38-0-slot-mask-r0-round{1,2}-review.md`.

## mnemonic-gui [0.37.0] — 2026-06-10

**SemVer-MINOR — the suppressed `*-stdin` secret toggles render disabled (greyed), so the dead checkbox isn't a lie.**

- The 6 `*-stdin` secret toggle names (24 sites: `--passphrase-stdin`, `--secret-stdin`, `--decrypt-password-stdin`, `--bip38-passphrase-stdin`, `--phrase-stdin`, `--ms1-stdin`) rendered as live checkboxes whose checked state never reached argv (the GUI runner has no stdin channel to feed them — `assemble_argv`'s secret branch suppresses them). A user could check a control that did nothing. Per the user's decision (grey out, not emit), `render_with_dispatch` now renders these disabled with a tooltip ("stdin toggles can't be driven from the GUI — type the value in the inline field, or run the CLI directly").
- **Predicate-mirror, no drift:** the grey-out condition (`flag_is_secret && FlagKind::Boolean`) is EXACTLY the assembler's suppressed set — verified by a converse-closure invariant test that fails RED if any future secret non-Text/non-Composite flag (e.g. a secret `Path`) is added that would be suppressed but not greyed. No state.values writeback (early return), so the flag stays absent from argv.
- Tests: `tests/greyout_stdin_toggles_v0_37_0.rs` — T1 (predicate==suppression across all 4 schemas + the converse-closure), T2 (kittest: the `--passphrase-stdin` checkbox renders disabled; verified RED without the branch). No schema/secret-bit change → `schema_mirror` + drift gates byte-unaffected; `assemble_argv` byte-identical. Resolves `boolean-stdin-secret-toggles-never-emit`. SPEC + R0: `design/SPEC_gui_v0_37_0_greyout_stdin_toggles.md`, `design/agent-reports/gui-v0-37-0-greyout-r0-round1-review.md`.

## mnemonic-gui [0.36.0] — 2026-06-10

**SemVer-MINOR — debounced autosave + atomic `state.json` writes (closes v0.35.0's two recorded gaps: crash session-loss + signal-torn writes).**

- **Atomic `save()`:** writes a per-PID sibling temp (`state.json.<pid>.tmp`) then `fs::rename`s over the destination. The PID suffix is load-bearing — a shared fixed-name temp lets two instances interleave writes and atomically install TORN content (the rename is atomic, the bytes aren't), which the malformed→`.bak` recovery would then convert into silent session resets; per-process names reduce two-instance contention to plain last-writer-wins. `fs::rename` replace-on-existing is the documented std contract on both Unix and Windows — no remove-then-rename fallback (it would recreate a destination-absent window). Power loss before the rename leaves the old file intact; the `.bak` leg recovers a torn post-rename file.
- **Change-gated autosave every ~30 s:** new `persistence::save_if_changed` serializes the redacted state once and skips byte-identical content (debounce-by-content, zero dirty-flag plumbing; correctness rides the `BTreeMap` fields' deterministic serialization — do not refactor them to `HashMap`); the cache updates only after a successful write so failures retry next interval. The timer rides `update()` after the geometry snapshot (the 1 Hz keepalive guarantees evaluation even when idle); the construction reuses the borrow-side `build_persisted_state()` extraction (never `mem::take` — the v0.35.0 invariants and their comments move verbatim); the final `on_exit` save stays unconditional with the load-bearing save-then-zeroize order untouched.
- Tests: `tests/persistence_autosave_v0_36_0.rs` (4 cells, TDD red-first: PID-temp consumption + no-stray-siblings, rename-over-existing, write/skip/write-on-change with the deleted-file no-recreate pin, failed-write-does-not-poison-the-cache). Resolves `gui-persistence-autosave-debounce`. SPEC + R0 ×3 + impl review: `design/SPEC_gui_v0_36_0_autosave_atomic.md`, `design/agent-reports/gui-v0-36-0-autosave-*.md`.

## mnemonic-gui [0.35.0] — 2026-06-10

**SemVer-MINOR — Phase-8 persistence WIRED: session restore via `state.json` (window geometry, tab/subcommand selections, output toggles, non-secret form values), saved on exit.**

- The feature-complete persistence module (built Phase 8, hardened v0.31.1→v0.34.0, previously ZERO callers) is wired into the app lifecycle per the converged §10 design: `main()` loads `state.json` before `run_native` (restored geometry seeds the `ViewportBuilder`); `new()` restores tab (validated + availability-fallback via the new `CliTab::from_bin_name`), per-tab subcommand (schema-validated, fallback to defaults — the new lib `persistence::restore_selections` helper), the form-state map (direct move; the demo seed applies only when its key is absent), and the 3 output toggles; `update()` snapshots geometry per frame (`Some`-guarded — egui sets the rects `None` while minimized and the keepalive thread keeps frames firing); `on_exit()` saves FIRST then runs the zeroize sweep (order is LOAD-BEARING: the sweep blanks every string-bearing value, not just secrets) with the `PersistedState` built borrow-side via `redact_for_persistence` (NOT `mem::take`, which would silently no-op the sweep).
- Robustness: a malformed `state.json` is now renamed `.json.bak` like a version mismatch (corrupt files preserved for diagnosis — also gracefully covers a signal-torn write); `MNEMONIC_GUI_STATE_PATH` overrides the path (documented; also the test seam — env-mutating cells isolated to a dedicated test binary); state path resolved ONCE and stored. Restored fresh-start toggles fixed against the `Default`-vs-serde-default trap (`unwrap_or_default` would have flipped them off).
- Secrets: nothing changes — `secret_widgets` is `#[serde(skip)]` (cannot round-trip, by type), and the end-to-end cell re-pins it at the wiring layer. Stale flag names in restored values are inert (schema-driven render/argv); stale tab/subcommand fall back safely.
- Tests: `tests/persistence_wiring_v0_35_0.rs` (10 cells: from_bin_name, restore validation ×5, `.bak`-on-malformed (RED-first ×1 + 2 symmetry/missing-file re-pins), end-to-end six-group round-trip + secret non-survival) + `tests/persistence_env_seam.rs` (the sole env-mutating cell). Resolves `persistence-unwired-redaction-never-runs`. SPEC + R0 ×2 + impl review: `design/SPEC_gui_v0_35_0_persistence_wiring.md`, `design/agent-reports/gui-v0-35-0-persistence-wiring-*.md`; recon `cycle-prep-recon-phase8-persistence-wiring.md`.

## mnemonic-gui [0.34.0] — 2026-06-10

**SemVer-MINOR — persistence-redaction hardening (audit I5 + I6): secret POSITIONALS never persist (by type) and the tree persist walk flips to a positive extended-public allowlist.**

- **`PositionalArgSchema.secret: bool`** (+ all 20 literal sites; census: exactly 5 secret, all ms.rs — `inspect`/`decode`/`verify`/`derive` bare `ms1` + `combine shares` — pinned by a frozen non-circular census cell). Secret positionals render as per-row masked `SecretLineEdit`s stored under the `secret_widgets["positional:<name>"]` reserved key (`#[serde(skip)]` — never-persist holds by TYPE; the `positional:` prefix cannot collide with `--`-prefixed flag keys), with the v0.31.1 add/remove row chrome for the repeating `shares`. The assembler emits them at argv end from the widget rows; stale `state.positionals` content is ignored for them. `should_confirm_run` gains a secret-positional loop. NEW INVARIANT: `has_positional` is permanently false for secret positionals (its only two callers are the non-secret md template/phrases conditionals — verified).
- **Redaction belt:** `redact_for_persistence` now drops ALL `state.positionals` content — the layer has no subcommand context, so the belt is unconditional/fail-closed (non-secret positionals are cheap re-pastes; persistence is still unwired and no `state.json` exists in the wild — no schema-version bump needed). Resolves `positional-secrets-not-redacted-at-persist` (audit I5).
- **Tree persist walk: deny-list → POSITIVE allowlist.** `blank_xprv_keys` (xprv-prefix-only — WIF / raw-hex private keys survived in cleartext) is replaced by `blank_non_extended_public_keys`: a key/keys entry survives persist ONLY if origin-stripped it starts with one of the 10 SLIP-132 extended-public prefixes (`xpub/tpub/ypub/Ypub/zpub/Zpub/upub/Upub/vpub/Vpub` — full 4-byte literals: a bare "`pub` at 1..4" probe would keep `Kpub…`-form WIFs). Documented fail-closed cost: raw-hex PUBLIC keys (shape-indistinguishable from hex private keys; no `from_str` backstop on this path) blank too. `is_xprv_like` stays for the render-side amber hint only. Resolves audit I6 `tree-wif-hex-privkey-in-key-fields-unredacted`.
- **Doc corrections:** the `SubcommandSchema.positional_args` doc claimed "mnemonic-toolkit's subcommands have zero positionals" — FALSE: the v0.53.1 `gui-schema` emits positionals for 7 subcommands, 6 secret-capable (everything except `decode-address`); this hand schema deliberately mirrors only `decode-address`. Filed `toolkit-secret-capable-positionals-unmirrored`.
- Tests: `tests/persist_redaction_v0_34_0.rs` (8 cells, TDD-red-first: redaction belt + serialized-absence, widget-row emission/row-order/blank-rows, stale-positional no-emit, non-secret path regression, run-confirm, frozen census, 14-row allowlist table incl. the `Kpub…` false-accept class). SPEC + R0 ×2 + impl review: `design/SPEC_gui_v0_34_0_persist_redaction_hardening.md`, `design/agent-reports/gui-v0-34-0-persist-redaction-*.md`.

## mnemonic-gui [0.33.0] — 2026-06-10

**SemVer-MINOR — toolkit pin v0.52.0 → v0.53.1 + the master-phrase secret flips (audit I4 GUI half): `xpub-search --phrase` finally renders MASKED with run-confirm.**

- **Toolkit pin bump v0.52.0 → v0.53.1** (the 6 lockstep sites: Cargo.toml + Cargo.lock + `pinned-upstream.toml` + README install line + `pinned_version` banner + module-doc). The bump carries toolkit v0.53.0 (multisig mk1 csi slot-unique — card bytes only, no schema surface) + v0.53.1 (the secret classifications). Measured pre-edit: the only gui-schema delta v0.52.0 → v0.53.1 is the 9 secret bits below; flag-NAME/conditional/archetype gates all green.
- **9 `FlagSchema.secret` flips** (`src/schema/mnemonic.rs`): `--phrase` / `--phrase-stdin` / `--ms1-stdin` × the three xpub-search modes (`path-of-xpub`, `account-of-descriptor`, `passphrase-of-xpub`). A raw master BIP-39 phrase typed into the GUI now renders as a masked `SecretLineEdit` with exit-zeroize, triggers the run-confirm modal, and the two stdin toggles join the redaction union. Resolves `xpub-search-inline-phrase-not-secret-classified` (= audit I4's mnemonic.rs half; toolkit source-of-truth fixed in v0.53.1).
- **`ms repair --ms1` → `secret: true` by deliberate GUI-side override** (`src/schema/ms.rs`): the to-be-repaired ms1 is master-secret material (BCH-corrupted BIP-39 entropy) — the lone false twin of the 8-site `--ms1` census. No automated gate covers ms.rs secret bits and ms-cli has no gui-schema surface to mirror, so the override is recorded at the site + pinned by `tests/secret_flips_v0_33_0.rs::t1`. Resolves `ms-repair-ms1-not-secret-classified` (= audit I4's ms.rs half).
- **Boolean stdin-toggle census 18 → 24** (`boolean-stdin-secret-toggles-never-emit`): the flipped `--phrase-stdin` ×3 / `--ms1-stdin` ×3 move from generic-checkbox emission into the assembler's secret-Boolean suppression — deliberate (the GUI runner has no stdin feed; a checked toggle previously produced a CLI that hangs awaiting stdin). The no-emit mechanism is pinned for the first time (`t2`).
- **Tests:** new `tests/secret_flips_v0_33_0.rs` (override pin + stdin-toggle no-emit + tri-mode `--phrase` secrecy); the 3 `xpub_search_widgets` argv cells converted to seed `secret_widgets` (the live form's routing — a values-synthesized Text-secret emits nothing by design) while KEEPING their `--phrase` emission asserts. The secret-drift gate goes green at the bumped pin (it was the forcing function: RED in both directions on exactly the 9 pairs).
- SPEC + R0 reviews: `design/SPEC_gui_v0_33_0_secret_flips_pin_bump.md`, `design/agent-reports/gui-v0-33-0-secret-flips-r0-round{1,2}-review.md`.

## mnemonic-gui [0.32.0] — 2026-06-10

**SemVer-MINOR — the recursive node-tree builder for `build-descriptor` (the wizard's tree stage): hand-build a miniscript policy tree node-by-node over the FULL 17-kind grammar, Validate it against the live binary with node-addressed diagnostic highlighting, and round-trip archetype presets into the tree via `--emit-spec`. NO toolkit pin change (stays `v0.52.0`) and NO `schema_mirror` delta (zero new flags — the tree is a GUI-side composition surface over the existing `--spec -` stdin contract).**

- **The node-tree model (`src/form/tree_model.rs` + `src/schema/nodes.rs`, library).** `NODE_KIND_SPECS`: the full 17-kind grammar (kind + payload shape + arity) transcribed from the v0.52.0 binary's `--spec-schema` `nodes` array, ORDER included (it is the kind-dropdown order). `TreeNode` is a WIDE node (every payload field coexists; switching kind preserves-and-flags rather than destroys) with `to_spec_json` / `from_spec_json` serializers projecting the active-kind fields + the in-arity child prefix; surplus children persist but never emit, and the render layer flags them amber ("surplus — will not emit" — the impl-review I1 cell pins both the flagged leg and the in-arity-silent leg). `TreeState` rides `FormState.tree` (persisted; `next_id` bookkeeping via the `import_root`/`recompute_next_id` invariant), with `diagnostics`/`validate_ok`/`strip` all `#[serde(skip)]` transient (wire-absence pinned by token, `strip` included — impl-review M3). Depth is capped (`MAX_TREE_DEPTH`) with an amber strip refusal, never a panic. **Persisted trees are xprv-redacted:** `redacted_for_persistence` blanks every `is_xprv_like` `key`/`keys[i]` entry (origin-prefix-aware — the toolkit gate's own heuristic; recursive incl. surplus children); hashlock hex digests survive.
- **The three-mode `build-descriptor` form (`src/form/tree_form.rs`, library — `main.rs` only dispatches).** A Generic / Archetype / Tree mode selector (never-destroys: switching modes preserves values, params, AND tree nodes; only the archetype dropdown SELECTION resets on the Generic gesture). Tree mode suppresses exactly the 12 mode-bound flags from argv (`--spec`, `--archetype`, the 9 params, `--emit-spec`) while the 6 mode-independent flags keep emitting; `conditional::build_descriptor` grows the tree arm (2 Disabled + 10 Hidden). Recursive node render: per-node kind ComboBox routing through `set_kind` (arity padding + the depth cap), payload widgets by shape, per-child remove (collect-then-apply), collapse/expand in egui memory (does NOT clear diagnostics). Every structural/payload edit routes through the `mark_dirty` chokepoint, clearing stale Validate results.
- **Validate (the live gate, in-form).** Fixed argv `build-descriptor --spec - --json` + the user's `--allow` rows passed through, spec piped via the new **`runner::run_with_stdin`** (the runner's first stdin channel; `run` delegates). The response parser is **descriptor-first — an accepted SPEC deviation:** the success envelope ALSO carries `"diagnostics": []` (probed against v0.52.0), so the `descriptor` key discriminates, not the diagnostics array. Node-addressed diagnostics (`root.or_d[1]`-style paths) tint the matching node red and render the message under it; unmatched paths (the `params` sentinel, version-skew drift) fail-soft to the global error strip — never dropped, never a panic; non-envelope failures route stderr to the strip; the `--allow` override banner (stderr on exit 0) strips too. Completeness gates Validate/Run/Copy alike (incomplete trees emit nothing).
- **Edit-as-tree (`--emit-spec`, archetype mode → tree).** One click lowers the selected archetype + its param rows to the spec AST via a **purpose-built argv — the second accepted SPEC deviation:** `--archetype` + the declared param rows (schema order) + `--emit-spec` only; the mode-independent flags must NOT ride (`--json`/`--format` clap-conflict with `--emit-spec`, probed). Success imports the AST (`import_root` — id renumbering + the `next_id` invariant) and switches to tree mode; failure stays in archetype mode with the binary's `[param]` diagnostics surfaced in a transient (never-persisted) error label. Filed: `edit-as-tree-overwrites-existing-tree` (a hand-built disabled tree is replaced unconditionally — confirm-on-overwrite posture deferred).
- **POSIX pipeline copy (tree mode).** A paste-runnable one-liner — `printf '%s' '<compact spec JSON>' | mnemonic build-descriptor … --spec -` — shlex-quoted (round-trip pinned byte-exact), carrying the user's mode-independent flags; gated on tree-mode + completeness. The exec cell runs the pasted string through a real `/bin/sh` against the real binary and asserts the descriptor comes back.
- **Three new drift gates (the GUI's standing idiom), incl. THE keystone:** `tests/spec_nodes_mirror.rs` (gate 1 — `NODE_KIND_SPECS` vs the pinned binary's `--spec-schema` `nodes`: kind set + order + payload strings byte-equal, plus refuse-loud pins on BOTH version axes `spec_schema_version == 1` / `supported_doc_schema_version == 1`); `tests/tree_path_parity.rs` (gate 2 — **live path-grammar parity**: the GUI's `child_path`/`key_path`/`node_paths` have no compile-time tether to the toolkit gate's path grammar; 6 planted-diagnostic recipes over all 5 path productions assert every live `node_path` equals the GUI-computed path AND matches a `node_paths()` entry); `tests/tree_round_trip.rs` (gate 3 — round-trip laws over the vendored fixtures + the wide-node projection law + serde/redaction cells). Skip-if-absent discipline throughout; CI exports `MNEMONIC_BIN`.
- **Test hygiene (impl-review folds).** `cell_2_tracing_init_logs_subprocess_spawn` de-flaked without a new dep (resolves `FOLLOWUPS.md::runner-tracing-test-flaky-under-parallel-load`): `set_default` is thread-local but tracing's callsite-interest cache is GLOBAL → `rebuild_interest_cache()` after subscriber install + up-to-3 fresh-subscriber retries (verified 10/10 consecutive `runner_integration` runs + a full-suite run, all green). Also filed: `tree-xprv-heuristic-only-covers-key-fields` (`hex`/`w` free-text fields could carry pasted xprv-like content unredacted — free belt-and-suspenders extension).
- **Coverage:** `tests/tree_form.rs` (22 cells — the mode-mutex argv matrix, Validate argv + parse contracts, selector gestures, kittest render/Validate-flow/edit-clears/depth-cap/surplus-flag cells, Edit-as-tree happy/failure/render-gate, pipeline quoting + real-shell exec), `tree_round_trip.rs` (13), `tree_path_parity.rs` (7), `spec_nodes_mirror.rs` (1), plus `run_with_stdin` unit cells in `src/runner.rs` and the model/grammar unit tests. Full suite green (4 pinned binaries) + clippy clean. Version `v0.31.1` → `v0.32.0` + README self-install pin. Audit trail: `design/SPEC_gui_v0_32_0_node_tree_builder.md` + `design/agent-reports/gui-v0_32_0-node-tree-r0-r{1,2}-review.md`.

## mnemonic-gui [0.31.1] — 2026-06-10

**SemVer-PATCH — repeating-secret flags reach argv (per-row `SecretLineEdit`). Resolves `FOLLOWUPS.md::repeating-secret-flags-never-reach-argv` with the fix direction INVERTED vs the entry** (routing secrets through `state.values` would have persisted seed material; the type-level never-persist invariant of `secret_widgets` is the safe rail). No flag-name change → no `schema_mirror` delta; no toolkit pin change.

- **THE bug (live, 4 sites):** secret + repeating + Text flags (`--ms1` on verify-bundle/import-wallet; `--share` on slip39-combine/ms-shares-combine, both **required**) rendered ONE `SecretLineEdit` into `state.secret_widgets` while `assemble_argv`'s repeating-secret arm read rows from `state.values` — a source the widget never wrote — so **live forms emitted NOTHING** for these flags (masked by cells that synthesized `state.values`). Fix: `FormState.secret_widgets` becomes **`BTreeMap<String, Vec<SecretLineEdit>>`** (scalar secrets = the 1-element vec, byte-identical behavior) and the assembler reads the per-row vec. **Lib-API note: this is a `pub` field TYPE change — a library-API break** (the crate is app-first; no external consumers known). The widget secret branch splits on `flag.repeating`: repeating secrets mirror the v0.30.0 multi-row UX (header label + `?` + required marker + "+ add" appending an empty row; per-row masked `TextEdit` + per-row `✕` collect-then-apply; per-frame required-zero-rows seed — `--share` seeds one row, optional `--ms1` seeds none; removing the last required row respawns it). Row order = vec order = argv order. Empty rows emit nothing (assembler-side guard).
- **Assembler secret branch is now KIND-GATED, mirroring the widget dispatch** (`flag_is_secret` is kind-blind): Text → the `secret_widgets` vec (unconditional `continue` — a values-synthesized Text-secret entry is a DEAD path); NodeValueComposite (seed-xor-combine `--share`) → falls through to the generic values paths (unchanged, the working counter-example); Boolean `*-stdin` secrets → `continue`, preserving the pre-existing no-emit (now recorded: FOLLOWUP `boolean-stdin-secret-toggles-never-emit`). The false v0.3 fold comment (values-routing intent + "paste-warn modal still fires" — the modal predicate exists but is not wired live) is rewritten as a supersession note. `FormState::has_value` gets per-row semantics (`rows.iter().any(|w| !w.is_empty())` — the silent `Vec::is_empty` inversion would have fired run-confirm on every blank required `--share` row); `zeroize_form_state` sweeps `values_mut().flatten()`.
- **Belt-and-suspenders persistence redaction (`secrets::schema_secret_flag_names()`):** `redact_for_persistence` additionally drops any `values` entry whose flag NAME is `secret: true` anywhere across the 4 CLI schemas (field-extracted union ∪ `SECRET_FLAG_NAMES`) — a name-level net closing the future-drift class. **Two LIVE plaintext-persistence leaks incidentally closed** via cross-CLI name collisions: (1) the three xpub-search `--phrase` (inline master BIP-39 phrase, `secret: false`) values entries no longer persist to `state.json`; (2) `ms repair --ms1` (master-secret material, `secret: false`) likewise. The underlying mis-classifications are filed: `xpub-search-inline-phrase-not-secret-classified`, `ms-repair-ms1-not-secret-classified`; the adjacent codex32 combine `shares` POSITIONAL leak (outside the name net, pre-existing) is filed as `positional-secrets-not-redacted-at-persist`. Deliberate side effect: the 5 Boolean `secret: true` `*-stdin` toggles join the union, so their persisted checkbox state resets across restarts (accepted — the toggles carry no secret material and a stale-persisted stdin toggle is a foot-gun).
- **Coverage:** new `tests/repeating_secret_rows.rs` (8 cells) — THE live-path bug cell drives the REAL widget via kittest `focus()`+`type_text()` into the masked rows (first in-repo use; egui registers password TextEdits as `Role::PasswordInput`) and asserts `--ms1 v1 --ms1 v2` in row order; the slip39 `--share` required-seed twin; required-row respawn; blank-row suppression; the values-synthesized dead-path cell (emits nothing + redacts at persist); the two §3 drift tests (every `secret: true` flag name redacts a dummy entry; union ⊇ {`--ms1`, `--share`} ∪ `SECRET_FLAG_NAMES`); scalar `--passphrase` live-typing regression. 3 cells migrated to the vec source (`cell_import_wallet_repeating_ms1_argv`, `cell_import_wallet_env_sentinel_literal_emission`, `cell_v0_3_slip39_combine_argv_assembles`); `cell_v0_3_seed_xor_combine_argv_assembles` KEPT unchanged as the NodeValueComposite counter-example pin; 8 mechanical `insert(name, vec![widget])` migrations (the `tests/secrets.rs` empty-widget negative migrates FAITHFULLY as `vec![SecretLineEdit::new()]` — the has_value net). Full suite green (4 pinned binaries) + clippy clean. Version `v0.31.0` → `v0.31.1` + README self-install pin. Audit trail: `design/SPEC_gui_v0_31_1_repeating_secrets.md` + `design/agent-reports/gui-v0_31_1-repeating-secrets-r0-r{1,2,3,4}-review.md`.

## mnemonic-gui [0.31.0] — 2026-06-09

**SemVer-MINOR — data-driven archetype param forms for `build-descriptor` (the wizard, archetype-forms stage). NO toolkit pin change** (the `archetypes` schema section shipped in toolkit `v0.51.0`; the `v0.52.0` pin is already past it) **and NO `schema_mirror` delta** (zero new flags — this cycle consumes surfaces that already exist).

- **Schema-driven archetype param form (`src/form/archetype_form.rs`, library).** Selecting a preset in the `--archetype` dropdown REPLACES the generic flag grid (for the 9 param flags + `--spec`) with a form rendering ONLY the selected archetype's parameters, in schema order, with the archetype's `summary` line directly under the dropdown. ParamKind → widget map (all v0.30.0 machinery): repeatable `key` → the repeating-row widget with a `(min N)` header annotation; archetype-NON-repeatable `key` → a single Text row, OR — when >1 row was carried over from another archetype — the repeating widget with "+ add" SUPPRESSED and an `(exactly 1)` annotation, so surplus rows stay visible/removable and **what emits is always what renders** (R0-r1 C1); `threshold` → Number whose `max` binds to the LIVE paired-key row count via the new `NumberMax::FromRowCount(&'static str)` (ALL rows incl. empty count; floored `.max(1)`; bespoke-synthesized `FlagSchema` only — the static entries keep `Static(20)`); `blocks`/`absolute_locktime` → the static Number widgets as-is; `hex_digest` → Text + a non-blocking `⚠ expected 64 hex chars` hint. The threshold↔key pairing is the static `paired_key_flag` fn (`--threshold`→`--key`, `--recovery-threshold`→`--recovery-key`), unit-tested against every archetype. `main.rs` only dispatches (the SlotEditor precedent): a name-set `continue` skips the 9 params + `--spec` in the generic loop (NOT `Visibility::Hidden` — declared params must emit); the 8 mode-independent flags (`--archetype`, `--spec-schema`, `--emit-spec`, `--allow`, `--format`, `--network`, `--json`, `--no-auto-repair`) keep their generic widgets. Deselecting ("(none)") restores the v0.30.0 generic form; rows of non-selected archetypes are left intact (data preservation).
- **Static mirror + drift gate (the GUI's standing idiom).** New `src/schema/archetypes.rs` (`ARCHETYPE_SPECS`: 5 archetypes × `flag`/`kind`/`required`/`repeatable`/`min` + `summary`, transcribed from the v0.52.0 binary's `build-descriptor --spec-schema` `archetypes` section) + new `tests/archetype_schema_mirror.rs` comparing the table **field-by-field including param ORDER** (load-bearing — render order) and `summary` verbatim against the pinned binary (skip-if-absent discipline; CI exports `MNEMONIC_BIN`). Self-consistency units: ids == `ARCHETYPES` minus the `""` sentinel (order included), every param flag ∈ `BUILD_DESCRIPTOR_FLAGS`, `ARCHETYPE_PARAM_FLAGS` == the exact declared-param union, kind-vocabulary pin.
- **Conditional extension (argv stays declarative).** `conditional::build_descriptor`: with a selected archetype, each of the 9 param flags NOT declared by it is `Hidden` (the assembler's established Hidden/Disabled suppression path) — stale rows carried over from another archetype never emit; declared params get NO entry. `--spec` handling UNCHANGED (still `Disabled`; cell_13 still pins it).
- **Widget seam.** `RepeatAnnotation { label, suppress_add }` threads into the repeating header (`Option`, `None` = v0.30.0 behavior byte-unchanged); `render_repeating` → `pub(crate)`; `FlagKind` gains `Clone, Copy`, `FlagSchema` gains `Clone` (bespoke per-mode synthesis off the static entries).
- **Coverage.** `tests/archetype_form.rs` (13 cells): per-archetype all-9-params argv cells (only the declared set emits; mode-independent flags keep emitting), kofn happy-path exact argv, `FromRowCount` resolve cell (3→2→0-rows floor + other-flag isolation), kittest mode-switch round-trip via the LIB seam (summary + 4 params render; deselect restores generic; `--key` rows survive), the C1 tiered→kofn switch cell (both surplus rows render + emit, add suppressed, removal restores the scalar arm), mode-independent visibility, threshold clamp-on-render (stored 5 clamps to 2 with 2 key rows), hex-digest hint, pairing unit. Version `v0.30.0` → `v0.31.0` + README self-install pin. Audit trail: `design/SPEC_gui_v0_31_0_archetype_forms.md` + `design/agent-reports/gui-v0_31_0-archetype-forms-r0-r{1,2}-review.md`.

## mnemonic-gui [0.30.0] — 2026-06-09

**SemVer-MINOR — bump the toolkit pin `v0.50.0` → `v0.52.0`, surface the 12 `build-descriptor` preset flags, and ship the generic repeating-row widget for non-secret repeating flags.**

- **`build-descriptor` presets + `--allow` surfaced (the capability addition).** toolkit `v0.51.0`/`v0.52.0` shipped descriptor-builder Release B: 5 curated archetype presets (`--archetype` + 10 parameter flags) and the reviewed sanity-rule opt-out `--allow`. `BUILD_DESCRIPTOR_FLAGS` grows 6 → 18 flags, closing the measured `schema_mirror` drift exactly (`--after --allow --archetype --emit-spec --final-key --hash --key --older --recovery-key --recovery-older --recovery-threshold --threshold`; zero drift on every other subcommand). New value-enum consts `ARCHETYPES` + `ALLOW_RULES`, both pinned **byte-equal to the v0.52.0 binary's possible-values** by `tests/build_descriptor_schema.rs` (`schema_mirror` gates flag NAMES only — Dropdown values are otherwise ungated). `--archetype` carries a GUI-side `""` UNSET sentinel (`ARCHETYPES[0]` + `default_value: Some("")`, displayed as **"(none)"**) so the seeded default form emits no `--archetype` and the archetype↔spec mutex never deadlocks; `--key`/`--recovery-key` are repeating Text (row order = argv order = quorum order); `--allow` is a repeating Dropdown (the `convert --to` precedent) whose added-untouched rows emit **nothing** (accidental emission of an allowance would be a funds-safety opt-out).
- **Generic repeating-row widget (non-secret).** Pre-v0.30.0 the form rendered at most ONE row per flag name, capping every repeating flag (`--md1`, `--cosigner`, `--to`, `--group`, `--add-path`, `--target-address`, and the new `--key`/`--recovery-key`/`--allow`) at a single value from the live form even though `assemble_argv` already emitted N rows. `widget::render_repeating` now renders a header (label + `?` help icon + required marker + "+ add") regardless of row count, one kind-appropriate widget per `state.values` row with a per-row remove button (collect-then-apply), and per-row egui ID salts (`("flag_dropdown", flag.name, row_idx)` — the v0.1.1 popup-state-leak class; the scalar path's salts are byte-unchanged). Seed rule: a REQUIRED repeating flag observing zero rows seeds one per frame via `default_flag_value_for_flag` (convert `--to` keeps seeding `phrase` — the default convert form stays runnable; removing the last required row respawns it, intended); optional repeating flags seed nothing. "+ add" seeds `Text("")` / `Dropdown("")` (never `opts[0]`). The Dropdown render arm maps the `""` option/value to the display-only label `"(none)"` (stored value stays `""`). Coverage: `tests/repeating_rows.rs` (10 cells, state-level + kittest UI-path incl. the popup "(none)" row and the required-row respawn).
- **Archetype↔spec mutex projected (`conditional::build_descriptor`).** `--archetype` non-empty → `--spec` Disabled; `--spec` non-empty → `--archetype` Disabled; re-selecting "(none)" round-trips `--spec` back to enabled (FormState persists per subcommand with no reset affordance). The OTHER preset clap edges (10 `requires = "archetype"` params, `--emit-spec` `conflicts_with_all`) stay deliberately **UN-projected** — recorded decision: the CLI is the gate, and the A2 archetype-forms wizard (v0.31.0) supersedes the generic form as the preset surface. The toolkit `gui-schema` emits `conditional_rules: []` for build-descriptor, so `gui_schema_conditional_drift` is unaffected (no `SUBCOMMAND_FLOORS` entry). 4 new `conditional_visibility` cells; `build-descriptor` joins the coverage-guard must-have list.
- **Repeating-secret bug FILED, not fixed.** `FOLLOWUPS.md::repeating-secret-flags-never-reach-argv`: secret+repeating+Text flags (`--ms1` ×2, `--share` ×2 — the seed-xor `--share` is NodeValueComposite and unaffected) render into the single `secret_widgets` entry while `assemble_argv` reads repeating secrets from `state.values` → live forms emit NOTHING for them (masked by cells that synthesize `state.values`). Pre-existing; the new repeating widget deliberately keeps the secret branch first/unchanged.
- **Toolkit pin bump `v0.50.0` → `v0.52.0`** (`Cargo.toml` + `Cargo.lock` rev = tag commit `2cad4b7` + `pinned-upstream.toml` `[mnemonic].tag` + `README.md` toolkit install pin + the `pinned_version` action-bar banner + the schema module-doc header — all in lockstep, `pin_coherence` + `readme_pin_coherence` gated). md/ms/mk pins unchanged. Version `v0.29.0` → `v0.30.0` + README self-install pin. Resolves `gui-build-descriptor-presets-pending-pin-bump` (both repos; the entry's "repeating Dropdown is a combination this schema has not used before" claim was wrong — `convert --to` is the live precedent). Audit trail: `design/SPEC_gui_v0_30_0_presets_pin_bump.md` + `design/agent-reports/gui-v0_30_0-presets-pin-bump-r0-r{1,2,3}-review.md`.

## mnemonic-gui [0.29.0] — 2026-06-09

**SemVer-MINOR — bump the toolkit pin `v0.47.3` → `v0.50.0` and surface the new `build-descriptor` subcommand in the schema mirror.**

- **`build-descriptor` surfaced (the capability addition).** toolkit `v0.50.0` shipped the descriptor-builder engine Release A (`mnemonic build-descriptor`: a JSON policy-tree spec → `wsh` descriptor + BIP-388 wallet policy + compare-cost preview + node-addressed diagnostics, with `--spec-schema`). Added `BUILD_FORMATS` + `BUILD_DESCRIPTOR_FLAGS` + the `SubcommandSchema` to `src/schema/mnemonic.rs` (flag-NAME set `{--format, --json, --network, --no-auto-repair, --spec, --spec-schema}`, matching `gui-schema`). `--spec` is modeled `FlagKind::Path { stdio_sentinel: true }` (the `--blob` precedent) — the toolkit `--spec` is a file path / `-` / stdin, **never inline JSON**, so the form must emit a valid path (a `Text` widget would emit raw JSON → ENOENT). The recursive node-tree wizard FORM is a later cycle; this entry yields a basic file-picker `--spec` form (render is sound by construction — Path/Boolean/Dropdown all render on existing forms). Characterization: `tests/build_descriptor_schema.rs` (presence + exact flag set + `--spec` is Path + argv no-panic).
- **Toolkit pin bump `v0.47.3` → `v0.50.0`** (`Cargo.toml` + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag` + `README.md` toolkit install pin + the `pinned_version` action-bar banner + the schema module-doc header — all in lockstep, `pin_coherence` + `readme_pin_coherence` gated). The toolkit releases `v0.47.4`–`v0.50.0` added **no flag-NAME / dropdown value-enum / conditional-rule / secret-projection drift on existing subcommands** (measured: `schema_mirror` GREEN against the v0.50.0 binary *before* the build-descriptor add → zero accumulated drift), so the only schema-content change is the new `build-descriptor` entry. md/ms/mk pins unchanged.
- **`schema_mirror` now validates `build-descriptor`** (6 flags vs the v0.50.0 `gui-schema`). Version `v0.28.0` → `v0.29.0` + README self-install pin. Resolves the toolkit FOLLOWUP `gui-build-descriptor-schema-mirror-pending-pin-bump`. Latent debt tracked in `FOLLOWUPS.md` (`manual-gui-build-descriptor-anchors-pending-pin-bump`): the next `docs/manual-gui` pin bump must add the `mnemonic-build-descriptor[-…]` anchors. Audit trail: `design/SPEC_gui_v0_29_0_build_descriptor.md` + `design/agent-reports/gui-v0_29_0-build-descriptor-r0-round1-review.md`.

## mnemonic-gui [0.28.0] — 2026-06-06

**SemVer-MINOR — bump the toolkit pin `v0.46.2` → `v0.47.3` and fix the `export-wallet --timestamp` default-value drift (resolves `gui-timestamp-default-value-drift-v0.47.3`).**

- **Timestamp default-value fix (the pin-necessitated change).** toolkit `v0.47.3` flipped `export-wallet --timestamp`'s default from `now` → `0` (rescan from genesis). The GUI's hand-maintained schema still declared `default_value: Some("now")`, so the D33 default-suppression (`is_at_default`) would treat an **explicit** `Now` selection as at-default and **drop `--timestamp` from the generated argv** — the toolkit would then apply its new `0` default, silently discarding the user's explicit `now`. Fixed: `src/schema/mnemonic.rs` `--timestamp` `default_value` `"now"` → `"0"` (+ help string). No `is_at_default` change needed — the timestamp widget seeds **Unset** in the default form (`widget.rs`), so the default form emits no `--timestamp` and the toolkit applies `0`; an explicit `Now` now correctly emits `--timestamp now`. Regression guard: `argv_assembler` tests inverted (`d33_timestamp_now_is_emitted_when_default_is_zero`, `cell_3b`).
- **Toolkit pin bump `v0.46.2` → `v0.47.3`** (`Cargo.toml` + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag` + `README.md` toolkit install pin + the `pinned_version` action-bar banner + the schema module-doc header — all in lockstep, `pin_coherence` + `readme_pin_coherence` gated). The toolkit releases `v0.46.3`–`v0.47.3` added **no** flag-NAME / dropdown value-enum / conditional-rule / secret-projection change (measured: `schema_mirror` + `gui_schema_conditional_drift` + `schema_mirror_secret_drift` + `xpub_search_schema_mirror` GREEN against the v0.47.3 binary), so the only schema change is the `default_value` (which `schema_mirror` does not gate). md/ms/mk pins unchanged (current).
- **No GUI `schema_mirror` impact** — `default_value` is not a flag-NAME or value-enum. Version `v0.27.0` → `v0.28.0` + README self-install pin. Audit trail: `design/SPEC_gui_v0_28_0_pin_bump_v0_47_3.md` + `design/agent-reports/gui-v0_28_0-pin-bump-r0-round{1,2}-review.md`.

## mnemonic-gui [0.27.0] — 2026-06-06

**SemVer-MINOR — consume `mnemonic-toolkit-v0.46.2`'s restore `conditional_rules` projection + add a README install-pin coherence guard.** Two follow-ons to the v0.25.0 cycle (both resolve toolkit FOLLOWUPs spawned there).

- **Drift-gate restore's `--from required-unless-md1` rule (slug 1).** Toolkit v0.46.2 now **projects** restore's `--from required_unless_present="md1"` as a `conditional_rule` (`not(flag_present "--md1") → {--from, required}`). The GUI's hand-authored `conditional::restore` fn already emitted exactly that shape, but while pinned to toolkit v0.46.0 (which emitted `conditional_rules: []` for restore) `gui_schema_conditional_drift` **skipped** it — the rule was GUI-authored/ungated. Bumping the pin to v0.46.2 makes the drift gate **exercise** restore (it passes — the fn matches the projection); `("restore", 1)` is added to `SUBCOMMAND_FLOORS` (total floor 34 → 35) so the now-gated rule can't silently vanish. The three now-stale "restore has no arm / emits `[]` / not drift-gated" comments (`src/form/conditional.rs`, `src/schema/mnemonic.rs`, `tests/conditional_visibility.rs`) are rewritten. **No GUI-logic change** — `conditional::restore` already matched. Resolves toolkit FOLLOWUP `gui-schema-restore-required-unless-md1-projection`.
- **README install-pin coherence guard (slug 2).** New pure-logic `tests/readme_pin_coherence.rs` asserts each README `cargo install … --tag <X>` line matches its source of truth — the GUI self-tag against `Cargo.toml` `version`, and the four sibling tags (`mnemonic-toolkit`/`md-cli`/`ms-cli`/`mk-cli`) against `pinned-upstream.toml`. The README block claims "pinned tags match `pinned-upstream.toml`" but nothing enforced it; the pins had drifted 3 versions before v0.25.0 backfilled them. Mirrors `pin_coherence.rs` + the toolkit's `readme_version_current.rs` (proven non-vacuous: RED on a deliberate desync). Resolves toolkit FOLLOWUP `gui-readme-install-pin-coherence-guard`.
- **Pin/version.** Toolkit pin `mnemonic-toolkit-v0.46.0 → v0.46.2` (`Cargo.toml` git-dep + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag`, `pin_coherence` lockstep). **No flag delta v0.46.0→v0.46.2** (v0.46.1 was a pure refactor; v0.46.2 added only the `conditional_rules` projection) → `schema_mirror` is a clean catch-up, no flag/secret/dropdown change. `pinned_version` banner + module-doc → `mnemonic 0.46.2` / `v0.46.2`; README install pins refreshed (GUI self-tag `v0.26.0 → v0.27.0`, toolkit `v0.46.0 → v0.46.2`). Sibling pins (md 0.6.2 / ms 0.7.0 / mk 0.7.0) unchanged. `gui_schema_conditional_drift` + `readme_pin_coherence` + `pin_coherence` + `schema_mirror` + full suite green (4 pinned binaries); clippy clean. The toolkit-side `manual-gui` (pinned `mnemonic-gui-v0.3.0`) is a separate deferred track, untouched.

## mnemonic-gui [0.26.0] — 2026-06-05

**SemVer-MINOR — `schema_mirror` lockstep for `mnemonic-toolkit-v0.46.0`'s `xpub-search passphrase-of-xpub --passphrase-candidates-file`.** Toolkit v0.46.0 added `--passphrase-candidates-file` (candidate-list passphrase scan) to `xpub-search passphrase-of-xpub`. Mirrors the one new flag in `src/schema/mnemonic.rs` `XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS`: a `FlagKind::Path { stdio_sentinel: false }`, **`secret: false`** entry (a PATH, not the secret value — mirrors `--decrypt-password-file`; the file itself holds secret candidates), auto-rendered by the schema-driven path widget. The flag trips the `schema_mirror` / `xpub_search_schema_mirror` flag-NAME-parity gates → MINOR. **No secret-projection delta** (`secret:false` → `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` unchanged; `schema_mirror_secret_drift` green) and **no conditional change** (the toolkit `gui-schema` emits `conditional_rules: []` for `passphrase-of-xpub` — the 3-way passphrase `ArgGroup` is not projected; `gui_schema_conditional_drift` unaffected). Toolkit pin `mnemonic-toolkit-v0.44.0 → v0.46.0` (`Cargo.toml` git-dep + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag` in lockstep — `pin_coherence` guard); `pinned_version` banner + module-doc → `mnemonic 0.46.0` / `v0.46.0`; `README.md` install-command pins refreshed (GUI self-tag `v0.25.0 → v0.26.0`, toolkit `v0.44.0 → v0.46.0`). Sibling pins (md 0.6.2 / ms 0.7.0 / mk 0.7.0) unchanged. `schema_mirror` + `xpub_search_schema_mirror` + `pin_coherence` + full suite green (4 pinned binaries); clippy clean. Resolves `mnemonic-toolkit` FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`. The toolkit-side `manual-gui` (pinned `mnemonic-gui-v0.3.0`) is a separate deferred track, untouched.

## mnemonic-gui [0.25.0] — 2026-06-05

**SemVer-MINOR — `schema_mirror` lockstep for `mnemonic-toolkit-v0.44.0`'s `mnemonic restore` multisig-cosigner flags.** Toolkit v0.44.0 added two flags to `mnemonic restore` (multisig-cosigner restore: reconstruct a watch-only multisig descriptor from the shared wallet-policy `md1` card) and made `--from` conditionally-required. Mirrors them in `src/schema/mnemonic.rs` `RESTORE_FLAGS`: `--md1` (text, **repeating**, non-secret — the shared wallet-policy md1 card chunk(s)) and `--cosigner` (text, **repeating**, non-secret — `@N=<mk1|xpub>` per-position cross-check assertion); both are watch-only so the only secret-bearing restore flags remain `--passphrase`/`--passphrase-stdin` → **no secret-projection delta** (`SECRET_NODE_TYPES` / `SECRET_SLOT_SUBKEYS` unchanged; `schema_mirror_secret_drift` green). `--from` flips `required: true → false` (toolkit `RestoreArgs.from` is now `Option<String>` with `required_unless_present = "md1"`; the v0.44.0 `gui-schema` emits `--from required:false`). The two new flags trip the `schema_mirror` flag-NAME-parity gate → MINOR. **Conditional-requiredness modeled (`conditional::restore`):** `--from` is marked `Required` while `--md1` is empty, and falls through to optional (own-cosigner cross-check) once `--md1` is supplied — a GUI-authored at-least-one rule (the toolkit `gui-schema` `conditional_rules` projection is a hand-encoded allowlist with no restore arm, so restore emits `conditional_rules: []`; the rule is not drift-gated, same posture as the GUI-authored `repair`/`inspect` at-least-one rules). `restore` `SubcommandSchema.conditional` flips `None → Some(restore)` and joins the `conditional_visibility` coverage-guard allowlist; 2 new `conditional_visibility` cells. Toolkit pin `mnemonic-toolkit-v0.43.0 → v0.44.0` (`Cargo.toml` git-dep + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag` in lockstep — `pin_coherence` guard); the `pinned_version` banner + `src/schema/mnemonic.rs` module-doc bump to `mnemonic 0.44.0` / `v0.44.0`. Sibling pins (md 0.6.2 / ms 0.7.0 / mk 0.7.0) unchanged. `schema_mirror` + `conditional_visibility` + `gui_schema_conditional_drift` + `pin_coherence` + full suite green (4 pinned binaries); clippy clean. Resolves `mnemonic-toolkit` FOLLOWUP `gui-restore-multisig-flags-pending-pin-bump`. Also backfills the two stale `README.md` install-command pins (GUI self-tag `v0.22.0 → v0.25.0`, toolkit `v0.41.0 → v0.44.0`) that had drifted silently since v0.23.0 — the README is ungated (no version-marker guard), so a `gui-readme-install-pin-coherence-guard` FOLLOWUP is filed (the md/ms/mk README pins were already current). The toolkit-side `manual-gui` (pinned `mnemonic-gui-v0.3.0`) is a separate deferred track, untouched.

## mnemonic-gui [0.24.0] — 2026-06-04

**SemVer-MINOR — `schema_mirror` lockstep for `mnemonic-toolkit-v0.43.0`'s new `mnemonic restore` subcommand.** Adds the `restore` `SubcommandSchema` (`RESTORE_FLAGS`) to `src/schema/mnemonic.rs` — 15 flags (14 explicit + the shared global `--no-auto-repair`) mirroring the toolkit v5 `gui-schema` for `restore`: `--from` (text, **required**), `--format` (dropdown `EXPORT_FORMATS`, the 11 export formats), `--template` (dropdown `TEMPLATES`), `--network` (dropdown `NETWORKS`), `--language` (dropdown `LANGUAGES`), `--account` (number, default 0), `--count` (number, default 1), `--expect-fingerprint` (text), `--expect-xpub` (text), `--allow-mismatch` (bool), `--passphrase` (text, **secret**), `--passphrase-stdin` (bool, **secret**), `--output` (path, default `-`), `--json` (bool), and the shared global `--no-auto-repair` (listed via `NO_AUTO_REPAIR_FLAG`, matching every other subcommand). All four reused dropdown value-enum consts (`EXPORT_FORMATS` / `TEMPLATES` / `NETWORKS` / `LANGUAGES`) already exist — no new const introduced. `restore` takes its input via `--from` not slots (`allows_slots: false`) and gui-schema emits `conditional_rules: []` (`conditional: None`). The net-new subcommand trips the `schema_mirror` flag-NAME-parity gate → MINOR (decode-address / verify-message / nostr / silent-payment new-subcommand precedent). The only secret-bearing flags are `--passphrase` / `--passphrase-stdin` — an already-classified secret flag-name pair, so **no secret-projection delta** beyond what `schema_mirror_secret_drift` already enforces (`SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` unchanged). Toolkit pin `mnemonic-toolkit-v0.42.0 → v0.43.0` (`Cargo.toml` git-dep + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag`, in lockstep — `pin_coherence` guard). `pinned_version` banner + the `src/schema/mnemonic.rs` module-doc header bumped to `mnemonic 0.43.0` / `v0.43.0`. `schema_mirror` + `schema_mirror_secret_drift` + `pin_coherence` green; full suite green; clippy clean.

## mnemonic-gui [0.23.0] — 2026-06-03

**SemVer-MINOR — `schema_mirror` lockstep for `mnemonic-toolkit-v0.42.0`'s new `export-wallet --format descriptor` value.** Adds `"descriptor"` to the `EXPORT_FORMATS` dropdown value-enum in `src/schema/mnemonic.rs` (the `export-wallet --format` picker), mirroring the toolkit's new `CliExportFormat::Descriptor` — a bare canonical multipath `<descriptor>#<BIP-380-checksum>` output (no JSON / wallet-file wrapper), single-sig + multisig, all input paths. New dropdown VALUE on an existing flag → `schema_mirror` value-enum lockstep (MINOR, per the precedent that user-facing dropdown-value additions paired with a toolkit feature ship MINOR). Toolkit pin `mnemonic-toolkit-v0.41.0 → v0.42.0` (`Cargo.toml` git-dep + `Cargo.lock` + `pinned-upstream.toml` `[mnemonic].tag`, in lockstep — `pin_coherence` guard). `pinned_version` banner + the `src/schema/mnemonic.rs` module-doc header bumped to `mnemonic 0.42.0` / `v0.42.0`. No flag NAME / subcommand / secret-projection delta (`SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` unchanged); the `--format descriptor` output is watch-only public material. `schema_mirror` + `schema_mirror_secret_drift` + `pin_coherence` green; full suite green; clippy clean.

## mnemonic-gui [0.22.0] — 2026-06-03

**SemVer-MINOR — catch-up pin bump across all four CLIs + the prepared ms1 slot-editor picker + the `md repair` schema entry.** Lands the slot-editor `Ms1` picker and the `SECRET_SLOT_SUBKEYS` snapshot `+= "ms1"` (in `src/secrets.rs` / `src/form/slot_editor.rs`), now that the toolkit lib pin reaches `mnemonic-toolkit-v0.41.0` — the picker had been authored ahead of time but was const-assert-blocked (the `v0_3_canonical_fallback` drift guard rejects a snapshot that the pinned toolkit's `secret_taxonomy::SECRET_SLOT_SUBKEYS` doesn't yet contain) until the pin caught up. Carries a **catch-up pin bump across all four CLIs** (`Cargo.toml` toolkit git-dep + `Cargo.lock` + all four `pinned-upstream.toml` tags): mnemonic `v0.38.0 → v0.41.0`, ms `v0.5.0 → v0.7.0`, mk `v0.6.0 → v0.7.0`, md `v0.6.1 → v0.6.2`. This **RESTORES `schema_mirror` to green** — the pins had lagged the hand-maintained schemas since v0.21.3, so against the current upstream binaries CI was effectively red; the pin bump is therefore a **bug-fix**, not just routine maintenance. Adds the `md repair` `SubcommandSchema` to `src/schema/md.rs` (`md repair [--json] <MD1_STRINGS>...`, BCH error-correction) — the sole remaining schema/binary subcommand-coverage gap on the `md` CLI. Introduces a new `tests/pin_coherence.rs` guard that asserts the `Cargo.toml` toolkit pin equals `pinned-upstream.toml`'s `[mnemonic].tag` (pure-logic, no binary, no network), promoting the prose lockstep invariant to a leading gate. This names and closes the bug class **"schema edits landed ahead of the pins, masked by local-binary `schema_mirror` runs"** — the live `schema_mirror` / `schema_mirror_secret_drift` gates exec a `*_BIN` binary (skipping when absent) and have no knowledge of the declared pins, so a schema-ahead-of-pin drift passes locally against a stale-but-present binary; this class has fired **twice** (the K-of-N v0.40.0 work and the `gui-ms1-slot-subkey-pending-pin-bump` FOLLOWUP). `SECRET_NODE_TYPES` is **unchanged** v0.37.3 → v0.41.0, so only the `SECRET_SLOT_SUBKEYS` snapshot moved (no node-type secret-projection delta). `pinned_version` banners bumped to `mnemonic 0.41.0` / `md 0.6.2` / `mk 0.7.0` (ms already `0.7.0`); README manual-install `--tag` block re-synced to the current pin set so its "match `pinned-upstream.toml`" claim holds again. New-subcommand-in-a-schema → MINOR (per the v0.20.0 silent-payment / v0.21.0 decode-address+verify-message precedent). `schema_mirror` + `schema_mirror_secret_drift` + `pin_coherence` green; full suite green; clippy clean.

## mnemonic-gui [0.21.3] — 2026-05-24

**SemVer-PATCH — schema-mirror lockstep for `mnemonic-toolkit-v0.37.3`'s new `repair --max-subst` flag.** Adds `--max-subst` (Number 0..=4, default 0, non-secret) to `REPAIR_FLAGS` in `src/schema/mnemonic.rs`. The flag lets indel recovery also tolerate up to E substitution (wrong-but-in-place) errors alongside the length-changing indels; a substitution-bearing recovery is surfaced as a VERIFY-ME candidate (toolkit exit 4), not a confident correction. Net-new flag NAME on an existing subcommand → `schema_mirror` lockstep (PATCH, per the v0.21.2 `--max-indel` precedent). Also de-stales the mirrored `--max-indel` help/comment from "ms1/mk1 only" to "ms1/mk1/md1" (toolkit v0.37.2 added md1 indel; help text is not gated, this is a correctness fix). Toolkit pin `mnemonic-toolkit-v0.37.1 → v0.37.3` (`pinned-upstream.toml` + `Cargo.toml` git-dep + `Cargo.lock`, in lockstep). `pinned_version` banner bumped to `mnemonic 0.37.3`. `schema_mirror` + `schema_mirror_secret_drift` green; full suite green; clippy clean. No cumulative drift — the v0.37.1 → v0.37.3 pin gap was a single flag addition (`--max-subst`); v0.37.2's md1 indel added no flag.

## mnemonic-gui [0.21.2] — 2026-05-24

**SemVer-PATCH — schema-mirror lockstep for `mnemonic-toolkit-v0.37.1`'s new `repair --max-indel` flag.** Adds `--max-indel` (Number 0..=4, default 0, non-secret) to `REPAIR_FLAGS` in `src/schema/mnemonic.rs`. The flag enables indel (insert/delete) recovery for transcription errors that changed the length of an ms1/mk1 chunk. Net-new flag NAME on an existing subcommand → `schema_mirror` lockstep (PATCH, per the v0.19.2 `import-wallet --network` precedent). Toolkit pin `mnemonic-toolkit-v0.36.1 → v0.37.1` (`pinned-upstream.toml` + `Cargo.toml` git-dep + `Cargo.lock`, in lockstep). `pinned_version` banner bumped to `mnemonic 0.37.1`. `schema_mirror` + `schema_mirror_secret_drift` green; full suite green; clippy clean. No cumulative drift — the v0.36.1 → v0.37.1 pin gap was a single flag addition.

## mnemonic-gui [0.21.1] — 2026-05-24

**SemVer-PATCH — schema-mirror lockstep for `mnemonic-toolkit-v0.36.1`'s three new `silent-payment` flags.** Adds to `SILENT_PAYMENT_FLAGS` in `src/schema/mnemonic.rs`: `--change-address` (Boolean, non-secret), `--passphrase` (Text, **secret**), `--passphrase-stdin` (Boolean, **secret**). The two passphrase flags inherit secret-projection from the toolkit's `flag_is_secret` (mask/zeroize/paste-warn). Net-new flag NAMES on an existing subcommand → `schema_mirror` lockstep (PATCH, per the v0.19.2 `import-wallet --network` precedent). Toolkit pin `mnemonic-toolkit-v0.36.0 → v0.36.1` (`pinned-upstream.toml` + `Cargo.toml` git-dep + `Cargo.lock`, in lockstep).

## mnemonic-gui [0.21.0] — 2026-05-23

**SemVer-MINOR — schema-mirror lockstep for `mnemonic-toolkit-v0.36.0`'s two new subcommands `decode-address` + `verify-message`.** Adds both `SubcommandSchema`s to `src/schema/mnemonic.rs`:

- `decode-address`: positional `<address>` + `--json` + global `--no-auto-repair` (the first GUI schema entry with a non-empty `positional_args`).
- `verify-message`: `--address` (text, required), `--format` (dropdown `auto`/`legacy`/`bip322`, default `auto`), `--message`, `--message-file` (path), `--message-stdin`, `--signature` (text, required), `--json`, global `--no-auto-repair`.

Both are public-data operations — **no secret flags**, so no secret-projection delta (`flag_is_secret` unchanged). Net-new subcommands trip the `schema_mirror` flag-NAME gate → MINOR (nostr/silent-payment precedent). Toolkit pin `mnemonic-toolkit-v0.35.0 → v0.36.0` (`pinned-upstream.toml` + `Cargo.toml` git-dep + `Cargo.lock`, in lockstep).

## mnemonic-gui [0.20.0] — 2026-05-23

**SemVer-MINOR — schema-mirror lockstep for `mnemonic-toolkit-v0.35.0`'s new `mnemonic silent-payment` subcommand (BIP-352 receiver address).** Adds the `silent-payment` `SubcommandSchema` to `src/schema/mnemonic.rs`: `--account` (number, hardened ≤ 2³¹−1, default 0), `--json` (bool), `--label` (repeating number, m≥1; m=0 refused), `--network` (dropdown `NETWORKS`, default mainnet), `--no-auto-repair` (global), `--secret` (text, **secret**), `--secret-file` (path), `--secret-stdin` (bool, **secret**). The net-new subcommand trips the `schema_mirror` flag-NAME-parity gate, hence the MINOR bump (matching the nostr/seedqr new-subcommand precedent). Secret projection (`--secret`/`--secret-stdin` masked + zeroized; `--secret-file` plain) inherits from the toolkit's `flag_is_secret`. Toolkit pin `mnemonic-toolkit-v0.34.7 → v0.35.0` (`pinned-upstream.toml` + `Cargo.toml` git-dep, in lockstep).

## mnemonic-gui [0.19.3] — 2026-05-23

**SemVer-PATCH — upstream pin bumps for the m-format argv-hardening rollout.** Bumps the pinned upstreams for the cross-repo `PR_SET_DUMPABLE` argv-hardening cycle: `mnemonic-toolkit-v0.34.6 → v0.34.7`, `descriptor-mnemonic-md-cli-v0.6.0 → v0.6.1`, `ms-cli-v0.4.0 → v0.4.1`, `mk-cli-v0.4.0 → v0.4.2` (the mk pin was a version behind — catch-up to v0.4.2). No schema change: the argv-hardening is process-internal (`prctl`), with no CLI flag/subcommand surface, so the `schema_mirror` gate is unaffected. `pinned-upstream.toml` + `Cargo.toml` git-dep, in lockstep.

## mnemonic-gui [0.19.2] — 2026-05-22

**SemVer-PATCH release — schema-mirror lockstep** for `mnemonic-toolkit-v0.34.6`'s new `import-wallet --network` flag. A net-new clap flag NAME trips the `schema_mirror` flag-NAME-parity gate.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.34.2 → mnemonic-toolkit-v0.34.6` (`pinned-upstream.toml` + `Cargo.toml` git-dep, in lockstep). v0.34.3/v0.34.4/v0.34.5 were toolkit-only (no CLI surface change → no GUI lockstep), so the only schema-mirror delta is `--network` on `import-wallet`.

### Added

- `IMPORT_WALLET_FLAGS`: `--network` (Dropdown `NETWORKS` = mainnet/testnet/signet/regtest) — the toolkit's signet/regtest disambiguation override (re-binds the imported network within the parsed coin-type class). Non-secret.

### Test totals

- All tests passing. Clippy clean. `schema_mirror` (flag-name parity, now incl. `import-wallet --network`) + `schema_mirror_secret_drift` green against the pinned v0.34.6 binary.

### Cycle topology

Cycle 25 — GUI lockstep for toolkit v0.34.6 (`import-wallet --network` signet/regtest override).

---

## mnemonic-gui [0.19.1] — 2026-05-22

**SemVer-PATCH release — schema-mirror lockstep** for `mnemonic-toolkit-v0.34.2`'s two new `mnemonic nostr` flags (`--import`, `--timestamp`). Net-new flag NAMEs trip the `schema_mirror` flag-NAME-parity gate.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.34.0 → mnemonic-toolkit-v0.34.2` (`pinned-upstream.toml` + `Cargo.toml` git-dep tag, in lockstep). v0.34.1 was internal-only (no CLI surface change → no GUI lockstep).

### Added

- `NOSTR_FLAGS`: `--import` (Text — value-valued `readonly`; emits a read-only Bitcoin Core `importdescriptors` recipe; `spending`/`both` are reserved/refused) and `--timestamp` (Text — `importdescriptors` rescan anchor `now`|unix, default `0`). Both non-secret, plain TEXT (toolkit value_parsers, NOT dropdowns).

### Secret-handling

- Neither new flag is secret-bearing: `--import` is a mode selector, `--timestamp` is a rescan anchor. `schema_mirror_secret_drift` confirms parity (toolkit's `flag_is_secret` does not classify either).

### Test totals

- All tests passing. Clippy clean. `schema_mirror` (flag-name parity, now including `nostr --import`/`--timestamp`) + `schema_mirror_secret_drift` both green against the pinned v0.34.2 binary.

### Cycle topology

Cycle 21 — GUI lockstep for toolkit v0.34.2 (`mnemonic nostr --import` read-only importdescriptors).

---

## mnemonic-gui [0.19.0] — 2026-05-22

**SemVer-MINOR release — schema-mirror lockstep** for `mnemonic-toolkit-v0.34.0`'s new `mnemonic nostr` subcommand (Nostr key derivation from BIP-39 seed / secret). A net-new subcommand trips the `schema_mirror` flag-NAME-parity gate. Closes `gui-nostr-schema-mirror`.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.33.2 → mnemonic-toolkit-v0.34.0`.

### Added

- `NOSTR_FLAGS` SubcommandSchema + `nostr` registration. Flags: `--all-script-types` (Boolean), `--json` (Boolean), `--network` (Dropdown mainnet/testnet/signet/regtest, default mainnet), `--no-auto-repair` (global), `--pubkey` (Text), `--script-type` (Text — custom value_parser, NOT a dropdown), `--secret` (Text, **secret**), `--secret-file` (Path), `--secret-stdin` (Boolean, **secret**).

### Secret-handling

- `--secret` + `--secret-stdin` carry `secret: true` (GUI masking + paste-warn/run-confirm + exit-time zeroize). This mirrors the toolkit's `flag_is_secret` classification for `nostr --secret` and `--secret-stdin` (Cycle 20 C1). `--secret-file` is non-secret (its value is a filesystem path); `--pubkey` is non-secret (public key).

### Test totals

- All tests passing; 1 ignored. Clippy clean. `schema_mirror` (flag-name parity, now including `nostr`) + `schema_mirror_secret_drift` (the two new secret flags) both green against the pinned v0.34.0 binary.

### Cycle topology

Cycle 20 C5 — GUI lockstep for toolkit v0.34.0 (`mnemonic nostr` subcommand).

---

## mnemonic-gui [0.18.1] — 2026-05-21

**SemVer-PATCH — schema-mirror lockstep** for `mnemonic-toolkit-v0.33.2`'s new `import-wallet --decrypt-password*` flags (Electrum BIE1 storage-encrypted wallet import, Cycle 19 Phase B). Net-new flag NAMEs on an existing subcommand trip the `schema_mirror` flag-NAME-parity gate. Closes `gui-import-wallet-decrypt-password-mirror`.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.33.1 → mnemonic-toolkit-v0.33.2`.

### Added

- Three flags on the `import-wallet` SubcommandSchema: `--decrypt-password` (Text, **secret**), `--decrypt-password-file` (Path), `--decrypt-password-stdin` (Boolean, **secret**).

### Secret-handling

- `--decrypt-password` + `--decrypt-password-stdin` carry `secret: true` (GUI masking + paste-warn/run-confirm + exit-time zeroize). This mirrors the toolkit's `flag_is_secret`, which has classified these flag names as secret since v0.33.1 — so the `schema_mirror_secret_drift` gate (the v0.3.0–v0.3.2 BIP-39 persistence-leak class) stays green for the new `import-wallet` projection. `--decrypt-password-file` is non-secret (its value is a filesystem path).

### Test totals

- 353 cells passing; 1 ignored. Clippy clean. `schema_mirror` (flag-name parity, now incl. the three import-wallet flags) + `schema_mirror_secret_drift` both green against the pinned v0.33.2 binary.

### Cycle topology

Cycle 19b — GUI lockstep for toolkit v0.33.2 (Electrum BIE1 storage import; completes the Electrum-encryption arc).

---

## mnemonic-gui [0.18.0] — 2026-05-21

**SemVer-MINOR release.** MANDATORY schema-mirror lockstep for `mnemonic-toolkit-v0.33.0`'s NEW `electrum-decrypt` subcommand (decrypt an Electrum field-encrypted secret → plaintext). A new subcommand is a hard `schema_mirror` trip. Closes `gui-electrum-decrypt-subcommand-mirror` FOLLOWUP.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.32.0 → mnemonic-toolkit-v0.33.1`. (The v0.32.1/.2/.3 patches between were test-only / behavior-expansion / `repeating`-cardinality — no flag-NAME or subcommand change — so they required no GUI bump; the schema_mirror gate confirmed zero accumulated flag-name drift across the jump beyond `electrum-decrypt`.)

### Added

- `ELECTRUM_DECRYPT_FLAGS` SubcommandSchema + `electrum-decrypt` registration. Flags: `--ciphertext` (Text, required), `--decrypt-password` (Text, **secret**), `--decrypt-password-file` (Path), `--decrypt-password-stdin` (Boolean, **secret**), `--json-out` (Path), `--no-auto-repair` (global).

### Secret-handling

- `--decrypt-password` + `--decrypt-password-stdin` carry `secret: true` so the GUI masks the password field, fires the paste-warn / run-confirm modals, and zeroize-sweeps the value at exit. This mirrors `mnemonic-toolkit-v0.33.1`'s `flag_is_secret` fix (the v0.33.0 emission omitted them — caught by this lockstep's `schema_mirror_secret_drift` gate, which is exactly the v0.3.0–v0.3.2 BIP-39 persistence-leak class). `--decrypt-password-file` is non-secret (its value is a filesystem path); `--ciphertext` is non-secret (encrypted material, not plaintext).

### Test totals

- 353 cells passing; 1 ignored. Clippy clean. `schema_mirror` (flag-name parity, now including `electrum-decrypt`) + `schema_mirror_secret_drift` (the two new secret password flags) both green against the pinned v0.33.1 binary.

### Cycle topology

Cycle 18b — GUI lockstep for Cycle 18 / toolkit v0.33.0+v0.33.1 (first of the final v0.32+ Electrum pair).

---

## mnemonic-gui [0.17.0] — 2026-05-21

**SemVer-MINOR release.** MANDATORY schema-mirror lockstep for `mnemonic-toolkit-v0.32.0` (CompactSeedQR variant). Closes `gui-seedqr-variant-flag-mirror` FOLLOWUP. `--variant` is a NET-NEW flag NAME on BOTH `mnemonic seedqr encode` AND `mnemonic seedqr decode` — trips the flag-NAME-parity gate on both subcommand schemas.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.31.6 → mnemonic-toolkit-v0.32.0`.

### Added

- `--variant` Dropdown (`["standard", "compact"]`, default `standard`) added to both `SEEDQR_ENCODE_FLAGS` and `SEEDQR_DECODE_FLAGS`. New `SEEDQR_VARIANTS` const.

### Note

No `SECRET_NODE_TYPES` change this cycle (CompactSeedQR added no new NodeType — it reuses the existing `seedqr` slot/node surfaces), so the supply-chain drift gate stayed quiet.

### Test totals

- 353 cells passing; 1 ignored. Clippy clean. schema_mirror green with the new `--variant` on both seedqr subcommands.

### Cycle topology

Cycle 14b — GUI lockstep for Cycle 14 / toolkit v0.32.0 (CompactSeedQR; close of the SeedQR-completion arc).

---

## mnemonic-gui [0.16.2] — 2026-05-21

**SemVer-PATCH release.** MANDATORY schema-mirror lockstep for `mnemonic-toolkit-v0.31.6` (SeedQR `--from` unification). Closes `gui-seedqr-decode-from-flag-mirror` FOLLOWUP. Unlike Cycles 10/12 (value-content additions the schema_mirror gate ignores), v0.31.6 adds a NET-NEW flag NAME (`--from`) to `mnemonic seedqr decode` — this trips the flag-NAME-parity gate.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.31.3 → mnemonic-toolkit-v0.31.6` (cumulative catch-up across v0.31.4 sparrow-regex-hardening + v0.31.5 SeedQR 15/18/21 word-counts + v0.31.6 SeedQR `--from` unification; only v0.31.6 has a GUI-surface change).

### Changed

- `src/schema/mnemonic.rs::SEEDQR_DECODE_FLAGS`: added `--from` flag (`NodeValueComposite(["seedqr"])`, canonical input) + made `--digits` `required: false` (deprecated alias). New `SEEDQR_DECODE_FROM_NODES` const.
- `src/secrets.rs::v0_3_canonical_fallback::SECRET_NODE_TYPES`: snapshot updated to include `"seedqr"` (the toolkit v0.31.6 added it to `SECRET_NODE_TYPES`; the compile-time supply-chain drift gate fired on the pin bump and was acknowledged via snapshot update).
- `tests/secrets.rs::secret_node_types_set_pinned`: expected-set updated to include `"seedqr"`.

### Known gap (deferred)

The `mnemonic convert` GUI form's `--from`/`--to` dropdowns share a single `NODE_TYPES` const; `seedqr` is input-only (valid for `--from`, rejected for `--to`). Adding it to the shared const would wrongly offer `--to seedqr`. Splitting into FROM/TO node lists is deferred (the `convert --from seedqr=` path is reachable via the toolkit CLI directly; the schema_mirror gate is flag-NAME-parity and is NOT affected). No FOLLOWUP filed — revisit if GUI convert-form seedqr-input demand surfaces.

### Test totals

- 353 cells passing; 1 ignored. Clippy clean.

### Cycle topology

Cycle 13b — GUI lockstep for Cycle 13 / toolkit v0.31.6.

---

## mnemonic-gui [0.16.1] — 2026-05-21

**SemVer-PATCH release.** Optional follow-on companion to `mnemonic-toolkit-v0.31.3` (SeedQR slot input). Closes `gui-seedqr-slot-subkey-help-mirror` FOLLOWUP filed at toolkit Cycle 10 close. The toolkit schema_mirror gate compares clap flag-NAME parity NOT value-content, so this bump is non-blocking — but desirable for `seedqr` slot subkey discoverability in the GUI SlotEditor dropdown.

### Lockstep

- Toolkit pin: `mnemonic-toolkit-v0.31.0 → mnemonic-toolkit-v0.31.3`. Cumulative catch-up across v0.31.1 (sparrow taproot multisig descriptor-passthrough; no GUI surface change), v0.31.2 (sparrow taproot singlesig template-mode; no GUI surface change), and v0.31.3 (SeedQR slot input).

### Changed

- `src/form/slot_editor.rs::SlotSubkey` — added `Seedqr` variant at enum position 1 (after Phrase, before Entropy) to mirror the toolkit's enum-position correctness. `ALL` constant + `as_str` + `is_secret_bearing` extended.
- `src/secrets.rs::v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS` — supply-chain drift snapshot updated to `["phrase", "seedqr", "entropy", "xprv", "wif"]` (the compile-time drift gate fired as designed on the toolkit pin bump; acknowledged by snapshot update). Snapshot history docstring extended.
- `tests/secrets.rs::secret_slot_subkeys_set_pinned` — expected-set updated to include `"seedqr"`.

### Test totals

- 353 cells passing; 1 ignored. +1 net assertion (the existing slot_subkey_set_pinned test continues to enforce the explicit set membership).

### Cycle topology

Cycle 10b — GUI mirror for Cycle 10 / toolkit v0.31.3. Both Cycle 10a (toolkit) and 10b (GUI) close their respective FOLLOWUPs in lockstep on 2026-05-21.

---

## mnemonic-gui [0.16.0] — 2026-05-21

**SemVer-MINOR release.** Paired companion to `mnemonic-toolkit-v0.31.0`'s new `--bsms-encryption-token` flag on `mnemonic import-wallet` (BIP-129 encryption envelope decrypt). Skips toolkit `v0.30.1` (PATCH; no clap-surface change).

### Lockstep

- Toolkit pin bumped `mnemonic-toolkit-v0.30.0` → `mnemonic-toolkit-v0.31.0` (skips v0.30.1 since that was no-clap-surface-change PATCH; schema-mirror gate enforces equivalence).
- `pinned-upstream.toml` `[mnemonic].tag` documentary mirror updated.
- `src/schema/mnemonic.rs`: new `FlagSchema` entry for `--bsms-encryption-token` on import-wallet (`FlagKind::Path { stdio_sentinel: true }`; not-required; not-repeating; non-secret-flag-level). Inserted alphabetically BEFORE `--bsms-round1`.

### Cycle context

Cycle 7 of v0.28+ residual FOLLOWUP release plan, executed as 7a (library `bsms_crypto.rs` + recon + opus R0) + 7b (CLI flag + parser integration + opus R0 plan-doc + ship). Both R0 cycles caught critical design issues pre-implementation. Library cross-validated byte-exact against BIP-129 §Test Vectors TV-3 + Coinkite Python ref. Toolkit `mnemonic-toolkit-v0.31.0` (`e2e62ce`) tag landed first; install-pin-check CI green. GUI tag lands second; closure-verification confirms GUI CI's `schema_mirror` gate passes against the new pin.

## mnemonic-gui [0.15.0] — 2026-05-21

**SemVer-MINOR release.** Paired companion to `mnemonic-toolkit-v0.30.0`'s new top-level `mnemonic seedqr` encode/decode subcommand.

### Lockstep

- Toolkit pin bumped `mnemonic-toolkit-v0.29.0` → `mnemonic-toolkit-v0.30.0`.
- `pinned-upstream.toml` `[mnemonic].tag` documentary mirror updated.
- `src/schema/mnemonic.rs`: **two new entries.** `SubcommandSchema { name: "seedqr-encode", ... }` + `SubcommandSchema { name: "seedqr-decode", ... }` placed between `seed-xor-combine` and `slip39-split` per verb-ordering convention (create-side `encode` before recover-side `decode`, matching the seed-xor/slip39 split-before-combine precedent).
  - `seedqr-encode` flags: `--from phrase=<VALUE|->` (NodeValueComposite, required) + `--json-out <PATH>` + `NO_AUTO_REPAIR_FLAG`.
  - `seedqr-decode` flags: `--digits <VALUE|->` (Text, required, secret=true) + `--json-out <PATH>` + `NO_AUTO_REPAIR_FLAG`.

### Cycle context

Cycle 5 of the v0.28+ residual FOLLOWUP release plan. Architectural pivot: original FOLLOWUP slug `wallet-import-jade-seedqr` superseded by vendor-neutral `seedqr-encode-decode-subcommand` (SeedQR is an open SeedSigner spec, not Jade-proprietary). Tracked in toolkit `design/BRAINSTORM_v0_30_0_seedqr.md` + `design/PLAN_mnemonic_toolkit_v0_30_0.md`. Toolkit-first ordering: toolkit `mnemonic-toolkit-v0.30.0` tag landed first (`56dd2b6`, install-pin-check CI green); GUI tag lands second; closure-verification confirms GUI CI's `schema_mirror` gate passes against the new pin.

## mnemonic-gui [0.14.0] — 2026-05-21

**SemVer-MINOR release.** Paired companion to `mnemonic-toolkit-v0.29.0`'s SemVer-minor cliff (xpub-search result tagged-enum conversion + `ImportProvenance::Bsms(Option<_>)` 2-variant split + `error.rs` retroactive alphabetical sort).

### Lockstep

- Toolkit pin bumped `mnemonic-toolkit-v0.28.4` → `mnemonic-toolkit-v0.29.0` (4 toolkit releases captured: v0.28.5 docs + v0.28.6 test-hygiene + v0.28.7 hardening + v0.29.0 SemVer-minor cliff).
- `pinned-upstream.toml` `[mnemonic].tag` documentary mirror updated.
- `src/schema/mnemonic.rs`: **no edit needed.** The clap flag-name surface is byte-identical across all 4 captured toolkit releases (the wire-shape break is serde-output-only). Cycle 4 P0 recon dossier + Phase 6 verification confirmed `gui-schema` JSON byte-identical between v0.28.7 and v0.29.0.

### Downstream wire-shape break (informational; not gated by `schema_mirror`)

The toolkit v0.29.0 xpub-search result types switch from struct → `#[serde(tag = "result", rename_all = "snake_case")]` tagged enums. Consumers of `mnemonic xpub-search --json` output checking `.path === null` (or similar null-on-no-match patterns) break — the `path` / `template` / `account` keys are absent on `no_match` rather than null. The discriminator field name is preserved as `"result"` (`"match"` / `"no_match"`).

**GUI's runtime consumers of xpub-search JSON output have NO automated drift gate** — the `schema_mirror` integration test enforces clap flag-name parity only, not JSON wire-shape. Tracked at toolkit FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` (filed v0.29.0).

### Cycle context

Cycle 4 of the v0.28+ residual FOLLOWUP release plan (Wave 3 SemVer-minor cliff + paired GUI). Tracked in toolkit `design/BRAINSTORM_v0_28_plus_residual_followups.md`. Toolkit-first ordering: toolkit `mnemonic-toolkit-v0.29.0` tag landed first; GUI tag lands second; closure-verification confirms GUI CI's `schema_mirror` gate passes against the new pin.

## mnemonic-gui [0.13.0] — 2026-05-20

Minor release: paired companion to `mnemonic-toolkit-v0.28.4`. Adds the `coldcard-multisig` value to the `export-wallet --format` dropdown via `EXPORT_FORMATS` schema-mirror constant. Closes the asymmetry where `--format coldcard-multisig` was accepted on the import side (already in `IMPORT_WALLET_FORMATS`) but rejected on the export side. The toolkit's new `CliExportFormat::ColdcardMultisig` variant refuses singlesig templates with a pointer to `--format coldcard`; the GUI dropdown shows the value but downstream refusal is the toolkit's responsibility.

### Changed

- `src/schema/mnemonic.rs:60-72` `EXPORT_FORMATS` constant — added `"coldcard-multisig"` between `"coldcard"` and `"jade"`. Total dropdown values: 9 → 10.
- `Cargo.toml` mnemonic-toolkit dep tag: `mnemonic-toolkit-v0.28.0` → `mnemonic-toolkit-v0.28.4` (catches up through v0.28.1/v0.28.2/v0.28.3 — all toolkit-only patches with no GUI lockstep).
- `pinned-upstream.toml` `[mnemonic].tag` documentary mirror: same bump v0.28.0 → v0.28.4.

### Cycle context

Cycle 3 of the A/B/C FOLLOWUP release plan tracked in mnemonic-toolkit's `design/BRAINSTORM_followups_abc_release_plan.md`. Wave 2 first ship; paired with toolkit `mnemonic-toolkit-v0.28.4`.

## mnemonic-gui [0.11.1] — 2026-05-19

### Changed

- **CI workflow triggers extended to release branches.** `build.yml` and `schema-mirror.yml` now run on PRs targeting `master` AND `release/**` (previously only `master`). Eliminates the silent-skip pattern v0.11.0 cycle worked around via `--admin` merges.
- **mnemonic-toolkit pin bump v0.26.0 → v0.27.2.** Catches up to v0.27.0 cross-format wallet conversion (envelope wire-shape replacement) + v0.27.1 PR-#26 fold + v0.27.2 cleanup. GUI envelope-consumer smoke cells added in `tests/cli_envelope_smoke.rs` for shape stability.
- **Schema mirror catch-up for v0.27.x surface additions.** `mnemonic bundle` gains `--import-json` + `--import-json-index`; `mnemonic export-wallet` gains `--bsms-form` + `--from-import-json` + `--from-import-json-index` + `bsms` format option; `mnemonic import-wallet` gains `--bsms-round1` + `--bsms-verify-strict`.

### Closed FOLLOWUPS

- `gui-workflow-trigger-include-release-branches`

## [0.11.0] — 2026-05-18

v0.26.0 cycle lockstep with `mnemonic-toolkit-v0.26.0`. Three-feature
release: adds the `mnemonic import-wallet` SubcommandSchema entry + 8
kittest cells pinning argv-emission contracts for the new BSMS Round-2 /
Bitcoin Core `listdescriptors` ingest surface; four matching
`SubcommandSchema` entries for the toolkit's new `xpub-search` umbrella
(4 modes); and a `compare-cost` SubcommandSchema + mutex helper for the
new wsh-vs-tr per-spending-condition cost comparison subcommand. Schema
stays at v5 — no version bump on the schema envelope itself; only the
`name`-keyed `SUBCOMMANDS` array grows by 6 (1 import-wallet + 4
xpub-search-* + 1 compare-cost).

### Added

- **`SubcommandSchema` entry for `mnemonic import-wallet`** in
  `src/schema/mnemonic.rs`. 7 flags: `--blob` (`Path { stdio_sentinel: true }`,
  required), `--format` (`Dropdown(["bsms", "bitcoin-core"])`),
  `--select-descriptor` (`Text`, free-form to accommodate the
  `N | active-receive | active-change | all` union), `--ms1` (`Text`,
  `secret: true`, `repeating: true`), `--slot` (`Text`, `repeating: true`),
  `--json` (`Boolean`), plus the global `--no-auto-repair` flag. New
  `IMPORT_WALLET_FORMATS` const carries the `--format` dropdown values
  for symmetric clap-derive ↔ schema mirror.

- **8 kittest cells** in `tests/kittest_import_wallet_form.rs`
  pinning the argv-emission contracts for the new subcommand:
  `cell_import_wallet_in_subcommands_set`,
  `cell_import_wallet_blob_path_argv`,
  `cell_import_wallet_blob_stdio_sentinel_argv`,
  `cell_import_wallet_repeating_ms1_argv`,
  `cell_import_wallet_select_descriptor_default_suppressed`,
  `cell_import_wallet_format_dropdown_argv`,
  `cell_import_wallet_slot_phrase_argv`,
  `cell_import_wallet_env_sentinel_literal_emission`.

- **Four `xpub-search` `SubcommandSchema` entries (toolkit v0.26.0
  lockstep):**
  - `xpub-search-path-of-xpub` — locate a target xpub's derivation path
    by iterating account-index candidates against a master seed.
    Toolkit commit `d28b170` (C1, P1 + umbrella scaffolding).
  - `xpub-search-account-of-descriptor` — locate the descriptor's
    account index across 4 descriptor shapes (single-sig + 3 multisig
    via SLIP-0132 prefixes). Toolkit commit `196cc8a` (C2, P2).
  - `xpub-search-address-of-xpub` — locate which BIP-44 address-class
    chain + index under a parent xpub produces a target address.
    Toolkit commits `a5bfbaf` (C3, P3) + `365c0d1` (P2PKH gap-fix fold).
  - `xpub-search-passphrase-of-xpub` — locate a BIP-39 passphrase
    candidate by iterating wordlist tokens against a target xpub.
    Toolkit commit `bc2a76a` (C4, P4).
- **New `XPUB_SEARCH_ADDRESS_TYPES` dropdown const**
  (`schema/mnemonic.rs`) — kebab-case mirrors of the toolkit's
  `ScriptType` JSON tag enumeration: `p2pkh / p2sh-p2wpkh / p2wpkh /
  p2tr`. Backs the `xpub-search-address-of-xpub --address-type`
  Dropdown widget.
- **Dedicated `tests/xpub_search_schema_mirror.rs`** asserting (a) all
  4 umbrella subcommand entries are present in `SCHEMA.subcommands`,
  (b) the GUI's flag-name set matches the toolkit's `gui-schema` JSON
  per-subcommand (skipped if `MNEMONIC_BIN` unset + bare `mnemonic`
  not on PATH), and (c) per-mode required-flag invariants (e.g.
  `--target-xpub`, `--xpub` + `--target-address`).
- **Lightweight kittest + argv-assembler cells** in
  `tests/xpub_search_widgets.rs` — one kittest cell per new
  subcommand confirming generic-renderer no-panic instantiation, plus
  argv-assembler cells per subcommand confirming the form-state →
  argv emission carries the expected flag list.

- **`compare-cost` subcommand:** new `SubcommandSchema` entry with
  `COMPARE_COST_FLAGS` (5 flags: `--miniscript`/`--descriptor` as
  `FlagKind::Text`, `--feerate` as `Text` for `f64` decimal support,
  `--max-conditions` as `Number`, `--json` as `Boolean`) and the mutex
  `form::conditional::compare_cost` helper that toggles
  `--miniscript`/`--descriptor` Disabled state based on the other's
  fill (mirrors clap's `conflicts_with`). Stdin fallback (toolkit
  Phase 3) is not surfaced as a GUI input slot — the GUI always passes
  one of the two flags. `--feerate` uses `FlagKind::Text` rather than
  `Number` because the toolkit's `f64` clap parser accepts decimals
  (`0.0..=10000.0`); the GUI's Number kind is `i64`-only. Decimal
  validation is toolkit-side (exit 64 on bad input).

### Changed

- **Toolkit pin bumped** from `mnemonic-toolkit-v0.24.0` to
  `mnemonic-toolkit-v0.26.0` across `Cargo.toml::[dependencies]
  mnemonic-toolkit` + `pinned-upstream.toml::[mnemonic].tag` +
  `src/schema/mnemonic.rs::SCHEMA.pinned_version` monospace label.
  The v0.26.0 toolkit ships the new `mnemonic import-wallet` subcommand
  + 4 modes of `mnemonic xpub-search` + cross-cutting `@env:<VAR>`
  value-source sentinel; the GUI schema-mirror drift gate auto-greens
  once all three pins point at v0.26.0.

### Security

- v0.11.0 GUI emits user-typed values for repeating `--ms1` (and other
  secret-bearing) flags VERBATIM on argv. The toolkit-side resolves
  `@env:<VAR>` sentinels at parse time, so GUI users who want argv-leak
  protection MUST type `@env:MY_VAR` themselves with `MY_VAR` exported
  in the calling shell. Auto-rewriting literal repeating-secret values
  to per-cosigner `@env:MNEMONIC_MS1_<i>` sentinels (the SPEC §9.3
  aspirational behavior) is tracked at FOLLOWUP
  `gui-import-wallet-env-var-secret-channel` (v0.12.0+). Pairs with the
  pre-existing `gui-run-confirm-modal-secret-redaction` FOLLOWUP
  (modal-redaction direction).

### Audit notes for v0.24.0 → v0.26.0 pin drift

The 8 kittest cells were authored against the v0.24.0 toolkit binary.
Two behavior changes landed in toolkit v0.25.x that COULD impact
GUI-side cells; per coordinator merge-plan §G1.5 audit:

- **v0.25.0 TTY-gate extension to `mnemonic convert` + `mnemonic inspect`**
  (`[[project-v0-9-0-mnemonic-gui-shipped]]` precedent): not
  applicable. None of the 8 cells spawn `mnemonic` as a subprocess
  (kittest exercises in-process argv assembly via the GUI invocation
  builder); none target `convert`/`inspect`.
- **v0.25.1 empty-`ms1` watch-only stderr NOTICE** in multi-cosigner
  `verify-bundle`/`repair`: not applicable. The 8 cells do not target
  `verify-bundle`/`repair` and do not assert on stderr at all.

No cell updates needed.

## [0.10.0] — 2026-05-17

v0.24.x cycle lockstep with `mnemonic-toolkit-v0.24.0`. Consumes
toolkit's gui-schema v5 envelope (default_value + global + secret
fields), retires the v0.9.0 R7 action-bar `--no-auto-repair`
fallback, fills 6 missing convert-subcommand conditional rules, and
migrates the v0.9.0 three-way card mutex to a softer
at-least-one-required policy (D36 lockstep with toolkit D35).

### Added

- **Schema v5 consumer (Tranche B.3 / D31 / D33):** `FlagSchema` struct
  gains `default_value: Option<&'static str>` + `global: bool` fields;
  every entry in `schema/{mnemonic,md,mk,ms}.rs` rebuilt against the
  toolkit v5 ground truth. `--no-auto-repair` declared
  `global: true` per-subcommand (drops the v0.9.0 action-bar
  affordance).
- **`is_at_default` argv-suppression (D33):** new predicate in
  `form/invocation.rs` per the 9-FlagKind table — argv assembler now
  suppresses flags whose user-set value matches the toolkit-declared
  `default_value`. 12 new D33 cells in `tests/argv_assembler.rs`; 5
  existing cells adjusted for the new default-suppression behavior.
- **Disabled-suppression regression matrix (Tranche B.2):** new file
  `tests/argv_assembler_disabled_suppression.rs` with 9 per-FlagKind
  cells (Boolean / Text / Number / Dropdown / NodeValueComposite /
  Range / Timestamp / Path + negative control). Locks the existing
  `assemble_argv::96-98` suppression mechanism against silent
  regression.
- **R7 removal regression file (Tranche B.3):** `tests/r7_no_auto_repair_removal.rs`
  (7 cells) covering the action-bar checkbox + runner helper deletion
  at the 5 sites in `src/main.rs` + `src/runner.rs`.
- **Secret-drift gate (Tranche B.3):** `tests/schema_mirror_secret_drift.rs`
  (1 cell) — cross-checks GUI `FlagSchema.secret` field against
  toolkit v5's `flag_is_secret` ground truth via `gui-schema` JSON
  output. `--reveal-secret` reconciled to `secret: false`.
- **6 missing convert-subcommand conditional rules (Tranche B.4):**
  `form/conditional.rs::convert` gains visibility rules for
  `--electrum-version`, `--electrum-language`, `--script-type`,
  `--template`, `--path`, `--xpub-prefix`. New `to_contains` helper
  at `:441-450` supports multi-value `--to` predicate. 17 new cells
  in `tests/conditional_visibility.rs`.
- **`three_way_card_at_least_one` helper (Tranche C.2 / D36):**
  renames v0.9.0's `three_way_card_mutex` to reflect the softer
  policy — at least one card must be supplied (lockstep with
  toolkit D35 cross-HRP mutex drop). Callers
  `conditional::repair` + `conditional::inspect` updated. 8 v0.22.x
  A.2 cells rewritten + 4 new cells (12 total in the at-least-one
  block).
- **Toolkit clippy --all-targets CI job (Tranche A.3):** new
  `clippy-all-targets` job in `.github/workflows/build.yml` running
  `cargo clippy --workspace --all-targets -- -D warnings` against
  HEAD. Catches test-target lints that the prior `--lib --bins` gate
  missed.

### Changed

- **Toolkit pin (3 sites):** `mnemonic-toolkit-v0.22.1` → `v0.24.0` in
  `Cargo.toml:42` git-dep tag, `pinned-upstream.toml:22`
  `[mnemonic] tag`, `src/schema/mnemonic.rs::SCHEMA.pinned_version`
  monospace label.
- **`schema/mnemonic.rs` prose (Tranche C.2):** 6 `help`-string
  updates + 2 module-level prose updates across `repair` / `inspect`
  flags (Mutually-exclusive → Combinable-with-X).
- **5 pre-existing clippy lints fixed in test files (Tranche A.3):**
  3× `doc_overindented_list_items` in `tests/manual_anchor_coverage.rs`;
  1× `field_reassign_with_default` in `tests/slot_editor_contiguity.rs`;
  1× `len_zero` in `tests/conditional_visibility.rs`. All caught by
  the new --all-targets CI job.

### Removed

- **R7 action-bar `--no-auto-repair` checkbox** at 5 sites:
  `src/main.rs:99` / `:260` / `:322-326` / `:788` + `src/runner.rs:48`
  + 3 unit cells. With toolkit v5 emitting `global: true` for the
  flag, the GUI mirrors it natively per-subcommand and no longer
  needs the load-bearing top-level fallback. `runner::prepend_no_auto_repair`
  retired.

### Resolved (FOLLOWUPS)

- `clippy-test-target-cleanup` — GUI-only; closed via the 5 lint
  fixes + the --all-targets CI job.
- `gui-schema-global-flag-emission` — companion close lockstep with
  toolkit v0.24.0 (Tranche B primary).
- `toolkit-mnemonic-force-tty-promote-from-test-only` — companion
  close lockstep with toolkit v0.24.0 (Tranche A primary).
- `md-codec-decode-with-correction-supports-non-chunked-md1` —
  companion close lockstep with md-codec v0.35.0 (Tranche D primary).
- `verify-bundle-watch-only-xpub-path-internal-consistency` —
  GUI-side observer companion to the toolkit primary at v0.24.0
  (GUI surfaces the new stderr WARNING through its existing stderr
  pane; no GUI code change required beyond the toolkit pin bump).

### Tests

- 240 → ~300+ (final count via `cargo test --workspace`): +9
  Disabled-suppression cells, +12 D33 argv-suppression cells,
  +17 conditional-visibility cells (B.4 convert rules), +12
  at-least-one cells (Tranche C.2), +7 R7-removal cells, +1
  secret-drift cell.

### Companion

`mnemonic-toolkit v0.24.0` (Tranche A + B + C primary).
`mk-cli v0.4.1` (independent patch, no GUI scope).
`md-codec v0.35.0` (Tranche D primary).

## [0.9.0] — 2026-05-17

Catchup release wiring the v0.22.0 + v0.22.1 toolkit BCH
error-correction surface into the desktop GUI. Lockstep with the
already-shipped `mnemonic-toolkit-v0.22.1` (BCH repair launch +
verify-bundle auto-fire) and `mk-cli-v0.4.0` (sibling-CLI repair
subcommand) cycle releases.

### Added

- `mnemonic repair` GUI surface: new `REPAIR_FLAGS` schema array
  (`--ms1` secret/Option, `--mk1` repeating, `--md1` repeating,
  `--json`) wired into `SUBCOMMANDS` with `conditional::repair`
  3-way card mutex.
- `mnemonic inspect` GUI surface: new `INSPECT_FLAGS` schema array
  (same 3-way card mutex + `--json` + `--reveal-secret`) with
  `conditional::inspect` mutex.
- `form::conditional::three_way_card_mutex` helper (shared between
  `repair` + `inspect`) — when all 3 cards unset, all 3 are
  Required; when exactly 1 is set, the other 2 are Disabled; ≥2
  set lets the CLI rejection surface naturally. NET-NEW conditional
  pattern (distinct from `verify_bundle::*`'s
  `bundle_json XOR cards-group` mutex).
- Action-bar `--no-auto-repair` checkbox (R7 fallback; load-bearing
  per Phase A.1 finding — see "Why a top-level checkbox" below).
  When checked, `runner::prepend_no_auto_repair` splices the global
  flag into argv after the binary name at spawn time.
- `render_exit_badge` helper (`src/main.rs`) — green badge
  `(60, 180, 75)` on exit 5 announcing "Repair Applied (BCH
  auto-fire succeeded)"; default label preserved for other exit
  codes. Matches existing slot_editor warning chroma.
- D23 `MNEMONIC_FORCE_TTY=1` spawn-time env-var
  (`src/runner.rs::run`) — toolkit's auto-fire gate is
  `std::io::stdout().is_terminal() && !no_auto_repair`; GUI
  subprocesses are piped (never TTY), so without this env override
  the GUI would never see auto-fire repair reports from
  `convert` / `inspect` / `verify-bundle` invocations. Filed
  `toolkit-mnemonic-force-tty-promote-from-test-only` toolkit-side
  + GUI-side companion to promote the env-var from its currently
  test-only documentation to a first-class public contract in a
  future toolkit minor.

### Changed

- Toolkit pin: `mnemonic-toolkit-v0.20.0` → `v0.22.1` across 3
  sites (`Cargo.toml:42` git-dep tag, `pinned-upstream.toml:22`
  `[mnemonic] tag`, `src/schema/mnemonic.rs` `pinned_version`
  monospace label).
- mk pin: `pinned-upstream.toml [mk] tag` →
  `mk-cli-v0.4.0` (lockstep with the mk-cli v0.4.0 release shipped
  concurrently; the prior `mk-cli-v0.3.1` → `v0.4.0` bump landed
  ahead of this release at `a15baf2`).

### Fixed

- Cleaned up stale "runtime soft-check (SPEC §11)" comment block at
  `src/schema/mnemonic.rs:1092-1099`. The `pinned_version` field is
  render-only at `main.rs:347` — there is no comparison logic;
  comment was misleading future maintainers.

### Why a top-level checkbox for `--no-auto-repair`

Phase A.1 surfaced that the toolkit's `mnemonic gui-schema` JSON
output (the schema-mirror drift-gate's source-of-truth) does NOT
emit global flags like `--no-auto-repair` for any subcommand — only
clap's per-subcommand `--help` TEXT propagates them. Adding
`--no-auto-repair` to the 10 existing `*_FLAGS` arrays hard-failed
the drift gate. The action-bar checkbox is therefore load-bearing
for v0.9.0 (~30 LOC across `runner::prepend_no_auto_repair` +
`MnemonicGuiApp.no_auto_repair` field + `main.rs` checkbox cell).
Tracked as `gui-schema-global-flag-emission` (toolkit-side primary
+ GUI companion); when the toolkit emits global flags per-
subcommand in a future cycle, the GUI can drop the action-bar
affordance in favor of native per-subcommand schema mirroring.

### Tests

- +12 cells (264 → 276): 8 conditional-visibility cells in
  `tests/conditional_visibility.rs` (4 repair × 4 inspect mutex
  states) + 4 runner cells (D23 env injection + 3
  prepend-helper edge cases).
- Schema-mirror + drift gates green against
  `mnemonic-toolkit-v0.22.1`.
- Manual smoke confirmed: corrupted ms1 in Convert → green exit-5
  badge + repair report in stderr + corrected ms1 in stdout;
  checkbox opt-out path correctly suppresses auto-fire.

## [0.7.2] — 2026-05-16

### Fixed — revert v0.7.0 disable_options for --template (UX flaw); migrate to inline warning banner

Lockstep with `mnemonic-toolkit-v0.18.1`. Drops the v0.7.0 bundle()
visibility pushes that disabled --template options based on
slot_count, and replaces them with a GUI-internal warning banner
adjacent to the slot grid.

#### Why the v0.7.0 emission was wrong

v0.7.0 added two `Visibility::DisableOptions` entries on bundle's
`--template`:
- slot_count >= 2 → disable single-sig template options
- slot_count == 1 → disable multisig template options

**Row 11 was a design flaw**: `slot_count == 1` is the natural
TRANSIENT state when a user is building UP to multisig (slots get
added one at a time, passing through 1 on the way to 2+). Disabling
multisig templates at that transient state prevents the user from
selecting their intended template before completing slot setup —
the user can only ever pick from single-sig, even when they meant
to build a multisig wallet. Row 10 had the symmetric flaw during
multisig→single-sig template switches.

Surfaced 2026-05-16 by user report: "for bundle command, i can not
select anything but the 4 single sig formats for --template".

#### Replacement: warning banner (Option A pattern)

Mirrors the v0.7.1 row-8 slot-contiguity warning. The --template
dropdown renders all options normally; an inline orange warning
banner fires adjacent to the slot grid when the chosen template +
slot_count combination would fail CLI rows 10/11 at runtime. The
warning text suggests both directions of fix (change template OR
adjust slot count) so the user can pick whichever matches their
intent. CLI's mode-violation ladder (§6.6 rows 10/11) remains the
authoritative gate.

#### Changes

- `src/form/conditional.rs::bundle`: row 10 + row 11 visibility
  pushes deleted. Replaced with explanatory in-line comment.
- `src/form/conditional.rs::template_slot_count_warning` (NEW):
  helper returning `Option<String>` when the chosen template +
  slot_count combination is invalid.
- `src/main.rs`: after `slot_editor::render`, calls the helper +
  renders the warning via `ui.colored_label`.
- `Visibility::DisableOptions` enum variant retained for forward-
  compat (still a defined v4 grammar surface; just unused after
  rollback).
- `tests/conditional_visibility.rs`: row 10/11 disable_options
  assertions DELETED; replaced with
  `cell_v0_18_1_bundle_emits_no_disable_options_after_row_10_11_rollback`
  (anti-regression guard) + 7 new cells covering the
  `template_slot_count_warning` helper (none for unset; valid
  single-sig/multisig configurations; row 10/11 fire conditions
  including the user's reported scenario).
- `tests/argv_assembler_visibility.rs::disable_options_does_not_suppress_argv_emission`:
  DELETED (no live emission to verify the no-suppress contract
  against; SPEC §6.10.4 still documents the contract for future
  grammar use).
- `tests/gui_schema_conditional_drift.rs::SUBCOMMAND_FLOORS`:
  bundle floor 13 → 11; total floor 36 → 34 (v0.17.1 baseline).

#### Closes

(No FOLLOWUP closures; same-cycle bugfix for a v0.7.0 design issue.)

#### Verification

- `MNEMONIC_BIN=...v0.18.1/mnemonic cargo test --offline`: 240
  passed, 0 failed, 1 ignored (was 235 at v0.7.1; net +5 cells —
  deleted 2 row-10/11 cells + 1 argv-no-suppress cell; added 1
  anti-regression guard + 7 warning-helper cells).

#### Companion

Toolkit pin bumped in lockstep: `Cargo.toml [dependencies]
mnemonic-toolkit` tag `v0.18.0 → v0.18.1`; `pinned-upstream.toml`
`[mnemonic].tag` matches.

## [0.7.1] — 2026-05-16

### Added — SPEC §6.6 row 8 GUI-internal slot-contiguity pre-check (Batch B-2 partial closure)

Closes the row-8 share of the v0.6.0-cycle FOLLOWUP `gui-schema-cross-
slot-predicate-projection`. Rows 13/14 are closed as wontfix (CLI
rejection is sufficient; GUI pre-check would add marginal UX at
significant code cost — full BIP-388 distinct-key enforcement requires
the toolkit's xpub derivation logic which can't be replicated GUI-side
for phrase-bearing slots).

#### What shipped

- `src/form/slot_editor.rs::detect_slot_index_gaps(rows)` helper —
  returns sorted `Vec<u8>` of missing indices that would cause the
  CLI to reject the bundle with `error: slot indices must be
  contiguous starting at @0; missing @{i}`. Operates on UNIQUE
  indices (duplicate-index rows with different subkeys are NOT a
  contiguity violation).
- `slot_editor::render()` calls the helper after the slot grid +
  Add-slot button. When gaps are detected, renders an inline orange
  warning banner: `⚠ slot indices must be contiguous starting at @0;
  missing @0, @2, ...`.

#### Design pattern

Option A (mirrors v0.7.0 `NumberMax::FromSlotCount` for row 9): pure
GUI-internal pre-check; no toolkit wire-format change. The CLI still
authoritatively rejects non-contiguous bundles at runtime; the GUI's
pre-check is purely UX.

#### NEW test file

- `tests/slot_editor_contiguity.rs` — 9 cells covering: empty set,
  single slot @0, contiguous N-slot, missing @0, single slot @3 (all
  lower missing), middle gap, multiple middle gaps, duplicate
  indices (non-violation), unsorted input.

#### Closes FOLLOWUPS

- `gui-schema-cross-slot-predicate-projection` (cross-repo) — row 8
  resolved GUI-side (Option A); rows 13/14 wontfix with rationale.
  All v0.6.0-cycle-close FOLLOWUPs are now closed (Batch A v0.6.1 +
  Batch B-1 v0.7.0 + Batch B-2 v0.7.1).

### Verification

- `MNEMONIC_BIN=...v0.18.0/mnemonic cargo test --offline`: 235
  passed, 0 failed, 1 ignored (was 226 at v0.7.0; +9 new cells in
  the contiguity test file).
- No toolkit functional change required; toolkit-side companion is
  docs-only (SPEC §6.10.7 row 8 → `ENCODED v3 (GUI-internal)` + the
  cross-repo FOLLOWUP closure with the same partition note).

## [0.7.0] — 2026-05-16

### Added — SPEC §6.10 v3-cycle GUI consumer (schema v4 disable_options Effect + GUI-internal NumberMax::FromSlotCount)

Lockstep with `mnemonic-toolkit-v0.18.0`. Closes the v0.6.0-cycle
FOLLOWUP `gui-schema-effect-on-dropdown-options-vocab` (Batch B-1).

#### Wire-format consumer (`src/schema_check.rs`)

- `VisibilityProjection` enum gains the `DisableOptions { values:
  Vec<String> }` variant. Custom `Deserialize` extended to accept the
  new tagged-object wire shape `{"disable_options": {"values": [<string>,
  ...]}}` alongside v3's `pin_value` + v2's bare-string. Fail-CLOSED
  posture preserved for any other tagged-object key.

#### GUI-internal NumberMax FlagKind extension (`src/schema/mod.rs`)

- `FlagKind::Number { max: i64 → max: NumberMax }` shape change
  (BREAKING for any out-of-tree consumer of `mnemonic_gui::schema::
  FlagKind`; this GUI is the sole consumer at release time, verified
  via `grep -rn "use mnemonic_gui::schema::FlagKind" /scratch/code/`).
- New `NumberMax = Static(i64) | FromSlotCount` enum closes SPEC §6.6
  row 9 (`--threshold` max equals `state.slot_count()`) GUI-side, with
  no toolkit wire-format change (Option A per the v0.7.0 design doc —
  toolkit does not emit `from_slot_count` and the bounds live in the
  GUI's per-flag declaration alone).
- `NumberMax::resolve(state)` helper falls back to `1` when
  `slot_count() == 0` (degenerate but valid range `min..=1`; CLI row 9
  catches the residual case).
- `Visibility` enum gains `DisableOptions { values }` (mirror of the
  wire-format consumer's `VisibilityProjection` extension).
- 25-site FlagKind::Number cascade migrated to `NumberMax::Static(N)`
  (or `FromSlotCount` for the 3 `--threshold` instances in
  `src/schema/mnemonic.rs`); zero out-of-tree consumers affected.

#### Conditional fn additions (`src/form/conditional.rs`)

- New `MULTISIG_TEMPLATES: &[&str]` const (mirror of
  `SINGLE_SIG_TEMPLATES`; order matches toolkit
  `CliTemplate::value_variants()` for drift-gate parity).
- `bundle()` gains two new rules: row 10 (`slot_count >= 2` →
  `--template DisableOptions { single_sig }`) + row 11 (`slot_count ==
  1` → `--template DisableOptions { multisig }`).

#### Render-time composition (`src/form/widget.rs` + `src/main.rs`)

- `render_with_dispatch` + `render` signatures gain `state: &FormState`
  (for `NumberMax::FromSlotCount` resolve) + `disabled_options:
  &[String]` (orthogonal Dropdown-option grey-out — extracted at
  the main.rs render-loop call-site by filtering the vis map for
  `Visibility::DisableOptions` entries).
- Number widget arm uses `max.resolve(state)` to compute the runtime
  upper bound; Dropdown widget arm iterates `disabled_options` to grey
  out + non-select listed values via `egui::Ui::add_enabled_ui`.
- A flag can now have BOTH a primary first-rule-wins Visibility (e.g.,
  `Required` red-asterisk decoration on the label) AND `DisableOptions`
  (per-option grey-out in the Dropdown) — orthogonal effects compose.

### Drift gate floors raised

`tests/gui_schema_conditional_drift.rs::SUBCOMMAND_FLOORS`: `bundle`
bumps `11 → 13`; total `34 → 36`. Other floors unchanged. The drift
gate's per-rule `find` was upgraded from "first entry per flag" to
"any entry matching the expected visibility variant" — the runtime
render-loop honours first-rule-wins for the primary visibility +
extracts `DisableOptions` separately, so the test should verify
presence-by-content rather than first-match.

### New test files / cells

- NEW `tests/number_max_from_slot_count.rs`: 4 cells unit-testing
  `NumberMax::resolve` (FromSlotCount happy path + slot_count==0
  clamp-to-1 + Static round-trip + monotonic increase across
  slot_count 1..=8).
- NEW cells in `tests/conditional_visibility.rs`: row 10 + row 11
  DisableOptions assertions via a new `disabled_options_for` helper
  (composition-aware lookup).
- NEW cells in `tests/argv_assembler_visibility.rs`: pins the
  "disable_options is schema-time only" argv contract +
  "threshold value above slot_count emits unchanged" stale-state
  contract (no auto-clamp at frame boundary).
- NEW cell in `tests/schema_mirror.rs`: MULTISIG_TEMPLATES const-vs-
  meta-block parity (sibling of the SINGLE_SIG cell).

### Closes FOLLOWUPS

- `gui-schema-effect-on-dropdown-options-vocab` (cross-repo) —
  toolkit emits the v4 `disable_options` Effect; GUI consumes via
  schema_check + render-time Dropdown filtering. Row 9 also closes
  GUI-side via `NumberMax::FromSlotCount` (no toolkit wire change).

### Verification

- `MNEMONIC_BIN=...v0.18.0/mnemonic cargo test --offline`: **226
  passed, 0 failed, 1 ignored** (was 222 at v0.6.1; +6 net new cells).
- Drift gate per-subcommand floor `bundle ≥ 13` + total ≥ 36 pass.
- Build green on local linux release profile.

### Companion

Toolkit pin bumped in lockstep: `Cargo.toml [dependencies]
mnemonic-toolkit` tag `v0.17.1 → v0.18.0`; `pinned-upstream.toml`
[mnemonic].tag matches (bumped separately in commit `c90d730`
as the schema-mirror entry-criterion).

## [0.6.1] — 2026-05-16

### Fixed — defense-in-depth folds (canary tests for serde-other dependency + drift gate per-subcommand floors + --slot PinValue debug_assert)

Patch release folding 3 FOLLOWUPs filed at the `mnemonic-gui-v0.6.0`
cycle-close opus reviewer audit. No SPEC grammar additions; no
wire-format changes; no widget refactors. Pure defense-in-depth +
toolkit-side cosmetic fix.

#### Companion toolkit bump: `mnemonic-toolkit-v0.17.1`

`mnemonic-toolkit-v0.17.0` → `mnemonic-toolkit-v0.17.1` (commit
`7ed3784`) on both the documentary `pinned-upstream.toml [mnemonic].tag`
(P2 commit `6d57a89`) and the load-bearing
`Cargo.toml [dependencies] mnemonic-toolkit` pin (this release commit).
The toolkit patch drops a spurious `meta.template_groups` block from
the `derive-child` subcommand's gui-schema output — silently emitted
in v0.17.0 despite derive-child having no `--template` flag. The GUI
does not consume derive-child's meta block, so no GUI source change is
required for the toolkit fix; the GUI just picks up the cleaner JSON
shape via the bump.

#### #4 (`gui-flag-value-unset-serde-other-externally-tagged-dependency`) — canary pair

Added 2 cells to `tests/widget_unset_sentinel.rs` covering distinct
serde branches; re-purposed the existing
`flag_value_unknown_tag_deserializes_to_unset_via_serde_other` cell
(at lines 154-165) as the load-bearing CANARY anchor.

- `flag_value_unset_canary_known_tags_still_deserialize_correctly` —
  regression guard for the canary pair; ensures `#[serde(other)]`
  doesn't accidentally swallow known tags too.
- `flag_value_unset_canary_unknown_tagged_object_currently_fails_to_deserialize`
  — **NEGATIVE** canary documenting an empirical v0.6.1 discovery:
  `#[serde(other)]` on externally-tagged FlagValue does NOT fall back
  tagged-object unknown variants (only bare-string unknown variants).
  The initial positive test failed (RED); inverted to a negative
  assertion that pins the observed asymmetry. If a future serde
  upgrade DOES make tagged-object fallback work, the canary fires and
  the v0.6.x forward-compat claim can be broadened.

**Forward-compat scope correction**: the v0.6.0 CHANGELOG claim
"v0.6+ readers map any unknown tag in state.json to Unset" is
**PARTIAL** — covers future *unit-variant* additions only; future
*data-carrying* variants would cause v0.6 readers to fail state.json
deserialization entirely. State.json files containing only known tags
(every variant currently shipped) continue to load fine; the
narrowing applies only to hypothetical future GUI versions adding new
data-carrying FlagValue variants.

#### #5A (`gui-pin-value-effect-on-slot-flag-gap`, sub-fold A) — slot-emit debug_assert

`src/form/invocation.rs::assemble_argv` lines 106-111 (the slot-emit
branch) gain a `debug_assert!(!matches!(flag_vis, Visibility::PinValue
{ .. }), ...)` + release-mode defensive `if-suppress`. The visibility
gate at lines 87-101 is `if flag.name != "--slot" || !subcommand.allows_slots`-
wrapped and so does not run for `--slot` on slot-bearing subcommands;
a future toolkit rule emitting PinValue for `--slot` would silently
fall through to the slot-emission branch and emit malformed argv
(pin_value's single-value emission semantic doesn't map onto `--slot`'s
multi-row `@N.subkey=value` grammar). The debug_assert fails loud in
dev/CI debug-profile; the release-mode `if-suppress` is the
defensive net.

A future cycle wanting legitimate pin_value-on-slot semantics must
remove this debug_assert and replace with the new design; the loud
fail makes that requirement visible at first encounter.

#### #5B (`gui-pin-value-effect-on-slot-flag-gap`, sub-fold B) — drift gate per-subcommand floors

`tests/gui_schema_conditional_drift.rs` replaces the prior
`assert!(total_rules > 0)` (vacuously satisfiable per
`[feedback-ci-snapshot-test-substring-vacuity]`) with per-subcommand
lower-bound floors. v0.17.1 baseline:

| Subcommand | Floor |
| --- | --- |
| `bundle` | 11 |
| `verify-bundle` | 10 |
| `export-wallet` | 6 |
| `convert` | 4 |
| `derive-child` | 3 |
| **Total** | **≥ 34** |

Failure message cites the floor table location so future
legitimate-reduction cycles (e.g., a grammar refactor consolidating
two rules into one) know exactly what to update. Added
`use std::collections::BTreeMap` import + `per_subcommand_rules`
accumulator populated only after the early-exit checks succeed.

### Verification

- `MNEMONIC_BIN=<path>/v0.17.1/mnemonic cargo test --offline`:
  23 test binaries, **222 passed**, 0 failed, 1 ignored (was 220 at
  v0.6.0; +2 net new cells in widget_unset_sentinel.rs).
- Drift gate exercises per-subcommand floors against the v0.17.1
  binary: all met (bundle 11 ≥ 11, etc.); total = 34.
- `cargo clippy --all-targets --offline` → no new lints from this
  cycle (pre-existing lints in unrelated test files unchanged).
- Bumps: `Cargo.toml [package].version 0.6.0 → 0.6.1`,
  `[dependencies] mnemonic-toolkit tag mnemonic-toolkit-v0.17.0 →
  mnemonic-toolkit-v0.17.1` (commit `7ed3784`), Cargo.lock in lockstep
  with both the workspace member bump and the git-dep tag bump.

### Closes FOLLOWUPS (3)

The cycle-close commit flips Status on these entries (toolkit + gui
repos):

- `gui-schema-derive-child-meta-template-groups-spurious` (cross-repo) —
  resolved at toolkit `7ed3784` / gui `4712a1c`-or-release-SHA.
- `gui-flag-value-unset-serde-other-externally-tagged-dependency` (gui-only)
  — resolved at GUI v0.6.1 SHA via the canary pair + scope-narrowed
  CHANGELOG claim.
- `gui-pin-value-effect-on-slot-flag-gap` (gui-only) — resolved at
  GUI v0.6.1 SHA (both sub-folds A + B).

## [0.6.0] — 2026-05-16

### Added — SPEC §6.10 v3 consumer (pin_value Effect + slot_count predicates + meta.template_groups)

Consumes `mnemonic-toolkit v0.17.0`'s schema-v3 `gui-schema` output.
Schema-version contract bumps 2 → 3 in lockstep with the toolkit;
back-compat preserved for the v2 wire shape per SPEC §6.10.6 (all
existing predicate kinds + bare-string Visibility variants still
deserialize bit-identically).

#### Phases shipped this cycle

- **P2 / P5** (`9d447d0`) — Predicate / Effect consumer + drift-gate
  extension.
  - `schema_check.rs::Predicate` gains `SlotCountEq` / `SlotCountGte` /
    `SlotCountLte` variants mirroring the toolkit's v3
    predicate-machinery (toolkit-side dead-code; emitted in the JSON
    schema-as-types, never in actual rules at v0.17.0).
  - `schema_check.rs::VisibilityProjection` gains
    `PinValue { value: serde_json::Value }`. Custom Deserialize
    accepts both v2 bare-string (`"hidden"` / `"disabled"` /
    `"required"`) and v3 tagged-object
    (`{"pin_value": {"value": V}}`) wire shapes. `Copy` dropped
    (`Value` isn't `Copy`); downstream consumers clone.
  - `schema::Visibility` (GUI-internal) +`PinValue` in lockstep;
    `Copy` dropped.
  - `FormState::slot_count()` accessor returning `slots.rows.len()`
    wires the Predicate evaluation path for `SlotCount*`.
  - `form::conditional::bundle()` pushes the SPEC §6.10.7 row 12
    rule when `--descriptor` is present:
    `("--account", Visibility::PinValue { value: json!(0) })`.
  - `form::invocation::assemble_argv` extended with a `PinValue`
    emission path — REPLACES the user-typed value before emit
    (vs Hidden/Disabled which suppress entirely). The visibility
    gate now executes the 3-way semantic per SPEC §6.10.4.
  - `tests/gui_schema_conditional_drift.rs::synthesize_satisfying`
    gains `SlotCount{Eq,Gte,Lte}` arms via a `set_slot_count`
    helper. `vis_to_visibility` extended with `PinValue` arm.

- **P3** (`84a69b8`) — `FlagValue::Unset` sentinel for
  Number / Range / Timestamp / TaggedOrIndexed widgets.
  - Numeric / structured-value widgets previously auto-seeded to a
    concrete value (`Number(min)`, `Range(0, 999)`, etc.) the
    moment they rendered; the auto-seeded value would then emit as
    `--<flag> <min>` into argv even when the user hadn't touched
    the widget. Bogus argv noise to the downstream CLI.
  - `FlagValue::Unset` (unit variant with `#[serde(other)]` for
    forward-compat) is the new default for those four kinds.
    `flag_value_is_present(Unset)` returns false → conditional fns
    + argv assembler treat Unset uniformly as absent.
  - `seeded_value_for(kind)` helper returns the kind-specific
    concrete seed (`Number(min)` etc.) used when the user clicks
    the new `Set` affordance on an Unset widget. The seeded
    widget also gains a small `✕` clear button that returns to
    Unset.
  - Persistence-schema delta: forward-compat preserved via
    `#[serde(other)]`; v0.5 readers (no Unset variant) CANNOT
    deserialize a v0.6 state.json that contains Unset entries —
    serde rejects the unknown tag and state-load fails (user sees
    a fresh form on first launch of v0.5 post-downgrade). Schema
    version intentionally NOT bumped — additive on the wire +
    bounded downgrade impact.

- **P4** (`538dc70`) — template-aware default form-state seed.
  - `form::conditional::template_defaults_for(template)` returns
    template-specific defaults: empty for single-sig
    (`bip44`/`bip49`/`bip84`/`bip86`); `[(--threshold, Number(2)),
    (--multisig-path-family, Dropdown("bip48"))]` for multisig
    templates (`wsh-multi`/`wsh-sortedmulti`/`tr-multi-a`/
    `tr-sortedmulti-a`). `bip48` is the canonical multisig path
    family; threshold-of-2 the smallest non-degenerate threshold.
  - `MnemonicGuiApp` gains a `last_template: BTreeMap<String,
    Option<String>>` field. Per-frame hook in `update()` detects
    `--template` transitions and applies
    `template_defaults_for(new_template)` ONLY to absent flags
    (seed-on-empty discipline — user-typed values preserved
    across template switches; never overwrites, never clears, no
    undo affordance needed). The visibility gate handles the
    inverse direction (single-sig template → Disabled
    threshold/path-family).
  - Closes the v0.16.0 `gui-default-form-state-template-aware-seed`
    FOLLOWUP — the previous `--multisig-path-family = bip87`
    hardcoded seed was REMOVED at v0.16.0 P5; v0.6.0 introduces
    the proper template-aware replacement.

#### SPEC reference

`mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10 (v3
extensions landed at `a26c809` toolkit-side):
  - §6.10.2: three new Predicate kinds (`slot_count_eq`/`gte`/`lte`).
  - §6.10.3: `pin_value` Visibility variant + wire-format details.
  - §6.10.4: Visibility-to-emission mapping table (NEW —
    enumerates per-visibility argv-emit semantic; PinValue is the
    only effect that produces argv with a value distinct from the
    user's input).
  - §6.10.6: version contract 2 → 3 + back-compat guarantee.
  - §6.10.7: row 12 (DESCRIPTOR_WITH_NONZERO_ACCOUNT) flipped
    DEFERRED → ENCODED v2 using pin_value.
  - §6.10.8 (NEW): per-subcommand meta-fields documentation
    (`meta.template_groups` is the first such field).

#### Closes FOLLOWUPS (5)

- `gui-schema-numeric-flag-value-pin-effect` (cross-repo —
  pin_value Effect grammar shipped both sides).
- `gui-schema-template-groups-meta-field` (cross-repo — toolkit
  emits `meta.template_groups`; GUI's `SINGLE_SIG_TEMPLATES` const
  retained as runtime source-of-truth gated by a new const-vs-meta
  parity test at
  `tests/schema_mirror.rs::single_sig_templates_const_matches_meta_template_groups`).
- `gui-schema-runtime-conditional-projection` (cross-repo —
  partial: predicate-machinery shipped; full encoding of SPEC §6.6
  rows 9/10/11 still deferred per §6.10.7 closing list, tracked
  going forward at NEW FOLLOWUP
  `gui-schema-effect-on-dropdown-options-vocab`).
- `gui-default-form-state-template-aware-seed` (gui-only — P4).
- `gui-number-widget-unset-sentinel` (gui-only — P3).

#### Files new FOLLOWUPS

- `gui-schema-effect-on-dropdown-options-vocab` (cross-repo) —
  dropdown-option-disable Effect grammar needed to close §6.6
  rows 9/10/11. Unblocked by this cycle's predicate-machinery.
- `gui-schema-cross-slot-predicate-projection` (cross-repo) —
  relational predicate types (cross-slot equality, all-distinct)
  needed to close §6.6 rows 8/13/14.

#### Verification

- `cargo test --offline` → 24 test binaries, **220** passed, 0
  failed, 1 ignored (was 187 at v0.5.1; +33 cells across this
  cycle: +8 P2/P5, +14 P3, +5 P4, +6 schema_mirror v3 deserialize).
- `MNEMONIC_BIN=<path>/v0.17/mnemonic cargo test --offline` →
  same, with the drift gate
  (`gui_schema_conditional_rules_match_hand_coded_conditionals`)
  exercising 11 rules against the v0.17.0 binary (was 10 at
  v0.5.x against v0.16.0; +1 new row 12 pin_value rule).
- `cargo clippy --all-targets --offline` → no new lints from this
  cycle (pre-existing lints in unrelated test files unchanged).
- Bumps: `Cargo.toml [package].version 0.5.1 → 0.6.0`,
  `[dependencies].mnemonic-toolkit tag mnemonic-toolkit-v0.16.0
  → mnemonic-toolkit-v0.17.0` (commit `4758168`), Cargo.lock in
  lockstep.

#### Companion

`bg002h/mnemonic-toolkit` v0.17.0 (`mnemonic-toolkit-v0.17.0`
tag, commit `4758168`). The toolkit + GUI ship in lockstep
per the cycle's tag-pair plan.

## [0.5.1] — 2026-05-16

### Changed — schema-mirror CI auto-tracks `pinned-upstream.toml`

`.github/workflows/schema-mirror.yml` adds a `parse-pinned-upstream`
pre-step that loads `pinned-upstream.toml` via Python 3.11+ stdlib
`tomllib` and exports per-CLI tag values (`mnemonic_tag`, `md_tag`,
`ms_tag`, `mk_tag`) as step outputs. The four `install-*-cli` steps
now consume those outputs via the `env:` → `$TAG` pattern (per
GitHub's hardening guidance for any `${{ }}` expression
substitution into `run:` scripts, even when the source is trusted).

The previous v0.5.0 cycle fix-commit `54865a7` was a v1 fold —
hand-bumping the hardcoded `mnemonic-toolkit-v0.14.0` literal to
`v0.16.0` after the master `schema-mirror` job failed at the new
drift gate. v0.5.1 is the v2 cleanup: future toolkit bumps in
`pinned-upstream.toml` flow automatically into CI without a
separate workflow edit. Same dynamic-tag pattern applied to md /
ms / mk install steps for symmetry, preventing the next divergence
class even though those entries are currently in lockstep.

### Closes FOLLOWUP

`schema-mirror-yml-toolkit-pin-tracks-pinned-upstream` (v2 cleanup
half; v1 fold previously landed at `54865a7`).

### Latent-bug fix (surfaced by this cycle)

`tests/schema_mirror.rs::ci_workflow_snapshot` asserted four
literal tag strings (`mnemonic-toolkit-v0.14.0`,
`descriptor-mnemonic-md-cli-v0.5.0`, `ms-cli-v0.2.1`,
`mk-cli-v0.3.1`) against the workflow body. The v0.5.0 cycle's
fix-commit `54865a7` bumped the actual mnemonic-toolkit pin from
v0.14.0 → v0.16.0 in the install-step's `--tag` literal, but left
the surrounding comment block mentioning v0.14.0. The snapshot
test's `body.contains("mnemonic-toolkit-v0.14.0")` continued to
pass as an incidental comment substring match while the real pin
had moved.

The v0.5.1 v2 cleanup removes the literal tags from the workflow
entirely, surfacing the gap. The snapshot test is refactored to
assert the v2 wiring directly: `parse-pinned-upstream` step
present, and each install step references its corresponding
`steps.pins.outputs.<cli>_tag`. The drift gate at
`tests/gui_schema_conditional_drift.rs` continues to enforce
toolkit-source-vs-GUI-mirror parity for the toolkit pin
specifically.

### Verification

- `actionlint .github/workflows/*.yml` clean (workflow + all
  sibling workflows lint with no warnings).
- Local dry-run of the `parse-pinned-upstream` step against the
  real `pinned-upstream.toml` emits the four expected tag values
  (`mnemonic-toolkit-v0.16.0`, `descriptor-mnemonic-md-cli-v0.5.0`,
  `ms-cli-v0.2.1`, `mk-cli-v0.3.1`).
- `cargo test --test schema_mirror --offline ci_workflow_snapshot`
  passes against the refactored assertions.
- Master CI green post-push (schema-mirror + build + tag-CI runs).

### No source / behavior changes

CI-only. No production Rust code touched; no API surface delta.
The Rust touch is confined to `tests/schema_mirror.rs`
(snapshot-test refactor in lockstep with the workflow surgery).

## [0.5.0] — 2026-05-16

### Added — SPEC §6.10 conditional-applicability consumer + drift gate

Consumes `mnemonic-toolkit v0.16.0`'s new
`mnemonic gui-schema` JSON v2 `conditional_rules` projection.
The toolkit emits machine-readable per-subcommand mutex/conditional
rules; the GUI maps them onto its per-frame visibility computation
and enforces parity via a drift-gate test.

#### Motivating bug closed

The GUI bundle form's default state (template `bip84`, single-sig)
previously emitted `--threshold 1 --multisig-path-family bip48` —
the CLI rejected the argv with the SPEC §6.6 byte-exact errors
`THRESHOLD_WITHOUT_MULTISIG` + `PATH_FAMILY_WITHOUT_MULTISIG`.
Three stacked defects:

1. `main.rs:203` pre-seeded `--multisig-path-family = bip87`
   unconditionally.
2. `assemble_argv` ignored the existing `Visibility` infrastructure.
3. No machine-readable conditional-applicability metadata flowed
   from toolkit to GUI; the GUI's hand-coded `conditional.rs` was
   the only source-of-truth and had drifted behind the CLI rule
   surface.

All three closed in v0.5.0.

#### Implementation surfaces (this cycle)

- **`src/schema_check.rs`** — `parse_gui_schema_conditional_rules`
  fn requiring `version >= 2`; relaxed `parse_gui_schema_json`
  version gate from `!= 1` to `< 1` (additive bump policy).
- **`src/form/conditional.rs`** — ~14 new visibility rules
  across `bundle` / `verify-bundle` / `export-wallet` /
  `derive-child`. Module-level `SINGLE_SIG_TEMPLATES` and
  `TAPROOT_INTERNAL_KEY_TEMPLATES` constants mirror the
  toolkit's `CliTemplate::is_multisig()` source-of-truth at
  `mnemonic-toolkit/src/template.rs:46-56` (parity enforced by
  the drift gate).
- **`src/form/invocation.rs`** — visibility gate at the TOP of
  the per-flag iteration loop. Both `Hidden` and `Disabled`
  suppress emission; `Required` does not (decorative marker
  only). Slot emission is exempt per SPEC §6.10 v1 scope.
- **`src/main.rs`** — removed `--multisig-path-family = bip87`
  default seed (bug-class default). A future cycle may
  re-introduce a template-aware seed; tracked at FOLLOWUP
  `gui-default-form-state-template-aware-seed`.
- **`.github/workflows/schema-mirror.yml`** — CI smoke steps
  for all four CLIs relaxed from `version == 1` to
  `version >= 1`.

#### Latent-bug fix

The visibility gate at `assemble_argv` also closes a pre-v0.5.0
latent bug: typed-then-mutex-disabled secret values (e.g., user
types `--passphrase=foo` then sets `--passphrase-stdin`) are now
suppressed at argv emission, preventing clap's `conflicts_with`
rejection downstream.

#### Drift gate

`tests/gui_schema_conditional_drift.rs` (NEW): shells out to
`<MNEMONIC_BIN> gui-schema`, parses the v2 `conditional_rules`,
synthesizes an exemplar `FormState` per rule's predicate,
invokes the corresponding hand-coded `SubcommandSchema.conditional`
fn, asserts the returned `FlagVisibility` contains the rule's
declared `(flag, visibility)`. Failure messages cite the rule's
`rationale` + `spec_ref` for forensic clarity.

Drift-gate cross-validation (§5.3): planting a divergence (e.g.,
commenting out the derive-child `--dice-sides Required` rule)
produces a failure of the form

    assertion `left == right` failed: drift in subcommand `derive-child`:
      rule rationale: --dice-sides is required when --application is set to dice.
      spec_ref: cmd/derive_child.rs clap-derive required_if_eq
      predicate: DropdownValueIn { flag: "--application", values: ["dice"] }
      target flag: --dice-sides
      expected visibility: Required
      actual visibility:   Visible

### Verification

- `cargo test --release` with all `*_BIN` env vars: 187 passed,
  0 failed, 1 ignored (the v0.4.3 baseline +30 new cells across
  P1/P2/P3/P4/P5).
- `default_bundle_form_state_cli_accepts` smoke test: passes
  (CLI exit 0 against the v0.16.0 toolkit binary; the motivating
  bug no longer reproduces).
- End-of-cycle opus reviewer-loop: R1 returned FOLD with 1
  Critical (CI workflow version gate) + 1 Important (missing
  companion FOLLOWUP entry); both folded in
  `6c2d019`. R2 returned PASS (0C / 0I).

### Companion / lockstep

- `bg002h/mnemonic-toolkit v0.16.0` (commit `519bcfc`, tag
  `mnemonic-toolkit-v0.16.0`) ships the producer side. The GUI's
  toolkit-dep pin bumps from `v0.15.0` (post-v0.4.3 catchup) to
  `v0.16.0` in this release.
- Plan + SPEC: `mnemonic-toolkit/design/
  IMPLEMENTATION_PLAN_gui_conditional_applicability_v1.md`
  + `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.

### Predecessor

- `mnemonic-gui v0.4.3` (toolkit v0.15.0 wire-format catchup;
  scope-isolated). v0.5.0 builds atop v0.4.3.

## [0.4.3] — 2026-05-16

### Scope-isolation catchup — bump toolkit dep to v0.15.0

`Cargo.toml` + `pinned-upstream.toml` pins bumped from
`mnemonic-toolkit-v0.14.2` to `mnemonic-toolkit-v0.15.0`. v0.15.0
was the toolkit's md-codec catchup release (md-codec 0.16.1 →
0.33.1 + mk-codec 0.2.1 → 0.3.0 + ms-codec git → 0.1.3); its
release commit (`5d92768`) describes the change as a wire-format
clean break (v0.14.x bundles forward-incompatible).

This release is a scope-isolated prerequisite for the in-flight
GUI conditional-applicability v1 cycle (toolkit v0.16.0 + GUI
v0.5.0 lockstep). Cutting v0.4.3 ahead of v0.5.0 ensures that any
§5.1 manual-reproduction failure in the v0.5.0 cycle is
attributable to v0.16.0 conditional-applicability work, not to
v0.15.0 wire-format drift. Architect-review rationale recorded in
the toolkit's plan-doc at
`design/IMPLEMENTATION_PLAN_gui_conditional_applicability_v1.md`
(top revision note + §4 prerequisite gate).

### Verification

- `cargo build --release` clean at v0.15.0 pin.
- Full `cargo test --release` green (156 passed, 0 failed,
  1 `#[ignore]`-gated sibling-dep test as expected) with all
  `*_BIN` env vars set (`MNEMONIC_BIN`, `MD_BIN`, `MS_BIN`,
  `MK_BIN`).
- Coldcard fixture (`tests/fixtures/coldcard_generic_bip84_mainnet.json`)
  verified **byte-identical** between v0.14.0 vendored copy and
  v0.15.0 master via `diff -q`; no re-vendor needed.

### Companion

`mnemonic-toolkit v0.15.0` (commit `5d92768`, tag
`mnemonic-toolkit-v0.15.0`) is the catchup target. No
toolkit-side change in this release.

## [0.4.2] — 2026-05-16

### Bug fix — bump toolkit dep to v0.14.2 (slip39 lib-internal mlock cfg-gate)

v0.4.1 bumped the toolkit dep to v0.14.1 expecting Windows builds
to pass. They didn't: v0.14.1's lib-cross-platform Windows CI job
caught FOUR additional `crate::mlock::*` call sites inside
`src/slip39/mod.rs` that v0.14.1's `lib.rs` cfg-gate missed.
mnemonic-gui v0.4.1 Windows CI (run 25952017502) inherited the
same failure: `error[E0433]: failed to resolve: could not find
'mlock' in the crate root` at slip39/mod.rs:159 + :314.

v0.14.2 cfg-gates the four slip39 call sites; this release just
pulls in the fix.

## [0.4.1] — 2026-05-16

### Bug fix — bump toolkit dep to v0.14.1 (Windows-compatible lib)

Cargo.toml + pinned-upstream.toml pins bumped from
`mnemonic-toolkit-v0.14.0` to `mnemonic-toolkit-v0.14.1`. v0.14.1
cfg-gates `pub mod mlock` behind `#[cfg(unix)]`, unblocking the
GUI's Windows build matrix. v0.4.0 CI run 25951528124 failed on
`x86_64-pc-windows-msvc` with `cannot find function 'sysconf' /
'mlock' / 'munlock' in crate 'libc'` — the v0.14.1 fix lands the
gate; this release just pulls it in.

### Docs / hygiene — architect-audit Important fixes

The v0.4.0 architect-audit (post-release LOCK pass) surfaced 2
Criticals (resolved at toolkit v0.14.1 + install.sh follow-up patch)
and 5 Importants. Folding the GUI-side Importants now:

- **NEW**: `tests/fixtures/SOURCE.md` — documents the provenance of
  the vendored Coldcard fixture (originally vendored from toolkit
  v0.14.0 in v0.4.0 to decouple from `MNEMONIC_GUI_UPSTREAM_ROOT`).
  Records re-vendor procedure for future toolkit cycles that revise
  Coldcard emission output.
- **DOCS**: `CHANGELOG.md` v0.4.0 section — replace the misnamed
  `CANONICAL_FALLBACK_*` reference (which exists only in v0.3.3
  historical context) with the live module path
  `v0_3_canonical_fallback::SECRET_NODE_TYPES` /
  `SECRET_SLOT_SUBKEYS`.

### Companion

`mnemonic-toolkit v0.14.1` (commit `bf54505`, tag
`mnemonic-toolkit-v0.14.1`) lands the actual Windows fix.
`scripts/install.sh` on the toolkit side will flip
`mnemonic-gui`'s `cratesio=yes` to `no` so the install path no
longer resolves `cargo install mnemonic-gui` to the leaky v0.3.1
on crates.io.

## [0.4.0] — 2026-05-16

### Structural fix — retire `build.rs` source-walker; consume `mnemonic_toolkit::secret_taxonomy`

Architect-recommended Option A from the cross-repo FOLLOWUP entries
(`secret-taxonomy-public-api-promotion` on the toolkit side,
`secret-taxonomy-public-api-consumption` here). Replaces v0.3.3's
tactical patch (committed `CANONICAL_FALLBACK_*` arrays in
`build.rs`) with the durable architectural fix.

### What changed

- **NEW**: `mnemonic-toolkit = { git = "...", tag =
  "mnemonic-toolkit-v0.14.0" }` added to `Cargo.toml` `[dependencies]`.
  Pulls in the toolkit's new `pub mod secret_taxonomy` module which
  exports `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` as compile-time
  contracts.

- **DELETED**: `build.rs` (the entire syn-based upstream-source walker).
  Was previously the source of the v0.3.0..v0.3.2 HIGH-severity bug:
  cargo-install sandboxes had no adjacent toolkit checkout, so the
  walker fell back to a `write_stub()` that emitted empty `&[]`
  arrays, silently disabling `persistence::redact_for_persistence`
  and leaking BIP-39 phrases to `state.json` in plaintext.

- **DELETED**: `tests/secrets_canonical_fallback.rs` (the v0.3.3 drift
  gate). Superseded by the toolkit's own per-variant parity tests at
  build time (`mnemonic-toolkit-v0.14.0` `secret_taxonomy_parity_tests`).

- **DELETED**: `[build-dependencies]` block in `Cargo.toml`. The
  `syn`/`quote`/`proc-macro2` crates are still in `[dev-dependencies]`
  because `tests/schema_mirror.rs` continues to re-parse upstream
  clap-derive flag surfaces for the flag-name parity gate.

- **CHANGED**: `src/secrets.rs`. The `include!(concat!(env!("OUT_DIR"),
  "/secrets_generated.rs"));` codegen line is replaced by:
  ```rust
  pub use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};
  ```
  Downstream consumers (`persistence::redact_for_persistence`,
  `secrets::slot_subkey_is_secret`, etc.) consume the toolkit-imported
  constants unchanged.

- **NEW**: compile-time supply-chain guard. A `const _: () =
  assert!(...)` block in `src/secrets.rs` asserts that the imported
  `SECRET_*` arrays equal the v0.3.3-committed snapshot (preserved as
  `v0_3_canonical_fallback::SECRET_NODE_TYPES` /
  `v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS`). Catches a
  supply-chain class of regression where the toolkit dep tag could
  resolve to a build with different `SECRET_*` arrays. Maintainers
  who deliberately bump the SECRET_* set must also update the
  snapshot (or the build fails). One-cycle belt-and-suspenders per the
  architect's recommendation; will be removed in v0.5.0.

- **NEW**: `tests/secret_taxonomy_pin.rs` — four runtime backstop
  tests asserting `SECRET_*` non-empty and contains the four
  BIP-39-class entries (`phrase`, `entropy`, `xprv`, `wif`). Runs
  under default `cargo test` with no env-var requirements; replaces
  the `secrets_canonical_fallback.rs` `#[ignore]`-gated drift gate.

- **CHANGED**: `.github/workflows/schema-mirror.yml` — the
  `cargo-test-secrets-canonical-fallback` step is retired. Backstop
  testing now runs as part of `cargo-test-full-suite`.

- **CHANGED**: `pinned-upstream.toml` — `[mnemonic].tag` is now
  documentary only. Load-bearing toolkit version pin lives in
  `Cargo.toml`'s `[dependencies]` table; both should bump in lockstep.

### Why this matters

End users running `cargo install --git mnemonic-gui` get a binary
that **correctly redacts secret slot rows and node-value composites
before persisting session state**, by compile-time guarantee. There
is no longer a code path through which the redaction filter can
silently become a no-op due to missing build-time environment setup.
The v0.3.0..v0.3.2 stub-fallback class of bug is structurally
impossible in v0.4.0+.

### User action

Re-install via:
```
./scripts/install.sh mnemonic-gui --from-git --force
```
Existing `~/.config/mnemonic-gui/state.json` files written by v0.3.3
are safe (v0.3.3 also redacted correctly). Files written by
v0.3.0..v0.3.2 may contain secret material — delete them if you used
any of those versions with `--slot @N.<phrase|entropy|xprv|wif>=…` or
`--from <phrase|entropy|xprv|wif|ms1|bip38|electrum-phrase>=…`:
```
rm -i ~/.config/mnemonic-gui/state.json
```

### Closes

`secret-taxonomy-public-api-consumption` (GUI half of the cross-repo
lockstep work). Companion: `mnemonic-toolkit` v0.14.0 (tag
`mnemonic-toolkit-v0.14.0`, commit `1a52612`) closed
`secret-taxonomy-public-api-promotion`.

### Reviewer trail

- Architect dispatch (opus): produced the Option A migration sketch
  in the cross-repo FOLLOWUP entries.
- R1 toolkit review (opus): caught a Critical (the original
  `every_*_variant` closure+driver pattern was not load-bearing —
  arm-only extension could escape parity tests).
- R2 toolkit review (opus): LOCK with-1-folded; the v0.14.0 fix was
  restructured around a declarative macro pattern that ties the
  variant array and exhaustiveness check to a single input.
- R1 GUI review (opus): caught a Critical missed deletion —
  `tests/schema_mirror.rs::source_audit` mod (250 LOC) +
  `tests/fixtures/mutated_convert.rs` were the OLDER Phase 7 audit
  re-doing the same source-walker pattern that this release's
  CHANGELOG claimed to retire, against a stale v0.13.0 toolkit
  clone. Plus five Important findings (stale comments in
  `src/persistence.rs` / `src/secrets.rs:138` / `tests/persistence.rs:303` /
  `tests/secrets.rs:241`; dead `[upstream]` table in
  `pinned-upstream.toml`; orphan `MNEMONIC_GUI_UPSTREAM_ROOT` coupling
  in `tests/runner_integration.rs` — fixture is now vendored at
  `tests/fixtures/coldcard_generic_bip84_mainnet.json`; misleading
  initial reviewer-trail framing in this CHANGELOG; lingering env
  var on `cargo-test-full-suite`). All six folded in the same commit.

## [0.3.3] — 2026-05-15

### Security fix — persistence-redaction bypass in cargo-install builds

**Severity: HIGH.** Affects v0.3.0, v0.3.1, v0.3.2.

`build.rs` generates `SECRET_NODE_TYPES` and `SECRET_SLOT_SUBKEYS` by
parsing the upstream `mnemonic-toolkit` source tree's
`NodeType::is_secret_bearing()` and
`SlotSubkey::is_secret_bearing()` impls. When the upstream source is
unresolvable at build time (Step 4 of the SPEC §B.11 resolution chain),
the prior `write_stub` emitted **empty arrays**:

```rust
pub const SECRET_NODE_TYPES: &[&str] = &[];
pub const SECRET_SLOT_SUBKEYS: &[&str] = &[];
```

This is the default outcome for **every `cargo install --git
mnemonic-gui` invocation** — `cargo install` runs `build.rs` in an
isolated sandbox with no adjacent toolkit checkout, so resolution
chain Steps 1/2/3 all fail and the stub fallback is used unless the
user explicitly sets `MNEMONIC_GUI_UPSTREAM_ROOT` or
`MNEMONIC_GUI_ALLOW_UPSTREAM_CLONE=1` before running cargo install.

`persistence::redact_for_persistence` filters slot rows by
`SECRET_SLOT_SUBKEYS.contains(...)` and `NodeValueComposite` entries
by `SECRET_NODE_TYPES.contains(...)`. With both arrays empty, the
filter is a no-op. The result: a BIP-39 seed phrase typed into a slot
field (e.g., `--slot @0.phrase=<phrase>`) or into a NodeValueComposite
field (e.g., `--from phrase=<phrase>`) **persists to `state.json` in
plaintext** when the GUI saves session state on exit.

The hand-maintained `SECRET_FLAG_NAMES` constant (`--passphrase`,
`--bip38-passphrase`, `--passphrase-stdin`) was NOT affected; only the
build-generated arrays were empty under the install-path build.

### Fix

`build.rs::write_stub` now emits the **canonical** secret-class sets
as a committed-in-source fallback, NOT empty arrays:

```rust
const CANONICAL_FALLBACK_NODE_TYPES: &[&str] = &[
    "phrase", "entropy", "xprv", "wif",
    "ms1", "bip38", "electrum-phrase",
];

const CANONICAL_FALLBACK_SLOT_SUBKEYS: &[&str] =
    &["phrase", "entropy", "xprv", "wif"];
```

These mirror the upstream `is_secret_bearing()` match-arm sets at
`mnemonic-toolkit@mnemonic-toolkit-v0.13.1`
(`crates/mnemonic-toolkit/src/cmd/convert.rs:85` +
`crates/mnemonic-toolkit/src/slot_input.rs:60`). When upstream source
IS resolvable, build.rs regenerates from source as before; when not,
the fallback ships canonical values rather than empty placeholders.

### Drift gate

New `tests/secrets_canonical_fallback.rs` re-parses the upstream
`is_secret_bearing()` impls and asserts set-equality against the
committed fallback arrays. `#[ignore]`-gated by default (needs
`MNEMONIC_GUI_UPSTREAM_ROOT`). Schema-mirror CI workflow runs it via
`cargo test --test secrets_canonical_fallback -- --include-ignored`;
any upstream change to the secret-class sets that isn't mirrored here
fails CI immediately.

A second (always-on) test asserts both fallback arrays are non-empty
and contain the four BIP-39-class items (`phrase`, `entropy`, `xprv`,
`wif`) — a backstop against future regression to `&[]`.

### Verification

Reproduction (before fix, post-`cargo install --git` build):

```
SlotRow { subkey: Phrase, value: "abandon abandon ..." }
→ save() → state.json contains "subkey": "Phrase",
                                "value": "abandon abandon ..."
```

After fix:

```
SlotRow { subkey: Phrase, value: "abandon abandon ..." } → save()
→ state.json contains "rows": []   ← phrase row stripped
```

### User action

Run `./scripts/install.sh mnemonic-gui --from-git --force` (or
re-install via your preferred path) to pick up v0.3.3. If you have an
existing `~/.config/mnemonic-gui/state.json` written by v0.3.0..v0.3.2
that may contain secret material, delete it manually before launching
v0.3.3:

```
rm -i ~/.config/mnemonic-gui/state.json
```

### CVE / GHSA

Recommend filing a GHSA advisory after release tag is live. Affected
versions: v0.3.0, v0.3.1, v0.3.2. Fixed in: v0.3.3.

## [0.3.2] — 2026-05-15

Patch: replace 4 non-ASCII glyphs in user-visible schema strings with
ASCII equivalents — matches the project's existing ASCII-first
convention (per `src/form/widget.rs:36` on the `?` button). User
reported a missing-glyph (open-square) render in the `mnemonic
final-word` subcommand dropdown; defensive sweep covers the other
three same-class chars that may also lack font support on some
systems.

Replacements:
- `—` (U+2014 EM DASH) -> `--`
- `→` (U+2192 RIGHTWARDS ARROW) -> `->`
- `↔` (U+2194 LEFT RIGHT ARROW) -> `<->`
- `≤` (U+2264 LESS-THAN OR EQUAL TO) -> `<=`

Affected user-visible strings (14 lines across 4 schema files):
- `mnemonic`: 5 help/human_name strings (final-word, slot threshold
  ×2, slot help paragraph, output-path help)
- `md`: 4 human_names (Encode/Decode/Verify/Compile)
- `ms`: 3 human_names (Encode/Decode/Verify)
- `mk`: 2 human_names (Encode/Decode)

Schema-mirror invariant: help-text content is NOT gated by the
bidirectional mirror (only flag presence/absence is), so this patch
ships independently of the toolkit. All 16 schema_mirror tests + full
suite green with proper sibling-binary env setup
(MNEMONIC_GUI_UPSTREAM_ROOT + MNEMONIC_BIN / MD_BIN / MS_BIN /
MK_BIN).

## [0.3.1] — 2026-05-15

(Backfill — v0.3.1 shipped at commit 407c5ef but its CHANGELOG section
was omitted from that release commit.)

Patch: GUI-side track of the manual-gui v1.0 cycle (lockstep with
`mnemonic-toolkit-v0.13.0` + manual-gui-v1.0.0 / v1.0.1 tags).

### Added

- `src/help/` module — `manual_url_for_subcommand` /
  `manual_url_for_dropdown` / `manual_url_for_composite` helpers, with
  `MANUAL_BASE_URL` build-time overridable via the
  `MNEMONIC_GUI_MANUAL_BASE_URL` env var (default
  `https://bg002h.github.io/mnemonic-toolkit/manual-gui/`).
- 91 `?` help-icon buttons across the form scaffolding
  (per-subcommand, per-Dropdown, per-NodeValueComposite,
  per-repeating-field) — SPEC §1.6 Option C selective placement.
- `tests/widget_help_icon.rs` kittest cell: clicks the `?` next to a
  flag, asserts the right tab/manual URL is targeted via
  `ctx.open_url`.

## [0.3.0] — 2026-05-14

v0.3 catches the GUI up to `mnemonic-toolkit-v0.13.0`. Five new
`mnemonic` subcommand surfaces (`slip39-split`, `slip39-combine`,
`seed-xor-split`, `seed-xor-combine`, `final-word`) close the
v0.11..v0.13 toolkit feature gap. A v0.10..v0.13 flag-drift
correction closes the schema-mirror-invariant breach for `bundle` /
`verify-bundle` / `convert` / `derive-child` (4 `*-stdin` flags
that shipped toolkit-side without companion GUI PRs). The
schema-mirror gate now prefers `gui-schema` JSON over `--help`
regex (required for flattened nested-subcommand names like
`seed-xor-split` that clap doesn't expose at the top level), and
`assemble_argv` now correctly emits repeating-secret flags (latent
v0.2 bug that affected `--ms1`/`--mk1`/`--md1` and was surfaced by
`--share`).

### Added — 5 new mnemonic subcommand surfaces

- `slip39-split` (8 flags) + `slip39-combine` (6 flags) — Trezor
  SLIP-39 K-of-N share splitter (toolkit v0.13.0 unblocks the
  GUI-side companion FOLLOWUP).
- `seed-xor-split` (5 flags) + `seed-xor-combine` (4 flags) —
  Coldcard all-or-nothing BIP-39 ↔ BIP-39 XOR splitter (toolkit
  v0.12.0).
- `final-word` (3 flags) — BIP-39 N-1 phrase → candidate Nth-word
  set (toolkit v0.11.0).
- 3 new schema constants (`SLIP39_FROM_NODES`, `SLIP39_TO_SHAPES`,
  `PHRASE_ONLY`).
- New `FormState::composite_node()` helper for NodeValueComposite
  value-inspect (slip39-split `--language` hide-when-entropy).
- 2 new conditional fns (`slip39_split`, `slip39_combine`).
- 5 new egui_kittest cells (one per new subcommand) +
  14 new conditional-visibility cells.

### Added — v0.10..v0.13 drift correction

Closes the schema-mirror-invariant breach (FOLLOWUPS.md
`mnemonic-gui-schema-mirror`): 4 toolkit cycles shipped new
`*-stdin` flags without companion `mnemonic-gui` PRs.

- `BUNDLE_FLAGS` / `VERIFY_BUNDLE_FLAGS` / `DERIVE_CHILD_FLAGS`:
  `--passphrase-stdin` (clap `conflicts_with = "passphrase"`).
- `CONVERT_FLAGS`: `--bip38-passphrase-stdin` (clap
  `conflicts_with = "bip38_passphrase"`).
- `src/form/conditional.rs`: `bundle` / `verify_bundle` / `convert`
  fns extended with passphrase XOR clauses; NEW `derive_child` fn
  (was `None` — gained conflict at v0.13.0); stale "no conditional
  fn needed" comment deleted.
- `derive-child` SubcommandSchema `conditional: None` →
  `Some(crate::form::conditional::derive_child)`.

### Changed — pinned-upstream + workflow

- `pinned-upstream.toml`: `[mnemonic].tag`
  `mnemonic-toolkit-v0.9.0` → `mnemonic-toolkit-v0.13.0` (sibling
  pins `md` / `ms` / `mk` unchanged).
- `.github/workflows/schema-mirror.yml`: install + clone steps
  bump to `v0.13.0`.
- `src/schema/mnemonic.rs`: `pinned_version: "mnemonic 0.9.0"` →
  `"mnemonic 0.13.0"`.
- `tests/schema_mirror.rs::ci_workflow_snapshot::required_tags`
  first entry bumped.

### Fixed — latent v0.2 bugs

- `src/form/invocation.rs::assemble_argv`: repeating-secret flags
  (`--ms1` / `--mk1` / `--md1` from v0.2; `--share` from v0.3) now
  emit N occurrences via `state.values` iteration. The pre-v0.3
  path only consulted `state.secret_widgets` (singular BTreeMap
  lookup) and silently emitted at most one token. Trade-off:
  zeroize-on-drop still applies per-widget; the values-map
  `String` copies during emission are plain heap allocations.
- `tests/schema_mirror.rs::assert_schema_matches_help`: prefer
  `gui-schema` JSON over `--help` regex (required for flattened
  nested-subcommand names like `seed-xor-split` that clap's
  top-level `--help` doesn't recognize). Falls back to `--help`
  regex for `gui-schema-capable = false` CLIs.

### Design

- `design/PLAN_v0_3.md` — 3-section reviewer-LOCKed plan
  (brainstorming + SPEC + implementation plan) with P0 drift-fold
  amendment, persisted from the plan-mode artifact directory to the
  in-repo design archive (matches v0.1 / v0.2 convention).

## [0.2.0] — 2026-05-12

v0.2 expands the GUI surface from the v0.1 baseline (one CLI's
`mnemonic` subcommand surface) to all four sibling CLIs of the
m-format constellation: `mnemonic`, `md`, `ms`, `mk` — 15 additional
subcommands across 4 binaries. The release also lands the SPEC §7
machine-readable schema contract (`<cli> gui-schema`), an OS-snapshot
occlusion baseline (Phase B.2), a secret-buffer abstraction
(Phase B.1), an egui_kittest harness scaffold (Phase A.3), and the
doubled-prefix release-artifact fix (Phase A.1).

### Added — sibling-CLI surface (Phase D)

- `ms` subcommands: `decode`, `encode`, `mnemonic`, `wordlist`
  (4 new schemas in `src/schema/ms.rs`).
- `mk` subcommands: `decode`, `encode`, `derive`, `fingerprint`
  (4 new schemas in `src/schema/mk.rs`).
- `md` subcommands: `decode`, `encode`, `compile`, `derive`,
  `address`, `policy`, `id` (7 new schemas in `src/schema/md.rs`).
- `src/form/conditional.rs` — per-subcommand conditional-visibility
  functions for `ms encode`, `mk encode`, `md encode`, `md compile`,
  `md address` (SPEC §8 enumeration-discipline conformance).

### Added — schema contract (Phase C)

- `<cli> gui-schema` JSON contract (SPEC §7). All four sibling CLIs
  now expose a `gui-schema` subcommand that walks their clap
  `Command` tree and emits a machine-readable schema
  (`{version: 1, cli, subcommands: [{name, flags, positionals}]}`).
- `src/schema_check.rs::parse_gui_schema_json` + `json_flag_names`
  — runtime consumer that shells out to `<cli> gui-schema`, parses
  the JSON, and exposes the canonical per-subcommand flag-name set
  to the schema-mirror gate. Falls back to v0.1
  regex-on-`--help` if the binary lacks `gui-schema` or exits
  non-zero.
- `.github/workflows/schema-mirror.yml` — 4 new
  `smoke-gui-schema-*` steps that validate each installed sibling
  binary emits the SPEC §7 envelope on CI before the test suite runs.

### Added — secret buffer + OS occlusion (Phase B)

- `src/form/secret_widget.rs` — `SecretLineEdit` widget backed by
  `Zeroizing<Vec<u8>>` for `--passphrase` and other secret-flag
  fields. Buffer zeroes on drop, on form reset, and on app exit.
  Excluded from `Serialize` / `Debug` derives; never persisted to
  disk via `redact_for_persistence`.
- `src/platform.rs` — first `unsafe` module in the codebase;
  cfg-gated macOS / Windows / Linux occlusion impl: macOS uses
  `NSWindowSharingType::NSWindowSharingNone` via `objc2` /
  `objc2-app-kit`, Windows uses
  `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` via
  `windows-rs`, Linux logs that no compositor API is available.
- `ctrlc` crate (windows-only) — Ctrl-C handler that runs the
  same `on_exit()` cleanup path as the unix `signal_hook::iterator`
  branch (Phase A.2).

### Added — testing infrastructure (Phase A.3 + D.4)

- `egui_kittest` (v0.31.1) dev-dependency; `accesskit` feature on
  `eframe`.
- `tests/widget_interaction.rs` — 4 cells driving real egui forms:
  slot editor (cell 1), conditional visibility (cell 2),
  `ms encode` argv assembly (cell 4), `md encode` dropdown
  value-inspection (cell 5).
- `tests/widget_secret.rs` — `cell_paste_warn_modal_trigger`
  validates the paste-warn PREDICATE (`should_warn_on_paste`) +
  modal-text constant. (The live paste→modal wiring was DEAD until
  v0.40.0, which adds `tests/paste_warn_wiring_v0_40_0.rs` for the
  live check; see that entry.)
- `tests/dropdown_id_salt.rs` — source-audit regression backstop
  for the v0.1.2 ComboBox ID-collision hotfix.

### Fixed

- `.github/workflows/build.yml` — `compute-version` step strips
  the `mnemonic-gui-` prefix from `github.ref_name` into a
  `VERSION` env var; 4 artifact-name template sites now reference
  `env.VERSION` instead of `env.REF_NAME`. Pre-fix this produced
  doubled-prefix artifacts like
  `mnemonic-gui-mnemonic-gui-v0.1.0-x86_64-linux.tar.gz`.

### Pinned

- `pinned-upstream.toml` — bumped all four sibling tags in
  lockstep with the Phase C.2 PR merges:
  `mnemonic-toolkit-v0.9.0`, `descriptor-mnemonic-md-cli-v0.5.0`,
  `ms-cli-v0.2.0`, `mk-cli-v0.3.0`. All four
  `gui-schema-capable = true` (Phase C.3).

### Internal

- `FormState` lost its `Clone` derive (the new `secret_widgets:
  BTreeMap<String, SecretLineEdit>` field is intentionally
  non-cloneable to prevent accidental secret duplication).
- `PersistedState` lost its `Clone` derive for the same reason.
- `Schema::pinned_version` strings bumped per CLI to match the
  bumped sibling-binary `--version` output.
- 122 tests pass across 15 binaries at the v0.2 release commit
  (vs. ~65 at v0.1.2).

## [0.1.2] — 2026-05-12

Dropdown-bug hotfix. The three `egui::ComboBox` instances in
`src/form/widget.rs` (the `FlagKind::Dropdown` selector, the
`NodeValueComposite` node selector, and the `TaggedOrIndexed` tag
selector) all used `ComboBox::from_label("")` or `from_label(" ")`.
`from_label(label)` derives the egui widget ID from `label`; egui then
keys popup open-state, hover-state, and selection-state by that ID.
With multiple ComboBoxes on the same page all using `""` (or `" "`)
they shared an ID, so:

- The popup failed to open ("no list opens" — egui couldn't
  disambiguate which popup state to drive), and
- Hover and selection state leaked across every ComboBox on the page
  ("every list on the page gets highlighted when one list is clicked
  on" — they all shared interaction state via the shared ID).

v0.1.2 switches each ComboBox to `ComboBox::from_id_salt((const,
flag.name))`, where `flag.name: &'static str` is unique per flag. This
matches the convention already used by `src/form/slot_editor.rs:160`
(`from_id_salt(("slot_subkey", i))`).

### Fixed

- `src/form/widget.rs:26` — `Dropdown` selector now
  `from_id_salt(("flag_dropdown", flag.name))`.
- `src/form/widget.rs:60` — `NodeValueComposite` node selector now
  `from_id_salt(("flag_nodevalue", flag.name))`.
- `src/form/widget.rs:84` — `TaggedOrIndexed` tag selector now
  `from_id_salt(("flag_tagged", flag.name))`.

### Added

- `tests/dropdown_id_salt.rs` — source-audit regression that fails if
  any future edit reintroduces `ComboBox::from_label` in `widget.rs` or
  removes the `from_id_salt` invariant. Pattern follows the existing
  Phase 7 source-audit tests.

### Unchanged

- `src/main.rs:291` uses `ComboBox::from_label("subcommand")` — the
  literal `"subcommand"` label is unique and there is only one such
  ComboBox in the application, so no ID collision. Not affected by the
  bug; not touched by the fix (scope discipline).
- `src/form/slot_editor.rs:160` was already correct
  (`from_id_salt(("slot_subkey", i))`).

## [0.1.1] — 2026-05-12

First functional GUI release. v0.1.0 shipped the full architecture
(schema, form widgets, slot editor, runner, secrets, persistence, CI
gates) but its eframe loop was wired against the `egui_glow` renderer,
which is broken on KDE/KWin Wayland: after the initial 1–2 paint
cycles, the wayland event loop went stuck, ignoring cross-thread
`request_repaint()` and `send_viewport_cmd(Close)`, ignoring KWin's
`xdg_toplevel.close` events, and never reaching `on_exit()` for clean
shutdown. KDE marked the window "Not Responding" in its title bar.

v0.1.1 swaps `egui_glow` → `egui_wgpu` (Vulkan via Mesa). With the
wgpu renderer, every cross-thread wake mechanism works: `update()`
runs at the 1 Hz keepalive cadence (CPU still ~0 % at idle), KWin
sees regular surface commits (no "Not Responding" label), SIGINT /
SIGTERM route through `ViewportCommand::Close` to `on_exit()` in
~2.5 s. Real user clicks would have failed under v0.1.0; they work
under v0.1.1.

### Changed

- `eframe = "0.31"` with `default-features = false` and explicit
  `features = ["wgpu", "default_fonts", "wayland", "x11"]` (was
  `eframe = "0.29"` with default glow renderer).
- `egui = "0.31"` (was `egui = "0.29"`).

### Added

- `signal-hook = "0.3"` dependency.
- SIGINT / SIGTERM handler thread that routes through
  `ViewportCommand::Close` for graceful shutdown (zeroize sweep +
  `on_exit()`), with `process::exit(130)` fallback after 3 s if the
  event loop is unresponsive.
- 1 Hz `wayland-keepalive` background thread keeping the surface
  alive for compositor liveness heuristics.
- `init_tracing` filter suppresses `egui_wgpu` / `wgpu_hal`
  swap-chain timeout warnings at default WARN level; visible under
  `--debug` / `RUST_LOG=info`.
- 3 demo screenshots in `screenshots/` against the working wgpu
  build (replacing the v0.1.0 captures that showed the
  frozen-at-first-paint "Not Responding" GUI).

### Fixed

- The event-loop-stuck bug described above (see
  `FOLLOWUPS.md` → Resolved → `gui-glow-wayland-loop-broken`).
- `on_exit()` signature updated to match the wgpu integration's
  `fn(&mut Self)` (was `fn(&mut Self, Option<&glow::Context>)`).

### Removed

- Diagnostic instrumentation added during v0.1.1 dev (TICK label,
  HEARTBEAT dialog, update() counter, keepalive tracing log) — kept
  only the working production code paths.
- Unused `with_position` viewport hint (Wayland ignores absolute
  window positioning by protocol design).

## [0.1.0] — 2026-05-12

First release. Cross-platform GUI overlay for the m-format constellation
CLIs (`mnemonic`, `md`, `ms`, `mk`), built with `egui` in Rust.

### Phases shipped

The build followed a 10-phase converged plan
(`/home/bcg/.claude/plans/declarative-tumbling-shell.md`, 3 sections —
Brainstorm + SPEC + IMPL_PLAN — all 0C/0I after iterative architect
review). Per-phase agent-report artifacts under `design/agent-reports/`.

- **Phase 0** — Repo scaffolding + dependency pin.
- **Phase 1** — Schema types + `mnemonic` schema (5 subcommands).
- **Phase 2** — Form widget renderer + argv assembler +
  CommandLineToArgvW-compatible copy-command shell-quoting.
- **Phase 3** — SlotEditor composite widget for the
  `--slot @N.<subkey>=<value>` repeating grammar.
- **Phase 4** — Subprocess runner (deadlock-safe via
  `wait_with_output`) + tracing init + integration test.
- **Phase 5** — Conditional visibility engine (12 clap-level and
  runtime constraints from upstream `mnemonic-toolkit-v0.8.1`).
- **Phase 6** — Sibling CLI schemas (`md`, `ms`, `mk`) + path-detect
  data layer.
- **Phase 7** — Secret-handling modals + `Zeroize` + `build.rs` codegen
  of `SECRET_NODE_TYPES` / `SECRET_SLOT_SUBKEYS` from upstream
  `is_secret_bearing()` impls via `syn::parse_file` + mutation-
  detection regression fixture.
- **Phase 8** — `state.json` persistence + never-persist redaction +
  schema-version migration.
- **Phase 9** — Schema-mirror CI workflow + cross-platform build
  matrix (5 targets) + 4 sibling-repo `FOLLOWUPS.md` PRs activating
  the mirror-invariant.
- **Phase 10** — This release roll-up.

### Test corpus

93 integration tests across 9 binaries:

| Binary | Cells |
|--------|-------|
| `argv_assembler` | 10 |
| `argv_assembler_slot` | 5 |
| `conditional_visibility` | 13 |
| `copy_command` | 15 |
| `path_detect` | 9 |
| `persistence` | 11 |
| `runner_integration` | 3 |
| `schema_mirror` | 9 (incl. CI workflow snapshot) |
| `secrets` | 18 |

### Pinned upstream tags

| CLI | Tag | --version output (runtime soft-check) |
|-----|-----|----------------------------------------|
| `mnemonic` | `mnemonic-toolkit-v0.8.1` | `mnemonic 0.8.0` |
| `md` | `descriptor-mnemonic-md-cli-v0.4.3` | `md 0.4.3` |
| `ms` | `ms-cli-v0.1.0` | `ms 0.1.0` |
| `mk` | `mk-cli-v0.2.0` | `mk 0.2.0` |

### v0.1 scope

5 subcommands of `mnemonic` (bundle / verify-bundle / convert /
export-wallet / derive-child) plus 1 subcommand each for the three
siblings (md / ms / mk inspect). See SPEC §A coverage table for the
15 subcommands deferred to v0.2.

### Explicit non-mitigations (v0.2 deferrals per SPEC §14)

- Code-signing (Mac Developer ID, Windows Authenticode): unsigned in
  v0.1; see `docs/onboarding/macos-gatekeeper-walkthrough.md` and
  `docs/onboarding/windows-smartscreen-walkthrough.md` for
  first-launch workarounds.
- Installer packages (AppImage, Flatpak, .dmg, .msi): tarball/zip only.
- Package-manager taps (Homebrew, winget): cargo install + release
  binaries only.
- Custom `SecretLineEdit` with `Zeroizing<Vec<u8>>`: best-effort
  `String`-level zeroize only. FOLLOWUPS
  `gui-secret-buffer-allocator-residue`.
- OS-snapshot occlusion (`NSWindowSharingNone` /
  `WDA_EXCLUDEFROMCAPTURE`): not yet mitigated; paste-warn modal
  copy mentions this. FOLLOWUPS `gui-os-snapshot-secret-occlusion`.
- Headless egui test harness: widget rendering paths unexercised by
  tests in v0.1. FOLLOWUPS `gui-headless-test-harness-evaluation`.

### Cross-repo mirror-invariant

Activated via 4 sibling-repo FOLLOWUPS PRs merged 2026-05-12:

- bg002h/mnemonic-toolkit#13 (tag `mnemonic-toolkit-v0.8.1`)
- bg002h/descriptor-mnemonic#28 (tag `descriptor-mnemonic-md-cli-v0.4.3`)
- bg002h/mnemonic-secret#4 (tag `ms-cli-v0.1.0`)
- bg002h/mnemonic-key#7 (tag `mk-cli-v0.2.0`)

Any flag add/remove/rename or new `conflicts_with` /
`required_unless_present_any` in those upstream CLIs lands in lockstep
with a companion `mnemonic-gui` PR that bumps the schema +
`pinned-upstream.toml` tag for that CLI.
