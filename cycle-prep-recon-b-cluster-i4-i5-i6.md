# cycle-prep recon — 2026-06-10 — B-cluster: xpub-search-inline-phrase-not-secret-classified (I4) + positional-secrets-not-redacted-at-persist (I5) + tree-wif-hex-privkey-in-key-fields-unredacted (I6)

**Origin/master SHA at recon time:** `385d062` (mnemonic-gui)
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** this recon file only

Slugs verified: `xpub-search-inline-phrase-not-secret-classified` (+ toolkit companion `gui-secret-mirror-phrase-ms1-stdin`), `ms-repair-ms1-not-secret-classified`, `positional-secrets-not-redacted-at-persist`, audit-I6 `tree-wif-hex-privkey-in-key-fields-unredacted` (index line `FOLLOWUPS.md:18`, NOT yet promoted to its own `###` entry). One STRUCTURALLY-WRONG instruction found in the freshly-filed companion; one audit file mis-cite; one behavior change the flips силently trigger.

---

## Per-slug verification

### `xpub-search-inline-phrase-not-secret-classified` (audit I4, GUI half — toolkit half DONE v0.53.1)

- **WHAT:** the 3 xpub-search `--phrase` flags (raw master BIP-39 phrase) + `--phrase-stdin` + `--ms1-stdin` are `secret: false` in the GUI hand schema → unmasked widget, no run-confirm/paste-warn. Toolkit v0.53.1 flipped the source of truth; GUI flips pend the pin bump.
- **Citations:**
  - 9 schema sites `src/schema/mnemonic.rs` (`--phrase` :2280/:2442/:2712 with `secret: false` at :2286/:2448/:2718; `--phrase-stdin` :2291/:2453/:2723; `--ms1-stdin` :2312/:2474/:2744) — **ACCURATE** (verified at 036776b by the toolkit R0-r2 agent; only FOLLOWUPS.md changed since → 385d062 identical).
  - `src/schema/ms.rs:321` `REPAIR_FLAGS --ms1 secret: false` — **ACCURATE** (read :315-325: `"--ms1"` / `FlagKind::Text` / `required: true` / `secret: false`).
  - **Companion instruction "(1) add the 3 tokens to `src/secrets.rs`'s token-for-token mirror" — STRUCTURALLY-WRONG.** The secret-drift gate (`tests/schema_mirror_secret_drift.rs:104-112`) collects the GUI side from **`FlagSchema.secret == true` only** (`for sub in schema::mnemonic::SCHEMA.subcommands … if flag.secret`); `SECRET_FLAG_NAMES` (`src/secrets.rs:141-145`) is a **3-token v0.1 legacy fallback** (`--passphrase`, `--bip38-passphrase`, `--passphrase-stdin`) consumed only by `flag_is_secret` as an OR-branch — it is NOT the gate's mirror and adding tokens there is redundant (harmless but misleading). The toolkit `secrets.rs` module-doc's "GUI mirrors a parallel enumeration … agree token-for-token" claim is STALE vs current GUI reality and propagated into both companion texts (toolkit FOLLOWUP `gui-secret-mirror-phrase-ms1-stdin` + GUI Companion: lines). **The real lockstep = the 9 `FlagSchema.secret` flips + the pin bump.** Fix both FOLLOWUP texts during the cycle.
  - Redaction union `schema_secret_flag_names()` (`src/secrets.rs:323`) is FIELD-extracted from the schemas → the 9 flips update it automatically (no hand edit).
- **NEW DESIGN POINT the flips trigger (needs R0 adjudication):** `--phrase-stdin` ×3 and `--ms1-stdin` ×3 are **Boolean** flags that today EMIT from the GUI form (secret:false, not in `SECRET_FLAG_NAMES` → generic checkbox branch). Flipping `secret: true` routes them into `assemble_argv`'s secret-branch `else { continue }` (see `boolean-stdin-secret-toggles-never-emit`, FOLLOWUPS.md:65) → **they STOP emitting**: census 18 → 24 suppressed sites. Practically an improvement (a checked stdin toggle emits a flag the GUI runner can't feed — the CLI would hang/error), and it ALIGNS them with `--passphrase-stdin`'s existing suppression — but it is a live behavior change that must be deliberate + the census entry updated.
- **`ms-repair-ms1-not-secret-classified` (sibling, audit I4's other half):** the drift gate asserts `cli == "mnemonic"` and walks `schema::mnemonic::SCHEMA` ONLY → **no automated gate covers `ms.rs` secret bits**; a GUI-side deliberate override flip of `ms.rs:321` is feasible NOW (the entry's option b) with zero gate conflict. Recommend folding it into the same cycle as a recorded GUI-side decision (it is the same widget-class exposure; waiting for an ms-cli-side classification source blocks indefinitely on a repo with no gui-schema surface).
- **Action for brainstorm spec:** cite GUI `385d062` + toolkit `87c33c5` (= v0.53.1). Plan = pin bump + 9 flips + optional ms.rs:321 flip + census update + companion-text corrections.

### `positional-secrets-not-redacted-at-persist` (audit I5)

- **WHAT:** secret-equivalent positionals (`ms combine <shares>`) persist to `state.json` verbatim — no redaction class, no run-confirm, plain widget.
- **Citations:**
  - `src/persistence.rs:115` `positionals: state.positionals.clone()` — **ACCURATE** (read :110-120; the clone is verbatim inside `redact_for_persistence`'s FormState construction).
  - `PositionalArgSchema` has no `secret` field — **ACCURATE but the audit's file cite is wrong**: the struct is at **`src/schema/mod.rs:53-64`** (fields: `name`, `required`, `repeating`, `help` — no `secret`), NOT `src/form/mod.rs:53-63` as the audit report cited. The GUI FOLLOWUPS entry itself cites no file for the struct, so only the audit report carries the mis-cite.
  - `src/secrets.rs:200-225` `should_confirm_run` never inspects positionals — **ACCURATE** (read it: loops `subcommand.flags`, `state.slots.rows`, `NodeValueComposite` values; no positional loop).
  - `src/schema/ms.rs::COMBINE_POSITIONALS` "Secret-equivalent" help — **ACCURATE** (per the v0.31.1 R0 record; entry filed from source).
  - Latency context — **ACCURATE**: persistence is UNWIRED (`persistence-unwired-redaction-never-runs`, FOLLOWUPS.md:28 [obs]: `save`/`load`/`redact_persisted_state` have no src/ callers) → I5 is latent until Phase-8 wires it; "fix-before-Phase-8" disposition stands.
  - Toolkit companion question (entry's last line) — **resolved by source**: toolkit subcommands have ZERO positionals (`src/schema/mod.rs:40-42` doc: "mnemonic-toolkit's subcommands have zero positionals — they pass slot data via `--slot`") → the `secret` field addition is purely GUI-side; NO toolkit gui-schema companion needed now.
- **Action for brainstorm spec:** `PositionalArgSchema.secret: bool` + redaction arm dropping secret positionals + route secret positionals through per-row `SecretLineEdit` (v0.31.1 repeating-secrets precedent gives the widget pattern) + extend `should_confirm_run`. Cite `385d062`.

### `tree-wif-hex-privkey-in-key-fields-unredacted` (audit I6 — index line `FOLLOWUPS.md:18`, promote when worked)

- **WHAT:** the tree redaction walk blanks only extended-private-prefix keys; a WIF (`K`/`L`/`c`-prefix) or raw-hex privkey pasted into a key/keys row survives `redacted_for_persistence` untouched.
- **Citations:**
  - `src/form/tree_model.rs:650-669` `is_xprv_like` / `blank_xprv_keys` — **ACCURATE** (read :645-675: `is_xprv_like` = strip `[origin]` via `rsplit(']')`, check `prv` at bytes 1..4; `blank_xprv_keys` recursive blank of matching `key`/`keys` only).
  - "runs unconditionally at :176-187" — **ACCURATE in substance**: `redacted_for_persistence` (:176-187) calls `blank_xprv_keys(&mut root)` unconditionally; note it's only invoked on the persistence path (unwired today — same latency as I5). Hashlock `hex` deliberately NOT redacted (doc :172-175) — that's the RESOLVED sibling `tree-xprv-heuristic-only-covers-key-fields` (:47) scope; I6 is the key/keys WIF/hex gap, distinct as the audit said.
  - Toolkit `gate.rs:275-276` "WIF / raw-hex … not prefix-detectable" — **ACCURATE** (toolkit `descriptor_builder/gate.rs:~270-278`, doc on `check_secret_key`: "WIF / raw-hex secrets are not prefix-detectable here; they are refused by the step-2 `from_str` type-check"). KEY NUANCE for the fix: in the toolkit, step-2 backstops the heuristic; in the GUI persistence path there is NO step-2 → the heuristic is the only line of defense, which is why prefix-only is insufficient exactly here.
- **Action for brainstorm spec:** decide the audit's two options — (a) positive allowlist (blank key/keys content unless it positively matches an xpub/descriptor-pubkey shape) vs (b) refuse to persist a tree that hasn't passed the watch-only gate. (a) is self-contained in `tree_model.rs` and fail-closed; (b) couples persistence to validation state. Lean (a); R0 adjudicates. Promote the index line to its own `###` entry. Cite `385d062`.

---

## Cross-cutting observations

1. **STRUCTURALLY-WRONG companion instruction** (filed 4 hours ago by the toolkit v0.53.1 cycle, propagating a stale toolkit module-doc claim): the secret-drift gate mirrors `FlagSchema.secret` per `(subcommand, flag)`, not a token list. Correct both repos' FOLLOWUP texts in this cycle.
2. **Audit file mis-cite:** I5's struct is `src/schema/mod.rs:53`, not `src/form/mod.rs:53`.
3. **Pin-bump lockstep census (v0.52.0 → v0.53.1) = the established 6 sites:** `Cargo.toml:42` + `Cargo.lock` + `pinned-upstream.toml:22` + `README.md:50` (`--tag` install line) + `src/schema/mnemonic.rs:3949` (`pinned_version: "mnemonic 0.52.0"` → `0.53.1`) + `src/schema/mnemonic.rs:1` (module-doc). 4 gated (`pin_coherence`, `readme_pin_coherence`), 2 ungated (banner + module-doc). The `archetypes.rs:5`/`nodes.rs:5` "transcribed from v0.52.0" comments are transcription provenance, not pins — leave unless `--spec-schema` output changed (it didn't in v0.53.x).
4. **Bump rides the v0.53.0 csi wire change:** `tests/fixtures/wallet_import/envelope_v0_27_0.json` carries a **3-cosigner multisig mk1 list** — if any GUI test feeds it through the bumped pinned binary and compares emitted mk1 bytes it will break (the toolkit recaptured its own copy at v0.53.0). Likely input-only (form-filling); verify with the full suite at the bumped pin before concluding (v0.29.0 lesson: run the suite at the BUMPED dep).
5. **Adjacent open entries touched by this cluster:** `boolean-stdin-secret-toggles-never-emit` (census 18→24 after flips), `slot-secret-values-rendered-unmasked` (audit minor — same widget-class family as I5's fix; cheap to fold if the cycle touches the slot editor, else leave), `run-confirm-and-preview-show-secrets-cleartext` [obs] (NOT in scope — preview rendering is its own design question).
6. Stale TOC impression: `gui-build-descriptor-presets-pending-pin-bump` (:99) is **resolved** (v0.30.0); pin is already v0.52.0. No build-descriptor debt rides this bump.

---

## Recommended brainstorm-session scope

**Cycle B1 — "I4 GUI half: pin bump + secret flips" (GUI MINOR, v0.33.0).** Pin v0.52.0 → v0.53.1 (6 sites) + flip the 9 `mnemonic.rs` sites + (recorded GUI-side decision) flip `ms.rs:321` + update the boolean-stdin census entry (18→24, or 25 with ms.rs's `--passphrase-stdin` 18th already counted — re-census in-cycle) + correct both companion texts. Forced pairing: the secret-drift gate REDs in BOTH directions (bump-without-flips and flips-without-bump) → single PR. schema_mirror green (no NAME change). Measure first (v0.29.0 lesson): run the full suite with `MNEMONIC_BIN` = a locally-built v0.53.1 binary BEFORE editing. ~30 LOC src + docs. SemVer MINOR (masking/run-confirm behavior turns ON; 6 checkboxes stop emitting).

**Cycle B2 — "persistence redaction hardening: I5 + I6" (GUI MINOR, v0.34.0).** One cycle, same subsystem (persist-path redaction): `PositionalArgSchema.secret` + redaction arm + per-row `SecretLineEdit` for secret positionals + `should_confirm_run` positional loop (I5); tree key/keys positive-allowlist redaction or refuse-ungated-persist per R0 (I6, promote index line). Both latent until Phase-8 persistence wiring — but both are its hard preconditions ("Persistence MUST NOT be wired until this lands"). ~150-250 LOC incl. tests.

Order: **B1 → B2** (B1 small + forced-lockstep + unblocks the audit's I4; B2 self-contained). No sibling-codec impact; no manual impact (docs/manual-gui pin is a separate cadence — its own FOLLOWUP exists for anchors).
