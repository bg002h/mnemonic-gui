# SPEC — GUI v0.31.1: repeating-secret flags reach argv (per-row SecretLineEdit)

**Status:** R0 GREEN (round 4 confirm, 0C/0I) — implementation may begin
**Source grounding verified at:** mnemonic-gui `origin/master` = `dabbdfe` (tag `mnemonic-gui-v0.31.0`)
**Resolves:** `FOLLOWUPS.md::repeating-secret-flags-never-reach-argv` — **with the fix direction INVERTED vs the entry** (recon `cycle-prep-recon-repeating-secret-flags.md`, in the toolkit repo root: routing secrets through `state.values` would persist seed material — `redact_for_persistence` drops only `SECRET_FLAG_NAMES`/node-types/slot-subkeys, NOT schema-`secret:true` Text flags; the type-level never-persist invariant of `secret_widgets` is the safe rail).
**Shape:** one GUI PATCH cycle (v0.31.1). No flag-name change → no `schema_mirror` delta; no toolkit involvement.

## 0. The bug + the corrected design stance

Secret+repeating+Text flags (`--ms1` on verify-bundle/import-wallet; `--share` on slip39-combine/ms-shares-combine, both **required**) render ONE `SecretLineEdit` (widget secret branch, `src/form/widget.rs:78`) while the assembler's repeating-secret arm reads rows from `state.values` (`src/form/invocation.rs:238-242`) → live forms emit NOTHING.

**Fix: per-row secrets stay in `secret_widgets` — the assembler comes to them.** `secret_widgets: BTreeMap<String, SecretLineEdit>` → **`BTreeMap<String, Vec<SecretLineEdit>>`** (scalar = the 1-element vec). Secrets never enter `state.values`; the never-persist invariant remains TYPE-level (`#[serde(skip)]` + freshly-defaulted in `redact_for_persistence`, `persistence.rs:99-103`); zeroize-per-widget is preserved per row (R0-r1 M1: the paste-warn MODAL is not wired live anywhere — only the predicate exists, deferral documented in tests/widget_secret.rs:19-25 — so no claim about it; the supersession note must NOT repeat the old fold comment's false "modal still fires" line). The v0.3 fold comment in `invocation.rs:222-237` (which documented the values-routing intent) is REWRITTEN to record the supersession and why.

## 1. Widget layer (`src/form/widget.rs`, the secret branch)

The secret branch (`flag_is_secret && FlagKind::Text` — branch order vs the non-secret repeating branch UNCHANGED) splits on `flag.repeating`:

- **Scalar (non-repeating)** — behavior byte-identical to today: one widget = `vec[0]` (`entry(name).or_insert_with(|| vec![SecretLineEdit::default()])`, render `[0]`).
- **Repeating** — mirror the v0.30.0 non-secret repeating UX, adapted:
  - Header row always: label + `?` help icon (repeating flags already qualify per `needs_help_icon`) + required marker + "+ add" button (appends an empty `SecretLineEdit`).
  - Render every row's `SecretLineEdit` (each its own masked `TextEdit`; positional auto-IDs are distinct — no salt work, noted from recon) + a per-row `✕` (collect-then-apply removal).
  - **Seed rule (the v0.30.0 rule, applied to the vec):** any render observing an empty vec for a `required` repeating secret flag seeds ONE empty widget (`--share` sites are required); optional (`--ms1`) seeds none. Removing the last required row respawns it next frame (intended).
  - Empty rows emit nothing (assembler-side guard, §2) — an added-but-blank row is inert.

## 2. Assembler (`src/form/invocation.rs`)

The secret-flag branch becomes uniformly `secret_widgets`-sourced:

```rust
// R0-r1 C1: the branch MIRRORS the widget dispatch (kind-gated), because
// flag_is_secret is kind-BLIND while the widget routes only Text secrets
// to secret_widgets. seed-xor's --share (secret+repeating+NodeValueComposite)
// renders via render_repeating into state.values and MUST keep emitting
// through the generic paths; Boolean *-stdin secrets currently emit NOTHING
// (the old kind-blind branch ate them) — that suppression is PRESERVED in
// this PATCH (FOLLOWUP boolean-stdin-secret-toggles-never-emit filed for
// whether they should emit).
if crate::secrets::flag_is_secret(flag) {
    if matches!(flag.kind, FlagKind::Text) {
        if let Some(rows) = state.secret_widgets.get(flag.name) {
            for w in rows {                      // scalar = 1-element vec
                if !w.is_empty() {
                    let value = w.as_string();   // Zeroizing<String>
                    argv.push(flag.name.to_string());
                    argv.push(value.as_str().to_string());
                }
            }
        }
        continue;
    }
    if matches!(flag.kind, FlagKind::NodeValueComposite(_)) {
        // fall through to the generic values paths (seed-xor --share keeps
        // emitting; values-routed composites are redaction-covered, §3)
    } else {
        continue; // Boolean *-stdin secrets: preserve today's no-emit
    }
}
```

- Row order = vec order = visual order (argv order preserved).
- The old repeating-secret `state.values` read is DELETED (it read a source the widget never wrote). The v0.3 fold comment block is replaced with the supersession note (§0).
- The transient plain `String` copies pushed into argv exist exactly as today's scalar path (the `as_string()` idiom) — unchanged posture.

## 3. Belt-and-suspenders: persistence redaction extension

`redact_for_persistence` (`src/persistence.rs:67`) additionally drops any `values` entry whose flag name is **schema-secret anywhere** — a new `schema_secret_flag_names()` union (every `FlagSchema` with `secret: true` across all 4 CLI schemas; currently `--ms1`, `--share` + the `SECRET_FLAG_NAMES` trio already covered). This closes the FUTURE-drift class (any later code path writing a secret-NAMED value would be redacted at persist) — note it is a NAME-level net, not a guarantee that secrets never enter `values` (see the xpub-search disclosure below). Drift test: every schema flag with `secret: true` is caught by the redaction filter (construct a FormState with a dummy value under each such name → redact → gone) AND `schema_secret_flag_names() ⊇ {--ms1, --share} ∪ SECRET_FLAG_NAMES` (an emptied union fails loud — R0-r1 M4). **Deliberate side effect (R0-r1 I3): the union also covers the 5 BOOLEAN `secret:true` `*-stdin` toggles (`--passphrase-stdin` ×12 sites, `--secret-stdin` ×2, `--decrypt-password-stdin` ×2, `--bip38-passphrase-stdin` ×1) — their persisted checkbox state now resets across restarts. Accepted: the `--passphrase-stdin` precedent already did this, the toggles carry no secret material, and a stale-persisted stdin toggle is itself a foot-gun. Those Booleans also currently EMIT nothing (the kind-blind secret branch ate them) — preserved in this PATCH; FOLLOWUP `boolean-stdin-secret-toggles-never-emit` filed.**

**Second deliberate side effect (R0-r2 I-NEW1 — a live leak incidentally closed):** the full schema-secret census also carries Text secrets `--decrypt-password` ×2, `--secret` ×2, `--digits` ×1, and ms.rs `--hex` ×3 / `--phrase` ×4 — all inert for the union (already widget-routed; twin-checked) EXCEPT TWO names (R0-r3 I-NEW2): **`--phrase`**, colliding with three `secret: false` Text flags on the mnemonic xpub-search subcommands ("Master BIP-39 phrase (inline)", `mnemonic.rs:2280-2286/:2442-2448/:2712-2718`). Those are values-routed and **persist master phrases to `state.json` in PLAINTEXT today**; under this union they silently stop persisting — accepted and welcomed (it closes the leak at the persistence boundary). The underlying mis-classification (xpub-search `--phrase` should plausibly be `secret: true`, which would flip its widget to a masked `SecretLineEdit`) is OUT of this PATCH's scope → FOLLOWUP `xpub-search-inline-phrase-not-secret-classified` filed.** **And `--ms1` (R0-r3 I-NEW2): `ms repair --ms1` (`ms.rs:314-324`) is `secret: false` Text/required — values-routed, so the to-be-repaired ms1 string (master-secret material, merely BCH-corrupted) persists to `state.json` in PLAINTEXT today; the union closes this second live leak the same way. FOLLOWUP `ms-repair-ms1-not-secret-classified` filed.** Third adjacent finding, OUTSIDE the flag-name net (R0-r3 m-NEW1): the codex32 combine `shares` POSITIONAL ("Secret-equivalent", `ms.rs:441-448`) rides `state.positionals`, cloned unredacted at persist — pre-existing, untouched by this PATCH; FOLLOWUP `positional-secrets-not-redacted-at-persist` filed. Census-method note (R0-r3 m-NEW2): the §3 drift test extracts the `secret` FIELD from the schema structs — never a text grep (2 comment lines in mnemonic.rs contain the literal `secret: true`).

## 4. Call-site migration (6 src sites + 9 test sites — R0-r1 C2/I1)

`src/schema/mod.rs:295` (type + `:308` default); **`src/schema/mod.rs:339-347` `FormState::has_value` — THE SILENT SITE (R0-r1 C2): `Vec::is_empty` compiles with inverted meaning; per-row semantics required: `rows.iter().any(|w| !w.is_empty())`** (otherwise run-confirm fires on every passphrase run and the conditional XOR gating misfires); `src/form/widget.rs:78` (scalar entry → `or_insert_with(|| vec![SecretLineEdit::default()])`, render `[0]`); `src/form/invocation.rs:238-253` (the secret branch rewrite per §2 — R0-r2 m1); `src/persistence.rs:103` (fresh default); **`src/secrets.rs:292-294` `zeroize_form_state` (R0-r1 I1): per-row sweep `values_mut().flatten()`** (compile-caught — `zeroize` is an inherent method). Plus 9 mechanical test sites (§5). "Compile errors enumerate missed sites" is FALSE for `has_value` — hence this explicit census.

## 5. Tests

- **THE bug cell (live-path, previously impossible):** drive the REAL widget for `import-wallet --ms1` — mechanism (R0-r1 I2): kittest `Node::focus()` + `type_text()` into the masked TextEdits (first in-repo use; masking is display-only so routed text events should land). FALLBACK if accesskit/password interaction misbehaves: seed the rows via `SecretLineEdit::from_text` and still drive the REAL `render_with_dispatch` for add/remove/seed coverage — that still pins the render→assemble seam the bug lived in (unlike the old values-synthesis). Assert `--ms1 v1 --ms1 v2` in row order. Twin for `slip39 --share` (required: the seeded row appears without clicking add).
- **Scalar regression:** `--passphrase` behavior byte-identical (existing cells keep passing).
- **MIGRATE to the vec source (R0-r1 I4):** `cell_import_wallet_repeating_ms1_argv` (kittest_import_wallet_form.rs:157-213), `cell_import_wallet_env_sentinel_literal_emission` (:322-350 — the `@env:` sentinel rides a secret row), `cell_v0_3_slip39_combine_argv_assembles` (widget_interaction.rs:296-336). **KEEP UNCHANGED as the counter-example pin:** `cell_v0_3_seed_xor_combine_argv_assembles` (:378-434 — NodeValueComposite keeps the values path per §2). Mechanical `insert(name, vec![widget])` migrations: argv_assembler.rs:41/:282/:322, argv_assembler_visibility.rs:198/:218, secrets.rs:164/**:171 (the empty-widget negative — migrate FAITHFULLY as `vec![SecretLineEdit::new()]`, it is the C2 net)**, persistence.rs:392. Unaffected: r7_no_auto_repair_removal.rs ambient values. Plus: a values-synthesized repeating **Text**-secret entry now emits NOTHING (assert the dead path) AND is redacted at persist.
- **Required-seed cell:** slip39-combine default form shows one `--share` row; removing it respawns.
- **Empty-row cell:** added-but-blank rows emit nothing.
- **Persistence cells:** (a) `secret_widgets` rows never persisted (type-level — existing invariant test extends to the Vec shape); (b) the §3 drift test (dummy secret-named values entries → redacted).
- **Paste-warn predicate tests** keep passing (pure-predicate; the modal is not live — M1). **Residue note (R0-r1 M2):** row removal orphans egui `TextEditState` (undo-ring plaintext snapshots) at the vacated trailing positional ID — same class as the existing `gui-secret-buffer-allocator-residue` FOLLOWUP; recorded, no new work this cycle.
- Full suite with the 4 pinned binaries + clippy clean.

## 6. Release

GUI **PATCH v0.31.1**: CHANGELOG `[0.31.1]` (note the pub `FormState.secret_widgets` type change — a lib-API break; the crate is app-first, R0-r1 M3); version bump + lock + README self-tag; full suite → push → CI green → tag `mnemonic-gui-v0.31.1` → tag-build green; toolkit `scripts/install.sh:44` GUI pin → v0.31.1 (checklist item); flip the FOLLOWUP (recording the inverted fix direction + why).

## 7. Source grounding (verified at `dabbdfe`)

- `src/form/widget.rs:76-93` secret branch (single-entry; the v0.30.0 dispatch comment documenting the deferral); `:78` the `entry().or_default()`.
- `src/form/invocation.rs:222-243` the v0.3 fold comment + the dead values-read (repeating) + the live scalar `secret_widgets.get`.
- `src/persistence.rs:67-105` `redact_for_persistence` (the three drop classes; `secret_widgets` freshly-defaulted `:99-103`).
- `src/form/secret_widget.rs:32-34` `SecretLineEdit { buf: Zeroizing<Vec<u8>> }`; `:68-84` `show` (positional TextEdit, transient zeroized); `as_string -> Zeroizing<String>`.
- `src/secrets.rs:141-151` `SECRET_FLAG_NAMES` (3 passphrase flags) + `flag_is_secret` (schema-secret OR name-list).
- Census: `--ms1` ×2 optional; `--share` ×2 Text **required** (+1 NodeValueComposite, unaffected — the working counter-example).

---

## Fold log

- **R0 round 1 (RED → folded, 2026-06-10; persisted at `design/agent-reports/gui-v0_31_1-repeating-secrets-r0-r1-review.md`):** C1 the assembler secret branch is now KIND-GATED mirroring the widget dispatch (Text → the vec; NodeValueComposite → falls through, seed-xor keeps working; Boolean *-stdin → today's no-emit preserved + FOLLOWUP filed). C2 `FormState::has_value` identified as THE silent migration site (Vec::is_empty compiles with inverted meaning) → per-row semantics pinned + the empty-widget negative migrates faithfully. I1 zeroize_form_state per-row sweep. I2 kittest focus+type_text named + from_text fallback. I3 the 5 Boolean-secret toggles' persistence reset stated deliberately. I4 migrate/keep test lists enumerated. M1 paste-warn claims removed (modal not live). M2 TextEditState residue recorded (existing FOLLOWUP class). M3 lib-API note. M4 union ⊇ census assertion. The review ADJUDICATED the inversion correct (values-routed secrets would persist; today's code even has a latent crafted-state.json variant that §2+§3 both close).
- **R0 round 2 (YELLOW → folded, 2026-06-10; persisted at `design/agent-reports/gui-v0_31_1-repeating-secrets-r0-r2-review.md`):** all 10 round-1 folds verified (C1 fall-through traced to emit_one's composite arm; Boolean render/no-emit sub-trace confirmed; no repeating Boolean secret exists). I-NEW1 folded: the union's `--phrase` name collision DISCLOSED (it incidentally closes a live plaintext master-phrase persistence leak on xpub-search) + §3's false "nothing writes secrets into values" sentence corrected (name-level net, not a guarantee) + FOLLOWUP `xpub-search-inline-phrase-not-secret-classified` to be filed. m1 citation tightened (:238-253).
- **R0 round 3 (YELLOW → folded, 2026-06-10; persisted at `design/agent-reports/gui-v0_31_1-repeating-secrets-r0-r3-review.md`):** round-2 folds verified applied. I-NEW2: a SECOND undisclosed secret:false twin — `ms repair --ms1` persists raw ms1 material in plaintext today; disclosure extended + FOLLOWUP `ms-repair-ms1-not-secret-classified`. m-NEW1: the codex32 `shares` positional leak (outside the name net) → FOLLOWUP `positional-secrets-not-redacted-at-persist`. m-NEW2: the drift test extracts the field, never greps text.
- **R0 round 4 (GREEN confirm, 2026-06-10; persisted at `design/agent-reports/gui-v0_31_1-repeating-secrets-r0-r4-review.md`):** all three round-3 folds verified accurate (the --ms1 twin census complete at 7+1; the positional + field-extraction notes exact). **Gate satisfied.**
