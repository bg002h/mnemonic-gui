# mnemonic-gui FOLLOWUPS

Cross-repo coordination items + deferred v0.2 work. Per the constellation's
mirror-invariant discipline, every entry that affects a sibling repo carries
a `Companion:` cross-cite, and the corresponding entry in the sibling repo
mirrors it.

## Active

### `audit-2026-06-10-backlog` — verified findings from the first independent Fable constellation audit

- **Surfaced:** 2026-06-10, the 23-agent read-only architecture audit (find → adversarial-verify → synthesize). 48 verified findings constellation-wide (0 critical); this repo's share below. **Full report + per-finding detail (claim/evidence/fix/disposition):** `../../mnemonic-toolkit/design/agent-reports/constellation-architecture-audit-2026-06-10.md` (committed in the toolkit repo). Promote any line to its own `### <id>` entry when worked; resolve here as fixed.
- **This repo's verified findings (14):**
  - **[IMPORTANT]** `gui-actions-v4-node20-deprecation` — Every JS action in both gui workflows is still on @v4 (checkout x3, upload-artifact, download-artifact) while the toolkit deliberately bumped all its JS-action sites @v4->@v5 for the GitHub Node-20 de (`.github/workflows/build.yml:19,56,112,125; .github/workflows/schema-mirror.yml:15`) [RESOLVED this session — audit I7/I8]
  - **[IMPORTANT]** `gui-tag-skips-all-gates` — On a mnemonic-gui-v* tag push only build.yml fires (clippy + cargo build --release + artifact upload + GitHub release) — it runs NO cargo test. The whole test suite (schema_mirror, archetype_schema_mi (`.github/workflows/build.yml:3-8, .github/workflows/schema-mirror.yml:3-7`) [RESOLVED this session — audit I7/I8]
  - **[IMPORTANT/I5] ✓ RESOLVED (v0.34.0, 2026-06-10)** `positionals-never-redacted` — PositionalArgSchema.secret + SecretLineEdit routing + unconditional drop-all-positionals belt; see `positional-secrets-not-redacted-at-persist`.
  - **[IMPORTANT/I4] ✓ RESOLVED (v0.33.0, 2026-06-10)** `secret-false-flags-render-cleartext-no-confirm` — toolkit v0.53.1 fixed the gui-schema source of truth; GUI v0.33.0 flipped the 9 mnemonic.rs sites + the ms.rs:321 deliberate override. See `xpub-search-inline-phrase-not-secret-classified` + `ms-repair-ms1-not-secret-classified`.
  - **[IMPORTANT/I6] ✓ RESOLVED (v0.34.0, 2026-06-10)** `tree-wif-hex-privkey-in-key-fields-unredacted` — promoted below; persist walk flipped to the positive extended-public allowlist.
  - **[minor]** `conditional-drift-gate-stale-binary-doc-lie` — The module doc (:13) states the gate is 'Skipped (returns early) when MNEMONIC_BIN is unset', but mnemonic_bin() always returns Some (PATH fallback to "mnemonic"), so a stale/unpinned `mnemonic` on PA (`tests/gui_schema_conditional_drift.rs:13,28-33,194-210`)
  - **[minor] ✓ RESOLVED (v0.40.0, 2026-06-11)** `paste-warn-live-wiring-untested` — `tests/paste_warn_wiring_v0_40_0.rs` adds the live kittest check (T-B2: inject `egui::Event::Paste` ≥/< threshold into a focused `SecretLineEdit`, assert the `paste_warn_id()` bus flag set/not-set; RED-proven). SPEC `design/SPEC_gui_v0_40_0_paste_warn_wiring.md`.
  - **[minor] ✓ RESOLVED (v0.40.0, 2026-06-11)** `paste-warn-modal-dead-code` — `SecretLineEdit::show` now detects an over-threshold `Event::Paste` into the focused field and raises a ctx-data bus flag (`secret_widget::paste_warn_id()`); `update()` `remove_temp`s it (read+clear) and renders the informational `PASTE_WARN_MODAL_TEXT` modal. The bus is the single chokepoint (no per-call-site wiring). SPEC + R0 ×2.
  - **[minor]** `secret-drift-gate-version-skip-silent` — fetch_v5_schema() returns None (→ silent PASS) when the binary can't spawn, exits non-zero, JSON won't parse, OR version<5. The version<5 branch means a regressed/pre-v5 pinned binary would silently d (`tests/schema_mirror_secret_drift.rs:61-91`)
  - **[minor] ✓ RESOLVED (v0.38.0, 2026-06-10)** `slot-secret-values-rendered-unmasked` — secret-bearing slot values now render `.password(true)` (gated on `is_secret_bearing()`); row removal zeroizes a secret value first (`SlotRow::zeroize_if_secret` + `remove_row`). Split-brain pin (T3): `is_secret_bearing()` == the persistence `SECRET_SLOT_SUBKEYS` gate for all 10 variants. Persistence was already safe. SPEC `design/SPEC_gui_v0_38_0_slot_secret_mask.md`. (paste-warn on slot edits deferred to the Item-1/3 cycle.)
  - **[obs]** `conditional-cells-lookup-not-live-form` — Every cell calls run_conditional(name,state) → the pure conditional fn → and asserts the returned (flag,Visibility) map; none drives the live form to confirm the rendered widget is actually hidden/dis (`tests/conditional_visibility.rs:36-73 (helper + all cells)`)
  - **[obs]** `json-envelope-wire-shape-ungated-stale-fixtures` — The runtime --json envelopes (bundle, import-wallet, export-wallet, xpub-search) remain ungated for wire-shape (schema_mirror is flag-name only). The new spec-schema surface IS now wire-shape-gated (a (`tests/cli_envelope_smoke.rs:1-59`)
  - **[obs] ✓ RESOLVED (v0.35.0, 2026-06-10)** `persistence-unwired-redaction-never-runs` — Phase-8 persistence WIRED: main() loads before run_native, on_exit saves (borrow-side redact, save-then-zeroize), per-frame Some-guarded geometry snapshot. See SPEC_gui_v0_35_0_persistence_wiring.md.
  - **[obs] ✓ RESOLVED (v0.39.0, 2026-06-11)** `run-confirm-and-preview-show-secrets-cleartext` — a parallel secret-mask (`assemble_argv_with_secret_mask`) now drives a masked display: the `Preview:` label, the run-confirm modal token list, and the last-run `argv:` line all render secret value tokens as `••••` (`render_copy_command_masked`), covering all four secret sources (Text/slot/positional/value-dependent composite). The copy buttons still copy the REAL command (deliberate reveal, relabeled). User decision (d). T-A3 pins `mask.any() ⟹ should_confirm_run`. SPEC `design/SPEC_gui_v0_39_0_mask_preview_paste_warn.md`. (Item 3 / paste-warn wiring is the separate v0.40.0 cycle.)
- **Status:** open (backlog index; individual items dispositioned in the report).
- **Tier:** audit-backlog.

### `tree-mode-posix-pipeline-spec-json-unmasked` — the build-descriptor tree-mode POSIX copy embeds the spec JSON unmasked

- **Surfaced:** 2026-06-11, v0.39.0 (Item 1 masking) cycle — deliberately scoped OUT.
- **Where:** `src/main.rs` tree-mode `posix_pipeline` (`tree_form::posix_pipeline_command(state, &argv)`); the copy embeds `spec_json_pretty(state)`.
- **What:** the v0.39.0 secret-mask covers the argv tokens but NOT the tree-mode spec JSON, which is copied/displayed via a different mechanism (JSON-string redaction, not the argv mask).
- **Why it is currently safe (verified R0-r2):** build-descriptor's policy tree is over XPUBS (watch-only) — none of its node kinds (`pk`/`pkh`/`multi`/`sortedmulti`/`older`/`after`/hash/child-ref) are in `SECRET_NODE_TYPES`. So there is no secret to leak TODAY. This is a defense-in-depth gap, not a live leak.
- **Fix when needed:** if a future build-descriptor node ever carries a secret-class value, add a JSON-redaction pass over the pipeline spec mirroring the argv mask. Re-verify the watch-only assumption at that time.
- **Tier:** deferred / conditional (no live leak).

### `gui-last-run-result-argv-stdout-not-zeroized` — `app.last_run: RunResult` holds the cleartext assembled argv + stdout/stderr, never scrubbed in RAM

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (sweep candidate #1). Source SHA `origin/master` @ `1999323` (v0.46.0). Report: `mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-gui.md`.
- **Secret type / gap class:** assembled-argv secret-value tokens (seed phrase / passphrase / xprv / WIF / minikey / ms1 slot value) + captured stdout/stderr — **in-RAM model residue (app holder outside `FormState`) + bare `Vec` with no zeroize-on-drop**.
- **Where:** `src/main.rs:104` (`last_run: Option<runner::RunResult>`); `src/runner.rs:18-31` (`RunResult { argv: Vec<String>, mask: Vec<bool>, exit_code, stdout: Vec<u8>, stderr: Vec<u8> }`); stored at `src/main.rs:1222` (`app.last_run = Some(result)`).
- **What:** after a secret-bearing Run, the FULL assembled argv lives in `app.last_run` with the secret value tokens in **cleartext** (`--passphrase <seed>`, `--from phrase=<seed>`, `@N.phrase=<seed>`), plus `stdout`/`stderr` bytes. `RunResult` is a bare struct (no `Zeroize`/`Drop`). The exit sweep (`on_exit` → `secrets::zeroize_form_state`) walks only `self.form_state`; it never touches `last_run`, so these bytes are never scrubbed in RAM at exit. The `mask` field governs DISPLAY only — the underlying `argv` bytes are cleartext.
- **Fix direction (NOT a plan):** bring the app-level run holder into the exit sweep + scrub-on-replace — `impl Zeroize`/`Drop` for `RunResult` (or zeroize the `argv`/`stdout`/`stderr` before reassigning `app.last_run`) and visit it from the on-exit path alongside `form_state`. In-RAM only; never serialized (`RunResult` is not `Serialize`).
- **Severity:** Med (in-RAM only; no disk/clipboard/log persistence). Same residue class as M9 but for an app holder M9 didn't reach.
- **Note:** M9 (`TreeNode::zeroize_keys`, v0.46.0) + the on-disk redactor are COMPLETE within their declared scope; this gap is in a surface NEITHER reaches — an app-level run holder that sits OUTSIDE `FormState` and is structurally invisible to `zeroize_form_state`.
- **Status:** **resolved** `mnemonic-gui-v0.47.0` (2026-06-21, cycle-15 Lane G, P1). `RunResult` gains `impl Zeroize + Drop` (whole-holder scrub: argv/mask/stdout/stderr; `exit_code` left, non-secret); `on_exit` scrubs it in the post-save sweep via the new pure `secrets::scrub_app_run_holders` seam (the holder sits outside `FormState`, so the `form_state` sweep never reached it). In-RAM only, never serialized. Tests: `tests/run_holder_zeroize.rs` (T1/T4). Per-phase R0 GREEN + whole-diff review.
- **Tier:** GUI-local (in-RAM secret residue).
- **Companion:** part of the constellation-wide "secret residue in surfaces outside the primary zeroize sweep" pattern from the 2026-06-21 secret-keymat sweep — siblings in `mnemonic-toolkit` (derived-output Strings) + `mnemonic-secret` (codec-internal). See the toolkit `design/agent-reports/sweep-keymat-*.md`.

### `gui-pending-confirm-argv-not-zeroized` — `pending_confirm_argv` holds the real cleartext argv + tree stdin while the confirm modal is open, cleared by non-scrubbing `= None`

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (sweep candidate #2). Source SHA `origin/master` @ `1999323` (v0.46.0). Report: `mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-gui.md`.
- **Secret type / gap class:** the real assembled argv (secret value tokens, cleartext) + `spec_stdin: Option<Vec<u8>>` (tree-mode `--spec -` bytes) held while the run-confirm modal is open — **in-RAM model residue (app holder outside `FormState`) + bare `Vec` with no zeroize-on-drop**.
- **Where:** `src/main.rs:114` (`pending_confirm_argv: Option<PendingConfirm>`); alias `src/main.rs:92` (`type PendingConfirm = (Vec<String>, Vec<bool>, Option<Vec<u8>>)`); set at `src/main.rs:1045`; cleared by `= None` at `:1092` / `:1096` (plain drop, no zeroize).
- **What:** the run-confirm modal exists BECAUSE a secret-bearing argv is pending; that pending argv (and any tree `spec_stdin`) holds the cleartext secret in app state until Run/Cancel. On dismiss it is dropped via `= None` with no zeroize; it is also never visited by the exit sweep (not in `FormState`). Same structural blind spot as `gui-last-run-result-argv-stdout-not-zeroized`.
- **Fix direction (NOT a plan):** scrub on clear (zeroize the `Vec<String>` argv + `Vec<u8>` stdin before setting `= None`) AND visit this app-level holder from the on-exit sweep so an exit during a pending modal does not leave it un-swept. Single combined "sweep the app-level secret holders on exit + scrub-on-clear" cycle covering this and `gui-last-run-result-argv-stdout-not-zeroized` is natural.
- **Severity:** Med (in-RAM only; bounded modal-lifetime residence, but the drop is non-scrubbing and exit during a pending modal leaves it un-swept).
- **Note:** M9 (`TreeNode::zeroize_keys`, v0.46.0) + the on-disk redactor are COMPLETE within their declared scope; this gap is in a surface NEITHER reaches — the other app-level holder of the real (unmasked) argv outside `FormState`.
- **Status:** **resolved** `mnemonic-gui-v0.47.0` (2026-06-21, cycle-15 Lane G, P1). `PendingConfirm` promoted from a bare tuple to a `Zeroize + Drop` struct in the `runner` lib module; the run-confirm modal consume site is rewritten E0509-safe (bind the whole struct, read by-ref, pass owned clones on Run) so the per-frame clone drops scrubbed; `on_exit` scrubs the holder via `secrets::scrub_app_run_holders`. Tests: `tests/run_holder_zeroize.rs` (T3/T4). Per-phase R0 GREEN + whole-diff review.
- **Tier:** GUI-local (in-RAM secret residue).
- **Companion:** part of the constellation-wide "secret residue in surfaces outside the primary zeroize sweep" pattern from the 2026-06-21 secret-keymat sweep — siblings in `mnemonic-toolkit` (derived-output Strings) + `mnemonic-secret` (codec-internal). See the toolkit `design/agent-reports/sweep-keymat-*.md`.

### `gui-composite-secret-value-rendered-cleartext-onscreen` — the `NodeValueComposite` value field is a plain (non-password) text edit even for secret nodes

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (sweep candidate #3). Source SHA `origin/master` @ `1999323` (v0.46.0). Report: `mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-gui.md`.
- **Secret type / gap class:** `NodeValueComposite` value for a secret node (`phrase`/`entropy`/`xprv`/`wif`/`minikey`/`bip38`/`electrum-phrase`/`seedqr`) — **on-screen widget renders the secret unmasked (on-screen reveal)**.
- **Where:** `src/form/widget.rs:653` — `let response = ui.text_edit_singleline(value);` in the `NodeValueComposite` arm (the in-source comment at `:649-651` notes it remains a plain non-password text edit).
- **What:** `convert --from phrase=<seed>` / `--from xprv=<key>` / `--from minikey=<key>` etc. render the typed/pasted secret in cleartext on screen (no `.password(true)`). v0.45.0 (H3) and v0.39.0 added paste-warn + Preview-label masking + persist-redaction + run-confirm for these nodes — but the widget itself still shows the secret. Contrast: secret Text flags route to `SecretLineEdit` (`.password(true)`), and secret slot values are `.password(true)` since v0.38.0. The composite value field is the lone secret-input widget NOT `.password(true)`.
- **Fix direction (NOT a plan):** render the composite value widget `.password(true)` when the selected node is argv-secret-class (`secrets::node_type_is_argv_secret(node)`), mirroring the slot v0.38.0 treatment. On-screen only.
- **Severity:** Low/Med (on-screen reveal only — not persisted (redacted), not logged, masked in Preview/confirm; exposure is shoulder-surf / OS-screenshot, Linux unmitigated per `gui-os-snapshot-secret-occlusion`).
- **Note:** the resolved `composite-paste-warn-parity` note claims the v0.39.0 masking covers the DISPLAY of these composites — but that masking is only the Preview LABEL, not the live `text_edit_singleline`. This is the on-screen-widget facet that note left open.
- **Status:** **resolved** `mnemonic-gui-v0.47.0` (2026-06-21, cycle-15 Lane G, P2). The `NodeValueComposite` value field renders `.password(is_secret_node)` gated on a SINGLE hoisted `secrets::node_type_is_argv_secret(node)` boolean that also drives the existing paste-warn condition (no double-eval / split-brain). Non-secret nodes (xpub/path/number) stay readable. The `ui.add(TextEdit::singleline(..).password(..))` form preserves the paste-warn `Response`. Tests: `tests/widget_secret_mask_cycle15g.rs` (T5/T5b/T6). Per-phase R0 GREEN + whole-diff review.
- **Tier:** GUI-local (on-screen widget cleartext).

### `gui-tree-key-field-rendered-cleartext-onscreen` — tree `key`/`keys[i]` build-descriptor fields render a mis-pasted xprv/WIF cleartext on screen (watch-only, gate-refused edge case)

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (sweep candidate #4). Source SHA `origin/master` @ `1999323` (v0.46.0). Report: `mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-gui.md`.
- **Secret type / gap class:** xprv/WIF/raw-hex private key mis-typed into a `TreeNode.key`/`.keys[i]` build-descriptor field — **on-screen widget renders the secret unmasked (on-screen reveal); non-canonical / gate-refused input**.
- **Where:** `src/form/tree_form.rs:697` (`ui.text_edit_singleline(&mut node.key)`) and `:717` (`ui.text_edit_singleline(&mut node.keys[i])`) — both plain (non-password). `xprv_hint` (`:785`) shows an amber "looks like an extended PRIVATE key … will not be saved and the gate will refuse it" warning but does not mask.
- **What:** the build-descriptor tree is watch-only by design (keys are XPUBs); the gate refuses private keys and persist blanks them (M9 + redactor). But if a user pastes an xprv anyway it renders cleartext on screen until exit. M9 zeroizes it from RAM on exit and the redactor blanks it on persist — but the live widget shows it.
- **Fix direction (NOT a plan):** render the tree key widgets `.password(true)` when the field is xprv-like (`is_xprv_like`), bundling with `gui-composite-secret-value-rendered-cleartext-onscreen`. On-screen only; arguably WONTFIX given the amber hint + watch-only contract.
- **Severity:** Low (on-screen only; non-canonical input — watch-only builder, gate-refused, persist-blanked, RAM-zeroized). Lowest of the four sweep candidates; filed for completeness / dedup.
- **Note:** distinct from `gui-tree-key-egui-undo-ring-residue` (that is the egui undo-ring residue after model zeroize; this is the live-widget cleartext render). M9 (`TreeNode::zeroize_keys`, v0.46.0) + the on-disk redactor are COMPLETE within their scope; this is the on-screen-widget surface neither reaches.
- **Status:** **resolved** `mnemonic-gui-v0.47.0` (2026-06-21, cycle-15 Lane G, P2). The build-descriptor tree `key` / `keys[i]` fields render `.password(is_xprv_like(..))` — the same predicate the amber `xprv_hint` keys off, so a mis-pasted xprv-shaped private key masks while watch-only xpubs stay readable (the input-then-mask flicker is the ratified-acceptable edge). Tests: `tests/widget_secret_mask_cycle15g.rs` (T7, Key + KeyQuorum). Per-phase R0 GREEN + whole-diff review.
- **Tier:** GUI-local (on-screen widget cleartext; watch-only edge case).

### `composite-paste-warn-parity` — paste-warn (v0.40.0) wires into SecretLineEdit but not the NodeValueComposite value field

- **Surfaced:** 2026-06-11, v0.39.0 cycle (round-1 R0 M1), carried forward.
- **Where:** Item 3 (paste-warn wiring, v0.40.0) threads through `SecretLineEdit::show`; `NodeValueComposite` values (e.g. `--from phrase=<seed>`) are typed in a different widget.
- **What:** a large paste of a seed into a composite VALUE field whose node is secret-classed does not trigger the paste-warn modal (v0.40.0 keys the warn on `SecretLineEdit`, and composites don't use it).
- **Fix:** extend the paste-detection to the composite value widget (it could set the same `secret_widget::paste_warn_id()` flag with the same `changed() + Event::Paste` check) OR keep documented. The v0.39.0 masking already covers the DISPLAY of these composites; this is only the paste-warn affordance.
- **Status:** **resolved** `mnemonic-gui-v0.45.0` (2026-06-21, constellation bug-hunt cycle-3 H3 fold). Wired node-aware paste-detection into the `NodeValueComposite` value widget (`src/form/widget.rs`): on `response.changed()` with `Event::Paste ≥ PASTE_WARN_THRESHOLD` AND `secrets::node_type_is_argv_secret(node)`, raise the same `secret_widget::paste_warn_id()` bus flag the existing modal already consumes (single chokepoint, no new modal wiring). Fires for every secret-class composite node's value (all 9 composites, incl. the cycle-3-newly-classified `minikey`); non-secret nodes (xpub/path/mk1/fingerprint/address) do not over-warn.
- **Tier:** RESOLVED (was: deferred after v0.40.0).

### `slot-field-paste-warn-uncovered` — paste-warn (v0.40.0) does not cover secret SLOT value fields

- **Surfaced:** 2026-06-11, v0.40.0 cycle (SPEC non-goal + R0).
- **Where:** v0.38.0 renders secret slot values via a raw `egui::TextEdit::singleline(&mut row.value).password(true)` in `src/form/slot_editor.rs`, NOT `SecretLineEdit`.
- **What:** a large paste into a `@N.phrase=`/`@N.ms1=` slot box does not fire the paste-warn modal (the v0.40.0 detection lives in `SecretLineEdit::show`, which the slot editor does not use). Sibling of `composite-paste-warn-parity` — both are "secret input not via SecretLineEdit."
- **Fix (cheap, by design):** the slot render can set the same `secret_widget::paste_warn_id()` bus flag with the same `response.changed() + Event::Paste ≥ PASTE_WARN_THRESHOLD` check on `is_secret_bearing()` rows; `update()`'s read-once already covers it. ~3 lines.
- **Tier:** deferred (low-risk; the slot value is already masked + zeroized since v0.38.0; this is only the paste affordance). **Considered + deferred again in cycle-3 (2026-06-21):** explicitly out of scope for the H3 funds-leak fix — `minikey` is `convert --from`-only and never reaches a slot (`SlotSubkey` has no minikey), so the slot gap carries no minikey funds-leak. Left open as the cheap sibling to `composite-paste-warn-parity`.

### `runner-tracing-test-flaky-under-parallel-load` — cell_2_tracing_init_logs_subprocess_spawn intermittently misses the exit event

- **Surfaced:** 2026-06-10, v0.31.1 impl review (observed once under full-suite load; passed 5/5 isolated + on rerun; the file was untouched by the cycle).
- **Where:** `tests/runner_integration.rs:140-168` — thread-local `tracing::subscriber::set_default` under parallel test threads (callsite-interest race class); the captured output contained `subprocess spawn` but not `subprocess exit 0` (`runner.rs:108`).
- **What:** serialize the cell (or use a dedicated subscriber guard) so CI can't intermittently red a green push.
- **Status:** **resolved** `mnemonic-gui-v0.32.0` (2026-06-10, impl-review M5 fold). No new dep (`serial_test` is not in the tree and one cell doesn't justify it). Two-layered no-dep fix in the cell itself: (1) `tracing::callsite::rebuild_interest_cache()` immediately after `set_default` flushes the stale GLOBAL interest decisions made under other tests' subscribers (the race mechanism: interest is cached per-callsite globally while `set_default` is thread-local, so a parallel thread's subscriber churn can transiently mark this cell's DEBUG callsites uninterested); (2) the spawn+capture loop retries up to 3 attempts, each with a fresh subscriber + rebuild, asserting only on the final capture — a transient race converges under retry. Verified 10/10 consecutive runs of the `runner_integration` binary green (its 3 cells on parallel threads) + one full-suite run green.
- **Tier:** test-hygiene.

### `edit-as-tree-overwrites-existing-tree` — `import_root` replaces a hand-built (disabled) tree unconditionally

- **Surfaced:** 2026-06-10, v0.32.0 impl review (M2).
- **Where:** `src/form/tree_form.rs::render_edit_as_tree` — the `--emit-spec` success arm calls `TreeState::import_root(root)` (`src/form/tree_model.rs`), which installs the lowered archetype AST over whatever `state.tree` held. A user who hand-built a tree, switched to archetype mode (the selector's never-destroys discipline deliberately PRESERVES the disabled tree's nodes), then clicks "Edit as tree…" gets the hand-built tree silently replaced.
- **What:** decide the overwrite posture: confirm-on-overwrite (a modal when the existing disabled tree is non-trivial — e.g. any node with a non-empty kind) or a merge/keep-both affordance. The silent replacement is surprising precisely BECAUSE never-destroys taught the user that mode switches keep their work.
- **Status:** open.
- **Tier:** GUI-local (UX; no funds-safety or on-disk impact — the replaced tree was never emitted).

### `tree-xprv-heuristic-only-covers-key-fields` — `hex`/`w` fields could carry pasted xprv-like content unredacted

- **Surfaced:** 2026-06-10, v0.32.0 impl review (M4).
- **Where:** `src/form/tree_model.rs::blank_xprv_keys` — the persistence-redaction walk sweeps `key` + `keys[i]` only (SPEC §1.3, deliberately: hashlock `hex` digests must survive). The `hex` (hashlock digest) and `w` (wrapper string) free-text widgets accept arbitrary paste, so a mis-pasted xprv in either persists to `state.json` verbatim.
- **What:** extend the `is_xprv_like` sweep to `hex` + `w` — free belt-and-suspenders, since neither field is ever legitimately xprv-shaped (the heuristic can't false-positive on a 64-hex digest or a wrap-char string: `prv` at byte offset 1..4). Keep the keep-hex-digests posture; only xprv-MATCHING content blanks.
- **Status:** open.
- **Tier:** GUI-local (belt-and-suspenders; the redaction walk + its unit cells are the touchpoints).

### `repeating-secret-flags-never-reach-argv` — live bug: secret+repeating Text flags render into `secret_widgets` but `assemble_argv` reads them from `state.values`

- **Surfaced:** 2026-06-09, GUI v0.30.0 cycle (SPEC §5; pre-existing — NOT introduced by the v0.30.0 repeating-row widget, whose dispatch deliberately leaves the secret branch first/unchanged).
- **Where:** `src/form/widget.rs::render_with_dispatch` (the secret branch: `flag_is_secret(flag) && FlagKind::Text` → ONE `SecretLineEdit` in `state.secret_widgets[flag.name]`, regardless of `flag.repeating`) vs `src/form/invocation.rs::assemble_argv` (the secret-flag branch: `flag.repeating` → reads rows from **`state.values`** — the v0.3 repeating-secret fold comment documents that intended routing; the widget layer never got the per-row half).
- **What:** for every secret + repeating + Text flag — `--ms1` (2 repeating+secret sites in `src/schema/mnemonic.rs`: `VERIFY_BUNDLE_FLAGS` + `IMPORT_WALLET_FLAGS`) and `--share` (2 sites: `SLIP39_COMBINE_FLAGS`, `MS_SHARES_COMBINE_FLAGS` — impl-review I1 corrected the census: `SEED_XOR_COMBINE_FLAGS` `--share` is `NodeValueComposite`, NOT Text, so it bypasses the secret-widget branch, routes through `state.values`, and already emits correctly — it is the counter-example that works, and as of v0.30.0 it gains multi-row UI) — a LIVE form renders a single secret widget whose buffer lives in `secret_widgets`, while emission reads repeating secrets from `state.values` → **the live form emits NOTHING for these flags**. Masked by the kittest/unit cells, which synthesize `state.values` entries directly (e.g. `cell_import_wallet_repeating_ms1_argv`) and so exercise only the assembler half.
- **Fix direction (the v0.3 fold comment's design):** per-row `SecretLineEdit` rendering routed through `state.values` — render N secret rows (one `SecretLineEdit` per row, keyed per-row), write each row's value into `state.values` so the existing assembler loop emits them; keep the paste-warn modal + zeroize-per-widget posture; accept (as the v0.3 fold already did) that the values-map String copies are plain heap allocations during emission.
- **Why deferred:** out of A1 scope (consult ruling + SPEC §3 — the v0.30.0 repeating-row widget covers NON-secret flags only; the secret branch order is load-bearing and unchanged). Needs its own cycle: secret-row UX (per-row paste-warn, per-row remove) + the `secret_widgets`→`state.values` migration story for persisted sessions.
- **Status:** **resolved** `mnemonic-gui-v0.31.1` (2026-06-10). **Fix direction INVERTED vs this entry** (R0 adjudicated): values-routing would have PERSISTED seed material (`redact_for_persistence` never dropped schema-secret Text names) — instead `secret_widgets` went per-row (`BTreeMap<String, Vec<SecretLineEdit>>`; type-level never-persist + per-row zeroize preserved) and the assembler secret branch is KIND-GATED mirroring the widget dispatch (Text→vec; NodeValueComposite falls through — seed-xor unaffected; Boolean no-emit preserved → `boolean-stdin-secret-toggles-never-emit`). Belt-and-suspenders: the redaction union (field-extracted schema-secret names) incidentally closed TWO pre-existing plaintext persistence leaks (`xpub-search-inline-phrase-not-secret-classified`, `ms-repair-ms1-not-secret-classified`) + filed the positional one. THE silent migration hazard was `FormState::has_value` (Vec::is_empty compiles with inverted meaning). SPEC `design/SPEC_gui_v0_31_1_repeating_secrets.md` (R0 4 rounds; impl review GREEN).
- **Tier:** GUI-local (no sibling-repo flag surface change; the toolkit CLIs already accept the repeats).

### `boolean-stdin-secret-toggles-never-emit` — the Boolean `*-stdin` toggles (24 sites) emit NOTHING from the GUI form (RESOLVED v0.37.0 — greyed out)

- **Surfaced:** 2026-06-10, v0.31.1 cycle (SPEC §2 R0-r1 C1/I3 — the kind-gating of the assembler secret branch made the pre-existing suppression explicit). **Impl-review amendment: 18 suppressed sites, not 17** — `ms.rs:275-281` carries an 18th `--passphrase-stdin` (`secret: false` but name-matched via `SECRET_FLAG_NAMES`, so the Boolean `continue` eats it identically). **v0.33.0 census: 18 → 24** — the audit-I4 secret flips moved `--phrase-stdin` (×3) + `--ms1-stdin` (×3) into the suppressed family (they previously emitted as generic checkboxes; a checked toggle produced a CLI waiting on stdin the GUI runner can't feed, so no working flow regressed — the flip ALIGNS them with `--passphrase-stdin`). The no-emit mechanism is now pinned by `tests/secret_flips_v0_33_0.rs::t2` (first cell to do so).
- **Where:** `src/form/invocation.rs::assemble_argv` secret-flag branch — the `else { continue }` arm (non-Text, non-NodeValueComposite secrets). Pre-v0.31.1 the kind-BLIND secret branch ate them the same way (`flag_is_secret` → repeating values-read finds nothing for a Boolean / scalar `secret_widgets.get` finds no widget for a checkbox flag → `continue`); v0.31.1 PRESERVES that no-emit byte-identically and records it.
- **What:** the 6 Boolean toggle names in the suppressed family — `--passphrase-stdin` (×12 `secret: true` sites + the ms.rs:275 name-matched 13th), `--secret-stdin` (×2), `--decrypt-password-stdin` (×2), `--bip38-passphrase-stdin` (×1), and since v0.33.0 `--phrase-stdin` (×3) + `--ms1-stdin` (×3); all `repeating: false` (the pre-v0.33.0 body said "5 names" while enumerating 4 — that count was the entry's own error) — render as generic checkboxes (they fail the widget's Text gate) but their checked state NEVER reaches argv: the secret branch `continue`s before the generic Boolean emission. A user checking `--passphrase-stdin` in the GUI gets an argv without it (the GUI runner also provides no stdin channel for the value — the flag would hang or error if it DID emit, which is why the suppression has never been reported). Decide deliberately: either emit them (requires a stdin-feed story in `runner.rs`) or keep suppressing and grey them out in the form so the dead checkbox isn't a lie.
- **Status:** **resolved** `mnemonic-gui-v0.37.0` (2026-06-10) — **user decision: GREY OUT** (not emit; the GUI runner has no stdin channel). `render_with_dispatch` renders a secret Boolean (`flag_is_secret && FlagKind::Boolean`) as a DISABLED checkbox + tooltip; the predicate is EXACTLY the assembler's suppressed set (verified by a converse-closure test that REDs if a future secret non-Text/non-Composite/non-Boolean flag is added) so render and emit can't drift. SPEC `design/SPEC_gui_v0_37_0_greyout_stdin_toggles.md` (R0 GREEN r1). The emit-alternative (a `runner.rs` stdin-feed story) remains a non-goal.
- **Tier:** GUI-local.

### `xpub-search-inline-phrase-not-secret-classified` — the 3 xpub-search `--phrase` (inline master phrase) flags are `secret: false`

- **Surfaced:** 2026-06-10, v0.31.1 cycle (SPEC §3 R0-r2 I-NEW1 — the `schema_secret_flag_names()` union census found the cross-CLI name collision).
- **Where:** `src/schema/mnemonic.rs` — the three xpub-search subcommand flag tables (`XPUB_SEARCH_PATH_OF_XPUB_FLAGS` / address-of-xpub / passphrase-of-xpub variants; "Master BIP-39 phrase (inline)" help text), each declaring `--phrase` `FlagKind::Text` + **`secret: false`**, mirroring the toolkit `gui-schema`'s own classification.
- **What:** a master BIP-39 phrase typed inline renders as a PLAINTEXT (unmasked) generic Text widget, routes through `state.values`, and — pre-v0.31.1 — **persisted to `state.json` in plaintext** (no redaction class caught it). v0.31.1's `schema_secret_flag_names()` union incidentally closes the persistence leak at the NAME level (`--phrase` is `secret: true` on 4 ms.rs sites → the name joins the union → every `--phrase` values entry is dropped at persist). The underlying mis-classification remains: `secret: true` would flip the widget to a masked `SecretLineEdit` (+ run-confirm + zeroize). Likely needs a toolkit-side `gui-schema` classification fix first (the GUI mirrors the toolkit's `secret` field; hand-overriding GUI-side would trip `schema_mirror_secret_drift`) — check the gate's scope before choosing the layer.
- **Companion (2026-06-10):** the toolkit-side classification fix SHIPPED — **toolkit v0.53.1** classifies `--phrase`, `--phrase-stdin`, AND `--ms1-stdin` as `secret: true` (`mnemonic-toolkit/design/FOLLOWUPS.md::vacuous-secret-flag-gate` + `::gui-secret-mirror-phrase-ms1-stdin`). CORRECTION (v0.33.0 recon): the original Companion line's "(1) add the 3 tokens to `src/secrets.rs`'s token-for-token mirror" was structurally wrong — the secret-drift gate compares `FlagSchema.secret` per `(subcommand, flag)` (`tests/schema_mirror_secret_drift.rs:105-112`); `SECRET_FLAG_NAMES` is a 3-token v0.1 legacy fallback needing NO additions. The real lockstep was the 9 `FlagSchema.secret` flips + the 6-site pin bump.
- **Status:** **resolved** `mnemonic-gui-v0.33.0` (2026-06-10) — toolkit pin v0.52.0 → v0.53.1 + the 9 flips (`--phrase`/`--phrase-stdin`/`--ms1-stdin` × path-of-xpub/account-of-descriptor/passphrase-of-xpub). Masked `SecretLineEdit` + run-confirm + redaction union now live for the inline master phrase; the 2 stdin toggles ×3 joined the Boolean no-emit family (census 18→24, see `boolean-stdin-secret-toggles-never-emit`). Pinned by `tests/secret_flips_v0_33_0.rs::t3` + the secret-drift gate at the bumped pin. SPEC `design/SPEC_gui_v0_33_0_secret_flips_pin_bump.md` (R0 2 rounds GREEN).
- **Tier:** cross-repo (classification source of truth is the toolkit `gui-schema`).

### `ms-repair-ms1-not-secret-classified` — `ms repair --ms1` is `secret: false` (master-secret material)

- **Surfaced:** 2026-06-10, v0.31.1 cycle (SPEC §3 R0-r3 I-NEW2 — the union twin-check found the second `secret: false` collision; blind-spotted earlier because `--ms1` sat in the union's headline).
- **Where:** `src/schema/ms.rs::REPAIR_FLAGS` — `--ms1` `FlagKind::Text` / `required: true` / **`secret: false`** ("ms1 string to repair via BCH error correction"), vs the 7 `secret: true` `--ms1` sites in mnemonic.rs (this is the lone false twin in the 8-site census — R0-r4).
- **What:** the to-be-repaired ms1 string is master-secret material (merely BCH-corrupted), yet it renders unmasked, routes through `state.values`, and — pre-v0.31.1 — **persisted to `state.json` in plaintext**. v0.31.1's `schema_secret_flag_names()` union closes the persistence leak (the `--ms1` NAME is secret elsewhere → all `--ms1` values entries drop at persist); the widget-side mis-classification (unmasked input, no run-confirm) remains. Same layering question as `xpub-search-inline-phrase-not-secret-classified` — the classification mirrors the toolkit's `gui-schema` for `ms repair`.
- **Companion (2026-06-10):** toolkit v0.53.1 (`mnemonic-toolkit/design/FOLLOWUPS.md::gui-secret-mirror-phrase-ms1-stdin`) fixed the toolkit-CLI classifications (`--phrase`/`--phrase-stdin`/`--ms1-stdin`) but explicitly does NOT cover this entry — `ms repair --ms1` mirrors the **ms-cli** surface (the toolkit's own `repair --ms1` was already secret). The fix here needs an ms-cli-side classification source or a deliberate GUI-side override decision.
- **Status:** **resolved** `mnemonic-gui-v0.33.0` (2026-06-10) — **deliberate GUI-side override** (`ms.rs` `--ms1` → `secret: true`, R0-adjudicated): (a) the value is master-secret material (the lone false twin of the 8-site `--ms1` census); (b) NO automated gate covers ms.rs secret bits (the drift gate asserts `cli == "mnemonic"` :92 and walks `schema::mnemonic::SCHEMA` only :105-112) → no gate conflict; (c) ms-cli has no gui-schema surface — waiting for one blocks indefinitely, and `ms repair --help` itself calls the value sensitive. If a future ms-cli gui-schema projection lands with `secret: false`, keep ours and file per the gate's own "broader secret-class semantic" escape. Override rationale lives as a comment at the `ms.rs` site + is pinned by `tests/secret_flips_v0_33_0.rs::t1`.
- **Tier:** cross-repo (classification source of truth is the sibling `ms-cli` projection consumed via the toolkit `gui-schema` mirror discipline).

### `positional-secrets-not-redacted-at-persist` — secret-equivalent POSITIONALS ride `state.positionals`, cloned unredacted

- **Surfaced:** 2026-06-10, v0.31.1 cycle (SPEC §3 R0-r3 m-NEW1 — adjacent family leak OUTSIDE the flag-name net).
- **Where:** `src/schema/ms.rs::COMBINE_POSITIONALS` — the codex32 combine `shares` positional (help text says "Secret-equivalent" outright) — vs `src/persistence.rs::redact_for_persistence`, whose `positionals: state.positionals.clone()` copies every positional to disk with NO redaction class (the three drop classes are flag-name / node-type / slot-subkey; positionals have none).
- **What:** distributed codex32 share strings typed into the `ms combine` positional rows persist to `state.json` in plaintext. Pre-existing (untouched by v0.31.1, which extended only the flag-NAME net). Fix shape: a `PositionalArgSchema.secret` field (mirroring `FlagSchema.secret`) + a redaction arm dropping secret positionals — or route secret positionals through per-row `SecretLineEdit`s like v0.31.1 did for repeating secret flags.
- **Status:** **resolved** `mnemonic-gui-v0.34.0` (2026-06-10) — BOTH fix shapes: `PositionalArgSchema.secret` (census = 5, all ms.rs: 4 bare `ms1` + `combine shares` — BROADER than this entry's framing) + secret positionals route through per-row `SecretLineEdit`s under `secret_widgets["positional:<name>"]` (never-persist by type) + an unconditional drop-ALL-positionals redaction belt (the layer has no subcommand context — fail-closed) + a `should_confirm_run` positional loop. Toolkit companion NOT needed for measured reasons (gui-schema positionals carry no `secret` field; schema_mirror is flag-NAME-only; the hand schema mirrors only decode-address) — see `toolkit-secret-capable-positionals-unmirrored` below for the residual. SPEC `design/SPEC_gui_v0_34_0_persist_redaction_hardening.md` (R0 2 rounds). **Phase-8 persistence wiring is now UNBLOCKED on this count** (I4 cleared in v0.33.0; remaining Phase-8 tracking: `persistence-unwired-redaction-never-runs` + `slot-secret-values-rendered-unmasked` [render-side only — secret slot VALUES already never persist]).
- **Tier:** GUI-local (the schema field addition is GUI-side; the toolkit `gui-schema` positional projection may want a companion if the field should mirror).

### `tree-wif-hex-privkey-in-key-fields-unredacted` — tree key/keys WIF / raw-hex privkey survived the persist redaction walk (RESOLVED v0.34.0)

- **Surfaced:** 2026-06-10 audit ([IMPORTANT], index line above). **Resolved:** 2026-06-10, `mnemonic-gui-v0.34.0`.
- **What:** the persist walk (`tree_model::redacted_for_persistence`) blanked only `is_xprv_like` matches (extended-PRIVATE prefix shape). The toolkit gate's own doc says WIF/raw-hex secrets are NOT prefix-detectable — there they're refused by the step-2 `from_str` type-check, but the GUI persist path has NO step-2, so a WIF or raw-hex privkey pasted into a key/keys row would have written to `state.json` in cleartext (latent — persistence unwired).
- **Fix:** deny-list → **positive allowlist** `blank_non_extended_public_keys`: keep ONLY origin-stripped 10-prefix SLIP-132 extended-public matches (full 4-byte literals — byte 0 constrained on purpose: a bare `pub`-at-1..4 probe keeps `Kpub…`-form WIFs, the leak-direction false-accept). Documented fail-closed cost: hex PUBLIC keys blank too (shape-indistinguishable from hex private). Rejected option (b) "refuse to persist ungated trees": couples persistence to transient Validate state (`validate_ok` is `#[serde(skip)]`, cleared by `mark_dirty`) → silent whole-tree loss mid-edit. `is_xprv_like` retained for the render-side amber hint only (false-negatives tolerable there; doc cross-pointers added). 14-row table test incl. the recursive leg; the surplus-children leg is re-asserted by the pre-existing `tree_round_trip` redaction cell, now driving the new walk. **Phase-8 persistence wiring is now UNBLOCKED on this count.**
- **Tier:** GUI-local (the walk is GUI-side; the toolkit gate analogy is doc-only).

### `toolkit-secret-capable-positionals-unmirrored` — 7 toolkit gui-schema positionals (6 secret-capable) are not mirrored in the hand schema

- **Surfaced:** 2026-06-10, v0.34.0 R0-r1 I1 (the "toolkit has zero positionals" doc claim was FALSE — measured against the pinned v0.53.1 binary).
- **What:** `mnemonic gui-schema` emits non-empty `positionals` for 7 subcommands: `decode-address` (`address`, required — the only one mirrored, and the only NON-secret one), `inspect`/`repair`/`verify-bundle` (`extra_strings`, repeating, HRP-autodetect — routes ms1 = master secret), and the 3 xpub-search modes (`[MS1]...` positional ms1 cards). **6 of 7 are secret-capable.** The hand schema deliberately mirrors only decode-address (30× `NO_POSITIONALS`) so GUI users simply lack those intake fields today — no leak, a capability gap. If ever mirrored: HRP-autodetect positionals are VALUE-dependent secret → the schema-level `secret: bool` must be conservatively `true`; note gui-schema's positional objects carry only `{name, required, repeating}` (no `secret` field) — a toolkit companion to project one would be the principled source of truth.
- **Status:** open.
- **Tier:** cross-repo (capability gap + future toolkit `gui-schema` positional-secret projection).

### `gui-build-descriptor-presets-pending-pin-bump` — bump toolkit pin → v0.52.0 + add the 12 build-descriptor flags to the SubcommandSchema

- **Surfaced:** 2026-06-09, toolkit descriptor-builder Release B ship (`mnemonic-toolkit-v0.51.0`); **extended same day at toolkit v0.52.0** (+`--allow` → 12 flags; pin target → v0.52.0). `mnemonic build-descriptor` gained 11 clap flags → the `schema_mirror` flag-NAME lockstep applies; the schema cannot add them until the toolkit pin is bumped to v0.52.0 (chicken-and-egg, same arc as the v0.29.0 build-descriptor surfacing). Pin bump = the usual **6 lockstep sites** (Cargo.toml + Cargo.lock + `pinned-upstream.toml` `[mnemonic].tag` + README pin marker + `pinned_version` banner + module-doc), 4 gated by `pin_coherence`/`readme_pin_coherence`, 2 ungated.
- **Where:** `src/schema/mnemonic.rs` — extend `BUILD_DESCRIPTOR_FLAGS` with: `--archetype` (**Dropdown** `["decaying-multisig","hashlock-gated","kofn-recovery","simple-timelocked-inheritance","tiered-recovery"]` — alphabetical, == the toolkit `CliArchetype` order), `--key` + `--recovery-key` (**Text, `repeating: true`** — xpub strings, NOT Path; argv order is load-bearing for quorum order), `--threshold` + `--recovery-threshold` + `--older` + `--recovery-older` + `--after` (**Number**), `--final-key` + `--hash` (**Text**), `--emit-spec` (**Boolean**), and (v0.52.0) `--allow` (**Dropdown** `["malleable","mixed-timelock","repeated-keys","resource-limit","sigless-branch"]`, **`repeating: true`** — NOTE: a repeating Dropdown is a FlagKind combination this schema has not used before; current repeats are Text). Measure drift with a local v0.52.0 binary via `MNEMONIC_BIN` BEFORE the add (`schema_mirror` is flag-NAME set-equality vs the pinned binary's `gui-schema`).
- **Un-gated surfaces this entry is the channel for (toolkit presets SPEC §3.3/§8 — "never assumed"):** (1) `--json` wire-shape: `Diagnostic` gains optional `flag` (skip-serialized when absent; spec-mode output byte-unchanged), new kind `param`, `node_path: "params"` sentinel; (1b) at toolkit v0.52.0 the success envelope gains `allowed_rules_fired` (only when non-empty) and `cost` is `null` on a sanity-overridden emit, and step-3 refusal messages carry a `; rerun with --allow <kebab> after review` suffix; (2) `--spec-schema` gains an `archetypes` section (`{flag, kind, required, repeatable, min}` per preset — the surface a future archetype-FORMS wizard consumes); (3) deliberately UN-projected clap rules (`SubcommandSchema.conditional` stays `None` unless this cycle decides otherwise): `--archetype`↔`--spec` mutex, 10 `requires = "archetype"` edges, `--emit-spec` `conflicts_with_all = [--format, --json]` — GUI forms could emit argv clap refuses; CLI is the gate, but make it a recorded decision (compare-cost mutexes ARE hand-projected; precedent both ways).
- **Status:** **resolved** `mnemonic-gui-v0.30.0` (2026-06-09). Pin v0.50.0 → v0.52.0 (6 sites; measured drift = exactly the 12 build-descriptor flags); `BUILD_DESCRIPTOR_FLAGS` 6 → 18 incl. `--archetype` w/ the `""` UNSET sentinel + `--allow` repeating Dropdown (precedent: `--to` — this entry's "combination not used before" note was wrong); generic repeating-row widget (non-secret) makes all 5 presets GUI-drivable; `conditional::build_descriptor` archetype↔spec mutex. Un-gated wire notes (1)/(1b)/(2) recorded in the v0.30.0 SPEC; un-projected clap rules ACCEPTED (CLI is the gate; the A2 wizard supersedes the generic form). SPEC `design/SPEC_gui_v0_30_0_presets_pin_bump.md` (R0 3 rounds RED→GREEN; impl review YELLOW→folded). Spawned `repeating-secret-flags-never-reach-argv` (live pre-existing bug, 4 sites).
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::gui-build-descriptor-presets-pending-pin-bump`; relates to `manual-gui-build-descriptor-anchors-pending-pin-bump` (below — the anchor debt grows by the 11 new flags when the manual-gui pin eventually catches up).

### `manual-gui-build-descriptor-anchors-pending-pin-bump` — next `docs/manual-gui` pin bump must add `build-descriptor` anchors

- **Surfaced:** 2026-06-09, GUI v0.29.0 (surfaced `build-descriptor` in the schema mirror; toolkit pin v0.47.3 → v0.50.0).
- **Where:** `docs/manual-gui/` lives in the **mnemonic-toolkit** repo, pinned `mnemonic-gui-v0.3.0` (`docs/manual-gui/pinned-upstream.toml:19`); the GUI gate `tests/manual_anchor_coverage.rs` (`#[ignore]`'d, no `MANUAL_GUI_HTML_PATH` in CI) + the toolkit `make lint` check `docs/manual-gui/tests/check_gui_schema_coverage.py`.
- **What:** adding `build-descriptor` to the live GUI `SubcommandSchema` (v0.29.0) creates a latent obligation. When `docs/manual-gui`'s pin is next bumped to a GUI version that includes `build-descriptor`, the manual-gui HTML must carry the SPEC §2.2 kebab anchors: `mnemonic-build-descriptor` (subcommand) + `mnemonic-build-descriptor-{spec,spec-schema,format,network,json,no-auto-repair}` (per-flag) + `mnemonic-build-descriptor-format-{descriptor,bip388}` and `mnemonic-build-descriptor-network-{mainnet,testnet,signet,regtest}` (per-variant) — or `manual_anchor_coverage --ignored` + `check_gui_schema_coverage.py` fail then.
- **Status:** open — NOT a v0.29.0 blocker (the gate is `#[ignore]`'d in CI; the toolkit lint is pinned to GUI v0.3.0, which predates build-descriptor). Discharge at the next `docs/manual-gui` pin bump.
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::gui-build-descriptor-schema-mirror-pending-pin-bump` (resolved by GUI v0.29.0).

### `xpub-search-gui-bespoke-hub-pane` — discoverable umbrella hub UI for `xpub-search` modes

- **Surfaced:** 2026-05-18, v0.11.0 plan-vs-codebase recon. Plan §7.2 (in toolkit `/home/bcg/.claude/plans/woolly-spinning-honey.md`) enumerated a "hub" navigation pane with nav cards linking to the 4 mode panes. The GUI has no pane abstraction — every subcommand is a flat row in the subcommand-name ComboBox.
- **Where:** `src/main.rs:346-602` (central panel renderer; net-new per-pane dispatch branch); `src/schema/mnemonic.rs` (a new SubcommandSchema entry for the hub itself, or a sibling navigation manifest).
- **What:** Introduce a "hub" pseudo-pane visible when the user picks the umbrella `xpub-search` from a dropdown above the subcommand selector. Hub renders 4 cards (one per mode) with mode-name + 1-line description + click-through. v0.12.0 UI polish; not a v0.11.0 blocker.
- **Why deferred:** v0.11.0 plan-vs-codebase recon revealed plan §7.2's "pane" architecture was overspecified; v0.11.0 ships the 4 modes via the generic flag-renderer + the existing subcommand-name ComboBox.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `xpub-search-gui-bespoke-hub-pane`.

### `xpub-search-gui-bespoke-widgets` — per-mode composite widgets (TargetXpubField / DescriptorIntakeField / TargetAddressField / etc.)

- **Surfaced:** 2026-05-18, v0.11.0 plan-vs-codebase recon. Plan §7.3 enumerated `SeedIntakeWidget`, `TargetXpubField`, `DescriptorIntakeField`, `TargetAddressField`, `AddPathRepeater`, `XpubSearchResultRenderer` as net-new widgets. GUI codebase has NO `PhraseField` / `PassphraseField` / `Ms1Field` named types; the plan's "widget reuse" framing was wrong.
- **Where:** `src/form/` (new modules).
- **What:** Per-mode composite widgets with affordances beyond the generic `widget::render` dispatch: TargetXpubField with prefix-detect badge; AddressTypeField that auto-suggests from xpub prefix; DescriptorIntakeField with multi-line textarea + shape-detect badge; AddPathRepeater with +/− buttons. v0.12.0 polish.
- **Why deferred:** v0.11.0 ships via the generic FlagKind dispatcher; the bespoke widgets are UX polish, not functional blockers.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `xpub-search-gui-bespoke-widgets`.

### `xpub-search-gui-positional-intake` — positional ms1 (HRP-autodetect) routing in mnemonic-gui

- **Surfaced:** 2026-05-18, v0.11.0. The toolkit accepts a positional ms1 (HRP-autodetect) on P1/P2/P4; the GUI's argv assembler does not surface this affordance — the GUI forces users into `--ms1` explicitly.
- **Where:** `src/form/invocation.rs::assemble_argv`; `src/schema/mnemonic.rs` `positional_args: NO_POSITIONALS` on the 4 xpub-search entries.
- **What:** Add a "drop any card" textarea/file-drop affordance that auto-routes via HRP detection (`ms1` → positional, `mk1`/`md1` → future modes' surfaces). v0.12.0 polish.
- **Why deferred:** v0.11.0 keeps GUI argv assembly simple; positional intake is a polish item.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `xpub-search-gui-positional-intake`.

### `xpub-search-gui-flag-mutex-visibility` — cross-flag conditional visibility for `xpub-search` mutex groups

- **Surfaced:** 2026-05-18, v0.11.0. The 4 xpub-search SubcommandSchema entries set `conditional: None` for v0.11.0; cross-flag mutex visibility (e.g., greying `--ms1` when `--phrase` is filled in) is open.
- **Where:** `src/form/conditional.rs` (new per-subcommand functions following the existing pattern at `slip39_split` / `slip39_combine` / `repair` / `inspect` / `derive_child` / etc.).
- **What:** Per-subcommand `fn(&FormState) -> FlagVisibility` functions that grey/hide flags based on cross-flag state. For xpub-search modes: enforce the seed-intake mutex visually (only one of `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` interactive at a time); surface the P4 mandatory-passphrase requirement before run-confirm; flag the multi-`--target-address` repeating affordance in P3. v0.12.0 polish.
- **Why deferred:** v0.11.0 ships with `conditional: None`; the user sees all flags simultaneously and clap-side handles the mutex at exec. The GUI's run-confirm modal will surface clap errors verbatim — functional but not ideal UX.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `xpub-search-gui-flag-mutex-visibility`.

### `gui-schema-global-flag-emission` — toolkit-side: surface global flags in `mnemonic gui-schema` JSON per-subcommand

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase A.1 execution (v0.9.0 catchup). Plan §5 R7 realized: Phase A.1 attempted to add `--no-auto-repair` to the 10 existing `*_FLAGS` arrays in `src/schema/mnemonic.rs` and the schema-mirror drift gate hard-failed. The toolkit's `mnemonic gui-schema` v4 JSON output (which the gate consumes as source-of-truth) does NOT emit global flags for any subcommand — only clap's per-subcommand `--help` TEXT propagates them. Phase A.0 reconnaissance only checked help-text propagation, missing the JSON gap.
- **Where:** toolkit-side `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (the emitter that omits global flags); GUI-side `src/runner.rs::prepend_no_auto_repair` + `MnemonicGuiApp.no_auto_repair` field + action-bar checkbox in `src/main.rs` (the v0.9.0 fallback; ~30 LOC; load-bearing until toolkit-side fix lands).
- **What:** Extend the toolkit's `cmd::gui_schema` JSON emitter to include global flags (e.g. `--no-auto-repair`, `--debug`, etc.) per-subcommand so downstream consumers can mirror them natively. Until then, mnemonic-gui v0.9.0 ships an action-bar `--no-auto-repair` checkbox (prepended to argv at spawn time via `runner::prepend_no_auto_repair`) as the load-bearing fallback. When toolkit emits global flags per-subcommand, GUI can drop the action-bar checkbox and surface `--no-auto-repair` in each subcommand's form natively.
- **Why deferred (historical):** R7 fallback was functionally complete at v0.9.0; the toolkit-side emitter fix shipped in v0.24.0 Tranche B.3.
- **Resolution:** RESOLVED in `mnemonic-gui-v0.10.0` (companion close lockstep with `mnemonic-toolkit-v0.24.0` Tranche B.3). Toolkit gui-schema v5 envelope emits `global: true` for `--no-auto-repair` per-subcommand; GUI consumes the v5 field, mirrors the flag in every subcommand's form natively, and retires the v0.9.0 R7 action-bar checkbox + `runner::prepend_no_auto_repair` helper at 5 sites.
- **Status:** RESOLVED in mnemonic-gui-v0.10.0
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-global-flag-emission` (toolkit-side primary; resolved at toolkit v0.24.0).

### `toolkit-mnemonic-force-tty-promote-from-test-only` — toolkit-side: promote `MNEMONIC_FORCE_TTY` env-var from test-only to first-class public contract

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle D23 lock execution (v0.9.0 catchup). Plan §5 R1 mitigation.
- **Where:** GUI-side `src/runner.rs::run` (sets `MNEMONIC_FORCE_TTY=1` via `Command::env`); toolkit-side `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` doc-comment (classifies the env-var as test-only) + `crates/mnemonic-toolkit/src/cmd/{convert,inspect}.rs` (same env-var consumed via `is_terminal()` gate).
- **What:** mnemonic-gui v0.9.0 sets `MNEMONIC_FORCE_TTY=1` in the toolkit subprocess env so that the toolkit's `std::io::stdout().is_terminal() && !no_auto_repair` auto-fire gate fires for GUI-spawned invocations (GUI subprocesses are piped, not TTY — without the env override the GUI would never see auto-fire repair reports from `convert` / `inspect` / `verify-bundle`). The env-var is currently documented test-only in toolkit's `verify_bundle::run` doc-comment. GUI consumption creates a load-bearing dependency on the env-var's behavior; promotion to a first-class public contract (with explicit semver guarantee on its semantics) would harden the GUI side against silent toolkit-internal refactors.
- **Why deferred (historical):** Functional risk was documentary, not behavioral; the env-var worked correctly at toolkit v0.22.1. Toolkit-side promotion shipped in v0.24.0 Tranche A.
- **Resolution:** RESOLVED in `mnemonic-gui-v0.10.0` (companion close lockstep with `mnemonic-toolkit-v0.24.0` Tranche A). Toolkit promoted `MNEMONIC_FORCE_TTY` from test-only to first-class public API (semver-stable contract); doc-comment rewritten in `cmd/verify_bundle.rs::run`; manual subsection added under verify-bundle auto-fire. GUI's load-bearing dep on the env-var is now backed by a public contract.
- **Status:** RESOLVED in mnemonic-gui-v0.10.0
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `toolkit-mnemonic-force-tty-promote-from-test-only` (toolkit-side primary; resolved at toolkit v0.24.0).

### `clippy-test-target-cleanup` — `cargo clippy --workspace --all-targets -- -D warnings` fails on 8 pre-existing lints in test files

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase A.1 execution. CI gates currently run `--lib --bins` only (not `--all-targets`); v0.9.0 builds + tests + lib/bin clippy all green. Test-target clippy errors verified as pre-existing via `git stash` + clippy on clean HEAD (before v0.9.0 work).
- **Where:** `tests/manual_anchor_coverage.rs:25-27` (3 × overindented doc-list), `tests/slot_editor_contiguity.rs:24` (1 × `field_reassign_with_default`), `tests/conditional_visibility.rs:685` (1 × `len_zero`), plus 3 `doc_lazy_continuation` matches added by newer rustc/clippy releases.
- **What:** A dedicated cleanup pass to fix all 8 test-target lints so `cargo clippy --workspace --all-targets -- -D warnings` runs clean. Optionally tighten CI gates to use `--all-targets` once the cleanup lands.
- **Why deferred:** Test-only; lib/bin clippy stays clean; v0.9.0 cycle scope didn't include test-target lint cleanup. Filed for a dedicated cleanup pass.
- **Status:** RESOLVED in mnemonic-gui-v0.10.0
- **Tier:** `v0.10+`
- **Companion:** None — gui-only.

### `md-codec-decode-with-correction-supports-non-chunked-md1` — GUI-side consumer: `mnemonic repair --md1` rejects non-chunked-form md1 post-toolkit-v0.23.0

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase B.8 (release-boundary docs). GUI-side companion to the descriptor-mnemonic primary entry (filed after Phase B.6 + B.7 surfaced the gap).
- **Where:** GUI invokes the toolkit's `mnemonic repair --md1` via `src/runner.rs::run` (spawned subprocess + `MNEMONIC_FORCE_TTY=1` env per D23). Post-toolkit-v0.23.0 (Phase B.7 D29 migration), the `--md1` branch delegates to `md_codec::decode_with_correction` (md-codec v0.34.0 — Phase B.2), which integrates via `chunk::split` + `chunk::reassemble` and only accepts chunked-form md1 input (those bearing a chunk header, as emitted by `md encode --force-chunked` or by automatic chunking when the payload exceeds 320 bits). Non-chunked single-string md1 (the form emitted by plain `md encode` for small payloads) is rejected with a wire-format error. GUI users attempting to repair a non-chunked md1 through the toolkit-spawn pathway will see the wire-format-mismatch error surface in the stderr pane, with no corrected output on stdout.
- **What:** GUI-side consumer tracker for the md-codec primary. No GUI code change in this cycle — the GUI is a passive consumer of whatever the toolkit's `mnemonic repair --md1` accepts. When the primary lands its non-chunked-form coverage (md-codec patch release) and the toolkit consumes the updated codec API, the GUI's repair surface inherits the broader input acceptance automatically (no GUI work required). For UX clarity in the meantime, consider documenting the constraint in the GUI's repair-form help text or tooltips so users understand why some md1 inputs are rejected.
- **Why deferred (historical):** Constraint lived in the codec; GUI scope was unaffected beyond the documentation suggestion. Tracked for cross-repo visibility until the codec primary lands.
- **Resolution:** RESOLVED in `mnemonic-gui-v0.10.0` (downstream-consumer companion close lockstep with `md-codec-v0.35.0`). md-codec v0.35.0 (Tranche D.1) added non-chunked-form detection in `decode_with_correction`; toolkit v0.24.0 consumes it transparently via the unchanged `repair_via_md_codec` delegation. GUI's repair surface now accepts non-chunked single-string md1 via the toolkit-spawn pathway — no GUI code change required beyond the toolkit pin bump.
- **Status:** RESOLVED in mnemonic-gui-v0.10.0
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (primary; resolved at md-codec v0.35.0); `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (toolkit-side consumer; resolved at toolkit v0.24.0); `bg002h/mnemonic-secret` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (sibling-codec mirror).

### gui-conditional-applicability-drift-fix

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle. Motivating bug: GUI bundle form default state (template = `bip84`, single-sig) emitted `--threshold 1 --multisig-path-family bip48` which the CLI rejected with SPEC §6.6 byte-exact errors (`crates/mnemonic-toolkit/src/cmd/bundle.rs:120, 207-220`).
- **Where:** `src/form/conditional.rs` (P2 — ~14 NEW rules across `bundle` / `verify-bundle` / `export-wallet` / `derive-child`); `src/form/invocation.rs` (P3 — visibility gate at top of per-flag loop; both Hidden + Disabled suppress emission, Required does not); `tests/gui_schema_conditional_drift.rs` (P4 — NEW drift gate consuming toolkit `mnemonic gui-schema` JSON v2 `conditional_rules`); `src/main.rs:197-206` (P5 — removed `--multisig-path-family bip87` default seed); `src/schema_check.rs` (P1 lockstep — `parse_gui_schema_conditional_rules` + relax `parse_gui_schema_json` version gate from `!= 1` to `< 1`); `.github/workflows/schema-mirror.yml:60-69` (CI smoke-step gate relaxed from `==1` to `>=1` per SPEC §6.10.6 additive-bump policy).
- **What:** Cross-repo mechanism + comprehensive rule coverage. Consumes toolkit-emitted `conditional_rules` JSON v2 (SPEC §6.10 Predicate AST + Effect grammar + drift invariant). Adds ~14 NEW per-frame visibility rules. Extends `assemble_argv` with visibility gate. Latent-bug fix: typed-then-mutex-disabled secret values (e.g., user types `--passphrase=foo` then sets `--passphrase-stdin`) are now suppressed at argv emission per the visibility gate.
- **Status:** `resolved 7b7e07d` — shipped at `mnemonic-gui v0.5.0` (2026-05-16). All P1–P5 surfaces landed; drift gate green; 187/187 GUI tests green against toolkit `v0.16.0` (commit `519bcfc`). End-of-cycle opus reviewer-loop R1 FOLD → R2 PASS (0C / 0I). One post-tag CI surface (`schema-mirror.yml` install-tag was stale) folded in `<next-commit-SHA>`; tracked separately at `schema-mirror-yml-toolkit-pin-tracks-pinned-upstream`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-conditional-rules-v1` (resolved at toolkit v0.16.0 commit `519bcfc`).

### schema-mirror-yml-toolkit-pin-tracks-pinned-upstream

- **Surfaced:** 2026-05-16, end of v0.5.0 cycle. After `mnemonic-toolkit-v0.16.0` + `mnemonic-gui-v0.5.0` tag-push, the master-branch `schema-mirror` CI failed at `tests/gui_schema_conditional_drift.rs` with `drift gate must exercise at least one rule; got 0`. Root cause: `.github/workflows/schema-mirror.yml:28` hardcoded `--tag mnemonic-toolkit-v0.14.0` (stale since the v0.14.0 release; harmless until v0.5.0 because prior `schema_mirror` tests only consumed flag-name extraction, which is version-agnostic). The v0.5.0 drift gate requires v2 `conditional_rules` emission. Tag CI for `mnemonic-gui-v0.5.0` itself was green (workflow scoped to `branches: [master]`); only master CI failed.
- **Where:** `.github/workflows/schema-mirror.yml` install-mnemonic-toolkit step (line ~30 post-fold). Same drift class applies to the md / ms / mk install steps below it (lines ~36/43/49) which still hardcode `md-cli-v0.5.0` / `ms-cli-v0.2.1` / `mk-cli-v0.3.1` — currently NOT stale vs `pinned-upstream.toml`, but the same drift-detection pattern would prevent future divergence.
- **What:** v1 fold (lands in the same v0.5.0 cycle): bump install-mnemonic-toolkit tag to `mnemonic-toolkit-v0.16.0`. v2 cleanup (future cycle): parameterize all four install steps' tag values from `pinned-upstream.toml` so they auto-track future bumps. Two options for the v2 implementation — (a) a workflow-pre step that parses `pinned-upstream.toml` and exports `MNEMONIC_TOOLKIT_TAG` / `MD_TAG` / `MS_TAG` / `MK_TAG` env vars, or (b) a per-CLI matrix that reads the pin via `dasel` / `jq` per step. (a) is simpler; (b) is more granular.
- **Why deferred:** v1 fold is mechanically trivial and ships this cycle; v2 cleanup is a UX-grade improvement that wasn't in the v0.5.0 cycle's scope.
- **Status:** `resolved 93c862a` — v2 cleanup shipped at `mnemonic-gui v0.5.1` (2026-05-16). Tag points at `93c862a`; the cycle split across two commits: workflow surgery + Cargo bump + CHANGELOG at `a445277`; snapshot-test refactor + CHANGELOG amendment at `93c862a` (the latter folded a latent bug — see below). Mechanism: new `parse-pinned-upstream` workflow-pre step loads `pinned-upstream.toml` via Python 3.11+ stdlib `tomllib` and exports per-CLI tag values; each install step consumes the matching `${{ steps.pins.outputs.<cli>_tag }}` via the `env:` → `$TAG` pattern (per GitHub's hardening guidance). Master CI green at `93c862a`: schema-mirror (run `25973805125`) + build (run `25973805129`). Tag CI green: build (run `25973933383`). Latent-bug fix folded: `tests/schema_mirror.rs::ci_workflow_snapshot` had been passing on an incidental v0.14.0 comment substring (the v0.5.0 fix-commit `54865a7` bumped the real install-step pin from v0.14.0 → v0.16.0 but left a surrounding comment mentioning v0.14.0); the v0.5.1 workflow surgery removed the comment and surfaced the gap. Refactored to assert v2 wiring directly: `parse-pinned-upstream` step present + four `steps.pins.outputs.<cli>_tag` references.
- **Tier:** `v0.6+`
- **Companion:** None — gui-only.

### `gui-schema-runtime-conditional-projection` — project SPEC §6.6 slot-count-dependent + runtime rules into gui-schema JSON

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle. Filed at cycle close per plan §7 item 1.
- **Where:** `src/form/conditional.rs` (gui side — slot-count signal from FormState to conditional engine); `src/schema_check.rs` (Predicate AST extension for `slot_count_op` / `slot_count_min` etc. when toolkit adds them).
- **What:** v1 cycle deferred slot-count-dependent + post-binding rules because the GUI's conditional engine consumes FormState snapshots without slot-count exposure. A future cycle will plumb a slot-count signal through FormState + extend the Predicate AST. Concrete rules to add: SPEC §6.6 row 9 (T-in-range vs N), row 10 (single-sig with N > 1), row 11 (multisig with N == 1), row 13 (BIP-388 distinct-key), row 14 (per-`@N` annotation inconsistency).
- **Why deferred:** Per plan §1.4 — runtime rules surface at Run time via the CLI's typed error. v1 ships argv-level submission.
- **Status:** `resolved 83efd2e` — fully closed 2026-05-16 across three sub-cycles. v2-cycle (`mnemonic-gui-v0.6.0`, `7d5e875`) shipped the **predicate-machinery**: schema v3 + `SlotCountEq`/`SlotCountGte`/`SlotCountLte` Predicate variants (toolkit `76db841` + GUI `9d447d0`); `FormState::slot_count()` accessor + drift gate `synthesize_satisfying` arms. Row 12 closed via separate `pin_value` Effect. Remaining row partition closed via two child FOLLOWUPs (both now resolved): `gui-schema-effect-on-dropdown-options-vocab` → resolved `f86a696` (Batch B-1: `mnemonic-gui-v0.7.0` — `disable_options` Effect consumer + GUI-internal `NumberMax::FromSlotCount` closing rows 9/10/11); `gui-schema-cross-slot-predicate-projection` → resolved `38ad066` (Batch B-2: `mnemonic-gui-v0.7.1` — row 8 GUI-internal `detect_slot_index_gaps`; rows 13/14 wontfix with CLI-rejection-sufficient rationale).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-runtime-conditional-projection` (resolved in lockstep).

### `gui-number-widget-unset-sentinel` — Number/Range/Timestamp/TaggedOrIndexed widgets lack a "no value" sentinel

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 2.
- **Where:** `src/schema/mod.rs:263-268` (`flag_value_is_present` always returns true for Number/Range/Timestamp/TaggedOrIndexed); `src/form/widget.rs:101-126` (`default_flag_value_for` seeds Number widgets to `min` regardless of user interaction).
- **What:** Numeric / Range / Timestamp / TaggedOrIndexed widgets have no "no value" sentinel — once `default_flag_value_for` seeds them, the value is always-present per `flag_value_is_present`. The v0.5.0 §6.10 visibility gate sidesteps this for the common case (Hidden/Disabled flags don't emit regardless of widget value). A future cycle may add an explicit unset state for UX clarity (e.g., a "clear" affordance next to numeric widgets so users can explicitly opt out of supplying a numeric flag).
- **Why deferred:** Per plan §1.4 — the visibility gate makes this unnecessary for the motivating bug. UX-quality improvement, not a correctness gap.
- **Status:** `resolved 84a69b8` — `mnemonic-gui-v0.6.0` P3 (2026-05-16). +`FlagValue::Unset` variant (unit, with `#[serde(other)]` for forward-compat); `flag_value_is_present(Unset)` returns false; `default_flag_value_for` returns Unset for the four Unset-default kinds; new `seeded_value_for(kind)` helper for click-to-seed; widget `Set` / `✕` affordances. Persistence-schema delta documented in CHANGELOG [0.6.0] — forward-compat preserved; v0.5 downgrade can't deserialize Unset entries (bounded impact). 14 new test cells at `tests/widget_unset_sentinel.rs`. Caveat: `#[serde(other)]` on the externally-tagged `FlagValue` enum works empirically but is not formally specified by serde — tracked at new FOLLOWUP `gui-flag-value-unset-serde-other-externally-tagged-dependency`.
- **Tier:** `v0.6+`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` cross-reference entry `gui-number-widget-unset-sentinel` (toolkit-side bookkeeping only — gui-impact-only).

### `gui-default-form-state-template-aware-seed` — replace static default-state seed with template-aware seed

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 3. Natural successor to P5 (the v0.5.0 cycle's static seed cleanup at `src/main.rs:203`).
- **Where:** `src/main.rs:197-211` (default form-state seed; v0.5.0's P5 removed the `--multisig-path-family bip87` line but left the static structure intact).
- **What:** Replace the static screenshot-mode default seed with a template-aware default. When the user picks a multisig template (e.g., `wsh-sortedmulti`), the form auto-seeds multisig defaults (e.g., `--multisig-path-family bip87`, `--threshold` to a reasonable default); when the user picks single-sig, the form omits those flags entirely.
- **Why deferred:** Out of v0.5.0 cycle scope per plan §7 — optional follow-on. The v0.5.0 P5 cleanup removes the unconditionally-wrong seed; the template-aware version is a UX enhancement.
- **Status:** `resolved 538dc70` — `mnemonic-gui-v0.6.0` P4 (2026-05-16). +`form::conditional::template_defaults_for(template)` returning `[]` for single-sig (`bip44`/`bip49`/`bip84`/`bip86`) and `[(--threshold, Number(2)), (--multisig-path-family, Dropdown("bip48"))]` for multisig. +`MnemonicGuiApp.last_template` per-form tracker + per-frame egui hook in `update()` that detects `--template` transitions and applies the defaults via **seed-on-empty discipline** (only seeds absent flags; preserves user-typed values across template switches; no overwrites, no clears, no undo machinery needed). 5 new test cells at `tests/template_aware_seed.rs` covering helper shape + seed-on-empty composition + multisig↔single-sig round-trip preservation. The `bip48` choice matches the canonical multisig path family; threshold-of-2 the smallest non-degenerate threshold.
- **Tier:** `v0.6+`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` cross-reference entry `gui-default-form-state-template-aware-seed` (toolkit-side bookkeeping only — gui-impact-only).

### `gui-schema-numeric-flag-value-pin-effect` — add `pin_value` Effect variant for SPEC §6.6 row 12 projection

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I3 reviewer fold. Plan §7 item 4.
- **Where:** `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary); `mnemonic-toolkit/src/cmd/gui_schema.rs` (Effect enum + serializer); `mnemonic-toolkit/src/cmd/bundle.rs:200-205` (the rule the projection would encode — `DESCRIPTOR_WITH_NONZERO_ACCOUNT`); `src/form/conditional.rs` (consumer — Number widget value-coerce-to-zero handler).
- **What:** Add a `pin_value: { flag, value }` Effect variant to SPEC §6.10.3 vocabulary so the GUI can coerce `--account` to 0 (or any pinned numeric value) when `--descriptor` is present, mirroring SPEC §6.6 row 12's CLI rejection at `bundle.rs:200-205`. v0.5.0's Number widget for `--account` defaults to `0` (per `default_flag_value_for`) — the safe value; the rule only fires when the user actively types a nonzero value, in which case the CLI's byte-exact error suffices for v0.5.0.
- **Why deferred:** Per R1 I3 reviewer fold — the GUI default of 0 makes this rare misuse; the CLI error is informative. Adding a `pin_value` Effect requires SPEC §6.10.3 expansion + GUI Number-widget coercion semantics not warranted by user evidence.
- **Status:** `resolved 9d447d0` — `mnemonic-gui-v0.6.0` P2 (2026-05-16). Toolkit-side `mnemonic-toolkit-v0.17.0` `76db841`: SPEC §6.10.3 v3 grammar extension (PinValue Visibility variant + wire shape `{"pin_value": {"value": V}}`); §6.10.4 NEW emission table enumerates PinValue's REPLACE-user-value semantic; §6.10.7 row 12 flipped DEFERRED → ENCODED v2; `gui_schema.rs::bundle_conditional_rules` emits the new rule. GUI-side `9d447d0`: `schema_check.rs::VisibilityProjection +PinValue` with custom Deserialize accepting both v2 bare-string and v3 tagged-object shapes (Copy dropped); `schema::Visibility +PinValue` in lockstep; `form::conditional::bundle()` pushes the row 12 rule; `assemble_argv` extended with PinValue emission path (+ `pin_value_to_argv_token` helper for Number/String/Bool primitives). 8 new tests across conditional_visibility/argv_assembler_visibility/schema_mirror (deserialize round-trip + reject-on-unknown).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-numeric-flag-value-pin-effect`.

### `gui-schema-template-groups-meta-field` — emit per-subcommand `meta.template_groups` to retire `SINGLE_SIG_TEMPLATES` const

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I4 reviewer fold. Plan §7 item 5.
- **Where:** `mnemonic-toolkit/src/cmd/gui_schema.rs` (toolkit side — emit `meta.template_groups: { single_sig: [..], multisig: [..] }` block sourced from `Template::is_multisig()`); `src/form/conditional.rs:23` (gui side — replace module-level `SINGLE_SIG_TEMPLATES: &[&str] = &["bip44", "bip49", "bip84", "bip86"]` with parse from JSON `meta.template_groups`); `mnemonic-toolkit/src/template.rs:46-56` (`is_multisig()` source-of-truth — unchanged).
- **What:** v0.5.0 cycle replicates the single-sig template set client-side as a module-level `SINGLE_SIG_TEMPLATES` const in `conditional.rs`. The drift gate test detects divergence, but a future cleanup cycle can collapse the const by having the toolkit emit `meta.template_groups` in the gui-schema JSON.
- **Why deferred:** Out of v0.5.0 cycle scope — the drift gate suffices for parity enforcement. Cleanup-class change.
- **Status:** `resolved 9d447d0` — `mnemonic-gui-v0.6.0` P2 (2026-05-16). Toolkit-side `mnemonic-toolkit-v0.17.0` `76db841`: SPEC §6.10.8 NEW per-subcommand `meta` block; `gui_schema.rs::build_subcommand_meta` emits `meta.template_groups: { single_sig, multisig }` sourced from `CliTemplate::is_multisig()`. GUI-side `9d447d0`: `SINGLE_SIG_TEMPLATES` const promoted `pub(crate) → pub`; new parity test `tests/schema_mirror.rs::single_sig_templates_const_matches_meta_template_groups` (MNEMONIC_BIN-gated) asserts the runtime const matches the toolkit-emitted meta block for every template-consuming subcommand. Pair-of-checks posture (drift gate for per-rule projection + const-vs-meta for the bulk list) closes the FOLLOWUP without coupling conditional-fn purity to a runtime subprocess fetch. Defect carried forward as a new FOLLOWUP: toolkit's `build_subcommand_meta` emits the meta block for `derive-child` but derive-child has no `--template` flag — see new FOLLOWUP `gui-schema-derive-child-meta-template-groups-spurious`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-template-groups-meta-field`.

### mnemonic-gui-schema-mirror

**What:** The `mnemonic-gui` GUI maintains a schema (per-CLI flag surface
description) that mirrors the four constellation CLIs (`mnemonic`, `md`,
`ms`, `mk`). Drift between the GUI schema and any CLI's clap-derive flag
set is enforced at CI time via `.github/workflows/schema-mirror.yml`,
which installs each pinned upstream binary and runs the in-process
`tests/schema_mirror.rs` cells.

Additionally, Phase 7's `build.rs` codegen reads the upstream
`NodeType::is_secret_bearing()` and `SlotSubkey::is_secret_bearing()`
impls to generate the `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS`
constants. The runtime `source_audit_*` tests re-parse the upstream
files and assert set-equality.

**Companion entries (per CLAUDE.md cross-repo discipline):**

| Sibling repo | Companion file | Current pinned tag | gui-schema PR (Phase C.2) |
|--------------|----------------|--------------------|---------------------------|
| `bg002h/mnemonic-toolkit` | `design/FOLLOWUPS.md` | `mnemonic-toolkit-v0.13.0` (v0.3); was `v0.9.0` at v0.2 | [#14](https://github.com/bg002h/mnemonic-toolkit/pull/14) |
| `bg002h/descriptor-mnemonic` | `design/FOLLOWUPS.md` | `descriptor-mnemonic-md-cli-v0.5.0` | [#29](https://github.com/bg002h/descriptor-mnemonic/pull/29) |
| `bg002h/mnemonic-secret` | `design/FOLLOWUPS.md` | `ms-cli-v0.2.0` | [#5](https://github.com/bg002h/mnemonic-secret/pull/5) |
| `bg002h/mnemonic-key` | `design/FOLLOWUPS.md` | `mk-cli-v0.3.0` | [#8](https://github.com/bg002h/mnemonic-key/pull/8) |

Each sibling-repo entry must cross-cite this entry + the
`mnemonic-gui` repo URL + this `mnemonic-gui-schema-mirror`
workflow URL. When the sibling CLI's flag surface changes (flag
add/remove/rename, conflict_with addition, etc.), both the
sibling-repo PR AND a companion `mnemonic-gui` PR (bumping the
schema + pinned-upstream.toml tag) land in lockstep — matching the
mnemonic-toolkit ↔ docs/manual mirror-invariant pattern.

**Suggested sibling-repo FOLLOWUPS body** (copy-paste into each sibling's
`design/FOLLOWUPS.md`):

```markdown
### mnemonic-gui-schema-mirror

**Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry
`mnemonic-gui-schema-mirror`; CI gate at
`.github/workflows/schema-mirror.yml`.

The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at
the pinned tag `<TAG>`. Any flag add / remove / rename / conflict_with
change in this repo's CLI surface must land in lockstep with a
companion `mnemonic-gui` PR that bumps the schema + the
`pinned-upstream.toml` tag for this CLI.
```

### slip39-gui-schema-flattening-companion

**Companion:** `bg002h/mnemonic-toolkit` `design/PLAN_v0_13_0_p2.md` §4.2 + `design/FOLLOWUPS.md` entry `slip39-shamir-secret-sharing`; toolkit P2.1 RED commit bumps `tests/cli_gui_schema.rs` from 7 → 10 subcommands.

**What:** v0.13.0 P2.1 GREEN lands a `cmd/gui_schema.rs` flattening fix in `mnemonic-toolkit`: nested clap subcommands now emit flattened hyphenated entries in the `gui-schema` JSON output. Specifically:

- `seed-xor` → `seed-xor-split` + `seed-xor-combine`
- `slip39` → `slip39-split` + `slip39-combine`

Schema `version` stays at `1` (additive: existing nested-parent names disappear; new hyphenated names appear; the schema document shape is unchanged).

**Pre-RED probe (executed at toolkit `81488e3`):** confirmed `mnemonic gui-schema | jq '.subcommands[] | select(.name == "seed-xor")'` returns `{name: "seed-xor", flags: [], positionals: []}` — i.e. `mnemonic-gui` v0.2 cannot see `seed-xor split` / `seed-xor combine` as discoverable subcommands. **This is a pre-existing v0.12.0 gap, NOT a v0.13.0 regression.** The toolkit-side flattening fix repairs both v0.12.0 (seed-xor) AND v0.13.0 (slip39) at the same patch.

**GUI-side companion work (gated on `mnemonic-toolkit-v0.13.0` shipping):**

1. Bump `pinned-upstream.toml` `mnemonic-toolkit` tag to `mnemonic-toolkit-v0.13.0` (toolkit PE rollup tag).
2. Refresh the schema-mirror tests (`tests/schema_mirror.rs`) to reflect the new flattened subcommand-name set — the test fixture pins `subcommands[]` names.
3. Audit any GUI surface that dispatched on the now-removed `seed-xor` name. The GUI's v0.2 release predates this fix; the seed-xor surface may have been an empty / unreachable code path (the upstream schema returned `flags: []` so per-flag dispatch had nothing to render). Verify before assuming a no-op.
4. Add `slip39-split` + `slip39-combine` GUI surfaces (new subcommand pair shipped at toolkit v0.13.0).

**Status:** `resolved at mnemonic-gui-v0.3.0` — all 4 GUI-side work items shipped 2026-05-14 in cycle v0.3. The bumped `mnemonic-toolkit-v0.13.0` pin + 4 v0.10..v0.13 drift flags (bundle/verify-bundle/convert/derive-child `*-stdin` adds, closes the `mnemonic-gui-schema-mirror` invariant breach) + 5 new subcommand surfaces (`slip39-{split,combine}`, `seed-xor-{split,combine}`, `final-word`) landed under release tag `mnemonic-gui-v0.3.0`. The latent v0.2 repeating-secret bug in `assemble_argv` was also surfaced and fixed in lockstep. See `design/PLAN_v0_3.md` for the 3-section reviewer-LOCKed plan + P0 drift-fold amendment.

**Tier:** shipped at `mnemonic-gui-v0.3.0`.

### gui-accesskit-production-side-effect (accepted in v0.2 Phase A.3)

**What:** v0.2 Phase A.3 introduced `egui_kittest = "0.31"` as a
dev-dependency (the egui-driven integration test harness). Cargo
feature unification then activates `egui/accesskit` globally because
`egui_kittest 0.31.1 → kittest 0.1.0 → accesskit 0.17.1` requires it,
and `egui-winit 0.31.1`'s `PlatformOutput` is destructured
exhaustively — without the matching feature on egui-winit, the build
fails. The minimal fix was to add `"accesskit"` to eframe's feature
list in `Cargo.toml`, which propagates the feature to both
`egui/accesskit` and `egui-winit/accesskit` (per eframe 0.31
`[features]`).

**Production-binary consequence:** the GUI binary now links the
accesskit family on all platforms (`accesskit_winit` 0.23.1,
`accesskit_unix` 0.13.1 + `atspi-*` transitive on Linux,
`accesskit_macos` 0.18.1, `accesskit_windows` 0.24.1). The
accessibility tree is active at runtime — screen readers and
accessibility tools can traverse the GUI's widgets.

**Disposition: accepted.** No cargo mechanism scopes a feature
activation to dev/test builds only (features are strictly additive
across the dep graph). No accesskit-free egui-0.31 testing harness
exists. The side effect is behaviorally benign (active accessibility
support is a positive externality), not a security concern.

**Revisit triggers:**

- If egui_kittest 0.32+ decouples the kittest/accesskit dep and a
  future GUI version drops the harness, the accesskit feature could
  be removed from eframe.
- If the accessibility tree exposure of mnemonic input fields becomes
  a threat-model concern (e.g., a screen-reader API leaks the secret
  buffer), revisit and audit the accesskit_winit accessible-name
  surface on `SecretLineEdit`.

**Trace:** v0.2 plan Phase A.3 R5 fold N-2 in
`/home/bcg/.claude/plans/v0_2-mnemonic-gui.md` Section C iterative
review log; report at
`design/agent-reports/v0_2-phase-A3-kittest-scaffold-r1.md`.

### `secret-taxonomy-public-api-consumption` — retire `build.rs` source-walker; consume `mnemonic-toolkit::secret_taxonomy` directly

**Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry
`secret-taxonomy-public-api-promotion`. Architect-vetted long-term fix
for the codegen pattern that caused the v0.3.0..v0.3.2 BIP-39-persistence
leak (HIGH-severity; tactically patched in v0.3.3 at commit `6851d1b`).

**Surfaced:** 2026-05-16, post-v0.3.3 emergency security fix.

**Where:** `build.rs` (the entire syn-based upstream-source walker is
the deletion target); `src/secrets.rs` (consumes `SECRET_*` via
`include!(concat!(env!("OUT_DIR"), ...))`; switches to `use
mnemonic_toolkit::secret_taxonomy::*`); `tests/secrets_canonical_fallback.rs`
(the v0.3.3 drift gate — deleted after one-cycle overlap);
`pinned-upstream.toml` (`[mnemonic]` `tag` becomes documentary;
load-bearing pin moves to `Cargo.toml`'s `[dependencies]` table);
`.github/workflows/schema-mirror.yml` (drop the
`cargo-test-secrets-canonical-fallback` step).

**What:** Today the GUI scrapes the toolkit's *private* `cmd/convert.rs`
+ `slot_input.rs` modules via `syn::parse_file` at build time. This is
the workaround for the toolkit's lack of a versioned, addressable
public contract for the secret-class taxonomy. Every fragility of the
codegen path descends from that contract gap — the cargo-install
sandbox stub-fallback bug (v0.3.0..v0.3.2 empty `&[]` arrays leaking
BIP-39 phrases to `state.json`) was a direct consequence. The
toolkit-side companion entry adds a new `pub mod secret_taxonomy` in
`mnemonic-toolkit v0.14.0`; this entry tracks the GUI-side switch to
that contract in `mnemonic-gui v0.4.0`.

**Why deferred:** v0.3.3 tactical patch is shipped + verified +
released; install path is no longer leaking secrets. Long-term fix
requires coordinated minor bump on both sides
(`mnemonic-toolkit v0.14.0` + `mnemonic-gui v0.4.0` lockstep). Filed
for the v0.4.x GUI cycle.

**One-cycle overlap recommended:** in GUI v0.4.0, retain the v0.3.3
`CANONICAL_FALLBACK_*` arrays + the `committed_fallback_is_non_empty`
backstop test, AND add a compile-time `const _: () = assert!(...)`
that they equal `mnemonic_toolkit::secret_taxonomy::SECRET_*`. Drop
the fallback in v0.5.0 once the new contract has been exercised
through one release cycle.

**Status:** `resolved 6fe44b6` (mnemonic-gui v0.4.0, 2026-05-16). Cargo.toml gains `mnemonic-toolkit = { git, tag = "mnemonic-toolkit-v0.14.0" }`; `build.rs` deleted; `src/secrets.rs` switches to `pub use mnemonic_toolkit::secret_taxonomy::*` + compile-time supply-chain guard against drift from v0.3.3's committed snapshot. R1 opus review caught a Critical (incomplete deletion sweep — `tests/schema_mirror.rs::source_audit` mod survived) + 5 Importants; all folded in the same commit before tag. Toolkit half closed at `bg002h/mnemonic-toolkit@1a52612` (mnemonic-toolkit v0.14.0).

**Tier:** `cross-repo / v0.4.0-coordinated`

**Architect's full evaluation** (Options A–E, recommendation A, migration
sketch, 6 non-obvious risks) is in the toolkit-side companion entry —
read that for the deeper rationale.

**Risks to surface at v0.4.0 planning time:**
1. Toolkit dep tree (bitcoin, miniscript, bip39, clap, etc.) gets linked
   into the GUI's cargo graph — ~30-60s cold compile cost increase.
   Mitigation: optional `cli` default-on feature-gate on the toolkit
   side; GUI depends with `default-features = false, features =
   ["secret-taxonomy"]`. Defer if compile cost is acceptable.
2. Toolkit's `secret_taxonomy` module becomes load-bearing semver
   surface — rename/relocate now requires a minor bump.
3. The GUI's pinned `mnemonic-toolkit` tag must stay current; a future
   toolkit-side `is_secret_bearing()` widening (e.g., new node type
   added) without a GUI bump means the GUI silently lacks the new
   secret class. Mitigation: future `mnemonic gui-schema` extension
   emitting the live taxonomy + GUI runtime cross-check against the
   installed `mnemonic` binary.
4. Re-export choice: `pub const &[&str]` (recommended) vs.
   `pub use NodeType` / `pub use SlotSubkey`. Stick with string slices;
   smaller semver surface; decouples GUI from toolkit's internal enum
   shape.
5. `mnemonic-toolkit` lib must build cleanly on GUI's full platform
   matrix (macOS, Windows, Linux × x86_64 + aarch64). `mlock.rs` uses
   `libc` and needs cfg-gating audit (likely already correct, but
   revisit during v0.14.0 release).
6. Lockstep release discipline (mirrors the manual-gui v1.0 cycle
   pattern): toolkit v0.14.0 PR + GUI v0.4.0 PR coordinated; both
   `Companion:` lines updated as each side closes.

### `mnemonic-gui-cratesio-publish` — re-enable `cargo install mnemonic-gui` from crates.io (blocked by toolkit publish)

**Companion:** `bg002h/mnemonic-toolkit/design/FOLLOWUPS.md` entry
`mnemonic-toolkit-cratesio-publish` (blocking).

**Surfaced:** 2026-05-16, post-v0.4.2 crates.io publish audit. v0.3.0
and v0.3.1 were published to crates.io and SHIPPED THE BIP-39
PERSISTENCE LEAK to any direct `cargo install mnemonic-gui` user;
both versions are now yanked (2026-05-16 17:36 UTC, cargo audit
records `bg002h` as the yanker). v0.3.2 / v0.3.3 / v0.4.0 / v0.4.1 /
v0.4.2 were tagged but never published.

**Where:** `Cargo.toml` line 36: `mnemonic-toolkit = { git = "...",
tag = "mnemonic-toolkit-v0.14.2" }` is the publish-blocking dep.
crates.io requires version-or-version+git/path; pure-git deps are
forbidden in published crates.

**What:** Once `mnemonic-toolkit` is on crates.io (toolkit-side
FOLLOWUP), this entry's work is:
1. Change the Cargo.toml dep from `{ git, tag }` to `{ version = "0.14" }` (or whatever the published version is).
2. Verify the v0.3.3 supply-chain guard's `v0_3_canonical_fallback` snapshot still equals the crates.io toolkit's `SECRET_*` (it should, since the toolkit-version pin determines both).
3. `cargo publish --dry-run` then `cargo publish` from `mnemonic-gui`.
4. Toolkit `install.sh` flips `mnemonic-gui` from `cratesio=no` back to `cratesio=yes` so direct `cargo install mnemonic-gui` users get a binary that's structurally incapable of the v0.3.x leak class.

**Why deferred:** Blocked by toolkit publish work; not blocking
install-script users (`./scripts/install.sh mnemonic-gui --from-git
--force` resolves through git+tag and already gets the latest fix).

**Status:** `open` (blocked by `mnemonic-toolkit-cratesio-publish`).

**Tier:** `v1+ / nice-to-have`.

### `gui-schema-effect-on-dropdown-options-vocab` — dropdown-option-disable Effect grammar for SPEC §6.6 rows 9/10/11

- **Surfaced:** 2026-05-16, GUI conditional-applicability v2 cycle (`mnemonic-gui-v0.6.0`) close. Filed per plan §6.10.7 closing list — unblocked by the v3 predicate-machinery (SlotCount* Predicate variants now expressible) but the *effect* side requires a new Effect grammar.
- **Where:** `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary extension); `mnemonic-toolkit/src/cmd/gui_schema.rs::VisibilityProjection` (toolkit emitter); `mnemonic-gui/src/schema_check.rs::VisibilityProjection` (GUI consumer); `mnemonic-gui/src/form/widget.rs` (Dropdown widget needs per-option-disable semantic).
- **What:** SPEC §6.6 rows 9/10/11 need to express "disable specific dropdown options" — e.g., row 9 disables `--threshold` values > N when slot-count is N; row 10 disables single-sig templates when N > 1; row 11 disables multisig templates when N == 1. The current v3 Effect grammar offers only `hidden` / `disabled` / `required` / `pin_value` — all of which act on the whole flag, not per-option. New Effect variant candidate: `disable_options: { values: [...] }` for Dropdown FlagKind.
- **Why deferred:** Out of v0.6.0 scope per plan; unblocked by this cycle's predicate-machinery. Requires SPEC grammar extension + GUI Dropdown widget rendering change.
- **Status:** `resolved f86a696` — Batch B-1 cycle (`mnemonic-toolkit-v0.18.0` + `mnemonic-gui-v0.7.0`, 2026-05-16) shipped the disable_options Effect grammar (rows 10/11) + GUI-internal NumberMax::FromSlotCount FlagKind extension (row 9). Schema bumps `v3 → v4`. Row 9 closes GUI-side without a toolkit wire-format change (Option A per the v0.7.0 design doc — single-consumer pragma; promotable to a toolkit-emitted Effect if a second `gui-schema` consumer ever appears). GUI consumer: `mnemonic-gui-v0.7.0` (`f86a696`) — VisibilityProjection::DisableOptions deserialize arm + Visibility::DisableOptions + NumberMax enum + render-time orthogonal composition (disabled_options extracted independently from primary first-rule-wins visibility, so --template can be both Required AND have DisableOptions). Toolkit emitter: `mnemonic-toolkit-v0.18.0` (`c7ac604`) — 2 new bundle_conditional_rules entries (count 11 → 13). Drift gate floors raised: bundle 11→13, total 34→36.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-effect-on-dropdown-options-vocab` (resolved at `c7ac604`).

### `gui-schema-cross-slot-predicate-projection` — cross-slot relational predicate types for SPEC §6.6 rows 8/13/14

- **Surfaced:** 2026-05-16, GUI conditional-applicability v2 cycle (`mnemonic-gui-v0.6.0`) close. Filed per plan §6.10.7 closing list — these rows need predicate types beyond the v3 `slot_count_*` extensions.
- **Where:** `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.2 (Predicate AST extension); `mnemonic-toolkit/src/cmd/gui_schema.rs::Predicate` (toolkit emitter); `mnemonic-gui/src/schema_check.rs::Predicate` (GUI consumer); `mnemonic-gui/tests/gui_schema_conditional_drift.rs::synthesize_satisfying` (drift gate extension).
- **What:** SPEC §6.6 rows 8/13/14 need relational predicates — row 8 (cross-slot equality, e.g., "two slots must NOT share an xpub"), row 13 (BIP-388 distinct-key invariant — all `@i` slots must be pairwise-distinct), row 14 (per-`@N` annotation consistency, e.g., "if `@1.xpub` is annotated `external`, all `@1.*` annotations must agree"). New Predicate variant candidates: `slot_subkey_distinct: { subkey: "xpub" }`, `slot_annotation_consistent: { annotation: "external" }`, etc.
- **Why deferred:** Predicate-machinery missing in v3; full design requires SPEC §6.10.2 grammar extension. Out of v0.6.0 cycle scope.
- **Status:** `resolved 38ad066` — Batch B-2 close 2026-05-16. **Row 8 resolved Option A** (`mnemonic-gui-v0.7.1` `38ad066`): GUI-internal `slot_editor.rs::detect_slot_index_gaps` helper + inline warning banner. Pure GUI-side pre-check; no toolkit wire-format change (mirrors the v0.7.0 row-9 NumberMax::FromSlotCount pattern). 9 new test cells in `tests/slot_editor_contiguity.rs`. **Rows 13/14 wontfix** with rationale: row 13 (BIP-388 distinct-key) requires xpub derivation that the GUI can't replicate for phrase-bearing slots (toolkit-binding-logic duplication is high-cost low-value); row 14 (per-`@N` annotation consistency) requires descriptor-string parsing + cross-slot annotation cross-reference (similarly high-cost low-value). Both surface authoritatively at CLI run-time per §6.6 rows 13/14 stderr. All v0.6.0-cycle-close FOLLOWUPs now closed (Batch A v0.6.1 + Batch B-1 v0.7.0 + Batch B-2 v0.7.1).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-cross-slot-predicate-projection` (resolved at toolkit master post-`38ad066`).

### `gui-schema-derive-child-meta-template-groups-spurious` — toolkit emits `meta.template_groups` on a subcommand with no `--template` flag

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle-close opus reviewer audit. Important finding (confidence 95): toolkit's `build_subcommand_meta` at `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:244-259` matches `name == "derive-child"` and emits a `template_groups` block, but `crates/mnemonic-toolkit/src/cmd/derive_child.rs` has ZERO `--template` references (grep-confirmed). SPEC §6.10.8 also lists derive-child as a template-consumer in error; toolkit test `derive_child_emits_meta_template_groups` enshrines the wrong invariant.
- **Where:** `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/gui_schema.rs:244-259` (the spurious match arm); `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.8 (matching mis-claim in prose); `mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_gui_schema_v3_extensions.rs` (the wrong-invariant test).
- **What:** Either (a) remove `derive-child` from the `build_subcommand_meta` match arm + delete the matching test cell + correct SPEC §6.10.8 prose, or (b) consciously document why derive-child gets the meta block despite having no `--template` widget. (a) is the source-faithful fix; the off-by-N pattern matches `[feedback-r0-must-read-source-off-by-n]`.
- **Why deferred:** Cosmetic — no GUI consumer reads derive-child's meta block today; the spurious emission is silent. Folding into the next toolkit cycle is lower-churn than cutting `mnemonic-toolkit-v0.17.1`.
- **Status:** `resolved 7ed3784` — toolkit-side fix shipped at `mnemonic-toolkit-v0.17.1` (2026-05-16). Took option (a): removed `derive-child` from `build_subcommand_meta` match arm (toolkit P0 `598b4ba`); deleted `derive_child_emits_meta_template_groups` test cell + added replacement negative-cell `derive_child_omits_meta_template_groups` as regression guard; corrected SPEC §6.10.8 paragraph 2 prose + added parenthetical noting the v0.17.1 correction. TDD discipline: negative cell ran RED against unmodified source (panic showed the spurious `multisig: [...], single_sig: [...]` block), GREEN after the match-arm fix. GUI-side picks up the cleaner JSON shape via the v0.17.1 pin bump at `mnemonic-gui-v0.6.1` (commit `6d57a89` for `pinned-upstream.toml` + commit `919866a` for the load-bearing `Cargo.toml` pin).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` `design/FOLLOWUPS.md` entry `gui-schema-derive-child-meta-template-groups-spurious` (to be filed at cycle close).

### `gui-persistence-autosave-debounce` — exit-only save loses session state on crash/SIGKILL; add debounced autosave (+ atomic temp+rename write)

- **Surfaced:** 2026-06-10, v0.35.0 persistence-wiring cycle (SPEC Decision 4: exit-only cadence accepted for the wiring cycle; the SIGINT/SIGTERM handlers DO route through `ViewportCommand::Close` → `on_exit` runs, so only hard kills lose state).
- **What:** add a debounced autosave (e.g. dirty-flag + N-second timer in `update()`, reusing the borrow-side `redact_for_persistence` construction — NEVER `mem::take`, see the v0.35.0 LOAD-BEARING order comment at the `on_exit` call site). While here: make `save()` atomic (write temp + rename) — the current `fs::write` can be torn by the signal handler's 3s `process::exit` grace; the v0.35.0 `.bak`-on-malformed leg already handles the torn result gracefully, so this is hardening, not a bug fix.
- **Status:** **resolved** `mnemonic-gui-v0.36.0` (2026-06-10) — change-gated autosave every 30s (`save_if_changed`: content-compare debounce, zero dirty-flag plumbing; cache updates only after Ok) + atomic `save()` (PER-PID sibling temp + `fs::rename` — the PID suffix is load-bearing: a shared temp name can atomically install torn content under two instances; no remove-then-rename fallback ever, `fs::rename` replaces on both platforms by documented contract). `build_persisted_state()` extraction keeps the v0.35.0 borrow-side/order invariants verbatim. SPEC `design/SPEC_gui_v0_36_0_autosave_atomic.md` (R0 3 rounds — r2 caught the spec's own fold-induced T1/PID mismatch whose cheapest green would have reversed the load-bearing fix).
- **Tier:** `GUI-local`

### `gui-flag-value-unset-serde-other-externally-tagged-dependency` — `#[serde(other)]` on externally-tagged FlagValue enum depends on undocumented serde behavior

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle-close opus reviewer audit (Important finding, confidence 85). The P3 forward-compat invariant ("v0.6+ readers map unknown FlagValue tags to Unset") depends on `#[serde(other)]` on `FlagValue::Unset` (a unit variant inside an externally-tagged enum). Per serde docs (https://serde.rs/variant-attrs.html): `#[serde(other)]` is "Only allowed on a unit variant inside of an internally tagged or adjacently tagged enum." Per serde issue #2010, on externally-tagged enums it "compiles but mysteriously doesn't work" — a request was filed to make it a compile-time error.
- **Where:** `src/schema/mod.rs:338-339` (the `#[serde(other)] Unset` variant); `tests/widget_unset_sentinel.rs:154-165` (`flag_value_unknown_tag_deserializes_to_unset_via_serde_other` — the forward-compat assertion); `CHANGELOG.md [0.6.0]` (the forward-compat claim).
- **What:** Empirical test passes at v0.6.0 — `serde_json::from_str::<FlagValue>(r#""FutureKitchenSink""#)` does return `FlagValue::Unset`, suggesting serde DOES handle the bare-string unit-variant fallback case correctly on externally-tagged enums. But this is undocumented behavior subject to silent change on future serde upgrades. Options for hardening: (a) bump `FlagValue` to an internally-tagged enum (breaks the wire shape — would need a persistence-schema-version bump); (b) write a custom Deserialize impl that explicitly handles the unknown-tag case (more code, but documented); (c) leave as-is + add a pinned serde version range + canary test that triggers on serde upgrades.
- **Now-live-relevant (2026-06-10, v0.35.0):** persistence wiring makes restored `state.json` values a LIVE deserialization surface for the first time — an unknown `FlagValue` tag in a stale file degrades per the `#[serde(other)]` fallback (or fails the whole parse → `.bak` + fresh default). This entry's hardening is now user-visible robustness, not just theory.
- **Status:** `resolved 919866a` — `mnemonic-gui-v0.6.1` P3 (2026-05-16). Chose option (c): canary pair in `tests/widget_unset_sentinel.rs`. Re-purposed existing `flag_value_unknown_tag_deserializes_to_unset_via_serde_other` cell (lines 154-165) as the load-bearing CANARY anchor + added 2 new cells: `flag_value_unset_canary_known_tags_still_deserialize_correctly` (regression guard) + `flag_value_unset_canary_unknown_tagged_object_currently_fails_to_deserialize` (NEGATIVE canary). **Empirical discovery this cycle**: serde's `#[serde(other)]` on externally-tagged `FlagValue` does NOT fall back tagged-object unknown variants — only bare-string ones. The v0.6.0 CHANGELOG forward-compat claim was therefore over-broad; CHANGELOG `[0.6.1]` narrows the scope to "future unit-variant additions only". Negative canary fires if a future serde upgrade DOES make tagged-object fallback work, at which point the v0.6.x forward-compat claim can be broadened.
- **Tier:** `v0.7+`
- **Companion:** None — gui-only.

### `gui-pin-value-effect-on-slot-flag-gap` — `assemble_argv` PinValue gate excludes `--slot` + drift gate lacks load-bearing rule count

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle-close opus reviewer audit (two related Important findings: confidence 80 each).
  - Finding A (PinValue / `--slot` gap): `src/form/invocation.rs::assemble_argv` wraps the visibility gate (including the new PinValue path) in `if flag.name != "--slot" || !subcommand.allows_slots {…}`. Future toolkit rules that target `--slot` with `pin_value` would silently fall through to the unguarded slot-emission branch, ignoring the rule. No current toolkit rule does this, but the gap is grep-detectable per `[feedback-r0-must-read-source-off-by-n]`.
  - Finding B (drift gate vacuous count): `tests/gui_schema_conditional_drift.rs:249-253` asserts `total_rules > 0` only; a regression that drops rules from ~34 to a non-zero handful would silently pass. Per `[feedback-ci-snapshot-test-substring-vacuity]`, this is a flagged class of project failure mode.
- **Where:** `src/form/invocation.rs:79-101` (the gate); `tests/gui_schema_conditional_drift.rs:249-253` (the assertion).
- **What:** Two defense-in-depth folds:
  - (A) Hoist the PinValue check above the slot-exemption guard, OR add `debug_assert!(!matches!(flag_vis, Visibility::PinValue { .. }))` inside the slot branch to fail-loud on future drift.
  - (B) Tighten the drift gate count: change `assert!(total_rules > 0, …)` to `assert!(total_rules >= 34, …)` (or similar load-bearing minimum derived from the actual v0.17.0 rule count), OR collect per-subcommand counts and assert individually.
- **Status:** `resolved 919866a` — `mnemonic-gui-v0.6.1` P3 (2026-05-16). Took **both** suggested folds: (A) `debug_assert!` + release-mode `if-suppress` at `src/form/invocation.rs:106-128` slot-emit branch (NOT hoist — pin_value's single-value emission semantic doesn't map onto `--slot`'s multi-row `@N.subkey=value` grammar; hoist would emit malformed argv). (B) per-subcommand floors at `tests/gui_schema_conditional_drift.rs:261-282` matching v0.17.1 baseline (`bundle ≥ 11`, `verify-bundle ≥ 10`, `export-wallet ≥ 6`, `convert ≥ 4`, `derive-child ≥ 3`; total ≥ 34) — bumped only on intentional rule-reduction cycles (rare). Added `use std::collections::BTreeMap` import. A future cycle wanting legitimate pin_value-on-slot semantics must remove the debug_assert and replace with the new design; the loud-fail-on-encounter makes that requirement visible.
- **Tier:** `v0.7+`
- **Companion:** None — gui-only.

## Deferred to v0.3+

Named for explicit closure per SPEC §14. Carried forward from v0.1
because not in v0.2 scope, or carried forward from v0.2 because
shipped partially.

### `gui-help-icon-per-flag-affordance` — extend help-icon coverage to every flag if Option C selective placement proves insufficient

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §2.7 (in-flight; archived to design/ at PE close).
- **Where:** `src/form/widget.rs` widget render. v1.0 ships Option C: per-subcommand `?` button + per-dropdown/NodeValueComposite/TaggedOrIndexed `?` button + per-repeating-field-flag `?` button (28+43+20=91 buttons). Per-flag `?` buttons would add ~100 more buttons across all 28 form views.
- **What:** If user feedback after v1.0 ships surfaces that hover-tooltip alone is insufficient for non-dropdown flags (e.g., users want click-through deep-links for `--passphrase`, `--json-out`, secret-bearing flags), extend Option C to Option A: per-flag `?` buttons on every FlagSchema.
- **Why deferred:** v1.0 ships Option C to balance UX-budget vs visual clutter (91 buttons / ~3 per visible form is sustainable; 200 buttons / ~7 per visible form is chaos). Wait for user feedback.
- **Status:** `open`
- **Tier:** `v1.1+`
- **Companion:** `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §1.6.

### `gui-manual-base-url-runtime-override` — `--manual-base-url` runtime flag if build-time env-var override proves insufficient

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §2.7.
- **Where:** `src/help/url.rs` MANUAL_BASE_URL constant + `src/main.rs` CLI argument parsing. v1.0 ships build-time env-var override `MNEMONIC_GUI_MANUAL_BASE_URL` via `option_env!` (CI staging vs prod). No runtime flag.
- **What:** If users in air-gapped environments need to point the GUI's help icons at a locally-hosted mirror (e.g., a corporate intranet copy of the manual), add a `--manual-base-url <URL>` runtime flag that overrides the compile-time default. Runtime override would also help self-hosting users without rebuilding from source.
- **Why deferred:** v1.0 ships with a stable GitHub Pages URL. Self-hosting / air-gap is a niche use case; defer until concrete demand surfaces.
- **Status:** `open`
- **Tier:** `v1.1+`
- **Companion:** `mnemonic-toolkit/design/PLAN_manual_gui_v1.md` §1.5 + §2.4.

- `gui-code-signing-mac-developer-id` — v0.1.x and v0.2.0 ship
  unsigned macOS binaries; users need to right-click → Open or
  `xattr -d com.apple.quarantine` on first launch (see
  `docs/onboarding/macos-gatekeeper-walkthrough.md`). v0.3+ plan:
  paid Apple Developer ID + notarization roundtrip.
- `gui-code-signing-windows` — v0.1.x and v0.2.0 ship unsigned
  Windows binaries; users need to click SmartScreen "More info →
  Run anyway" on first launch (see
  `docs/onboarding/windows-smartscreen-walkthrough.md`). v0.3+ plan:
  Authenticode certificate (EV variant for SmartScreen reputation).
- `gui-os-snapshot-secret-occlusion-linux` — v0.2 Phase B.2 shipped
  macOS (`NSWindowSharingType::NSWindowSharingNone` via
  `objc2-app-kit`) and Windows
  (`SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` via
  `windows-rs`) occlusion. Linux has no compositor API for this at
  v0.2 — see `src/platform.rs` cfg-not-any branch for the deferral
  notice and the paste-warn modal copy that surfaces the gap to
  users. Tracking entry kept open for the Linux-specific
  follow-up.

### `gui-bundle-multisig-flags-conditional` — `--multisig-path-family` and `--threshold` should be Disabled (conditional-visibility) under single-sig templates

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle M-P2.4 sub-batch 5b R0 fold (worked example for `mnemonic bundle` single-sig had to add an explicit "clear `--multisig-path-family`" step because the field is seeded to `bip87` by default at `src/main.rs:188-211`, and leaving it set under `--template bip84` triggers the `mode_text::PATH_FAMILY_WITHOUT_MULTISIG` refusal).
- **Where:** `src/form/conditional.rs::bundle` (line 21-45). The current rules enforce `--template`-required-unless-descriptor, `--descriptor`/`--descriptor-file` XOR, and `--passphrase`/`--passphrase-stdin` XOR. They do NOT disable `--multisig-path-family` or `--threshold` when the active template is in the single-sig set (`bip44`, `bip49`, `bip84`, `bip86`).
- **What:** Extend `pub fn bundle(state: &FormState) -> FlagVisibility` to disable `--multisig-path-family` and `--threshold` when `state.dropdown_value("--template")` is in the single-sig template set. Mirror the same fix in `verify_bundle` (same constraint applies). The argv assembler will then skip these fields (per `form/invocation.rs::emit_one`'s "empty / false / absent values are NOT emitted" rule at the schema docstring) and the user no longer needs to manually clear the seeded default.
- **Why deferred:** Surfaced AFTER v0.3.0 ship; a reasonable fix but not blocking the manual-gui v1.0 cycle. v1.0 manual instead documents the manual-clear workaround.
- **Status:** `resolved 6c2d019` — closed by the GUI conditional-applicability v1 cycle (mnemonic-gui v0.5.0 + mnemonic-toolkit v0.16.0 lockstep, in-flight). P2 (`16b15de`) extended `bundle()` + `verify_bundle()` + `export_wallet()` with single-sig-template Disabled rules + single-sig-template + descriptor-mode mutexes. P3 (`f2a985b`) added the `assemble_argv` visibility gate that suppresses Hidden/Disabled flags from argv emission. P5 (`2afd603`) removed the `--multisig-path-family bip87` default seed at `main.rs:203` (the root of the surfacing). The manual workaround documented in the worked example may now be retired in a future manual cycle.
- **Tier:** `v0.4`
- **Companion:** `mnemonic-toolkit/docs/manual-gui/src/40-mnemonic/42-bundle.md` worked-example step 3 documents the workaround and cites this FOLLOWUP; superseded by the v1 cycle.
- **Successor:** `gui-conditional-applicability-drift-fix` (this file, above) is the mechanism + drift-gate generalization of which this entry is the originating specific case.

### `gui-import-wallet-env-var-secret-channel` — auto-rewrite literal seeds in repeating `--ms1` widgets to `@env:MNEMONIC_MS1_<i>` sentinels + spawn-time env-var injection

- **Surfaced:** 2026-05-18, Phase 6 R0 architect review C1 (mnemonic-toolkit v0.26.0 wallet-import cycle).
- **Where:**
  - `src/runner.rs:74-114` — current spawn flow injects only `MNEMONIC_FORCE_TTY`; no per-cosigner secret env-var bag.
  - `src/form/invocation.rs:236-251` — repeating-secret branch routes values verbatim through `state.values`.
  - `src/main.rs:683-688` — run-confirm modal renders argv verbatim (per `[[feedback-run-confirm-modal-renders-argv-verbatim]]`).
  - `tests/kittest_import_wallet_form.rs:44-46,154-213` — module-doc cites this FOLLOWUP; cell `cell_import_wallet_repeating_ms1_argv` pins the literal-pass-through contract until this FOLLOWUP lands.
  - `mnemonic-toolkit/design/SPEC_wallet_import_v0_26_0.md` §9.3 — describes the aspirational behavior (toolkit-side accepts `@env:VAR` sentinels at parse time, but GUI does NOT pre-rewrite in v0.11.0).
  - `mnemonic-toolkit/docs/manual-gui/src/40-mnemonic/4c-import-wallet.md` (post-Phase-6-R0-fold) — documents the v0.11.0 user-must-type-explicitly fallback.
- **What:** v0.12.0+: on subprocess spawn, collect per-cosigner-index secret values from `--ms1` repeating-widget state into a per-spawn env-var bag (`MNEMONIC_MS1_<i>=<value>`), rewrite `args[--ms1+1]` to `@env:MNEMONIC_MS1_<i>` sentinels, render the sentinel-bearing argv in the run-confirm modal (so the raw seed never appears), drop the env-vars on subprocess exit. Same pattern for `--passphrase`, `--share` (slip39-combine, seed-xor-combine), and other secret-bearing repeating flags. Toolkit-side already accepts the sentinel at parse-time per the cross-cutting Phase 1 `resolve_env_var_sentinel` helper.
- **Why deferred:** v0.26.0 scope was wallet-import-side parse + watch-only invariant + round-trip discipline; the env-var-channel rewrite is GUI-side runner work that affects ALL repeating-secret surfaces, not just `--ms1`. Pre-existing `gui-run-confirm-modal-secret-redaction` covers the modal-redaction direction; this FOLLOWUP covers the argv-rewrite direction. Both need to land together in v0.12.0.
- **Status:** open
- **Tier:** `v0.12.0`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::gui-import-wallet-env-var-secret-channel` (cross-citing companion). v0.11.0 manual prose at `mnemonic-toolkit/docs/manual-gui/src/40-mnemonic/4c-import-wallet.md` documents the user-must-type-explicitly fallback.

### `gui-run-confirm-modal-secret-redaction` — run-confirm modal renders secret-bearing argv tokens in plaintext (security-relevant gap)

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle M-P2.4 batch 4 R0 source-grep. The `mnemonic-toolkit/docs/manual-gui/src/10-foundations/14-secret-handling.md` Defense-2 prose (LOCKed in M-P2.4 batch 2) claims the run-confirm modal "shows the assembled argv with secret values replaced by `***`". `src/main.rs:512-535` shows the modal renders each argv token verbatim in monospace via `ui.monospace(format!("  {}", tok))`; no redaction step exists anywhere in the source tree (`grep -rn "redact" src/` returns only `persistence.rs` on-disk-save paths).
- **Where:** `src/main.rs:512-535` (modal render block); `src/secrets.rs:65-66` (`RUN_CONFIRM_MODAL_PREFIX` const has no continuation that would substitute a redacted argv); `src/form/invocation.rs:42-100` (`assemble_argv` returns the full plaintext argv including secret-class flag values).
- **What:** Add a redaction step that mirrors `persistence::redact_for_persistence`'s flag-class logic so the modal displays e.g. `--passphrase ***` instead of `--passphrase the-actual-secret-mnemonic`. Two implementation options: (a) build a parallel `redact_argv_for_display(sub, state, &argv)` in `secrets.rs` and call it from the modal site only — preserves the actual `argv` that's passed to `spawn_and_capture` after Run-confirm; (b) inline a per-token check in the modal render loop using `secrets::flag_is_secret` against the preceding flag-name token. Option (a) is cleaner; option (b) is smaller-LOC. Either way the secret-class boundary already exists.
- **Why deferred:** Surfaced AFTER v0.3.0 ship; remediation requires (i) a new `mnemonic-gui` cycle (`mnemonic-gui-v0.4.0` or a v0.3.1 patch) and (ii) lockstep manual prose patch landing in the `manual-gui-v1.0` PR's batch-4 commit so the v1.0 manual ships consistent with what shipped GUI v0.3.0 actually does. Until the GUI fix lands, the manual MUST describe the actual (undesired) behavior plus an operational mitigation: only run the GUI on a cold/airgapped machine where on-screen secret display does not constitute a network-exfiltration vector. Compromise: the v1.0 manual ships honestly-broken; v1.1 ships fixed. Severity is high but not P0-block-v1.0 for the manual cycle because the manual cannot fix the GUI behavior — only describe it.
- **Status:** `open`
- **Tier:** `v0.4-cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` `gui-run-confirm-modal-secret-redaction-manual-companion`; `mnemonic-toolkit/docs/manual-gui/src/10-foundations/14-secret-handling.md` Defense-2 prose patch in M-P2.4 batch 4 commit. Closure requires: (i) GUI source patch implementing redaction; (ii) manual prose patch undoing the v1.0 honest-broken framing and restoring the `***` claim; (iii) `pinned-upstream.toml` bump in this manual to whatever GUI tag ships the fix.

## Resolved in v0.2

- `gui-secret-buffer-allocator-residue` — **shipped Phase B.1.**
  `SecretLineEdit` widget backed by `Zeroizing<Vec<u8>>` replaces
  the v0.1 best-effort-on-`String` `SecretBuffer`. Buffer zeroes on
  drop / form reset / app exit. Excluded from `Serialize` /
  `Debug` derives; never persisted to disk via
  `redact_for_persistence`. See `src/form/secret_widget.rs`.
- `gui-os-snapshot-secret-occlusion` (macOS + Windows) —
  **shipped Phase B.2.** macOS uses
  `NSWindowSharingType::NSWindowSharingNone` via `objc2-app-kit`;
  Windows uses `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`
  via `windows-rs`. Both applied from `MnemonicGuiApp::new()`.
  Linux gap moved to a separate entry (above) since the platform
  has no compositor API for this.
- `gui-headless-test-harness-evaluation` — **shipped Phase A.3.**
  `egui_kittest` v0.31.1 dev-dep + `accesskit` feature on
  `eframe`. Five widget-driving cells across
  `tests/widget_interaction.rs` (slot editor, conditional
  visibility, `ms encode` argv, `md encode` dropdown) and
  `tests/widget_secret.rs` (paste-warn modal). See
  `gui-accesskit-production-side-effect` (above, Active) for the
  production-side-effect note that accepting `egui_kittest`
  introduced.
- `gui-schema-json-subcommand-evaluation` — **shipped Phase
  C.1 / C.2 / C.3.** `<cli> gui-schema` subcommand on each of the
  four sibling CLIs emits a SPEC §7 JSON envelope
  (`{version:1, cli, subcommands:[{name, flags, positionals}]}`).
  GUI consumes via `src/schema_check.rs::json_flag_names`. Falls
  back to v0.1 regex-on-`--help` if the binary lacks `gui-schema`
  or exits non-zero. Schema-mirror CI gate now runs
  `<cli> gui-schema | python3 -c 'json.load...'` smoke for each
  CLI before the in-process test suite.
- **15 sibling-CLI subcommands** — **shipped Phase D.1 / D.2 /
  D.3 / D.4.** D.1 audited `--help` across `ms` (×4) + `mk` (×4)
  + `md` (×7). D.2 + D.3 added the schema entries to
  `src/schema/{ms,mk,md}.rs`. D.4 added two egui_kittest cells
  (`ms encode` argv-assembly + `md encode` dropdown
  value-inspect) covering representative new surface. All 15
  subcommand tabs render in the GUI at v0.2.

### `gui-tree-key-egui-undo-ring-residue` — egui undo ring may retain tree-key keystrokes after model zeroize

- **Surfaced:** 2026-06-21, cycle-11a (M9 follow-on). The cycle-11a M9 fix (`gui-tree-key-not-zeroized-on-exit`) scrubs the model-side `TreeNode.key`/`.keys[i]` `String`s on exit, but the egui `text_edit_singleline` widgets rendering those fields (`src/form/tree_form.rs`) maintain an internal **undo ring** that may retain prior keystrokes after the model `String` is zeroized.
- **Class:** identical to the already-tracked `gui-secret-buffer-allocator-residue` / `gui-os-snapshot-secret-occlusion` caveats (referenced in `PASTE_WARN_MODAL_TEXT`). The values/slots/positionals/secret_widgets the exit sweep already covers carry the same undo-ring caveat; M9 brought `state.tree` to parity with them. A separate, deferred mitigation.
- **Status:** open (deferred).

### `gui-canonicity-regex-coarse-pin-gate-not-validator` — canonicity regex is intentionally a coarse pin-gate, not a full validator

- **Surfaced:** 2026-06-21, cycle-11a (L12 follow-on). `src/form/conditional.rs::classify_descriptor_canonicity` is by design a COARSE pin-gate for the `--account → PinValue(0)` push (the toolkit is the authority on canonicity). After the L12 fix it over-accepts the double-origin form `[fp]@N[fp]` (classifies Canonical). This is benign under both toolkit versions: the pinned v0.60.0 ACCEPTS the double-origin form (its suffix-only lexer silently skips a leading `[fp]`), and master/v0.62.0+ REFUSES it at parse ("supply exactly one"), so the pin value is moot.
- **Alternative (deferred):** replace the regex with a single oracle — shell out to `gui-schema --classify-descriptor` — making the GUI exactly track the toolkit. A larger change (a subprocess per keystroke / classification site) for zero behavioral gain over the coarse gate, since v0.62.0+ already enforces the tightening toolkit-side. Doc-only record.
- **Status:** open (deferred, doc-only).

## Process notes

### v0.2: enforce PR-CI gate before tag-push

**Phase 10 R1 I-2 finding (confidence 85).** v0.1.0 was tagged via direct
push to master on a fresh repo, bypassing the `pull_request` build.yml
trigger that SPEC §B.12 R1 I-3 fold explicitly required ("PR must pass
full matrix BEFORE tag"). For v0.1.0 on a fresh repo with no prior master
history, this was mechanically the only path. For v0.2 and beyond — when
master has history and PRs are the normal flow — feature work must land
via PR with full 5-target CI green before tagging. This entry exists so
the v0.2 release prep doesn't repeat the v0.1 deviation.

## Resolved

### `gui-runner-debug-logs-unmasked-secret-argv` — runner `debug!`-logged the full spawn argv in cleartext (RESOLVED v0.45.0)

- **Surfaced:** 2026-06-20, constellation bug-hunt (H2). **Resolved:** `mnemonic-gui-v0.45.0` (2026-06-21, cycle-3).
- **Where:** `src/runner.rs` `run_with_stdin` — `debug!(target: "mnemonic_gui::runner", argv = ?argv, …, "subprocess spawn")` Debug-formatted the ENTIRE argv. Under `--debug` / `RUST_LOG=…=debug`, secret-bearing argv tokens (BIP-39 phrase / entropy / WIF / minikey) printed verbatim to stderr (terminal / journald / log file).
- **Fix:** the spawn log now emits `program = %argv[0]` (the resolved binary — never secret) + `argv_len` + `stdin` only; zero secret bytes, no `run_with_stdin` signature/behavior change. Pinned by `runner_no_argv_leak` in `tests/runner_integration.rs` (capture-gated sentinel-absence + `argv_len` positive control, reusing the `cell_2` 3-attempt race harness).
- **Report:** `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (H2).

### `gui-minikey-secret-not-masked-in-argv-preview` — minikey (Casascius mini PRIVATE KEY) leaked across 4 GUI surfaces (RESOLVED v0.45.0)

- **Surfaced:** 2026-06-20, constellation bug-hunt (H3; folds the Wave-3 `w3-gui-minikey-persist-plaintext` facet). **Resolved:** `mnemonic-gui-v0.45.0` (2026-06-21, cycle-3).
- **Where:** the GUI classified composite-node secrecy via the narrow `mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES`, which OMITS `minikey`. So `convert --from minikey=<key>` fell through four surfaces: argv-mask (unmasked in Preview / last-run), `should_confirm_run` (no run-confirm), paste-warn (never fired), and `redact_for_persistence` (written **plaintext** to `~/.config/mnemonic-gui/state.json`).
- **Fix:** new `secrets::node_type_is_argv_secret` predicate backed by the wider `SECRET_NODE_TYPES_ARGV` (= narrow + minikey, already present at the pinned `mnemonic-toolkit-v0.60.0`) routes argv-mask (`form/invocation.rs`), run-confirm (`secrets::should_confirm_run`), and persist-redaction (`persistence.rs`, autosave + on-exit); the composite value widget gained node-aware paste detection (see `composite-paste-warn-parity`). Compile-time drift-guard mod `argv_canonical_fallback` + `secret_taxonomy_pin` assertions prevent silent re-narrowing. No toolkit pin bump; no `schema_mirror` delta.
- **Report:** `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (H3).

### `gui-tree-key-not-zeroized-on-exit` — node-tree builder key material never scrubbed in RAM on exit (RESOLVED v0.46.0)

- **Surfaced:** 2026-06-20, constellation bug-hunt (M9). **Resolved:** `mnemonic-gui-v0.46.0` (2026-06-21, cycle-11a).
- **Where:** `src/secrets.rs::zeroize_form_state` swept `values`/`slots`/`positionals`/`secret_widgets` but never referenced `state.tree`. The node-tree builder's `TreeNode.key` / `.keys[i]` (plain `String`s accepting xprv/WIF/raw-hex PRIVATE keys) were therefore never zeroized in RAM on app shutdown. (On-disk was already defended via `redacted_for_persistence`; this was the strictly-in-memory non-scrub.)
- **Fix:** new recursive `TreeNode::zeroize_keys` (`src/form/tree_model.rs`, sibling to the on-disk redactor `blank_non_extended_public_keys`) zeroizes `key` + every `keys[i]` through all `children`; wired into the exit sweep via `state.tree.as_mut()`. The public `hex` digest is deliberately excluded (commitment, never a private preimage — matches on-disk redaction). `String: Zeroize` is live (zeroize 1.8.2). Pinned by `zeroize_form_state_clears_tree_keys` in `tests/secrets.rs` (recursion + `hex`-untouched guard). No toolkit pin bump; no `schema_mirror` delta. **Deferred residue:** the egui `text_edit_singleline` undo ring (`gui-tree-key-egui-undo-ring-residue`).
- **Report:** `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (M9).

### `gui-canonicity-regex-suffix-origin-misclassify` — canonicity regex missed the suffix-origin form `@N[fp/path]` (RESOLVED v0.46.0)

- **Surfaced:** 2026-06-20, constellation bug-hunt (L12). **Resolved:** `mnemonic-gui-v0.46.0` (2026-06-21, cycle-11a).
- **Where:** the three single-key canonicity regexes in `src/form/conditional.rs` (`pkh`/`wpkh`/`tr`) anchored the optional origin bracket BEFORE `@N` (prefix form `[fp]@N` only). The pinned toolkit ALSO accepts the SUFFIX form `@N[fp/path]` and classifies it canonical (empirically, v0.60.0: `gui-schema --classify-descriptor` → `canonical`). The GUI misclassified it NonCanonical → LIFTED the `--account → PinValue(0)` push → user `--account N` reached the toolkit → confusing hard-error `DESCRIPTOR_WITH_NONZERO_ACCOUNT` on a valid input (NOT a silent wrong-address).
- **Fix:** insert a SECOND optional origin bracket between `@\d+` and the multipath group (byte-identical to the prefix bracket; the prefix group is kept). The wsh/sh-wsh multisig regexes carry no origin bracket and are untouched. Added a suffix-form Canonical fixture to `tests/canonicity_drift.rs` (the live GUI-vs-toolkit parity gate that should have caught L12 — the prefix-only corpus let GUI + toolkit happen to agree) and a `suffix_form_origin_descriptor_pins_account_to_zero` pin test (+ prefix/no-origin positive controls). The GUI regex remains a coarse pin-gate (the toolkit is the authority); the double-origin over-acceptance is benign under both the pin (v0.60.0 ACCEPTS) and master/v0.62.0+ (REFUSES at parse) — see `gui-canonicity-regex-coarse-pin-gate-not-validator`. No toolkit pin bump.
- **Report:** `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (L12).

### `gui-convert-from-dropdown-missing-seedqr` — `convert --from seedqr` unreachable from the GUI (RESOLVED v0.46.0)

- **Surfaced:** 2026-06-20, constellation bug-hunt (L13). **Resolved:** `mnemonic-gui-v0.46.0` (2026-06-21, cycle-11a).
- **Where:** a single shared `NODE_TYPES` const (seedqr-free) in `src/schema/mnemonic.rs` backed BOTH `--from` (`NodeValueComposite`) and `--to` (`Dropdown`). The toolkit ACCEPTS `--from seedqr=<digits>` (`NodeType::as_str` lists `seedqr` at index 1) but REJECTS `--to seedqr` (its `--to` `PossibleValuesParser` omits it — seedqr is decode/input-only). So `--from seedqr` was un-offerable from the GUI.
- **Fix:** split the list — `CONVERT_FROM_NODES` (14, seedqr@1, mirrors the INPUT enum `NodeType::as_str`) backs `--from`; `CONVERT_TO_NODES` (13, seedqr-free, mirrors the `--to` `PossibleValuesParser`) backs `--to` (value-preserving rename of the old `NODE_TYPES`). Pinned by `convert_from_dropdown_includes_seedqr` + the `convert_to_dropdown_excludes_seedqr` leak-guard in `tests/convert_node_dropdowns.rs`. **No toolkit pin bump / no `schema_mirror` trigger:** seedqr present since toolkit v0.31.6 < pinned v0.60.0; `--from` uses a custom `parse_from_input` value_parser (gui-schema emits NO `--from` value enum); `--to` keeps the seedqr-free list (schema_mirror compares flag NAMES only, and `schema_mirror_secret_drift` the per-(sub,flag) secret-bit — both stay GREEN).
- **Report:** `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (L13).

### gui-combobox-id-collision (resolved in v0.1.2 by from_id_salt switch)

**Symptom (reported 2026-05-12, post-v0.1.1):**

> "There is a bug involving every dropdown list. No list opens and
> sometimes every list on the page gets highlighted when one list is
> clicked on."

**Root cause:** The three `egui::ComboBox` instances in
`src/form/widget.rs` (the `FlagKind::Dropdown` selector at line 26, the
`FlagKind::NodeValueComposite` node selector at line 60, and the
`FlagKind::TaggedOrIndexed` tag selector at line 84) all used
`ComboBox::from_label("")` or `from_label(" ")`.
`ComboBox::from_label(label)` derives the egui widget ID from `label`,
and egui keys popup open-state, hover-state, and selection-state by ID.
All ComboBoxes sharing the same `""`/`" "` label thus shared an ID:

- "no list opens" — egui couldn't disambiguate which popup-state to
  drive when the click landed on a widget with a non-unique ID.
- "every list on the page gets highlighted when one is clicked" — the
  hover and selection state propagated to every widget sharing the ID.

**Fix:** Switched each of the three sites to
`ComboBox::from_id_salt((const, flag.name))` — the
`flag.name: &'static str` field is unique per `FlagSchema`, so each
ComboBox gets a unique egui widget ID. This matches the convention
already used by `src/form/slot_editor.rs:160`, which had been correct
since v0.1.0 (`from_id_salt(("slot_subkey", i))`).

**Audit pinned at `tests/dropdown_id_salt.rs`:** the test reads
`src/form/widget.rs` and asserts (a) no `ComboBox::from_label` calls
remain and (b) `ComboBox::from_id_salt` is used. Future regressions —
e.g., someone reaching for the quicker-typing `from_label("")` again —
fail the audit at test-time.

**Out of scope (left intentionally):** `src/main.rs:291` uses
`ComboBox::from_label("subcommand")`. The label is non-empty and
unique, and there is only one such ComboBox in the application, so no
ID collision occurs. Not touched by this hotfix; the
`from_id_salt`-everywhere stylistic sweep can be a v0.2+ janitorial
follow-up if desired.

**Files changed in v0.1.2:** `src/form/widget.rs` (3 `from_label` →
`from_id_salt` swaps), `tests/dropdown_id_salt.rs` (new audit),
`Cargo.toml` (version bump 0.1.1 → 0.1.2), `CHANGELOG.md` (`[0.1.2]`
entry), this `FOLLOWUPS.md` (Resolved entry).

### gui-glow-wayland-loop-broken (resolved in v0.1.1 by renderer swap)

**Symptom:** With `eframe = "0.29"` + `egui_glow` renderer on KDE/KWin
Wayland, the eframe event loop went stuck after the first 1-2 paint
cycles. Cross-thread `Context::request_repaint()` and
`Context::send_viewport_cmd(ViewportCommand::Close)` calls were silently
dropped — they didn't wake winit's event loop. Symptoms observed during
v0.1.1 dev:

- `update()` called 2 times at startup, never again over 90+ seconds of
  runtime (despite a background keepalive thread calling
  `request_repaint()` at perfect 1 Hz cadence).
- KWin sent `xdg_toplevel.close` via the wayland protocol after a
  Scripting `closeWindow()` call — the GUI process did not process the
  close, did not call `on_exit()`, and stayed alive until SIGKILL.
- Signal-hook handler thread sent `ViewportCommand::Close` on SIGINT —
  ignored the same way; only a `process::exit(130)` fallback after 3 s
  could terminate the process.
- KDE's title bar marked the window "Not Responding" because the
  surface stopped committing frames between input events.

**Root cause:** Bug in the `egui_glow`/`egui_winit` wayland integration's
cross-thread wakeup. Verified across `eframe = "0.29"`, `"0.30"`, and
`"0.31"` — same broken behavior in all three.

**Fix:** Switched eframe to the `wgpu` renderer (Vulkan via Mesa) by
configuring `eframe = { version = "0.31", default-features = false,
features = ["wgpu", "default_fonts", "wayland", "x11"] }` in Cargo.toml.
With wgpu:

- `update()` runs at the keepalive's 1 Hz cadence (CPU still ~0 % at idle)
- Cross-thread `request_repaint()` works
- Cross-thread `send_viewport_cmd(Close)` works
- SIGINT/SIGTERM → handler → `ViewportCommand::Close` → `on_exit()`
  fires cleanly within ~2.5 s (well under the 3 s timeout grace)

A residual cosmetic issue: `egui_wgpu` logs `Dropped frame with error:
A timeout was encountered while trying to acquire the next frame` at the
1 Hz keepalive cadence. These are suppressed at the default WARN level
via the `init_tracing` filter (`wgpu_hal=error,egui_wgpu=error`); only
visible under `--debug` / `RUST_LOG=info`. They don't affect
functionality.

**Files changed in v0.1.1:** `Cargo.toml` (eframe feature flags +
signal-hook), `src/main.rs` (signal-hook handler, keepalive thread,
on_exit signature for wgpu renderer, tracing filter for wgpu warnings).

### `gui-workflow-trigger-include-release-branches` — CI gates silently skip PRs targeting release branches

- **Surfaced:** 2026-05-19, v0.11.0 cycle — discovered mid-G2/G3 when no CI workflows queued for 14+ min after force-pushes on PR #6 + PR #5 against base `release/v0.11.0`.
- **Where:**
  - `.github/workflows/build.yml` — `on: pull_request: branches: [master]`
  - `.github/workflows/schema-mirror.yml` — `on: pull_request: branches: [master]`
- **What:** Both workflow files currently filter `pull_request: branches: [master]`, which means **no CI fires for PRs targeting `release/v0.11.0`** (or any future integration branch). v0.11.0 cycle worked around this via local pre-merge vetting (`cargo build` + `cargo clippy --all-targets -- -D warnings` + `cargo test` with `MNEMONIC_BIN` pointing at the v0.26.0 toolkit binary) plus `--admin` merges against the integration branch. The integration PR (`release/v0.11.0 → master`) DID trigger workflows normally (base=master), so the load-bearing schema-mirror gate worked. Fix: extend trigger filter to `branches: [master, release/*]` so per-PR CI runs on integration branches too. Reduces reliance on out-of-band local vetting in future multi-instance cycles.
- **Why deferred:** Cycle workaround was sound and architecturally consistent (per merge-plan §G3.5.2, the integration PR is the load-bearing gate); the per-PR gate failures during G1/G2/G3 were already known to be structurally red. Trigger-filter fix is a future-cycle ergonomics improvement.
- **Status:** resolved (mnemonic-gui-v0.11.1; commit 5254b59 — v0.27.2 + v0.11.1 lockstep cycle close)
- **Tier:** `v0.12` (next GUI cycle).
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::gui-workflow-trigger-include-release-branches`.

### `gui-timestamp-default-value-drift-v0.47.3` — `export-wallet --timestamp` `default_value: Some("now")` silently suppressed an explicit `now` after toolkit v0.47.3 flipped the default to `0`
- **Surfaced:** 2026-06-06, toolkit-v0.47.3 R0 round-1 (I2). NOT caught by `schema_mirror` (gates flag-NAMES + value-enums, not `default_value`).
- **Where:** `src/schema/mnemonic.rs:1044` (`export-wallet --timestamp`, `FlagKind::Timestamp, default_value`) + the D33 default-suppression `src/form/invocation.rs:78`.
- **What:** toolkit v0.47.3 flipped `export-wallet --timestamp` default `now → 0`. With the GUI schema still declaring `default_value: Some("now")`, `is_at_default` treated an **explicit** `TimestampValue::Now` as at-default and dropped `--timestamp` from argv → the pinned toolkit then applied its new `0` default → the user's explicit `now` was silently discarded.
- **Status:** `resolved` mnemonic-gui-**v0.28.0** (this cycle, alongside the toolkit pin bump v0.46.2 → v0.47.3). Fix = `src/schema/mnemonic.rs` `--timestamp` `default_value "now" → "0"` (+ help string + `pinned_version` banner + module-doc header). **MINIMAL — no `is_at_default` change:** `widget.rs::default_flag_value_for_flag` seeds `FlagKind::Timestamp → FlagValue::Unset`, so the default export-wallet form emits no `--timestamp` (toolkit applies `0`); an explicit `Now` now correctly emits `--timestamp now`; `Unix(n)` always emits. Regression guards: `tests/argv_assembler.rs::{d33_timestamp_now_is_emitted_when_default_is_zero, cell_3b_export_wallet_timestamp_now_argv}` (inverted RED→GREEN). **No manual-gui change in this repo** (the GUI repo has no `docs/`); the stale `now`-default PROSE in the toolkit repo's `docs/manual-gui/src/40-mnemonic/45-export-wallet.md:30,340-343` is tracked separately by `mnemonic-toolkit/design/FOLLOWUPS.md::manual-gui-export-wallet-timestamp-default-now-stale`. Audit trail: `design/SPEC_gui_v0_28_0_pin_bump_v0_47_3.md` + `design/agent-reports/gui-v0_28_0-pin-bump-r0-round{1,2}-review.md`.
- **Tier:** `cross-repo`.
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::gui-timestamp-default-value-drift-v0.47.3` (resolves this side).

### `canonicity-drift-gate-floor-too-lenient` — F2 drift gate's 50% floor let a broad toolkit-parser regression pass silently
- **Surfaced:** 2026-05-17, toolkit v0.20.0 cycle Phase 5 end-of-cycle opus review I4 (filed toolkit-side; this is the lockstep GUI companion).
- **Where:** `tests/canonicity_drift.rs` — the prior end-of-test floor `assert!(classified >= FIXTURES.len() / 2, …)` (was `:131-136`; the toolkit FOLLOWUP's snapshot `:138` had drifted).
- **What:** The drift gate iterates 18 canonical/non-canonical fixtures, shells each to `mnemonic gui-schema --classify-descriptor`, and asserted (a) GUI-vs-toolkit agreement on the classifiable ones + (b) a count floor of `≥ FIXTURES.len()/2` (= 9). The floor meant a broad toolkit-parser regression — where 9+ of 18 fixtures silently start parse-failing — still passed (`feedback-ci-snapshot-test-substring-vacuity`: tight floors). The toolkit FOLLOWUP's right answer was a per-fixture classified-expectation table, not a count floor.
- **Status:** `resolved` — `mnemonic-gui` master (test-only, **no version bump / no tag**). Replaced the floor with a per-fixture `enum Expect { Canonical, NonCanonical, ParseFails }` table (`const FIXTURES: &[(&str, Expect)]`, 11 Canonical / 4 NonCanonical / 3 ParseFails = 18) and four accumulators: `newly_parsed` (a `ParseFails` fixture that now parses → "promote it" FAIL), `regressed` (an expected-classify fixture that now parse-fails → the broad regression the floor tolerated), `wrong_verdict` (toolkit verdict ≠ pinned `want` → pins the *absolute* canonical↔non-canonical verdict, stronger than agreement-only), and `disagreements` (GUI verdict ≠ live toolkit verdict — the original drift check, kept keyed on the live toolkit verdict). Strictly dominates the old floor + agreement-only check: every one of the 18 fixtures now has an exact expectation. Empirically captured + verified against the **CI-pinned** toolkit binary `mnemonic-toolkit-v0.47.3` (= commit `8502723`, `pinned-upstream.toml:22`) — the binary the `schema-mirror.yml` gate `cargo install`s and runs as `MNEMONIC_BIN`. Negative checks proved all three new accumulators bite (flip Canonical→NonCanonical → `wrong_verdict`; ParseFails→Canonical → `regressed`; Canonical→ParseFails → `newly_parsed`). Audit trail: `design/SPEC_canonicity_drift_per_fixture_table.md` + `design/agent-reports/canonicity-drift-per-fixture-table-r0-round{1,2}-review.md` (R0 converged YELLOW→GREEN, 0C/0I).
- **Tier:** `cross-repo` (test-hygiene).
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::canonicity-drift-gate-floor-too-lenient` (resolved in lockstep).
