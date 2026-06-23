# SPEC — Wave-2 secret-hygiene, GUI lane (G1 / G2 / G3 / G4)

**Repo:** `mnemonic-gui` (`/scratch/code/shibboleth/mnemonic-gui`)
**Source SHA pin:** `7ce777d4e364eb2a485a4aaf130644cc68d0d9fc` (`mnemonic-gui-v0.48.0`)
**Cross-repo SHA pin:** `mnemonic-toolkit` @ `34d3a724e8ac0ccb10ad13cbb5293b9bc844ae3c` (for the G1-B manual-prose revert)
**Author:** single author (this spec). **Gate:** mandatory R0 (0C/0I) before any code.
**Ship mechanism:** mnemonic-gui PR + CI-before-merge (clippy `-D warnings` all-targets, build, `schema_mirror`; **NO fmt gate — do NOT `cargo fmt` the GUI**) + a paired cross-repo toolkit `docs/manual-gui` prose PR.

> **All cited paths/line numbers were re-grepped against the pinned SHAs above** (citations decay; the FOLLOWUP entries that motivated this lane carry decayed line numbers — corrected inline below).

---

## 0. Scope & disposition summary

| Item | Slug | Disposition | Code? | SemVer |
|------|------|-------------|-------|--------|
| **G1-A** | `gui-run-confirm-modal-secret-redaction` (GUI) | **RECONCILE → resolved** (already shipped v0.39.0) | no | docs-only |
| **G1-B** | `gui-run-confirm-modal-secret-redaction-manual-companion` (toolkit) | **LOCKSTEP prose revert + pin bump** | no (cross-repo docs) | manual PATCH + `pinned-upstream.toml` bump |
| **G1-co** | `gui-import-wallet-env-var-secret-channel` (GUI) | **DEFER** (distinct feature, own R0 cycle) — *fenced spec provided* | yes (if opted in) | MINOR (0.49.0) if shipped |
| **G2** | `tree-xprv-heuristic-only-covers-key-fields` (GUI) | **SHIP** (narrow deny-list extension) | yes | **PATCH 0.48.1** |
| **G3** | `gui-tree-key-egui-undo-ring-residue` (GUI) | **DOCUMENTED CAVEAT** (upstream-egui-blocked) | no (optional doc) | NO-BUMP |
| **G4** | `tree-mode-posix-pipeline-spec-json-unmasked` (GUI) | **DEFER** (no live leak; conditional) | no (optional doc) | NO-BUMP |

**Bundled GUI SemVer for this cycle (G2 only): `0.48.0 → 0.48.1` (PATCH).** G1-A/G3/G4 are GUI-docs/FOLLOWUPS-only and add no version-gated surface. G1-B is toolkit-manual-side (manual versioning, not a crate bump). If the user opts the G1 env-var co-lander in, the GUI bump becomes **`0.49.0` MINOR** and the co-lander runs its **own** R0 sub-cycle (do not fold into the PATCH).

**Why this lane is mostly docs:** the headline ask ("add modal redaction" / "add `redact_argv_for_display`") was **already satisfied at v0.39.0** (`SECRET_MASK` + `assemble_argv_with_secret_mask` + `render_copy_command_masked` + `PendingConfirm.mask` + the inline modal substitution). Scheduling it as code would re-implement and risk regressing shipped work. The only **GUI code** remaining in scope is G2 (a 2-field redaction-walk extension). This is a `followup-status-discipline` case (per MEMORY: 3 of 4 prior "open" followups were already done) — surface it; flip status in the shipping commit.

---

## 1. G2 — extend the persist-redaction sweep to `hex` / `w` (SHIP, PATCH)

### 1.1 Current behavior (verified @ `7ce777d`)

- `src/form/tree_model.rs:714` `fn blank_non_extended_public_keys(node: &mut TreeNode)` recurses over `node.key` + `node.keys[i]` **only**, blanking any entry that is **not** `is_extended_public_like` (the positive allowlist installed by v0.34.0 audit I6). `node.hex` and `node.w` are **never visited** — a mis-pasted xprv (or WIF/raw-hex/garbage) in either field persists verbatim to `state.json`.
- `src/form/tree_model.rs:176` `TreeState::redacted_for_persistence()` clones `self.root` then calls `blank_non_extended_public_keys(&mut root)`. Its doc-comment (lines ~170-176) asserts *"Hashlock `hex` is deliberately NOT redacted."* — **must be amended.**
- `src/form/tree_model.rs:675` `pub fn is_xprv_like(key: &str) -> bool` = `rsplit(']')` origin-strip then `key_part.is_char_boundary(4) && key_part.as_bytes().get(1..4) == Some(b"prv")`. **Cannot false-positive** on a 64-hex digest (`0000…` → bytes 1..4 = `000`) or a wrap string (`sv` → too short, `is_char_boundary(4)` false). Currently consumed only render-side (`tree_form.rs` `.password`/amber-hint) + its own unit test; **not** by the persist walk anymore.
- `src/form/tree_model.rs:695` `fn is_extended_public_like` is the positive allowlist the persist walk uses; returns `false` for hex/w content → reusing it would blank **all** non-empty hex/w (data-loss).
- `src/form/tree_model.rs` `TreeNode` struct: `hex: String` (line ~94), `w: String` (line ~99), both plain `String`, `#[serde(default)]`, serialized to `state.json`. **Not** secret-typed.
- `src/persistence.rs:77` `redact_for_persistence` → `:145` maps `TreeState::redacted_for_persistence` per tree form. **No change here** — the tree leg is the only persist path for hex/w.

### 1.2 Exact change (shape (A) — NARROW deny-list; (B) fail-closed REJECTED)

In `blank_non_extended_public_keys`, after the existing `key`/`keys` loop and **before** the `children` recursion, add a **DENY-list** (`is_xprv_like`) arm for the two free-text fields:

```rust
// hex (hashlock digest) + w (wrapper) are NOT key fields — they have a
// legitimate non-key shape (64-hex digest / "sv"), so an allowlist would
// destroy valid data. Apply the NARROW is_xprv_like deny-list: blank ONLY
// xprv-SHAPED mis-pastes, preserve legit digests/wrap strings.
// (Asymmetric to the is_extended_public_like ALLOWLIST used for key/keys
// above — correct: key/keys have NO legitimate non-key shape, hex/w do.)
if !node.hex.is_empty() && is_xprv_like(&node.hex) {
    node.hex.clear();
}
if !node.w.is_empty() && is_xprv_like(&node.w) {
    node.w.clear();
}
```

**Shape (B) — fail-closed via `is_extended_public_like` on hex/w — is REJECTED**: it would blank every legitimate 64-hex hashlock digest and every wrap string (data-loss regression contradicting the "keep-hex-digests posture"). The survive-the-digest test (T6 case 3) is the guard against accidentally landing (B).

**Also amend** the `redacted_for_persistence` doc-comment (`tree_model.rs:~170-176`): the line *"Hashlock `hex` is deliberately NOT redacted."* becomes — *"Hashlock `hex` (and the `w` wrapper) are NOT redacted when they hold legitimate digest/wrap content; an xprv-SHAPED mis-paste into either is blanked via the narrow `is_xprv_like` deny-list (preserves valid digests; belt-and-suspenders against a private key fat-fingered into a non-key field)."*

**No `ScrubbedXpriv`/`SecretString`/`Zeroizing` migration** — those are toolkit-only types **absent from this repo** (grep-verified). This is a redaction-walk extension on plain `String` fields, **not** a secret-type change → the constellation MINOR-for-secret-type-migration rule does **not** trigger.

### 1.3 SCOPE DECISION — on-disk persist walk ONLY; the in-RAM twin is OUT (explicit)

The parallel in-RAM exit zeroizer `TreeNode::zeroize_keys` (`tree_model.rs:258`, M9 / v0.46.0) **also** scrubs `key`/`keys[i]` only and explicitly excludes `hex`/`w`. The FOLLOWUP slug names **only the on-disk persist walk**. **Decision: this cycle covers the persist walk ONLY; `zeroize_keys` is left unchanged.** Rationale: (a) the slug scope is persist-on-disk; (b) the threat (a private key fat-fingered into a non-key field) is bounded — persist-blanking removes the durable on-disk exposure, which is the higher-value leg; (c) keeping the in-RAM twin out avoids scope-creep and a second doc-amend. *If the reviewer escalates the in-RAM twin to in-scope, the mirror change is one `if … { self.hex.zeroize(); }` + `self.w.zeroize();` in `zeroize_keys` + a doc-amend at lines ~250-257 + a mirror test cell in `tree_round_trip.rs`.* This is flagged as a deliberate, reviewable spec decision, not an oversight.

### 1.4 SemVer

**PATCH (`0.48.0 → 0.48.1`).** `blank_non_extended_public_keys` is private; `redacted_for_persistence` keeps its signature; no clap/flag/dropdown/`schema_mirror` surface; no wire-shape change; no `pub`-struct field change; no `impl Drop` added (no pub-struct-Drop trap). Purely internal redaction-coverage widening. May alternatively ride NO-BUMP into a later cycle (persistence is the only consumer) — but recommend the standalone PATCH so the FOLLOWUP closes with a citable tag and the G1-B manual pin has a concrete `≥`-redaction-fix target.

### 1.5 Test surface (TDD — write RED first)

**Home:** `tests/persist_redaction_v0_34_0.rs` (the home of T5, the sibling allowlist test at line 187). Add `t6_tree_persist_blanks_xprv_in_hex_and_w` (do NOT overload T5 — keep the allowlist vs deny-list cases separate for diagnosability):

1. `node.hex = "<an xprv string>"` (reuse the T5 `BLANKED[0]` xprv vector) → **blanks** (`is_empty()` after `redacted_for_persistence`).
2. `node.w = "<the same xprv string>"` → **blanks**.
3. `node.hex = "<a legit 64-char hashlock digest>"` (e.g. `"0000…0001"` or a real `sha256` digest hex) → **SURVIVES** unchanged. *This case proves `is_xprv_like` (narrow), NOT `is_extended_public_like` (fail-closed), was used — it is the regression guard against accidentally shipping shape (B).*
4. `node.w = "sv"` → **SURVIVES** unchanged.
5. **Recursion:** the same hex-xprv-blanks / digest-survives pair in a `children[0]` node → proves the deny-list arm runs under the recursive walk, not only at root.

Assertions mirror T5's style (per-case `assert!(… .is_empty(), "…")` / `assert_eq!(…, "…", "…survives…")`). **Amend** the `redacted_for_persistence` doc-comment in the same PR (1.2 above).

**No lint floor / allowlist row:** grep of `tests/` + `src/` for `lint_zeroize` / `argv_secret` / `ALLOWLIST` returns nothing — those gates are **toolkit-only**. This GUI repo's hygiene gates are behavior + taxonomy tests (`secret_taxonomy_pin.rs`, `schema_mirror_secret_drift.rs`, `persist_redaction_v0_34_0.rs`, etc.); none gates a numeric floor → nothing to bump. The existing `wide_node_projection_law` (hex/w project to empty on kind-mismatch) does not collide.

### 1.6 G2 risks (verified)

1. **STALE-CITATION (primary):** the FOLLOWUP frames the fix as "extend the `is_xprv_like` sweep" / cites `blank_xprv_keys` — both predate the v0.34.0 allowlist inversion. The **live** target is `blank_non_extended_public_keys` + a NEW deny-list arm, asymmetric to the allowlist used for key/keys. An implementer copying the entry verbatim would patch a fn that no longer exists. R0 must confirm the reconciled framing.
2. **FAIL-CLOSED-ON-HEX:** reusing `is_extended_public_like` for hex/w blanks every legit digest — the T6 case-3 survive-the-digest test is the guard.
3. No pub-struct-Drop trap (no `impl Drop`); no double-Zeroizing (no `Zeroizing` on the persist path); no signature fan-out (private fn).

---

## 2. G1 — modal redaction (RECONCILE + LOCKSTEP; co-lander DEFERRED)

### 2.1 The core ask is ALREADY SHIPPED (v0.39.0) — verified @ `7ce777d`

The FOLLOWUP `gui-run-confirm-modal-secret-redaction` premise ("the modal renders each argv token verbatim"; "`grep -rn redact src/` returns only `persistence.rs`") is **now FALSE**:

- `src/main.rs:1081-1124` (the run-confirm modal render block; the FOLLOWUP's cited `:512-535` / `:683-688` are **decayed**) binds the cloned `PendingConfirm` **whole** (Drop-type, E0509-safe), `debug_assert_eq!`s `argv.len() == mask.len()`, then per token renders `SECRET_MASK` (`"••••"`) iff `pending.mask[i]`, else the raw token.
- `src/form/invocation.rs`: `SECRET_MASK` const (`:137`), `assemble_argv_with_secret_mask` (`:152`, returns parallel `(Vec<String>, Vec<bool>)`, correct-by-construction across all 4 secret-value sources incl. repeating `--ms1`), `render_copy_command_masked` (`:524`).
- `src/runner.rs:77-98`: `PendingConfirm` carries `mask: Vec<bool>` + `impl Zeroize` + `impl Drop` (scrub-on-drop, cycle-15 Lane G). The copy buttons relabel "…— reveals secret" so the real-command copy is a deliberate, informed click.
- Coverage: `tests/secret_mask_preview_v0_39_0.rs` (11 tests), `tests/secrets.rs` run-confirm gate, `tests/run_holder_zeroize.rs` (PendingConfirm scrub), `tests/widget_secret_mask_cycle15g.rs` (masking-gate==argv-secret-gate split-brain pin). The render is bin-crate (not kittest-isolable) → coverage pins at the `render_copy_command_masked` seam (T-A4), the established pattern.

**→ DO NOT re-implement modal redaction. The FOLLOWUP's option (b) was taken.** The `redact_argv_for_display` the entry asks to "add" effectively exists as `render_copy_command_masked` + the inline modal substitution.

### 2.2 G1-A — reconcile the stale GUI FOLLOWUP (docs-only)

In `mnemonic-gui/FOLLOWUPS.md`, the `gui-run-confirm-modal-secret-redaction` entry (lines 713-721, `Status: open`) → **flip to resolved**, citing: shipped v0.39.0 (CHANGELOG line 81); `src/main.rs:1098-1104` (the mask substitution); `src/form/invocation.rs:137/152/524`; `tests/secret_mask_preview_v0_39_0.rs`. **No code.** Flip in the **shipping commit** (followup-status-discipline). The companion `gui-import-wallet-env-var-secret-channel` entry (lines 697-711) stays **open** (genuinely unimplemented — see 2.4).

### 2.3 G1-B — LOCKSTEP the toolkit manual-gui companion (cross-repo docs)

Per the toolkit FOLLOWUP `gui-run-confirm-modal-secret-redaction-manual-companion` (`mnemonic-toolkit/design/FOLLOWUPS.md:1005`, `Status: open`), the manual is **factually stale** (claims v0.3.0-era plaintext; the GUI fixed it at v0.39.0, current v0.48.0). Required edits in `mnemonic-toolkit` @ `34d3a724`:

1. **`docs/manual-gui/src/10-foundations/14-secret-handling.md`** — the Defense-2 `:::danger` block (lines **79-114**): revert the "renders secret-bearing argv tokens in plaintext, NOT as `***`" prose; **restore the `***`/`••••` redaction claim** ("the modal shows the assembled argv with secret values replaced by a fixed `••••` sentinel"); **demote** the cold/airgapped-only operational mitigation from load-bearing (`:::danger`) to a general-hygiene remark (drop to a normal note; the cold-node guidance is still useful but no longer the security model's load-bearing element).
2. **`docs/manual-gui/src/10-foundations/11-what-is-mnemonic-gui.md`** — feature-2 description (lines **37-48**, esp. the line-46 *"At v0.3.0 the modal renders secret-bearing argv tokens in plaintext"*): revert to the accurate redaction claim.
3. **`docs/manual-gui/pinned-upstream.toml`** — `[mnemonic-gui] tag = "mnemonic-gui-v0.3.0"` → bump to the GUI tag this cycle ships (recommend **`mnemonic-gui-v0.48.1`**, the cut that demonstrably contains the v0.39.0 fix). *Note:* the same file's `[manual-gui]` implied toolkit/md/ms/mk tags should be re-checked for coherence with the GUI's current pins, but the load-bearing edit for THIS FOLLOWUP is the `[mnemonic-gui]` tag.
4. **Flip** the toolkit FOLLOWUP `gui-run-confirm-modal-secret-redaction-manual-companion` → resolved in the manual PR's shipping commit.

**All three edits (i)+(ii)+(iii) MUST land together** — missing any one leaves the manual internally inconsistent (the FOLLOWUP explicitly requires all three). This is a manual docs PATCH + a `pinned-upstream.toml` bump (manual versioning, not a crate bump). The manual-gui lint (`tests/lint.sh::gui-schema-coverage`) clones the pinned GUI ref — bumping the pin to v0.48.1 means the lint resolves against a ref that has `gui-schema`; confirm the lint passes against the new pin (CI `manual-gui.yml`).

### 2.4 G1 env-var co-lander — DEFER (distinct feature; fenced spec for IF-opted-in)

`gui-import-wallet-env-var-secret-channel` (GUI FOLLOWUPS:697-711, `Status: open`) is **genuinely open** — verified: `src/runner.rs:199` injects **only** `MNEMONIC_FORCE_TTY`; there is **no** per-cosigner `@env:` bag, no argv→sentinel rewrite. This is a **DISTINCT defense layer** (keeps literal seeds out of the spawned child's `/proc/<pid>/cmdline` / `ps`) from the on-screen modal redaction — the FOLLOWUP *pair* conflated two orthogonal directions. **Recommendation: DEFER to its own R0-gated cycle** (Open Question 1). It is NOT a continuation of the redaction fix and must NOT be folded into the 0.48.1 PATCH.

**Fenced spec (execution-ready IF the user opts in → GUI 0.49.0 MINOR, own R0 sub-cycle):**

- **Rewrite** the value tokens of `--ms1` / `--passphrase` / `--share` at spawn time to `@env:MNEMONIC_<FLAG>_<i>` sentinels; **inject** `MNEMONIC_<FLAG>_<i>=<value>` into the child env at spawn; **drop/scrub** the env bag on child exit. Reuse the existing mask: `mask[i] == true` marks exactly the value tokens to rewrite (no new classifier needed).
- **Secret-hygiene bar (REUSE precedents, do NOT reinvent):** hold the per-spawn secret env VALUES in **`SecretString`** (toolkit v0.67.0 — newtype over `Zeroizing<String>` with a length-only **redacting** Debug) or an equivalent GUI-local newtype — **NOT raw `Zeroizing<String>`** (raw `Zeroizing` derives a NON-redacting Debug that leaks the secret into `{:?}`/panic — the v0.67.0 lesson). **Scrub the env bag on child exit** (mirror `PendingConfirm`'s `Drop`). Avoid double-Zeroizing already-wrapped values.
- **Signature fan-out (note):** `spawn_and_capture` / `run_with_stdin` (`src/runner.rs:172`) must accept the env bag; the `MNEMONIC_FORCE_TTY` injection at `:199` is **extended, not replaced**; the main.rs consume site at `:1110` threads the bag. The `PendingConfirm` is already `pub` with `impl Drop` — **no NEW pub-struct-Drop trap is introduced** (do not add a second `Drop` or remove the existing one to fix a borrow error; the comment at `main.rs:1075-1080` warns this silently reverts the scrub).
- **Test flip (load-bearing):** `tests/kittest_import_wallet_form.rs` cell `cell_import_wallet_repeating_ms1_argv` (`:157`, currently at lines 154-213) PINS the **literal-pass-through** contract (two `--ms1` literal seeds flow to argv verbatim). It **MUST be flipped** when the co-lander lands → assert argv carries `@env:MNEMONIC_MS1_<i>` sentinels (not literal seeds). New tests: (i) spawned child's argv contains `@env:` sentinels not literal seeds for `--ms1`/`--passphrase`/`--share`; (ii) child env carries `MNEMONIC_*_<i>`; (iii) env bag scrubbed on exit; (iv) toolkit-side `resolve_env_var_sentinel` already resolves `@env:` (CHANGELOG/cell 8 confirm) → end-to-end round-trip testable.
- **Cross-repo:** toolkit-side already accepts `@env:VAR` at parse time (`resolve_env_var_sentinel`), so no toolkit code change; but the manual prose at `mnemonic-toolkit/docs/manual-gui/src/40-mnemonic/4c-import-wallet.md` (currently documents the "user-must-type-explicitly" fallback) updates in lockstep when the co-lander ships.

---

## 3. G3 — egui undo-ring residue (DOCUMENTED CAVEAT; upstream-blocked)

### 3.1 Feasibility verdict (verified against egui 0.31.1 source)

A true RAM-scrub fix is **NOT cleanly achievable** against egui 0.31.1 → **disposition = DOCUMENTED CAVEAT (won't-fix-cleanly / accepted residue)**, matching the sibling rulings `gui-secret-buffer-allocator-residue` and `gui-os-snapshot-secret-occlusion`.

- egui owns the undo `String`s inside `Arc<Mutex<Undoer<(CCursorRange,String)>>>` (`egui-0.31.1/src/widgets/text_edit/state.rs:42`), stored in `ctx.data` keyed by widget Id. `#[serde(skip)]` on the undoer (state.rs:44) → **RAM-only**, never on disk even with `persist_egui_memory=true`.
- The undoer is fed the **real** buffer every frame (`builder.rs:905-908` / `1116-1118`), **independent of `.password(...)`** — masking the display does NOT keep the secret out of the ring.
- The only public lever, `TextEditState::clear_undoer()`, REPLACES the `Undoer` with `default()` and **DROPS** the old `VecDeque`s — **drop frees but does not overwrite** (egui has **zero** Zeroize, repo-wide grep empty). So even the "clean" path leaves the secret in **freed-but-unscrubbed heap** = the same allocator-residue class already ratified as accepted → **zero net hygiene gain** over the existing caveat.
- To even reach `clear_undoer`, an exit sweep needs (a) a cached `egui::Context` (feasible — pattern exists at `main.rs:195/219/247`) **and** (b) the exact per-frame-derived widget `Id`s of every tree-key `TextEdit` (NOT recorded in the model) — brittle Id-reconstruction that silently no-ops on an egui Id-derivation shift.
- eframe `on_exit(&mut self)` has **no `&egui::Context`** param (`eframe-0.31.1/src/epi.rs:195`; `main.rs:144` already notes this).

**Already shipped (do NOT redo):** `TreeNode::zeroize_keys` scrubs the MODEL Strings (M9, v0.46.0); `.password(is_xprv_like(..))` masks the display (cycle-15g, v0.47.0); the gap is already documented in-source at `src/form/secret_widget.rs:11-14`, `src/secrets.rs:194` (`PASTE_WARN_MODAL_TEXT`), and in FOLLOWUPS.

### 3.2 Action (NO-BUMP, optional)

- Flip the FOLLOWUP `gui-tree-key-egui-undo-ring-residue` disposition to **won't-fix-cleanly / accepted-residue (documented caveat)**, citing the egui-0.31.1 findings above and the consistent sibling rulings.
- **Optional** one-line caveat-text tightening to NAME the tree-key facet (e.g. in `secret_widget.rs:11-14` or the FOLLOWUP entry: "…this includes the build-descriptor tree key/keys `TextEdit`s, whose model String is zeroized on exit but whose egui undo ring is not"). Doc-only.
- **Do NOT** ship a `clear_undoer()` pass and flip to RESOLVED — that would over-claim (drop ≠ scrub; secret remains in freed heap) and risk a silent regression on the next egui bump. A genuine elimination is **BLOCKED ON UPSTREAM egui** (needs a zeroizing/scrubbing undo buffer or opt-out; neither exists in 0.31.1).

---

## 4. G4 — tree-mode POSIX-pipeline spec-JSON (DEFER; no live leak)

### 4.1 Verdict (verified @ `7ce777d`)

**DEFER (keep the FOLLOWUP open; do NOT schedule a fix).** There is **no current secret-class node** in the build-descriptor tree:

- The 17 `NODE_KIND_SPECS` kinds (`src/schema/nodes.rs:60+`: pk/pkh/multi/sortedmulti/older/after/sha256/hash256/hash160/ripemd160/and_v/or_d/or_i/or_b/andor/thresh/wrap) are **disjoint** from the 9 `SECRET_NODE_TYPES_ARGV` tokens (`src/secrets.rs:35` re-export: phrase/entropy/xprv/wif/ms1/bip38/electrum-phrase/seedqr/minikey). `TreeNode.kind` is always a `NODE_KIND_SPECS` kind — never a convert NodeType.
- `to_spec_json` (`src/form/tree_model.rs:434`) serializes only: `node.key`/`node.keys` (descriptor KEYS = xpubs, watch-only by contract), `node.k`/`node.n` (uints), `node.hex` (PUBLIC hash digests), `node.w` (wrap prefix). No field can structurally carry a secret-class value.
- An xprv mis-paste into a key field is persist-BLANKED + RAM-zeroized (and after **G2**, an xprv into `hex`/`w` is also persist-blanked).
- **Trip-wire:** the compile-time `const _: () = assert!(secret_slice_eq(...))` at `src/secrets.rs:80-101` FAILS THE BUILD if a toolkit pin bump adds a secret node-type → auto-re-arms this slug. Plus `tests/secret_taxonomy_pin.rs` + `tests/spec_nodes_mirror.rs`.

### 4.2 Action (NO-BUMP, optional) + the conditional future fix (NOT scheduled)

- **Optional:** refresh the FOLLOWUP's stale `Where:` line numbers — the function now lives at `src/form/tree_form.rs:124` (`posix_pipeline_command`), `:104` (`spec_json_pretty`), `:94` (`spec_stdin_bytes`); call sites `src/main.rs:949-950, 1033`. Add a one-line trip-wire pointer to `src/secrets.rs:80` so the conditional re-fires automatically.
- **Future conditional fix (NOT this cycle):** IF a build-descriptor node ever carries a secret-class value, add a JSON-string redaction pass **chokepointed at `to_spec_json`** (or a single shared redact-pass) covering **all three** surfaces (`posix_pipeline_command`, `spec_json_pretty` / "Copy spec JSON" button, `spec_stdin_bytes` / live `--spec -` stdin — the slug names only the first; a real fix MUST cover all three), keyed on `secrets::node_type_is_argv_secret` (the wider ARGV superset, NOT the narrow `SECRET_NODE_TYPES`). No secret-type wrapper warranted (string-redaction of already-public JSON, not a secret-buffer-lifetime problem). PATCH if ever built.

---

## 5. Bundled SemVer + ALL version / ship sites

**This cycle (G2 only): `mnemonic-gui 0.48.0 → 0.48.1` (PATCH).**

GUI version sites to touch for the 0.48.1 cut (per `project_toolkit_release_ritual_version_sites` — several are NOT gate-enforced):

1. `Cargo.toml` `version = "0.48.0"` → `0.48.1` (line 3).
2. `README.md` install line `--tag mnemonic-gui-v0.48.0` → `…v0.48.1` (line 42). *(Gated by `readme_pin_coherence`/`pin_coherence` — verify.)*
3. `Cargo.lock` (the `mnemonic-gui` package version entry) — re-resolve.
4. `CHANGELOG.md` — new `## mnemonic-gui [0.48.1]` section.
5. Tag `mnemonic-gui-v0.48.1` after CI-green.

**Toolkit manual side (G1-B), no crate bump:**

6. `mnemonic-toolkit/docs/manual-gui/src/10-foundations/14-secret-handling.md` (Defense-2 prose, lines 79-114).
7. `mnemonic-toolkit/docs/manual-gui/src/10-foundations/11-what-is-mnemonic-gui.md` (feature-2, lines 37-48).
8. `mnemonic-toolkit/docs/manual-gui/pinned-upstream.toml` (`[mnemonic-gui].tag` → `mnemonic-gui-v0.48.1`).
9. Toolkit `design/FOLLOWUPS.md` — flip `gui-run-confirm-modal-secret-redaction-manual-companion` → resolved.

**FOLLOWUPS flips (in the shipping commits):**

10. `mnemonic-gui/FOLLOWUPS.md` — `gui-run-confirm-modal-secret-redaction` → resolved (G1-A); `tree-xprv-heuristic-only-covers-key-fields` → resolved (G2); `gui-tree-key-egui-undo-ring-residue` → won't-fix-cleanly/documented (G3, optional); `tree-mode-posix-pipeline-spec-json-unmasked` → keep open / optional line-refresh (G4).

**NOT touched:** `schema_mirror` (no clap flag/dropdown change), sibling-codec CLIs (no flag surface), crates.io (GUI is git-installed), `cargo fmt` (no GUI fmt gate). CI gates that MUST stay green: clippy `-D warnings --all-targets`, build matrix, `schema_mirror` (21 subtests), `pin_coherence`/`readme_pin_coherence`, `archetype_schema_mirror`, `gui_schema_conditional_drift`, `xpub_search_schema_mirror`, `schema_mirror_secret_drift`, `canonicity_drift`, **full suite**.

> **R0 reminder (per `feedback_r0_review_run_full_package_suite`):** the per-phase R0 review and the post-impl whole-diff review MUST run the FULL `cargo test` suite (not targeted `--test` targets) — a CLI/secret-classification touch can ripple into taxonomy/schema-drift tests outside any one phase's targets.

---

## 6. Deferred / blocked (clean exclusion)

- **G1 env-var co-lander** — DEFERRED to its own R0-gated MINOR cycle (0.49.0). Genuinely open, real feature, distinct defense layer; fenced spec in §2.4. **Excluded from the 0.48.1 PATCH.** Ship-vs-defer = Open Question 1.
- **G3 genuine RAM-scrub** — BLOCKED ON UPSTREAM egui 0.31.1 (no zeroizing undo buffer); ship the doc caveat only (§3).
- **G4 conditional redaction pass** — DEFERRED (no triggering condition; the compile-time drift assert auto-re-arms it on a toolkit pin bump that adds a secret node-type) (§4).
- **G2 in-RAM twin (`zeroize_keys` hex/w)** — explicitly OUT of scope (§1.3); reviewable spec decision, one-line escalation path documented.

---

## 7. Open questions

1. **G1 env-var co-lander — ship IN this cycle (→ 0.49.0 MINOR, own R0 sub-cycle, runner.rs fan-out + kittest cell flip) or DEFER?** Headline fork. Recommendation: **DEFER** (keep this cycle docs+G2-PATCH; it is a distinct feature, not a continuation of the redaction fix) — unless the user wants the argv-on-disk/`ps` leak closed now.
2. **G1-B pin-bump target** — bump `[mnemonic-gui].tag` to **0.48.1** (the cut this cycle makes, demonstrably ≥ the v0.39.0 fix) vs the exact v0.39.0 fix tag? Recommendation: **0.48.1** (pinned-upstream also implies the GUI's current toolkit/md/ms/mk pins).
3. **G2 release vehicle** — standalone **0.48.1 PATCH** vs ride NO-BUMP into a later cycle? Recommendation: **standalone 0.48.1** (citable tag for the FOLLOWUP close + a concrete pin target for G1-B).
4. **G3/G4 optional doc-tightening** — apply the caveat-text tightening (G3) + stale-line-number refresh (G4) now, or leave verbatim? Recommendation: **apply both** (cheap, reduces future citation-decay); strictly optional, no code.
5. **followup-status-discipline** — confirm the cycle flips the stale-open `gui-run-confirm-modal-secret-redaction` (shipped v0.39.0, never flipped) in the **shipping commit**, not a separate housekeeping pass. (Per MEMORY: another instance of the "open" status lagging shipped code — surfaced explicitly.)