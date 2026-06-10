# SPEC — GUI v0.33.0: toolkit pin v0.52.0 → v0.53.1 + secret flips for `--phrase`/`--phrase-stdin`/`--ms1-stdin` (+ `ms repair --ms1` override) — audit I4 GUI half

**Cycle:** mnemonic-gui v0.33.0 (MINOR) · **Source SHAs:** GUI `385d062`, toolkit `87c33c5` (= tag `mnemonic-toolkit-v0.53.1`) · **Recon:** `cycle-prep-recon-b-cluster-i4-i5-i6.md` (repo root)
**Resolves:** `xpub-search-inline-phrase-not-secret-classified` + `ms-repair-ms1-not-secret-classified` (FOLLOWUPS.md:73/:82; = audit I4, both halves). **Companion:** toolkit `gui-secret-mirror-phrase-ms1-stdin` (flips to resolved; its text needs a correction — see §5).

## 0. Measured baseline (pre-edit, 2026-06-10)

Against a locally-built v0.53.1 binary (`MNEMONIC_BIN=…/mnemonic-toolkit/target/release/mnemonic`):

- `schema_mirror_secret_drift` — **RED on exactly the 9 expected pairs** (`--phrase`, `--phrase-stdin`, `--ms1-stdin` × the 3 xpub-search modes), `only_in_gui` empty. Scope confirmed; nothing else rides the bump.
- `schema_mirror` (mnemonic flag-NAME set) + `gui_schema_conditional_drift` + `archetype_schema_mirror` — **GREEN** vs v0.53.1 (no name/conditional/archetype drift v0.52.0 → v0.53.1; v0.53.0's csi change is card-bytes only).
- `schema_mirror`'s `ms_schema`/`mk_schema` help-text cells FAIL **locally only** — stale `~/.cargo/bin/ms` 0.4.0 / `mk` 0.4.1 on PATH vs pinned ms-cli/mk-cli v0.7.0; CI installs the pinned tags. Environment artifact, not drift. Final verification must use pinned sibling binaries (install to a temp prefix or set `MS_BIN`/`MK_BIN`) — do NOT chase these as regressions.

## 1. Pin bump — the established 6 lockstep sites

`mnemonic-toolkit-v0.52.0` → `mnemonic-toolkit-v0.53.1` at:
1. `Cargo.toml:42` (git tag dep — the load-bearing pin),
2. `Cargo.lock` (refresh; rev must equal the TAG'S commit `87c33c5` — annotated-tag gotcha: `git rev-parse <tag>^{commit}`),
3. `pinned-upstream.toml:22` (documentary cross-cite + CI install source),
4. `README.md:50` toolkit `--tag` install line (gated by `readme_pin_coherence`),
5. `src/schema/mnemonic.rs:3949` `pinned_version: "mnemonic 0.52.0"` → `"mnemonic 0.53.1"` (ungated),
6. `src/schema/mnemonic.rs:1` module-doc (ungated).

NOT pins (leave): `archetypes.rs:5`/`nodes.rs:5`/`conditional.rs:626`/`archetype_form.rs:232` "v0.52.0" transcription-provenance comments (`--spec-schema` output unchanged in v0.53.x).

## 2. The 9 `FlagSchema.secret` flips (src/schema/mnemonic.rs)

`secret: false` → `secret: true` at the 9 sites (line numbers at `385d062`): `--phrase` :2286/:2448/:2718, `--phrase-stdin` (:2291/:2453/:2723 blocks), `--ms1-stdin` (:2312/:2474/:2744 blocks) — the 3 xpub-search modes `path-of-xpub` / `passphrase-of-xpub` / `account-of-descriptor` (NOT `address-of-xpub`, which has `--xpub`). This is the entire drift-gate delta (§0). Effects that follow AUTOMATICALLY (no further edits): masked `SecretLineEdit` + zeroize for the Text `--phrase` (widget dispatch keys on `flag_is_secret` = `FlagSchema.secret ||` legacy names), `should_confirm_run` true when non-empty, `should_warn_on_paste` eligible, redaction union `schema_secret_flag_names()` (field-extracted) — already contains `--phrase` (4 ms.rs secret sites); the flips' NEW union members are `--phrase-stdin` and `--ms1-stdin` (R0-r2 M-NEW2c).

## 3. `ms repair --ms1` flip (src/schema/ms.rs:321) — recorded GUI-side decision

Flip `secret: false` → `true`. Adjudication basis (R0 to confirm): (a) the value is master-secret material (BCH-corrupted ms1; same class as the 7 `secret: true` `--ms1` sites in mnemonic.rs — it is the lone false twin of the 8-site census); (b) **no automated gate covers ms.rs secret bits** (the drift gate asserts `cli == "mnemonic"` (:92) and walks `schema::mnemonic::SCHEMA` only (:105-112) — `tests/schema_mirror_secret_drift.rs`) so no gate conflict in either direction; (c) waiting for an "ms-cli-side classification source" blocks indefinitely on a repo with no gui-schema surface; ms-cli's own runtime DOES treat the value as secret-equivalent (its help text says so). Risk: if a future ms-cli gui-schema projection lands with `secret:false`, a then-new gate would flag OUR true as `only_in_gui` — the drift-gate's own failure text explicitly sanctions "file a FOLLOWUP if the GUI's broader secret-class semantic should stay". Record the decision in the FOLLOWUPS resolution.

## 4. Boolean stdin-toggle emission change (deliberate, censused)

Flipping `--phrase-stdin` ×3 + `--ms1-stdin` ×3 to secret moves them from the generic Boolean checkbox emission into `assemble_argv`'s secret-branch `else { continue }` → **they stop emitting** (today they emit; a checked toggle produces a CLI that waits on stdin the GUI runner cannot feed — hang/error). This ALIGNS them with the existing suppressed family (`--passphrase-stdin` et al.). Census in `boolean-stdin-secret-toggles-never-emit` (FOLLOWUPS.md:65): 18 → **24** suppressed sites (+6; re-verify the count in-cycle by grepping the schema tables). Update that entry's census + note the alignment; its "emit-or-grey-out" decision stays open (unchanged scope, now 24 sites).

## 5. Docs / FOLLOWUPS / companion-text corrections

- **Resolve** `xpub-search-inline-phrase-not-secret-classified` + `ms-repair-ms1-not-secret-classified` (status → resolved v0.33.0, with the §3 decision recorded).
- **Correct the structurally-wrong companion instruction** (both repos, surfaced by recon): the secret-drift gate mirrors `FlagSchema.secret` per `(subcommand, flag)` — NOT a token list; `SECRET_FLAG_NAMES` (`src/secrets.rs:141-145`) is a 3-token v0.1 legacy fallback and needs NO additions. Fix the GUI Companion: lines + the toolkit `gui-secret-mirror-phrase-ms1-stdin` entry (flip it to resolved at the same time, citing GUI v0.33.0). Also fix the toolkit `src/secrets.rs` module-doc's stale "token-for-token" claim — add to the toolkit entry as a one-line doc errand (next toolkit touch; do NOT cut a toolkit release for a comment).
- **Census update** per §4 — the `boolean-stdin-secret-toggles-never-emit` entry needs BOTH its header count "(18 sites)" → 24 AND its body's name list re-enumerated to the **6** names (R0-r1 M3 corrected by R0-r2 M-NEW1: today's distinct names = 4 — `--passphrase-stdin`/`--secret-stdin`/`--decrypt-password-stdin`/`--bip38-passphrase-stdin` — the entry's own "5" already disagrees with its 4-name enumeration; +`--phrase-stdin` +`--ms1-stdin` = 6). Audit-backlog index line `secret-false-flags-render-cleartext-no-confirm` (FOLLOWUPS.md:17) → mark resolved. Toolkit-side correction also fixes its GUI cross-cite ":81" → ":82" (`mnemonic-toolkit/design/FOLLOWUPS.md:50`).
- GUI CHANGELOG `[0.33.0]`; version `Cargo.toml 0.32.0 → 0.33.0`; README self-pin line :42 `mnemonic-gui-v0.32.0` → `v0.33.0` (gated by `readme_pin_coherence` self-line rule).
- No manual-gui impact this cycle (docs/manual-gui lives in the TOOLKIT repo on its own pinned cadence; anchors FOLLOWUP already tracks the debt).

## 6. Tests (TDD shape)

The forcing gate already exists (`schema_mirror_secret_drift`) and is RED at the bumped pin pre-flip (§0) — that IS the red cell for §2. Add:
- **T1 (ms.rs override pin):** a unit/integration cell asserting `ms repair --ms1` is `secret: true` AND documenting the override decision (test-local comment citing §3) — guards against a future "reconcile ms.rs to a false upstream" sweep silently reverting it.
- **T2 (stdin-toggle suppression):** extend the argv-assembler suite: a checked `--phrase-stdin` / `--ms1-stdin` on an xpub-search form emits NOTHING. NEW coverage (R0-r1 M4: no cell anywhere pins a checked secret Boolean's no-emit — `argv_assembler_visibility.rs:181-196` tests typed-value suppression UNDER a toggle, not the toggle itself) — T2 pins the mechanism shared by all 24 census sites for the first time.
- **T3 (widget class):** `--phrase` on the 3 modes routes to the secret widget branch (`flag_is_secret` true) — cheapest as a pure-predicate cell over the schema tables.
- Full suite at the bumped pin with pinned sibling binaries (§0 caveat). Watch `envelope_v0_27_0.json` consumers — the fixture carries a 3-cosigner multisig mk1 list; v0.53.0 changed multisig mk1 csi bytes. Expectation: input-only (form-filling) → unaffected; if a cell compares binary-emitted mk1 bytes to the fixture, recapture from the v0.53.1 binary (toolkit precedent).

## 7. Phases

1. **Phase 1:** pin bump (6 sites) → confirm `schema_mirror_secret_drift` RED on exactly the 9 pairs **with `MNEMONIC_BIN=<locally-built v0.53.1 binary>` set explicitly** (R0-r1 I1: the gate has NO pinned-dep path — `tests/schema_mirror_secret_drift.rs:54-56` (`resolve_mnemonic_bin`, env read :55; silent-skip return :85-90) resolves `MNEMONIC_BIN` else bare `mnemonic` on `$PATH`, and this machine's `$PATH` binary is `mnemonic 0.24.0` = pre-v5 → the cell SILENTLY PASSES BY SKIP without the env-var; the Cargo dep feeds only `secret_taxonomy` constants, byte-identical across the bump). The pin bump's gate effect is **CI-only** (`pinned-upstream.toml:22` → schema-mirror.yml:49-62 `install-mnemonic-toolkit`). Everything else green under the same env.
2. **Phase 2:** §2 + §3 flips + T1-T3 → full suite GREEN (pinned siblings for ms/mk cells).
3. **Phase 3:** §5 docs/census/version/CHANGELOG → commit (explicit paths) → push → CI green (schema-mirror.yml full suite fires on master push) → tag `mnemonic-gui-v0.33.0` → tag CI green (build.yml release + schema-mirror.yml tag-gated since I7) → resolve FOLLOWUPS both repos.

## 8. SemVer + risks

**MINOR (v0.33.0):** user-visible behavior turns ON (masking, run-confirm, paste-warn *eligibility* — the modal wiring is still dead code, see `paste-warn-modal-dead-code`) and 6 checkboxes stop emitting; precedent v0.28.0 (pin bump + drift fix = MINOR).
Risks: (a) the 6-checkbox no-emit is a behavior REGRESSION only if someone fed stdin via a wrapper — the GUI runner has no stdin channel, so no working flow breaks; (b) **three deterministic test breaks (R0-r1 I2, not speculative):** `tests/xpub_search_widgets.rs` argv cells `cell_path_of_xpub_argv_assembles` (:46, assert :75-76), `cell_account_of_descriptor_argv_assembles` (:117, assert :144), `cell_passphrase_of_xpub_argv_assembles` (:263, assert :290) push `--phrase` into `state.values` and assert emission — post-flip, `assemble_argv`'s secret branch reads Text secrets from `secret_widgets` and a values-synthesized entry emits NOTHING (`src/form/invocation.rs:255-273`). **Prescribed conversion (do NOT delete the emission asserts):** seed `state.secret_widgets` via `SecretLineEdit::from_text(...)` (`src/form/secret_widget.rs:55`; live pattern `tests/repeating_secret_rows.rs:210-218`) and keep asserting `--phrase <value>` emission — converted, they double as the positive cells for the new path. Everything else survives by construction (`has_value` spans both maps `src/schema/mod.rs:378-385`; `xpub_search_schema_mirror.rs:163-201` asserts only secret=true; union census cells are field-extracted); (c) local ms/mk staleness (§0) must not be chased as drift.

## Non-goals

I5 (positional redaction) + I6 (tree WIF/hex) — next cycle (B2). `boolean-stdin` emit-or-grey-out decision. `slot-secret-values-rendered-unmasked`. Preview/run-confirm cleartext rendering [obs].
