# SPEC — GUI v0.39.0: mask secret values in every on-screen command display (Item 1)

**Cycle:** mnemonic-gui v0.39.0 (MINOR) · **Source SHA:** `71c7ecd` (= v0.38.0) · **Recon:** `cycle-prep-recon-secret-exposure-cluster.md` §ITEM 1.
**User decision:** Item 1 = **(d) mask the on-screen command (preview + run-confirm modal + last-run display); copying the real runnable command is a deliberate, labeled action.**
**Resolves:** `run-confirm-and-preview-show-secrets-cleartext`.
**RE-SCOPED from the original combined spec** (R0 round-1 split recommendation + C1 scope inflation): this cycle is **Part A (masking) ONLY**. Item 3 (paste-warn wiring) + the CHANGELOG-honesty correction (round-1 I1) move to **v0.40.0** — see §Deferred. The round-1 review is `design/agent-reports/gui-v0-39-0-mask-paste-warn-r0-round1-review.md`.

## Problem (verified @ `71c7ecd`)

`assemble_argv` (`src/form/invocation.rs:126-315`) pushes real secret VALUES into argv. There are **FOUR** secret argv-token sources (NOT two — round-1 C1 found the composite; this re-scope adds the slot token), exactly mirroring the four arms of `should_confirm_run` (`src/secrets.rs:200-236`):

1. **Secret Text flag** — value token at `invocation.rs:268` (`argv.push(value.as_str().to_string())`).
2. **Secret slot row** — `SlotState::to_slot_argv` (`slot_editor.rs:175-190`) emits `["--slot", "@{index}.{subkey}={value}"]`; for a secret-bearing subkey (Phrase/Seedqr/Entropy/Ms1/Wif/Xprv) the SECOND token carries the cleartext secret (`:182-187`).
3. **Secret positional** — value token at `invocation.rs:302` (`argv.push(value.as_str().to_string())`).
4. **`NodeValueComposite` value** — `emit_one` (`invocation.rs:380-390`) pushes `format!("{}={}", node, value)` for BOTH the `secret:true` flag `--share` (falls through the secret branch at `:274-276`) AND the `secret:false` but value-dependent `--from phrase=<seed>` (not a secret flag; reaches `emit_one` at `:285-287`; secret because the NODE is in `SECRET_NODE_TYPES`).

`render_copy_command` (`:422-435`) shell-quotes only — no masking. These secret tokens are then shown cleartext on **three pure-display surfaces** + copied by the copy buttons:

- **D1 — live `Preview:` label** (`main.rs:986`): always on screen, no opt-in. The highest-value leak.
- **D2 — run-confirm modal token list** (`main.rs:1023-1026`): the modal that exists *because* a secret is present prints it cleartext.
- **D3 — last-run `argv:` display** (`main.rs:459-465`): the executed command, behind the `show command-line` checkbox.
- **Copy buttons** (`main.rs:952-959`, copy at `:988-999`): POSIX/Windows copy the REAL command — this is the deliberate-reveal half of decision (d); keep real, relabel.

## Design — one mask, computed once, flowing to every display surface

### Mechanism: a parallel secret-mask on argv

1. **`invocation.rs` — `assemble_argv_with_secret_mask(schema, sub, state) -> (Vec<String>, Vec<bool>)`** becomes the real implementation; `assemble_argv` is a thin wrapper returning `.0` (signature unchanged — many callers). The mask is **correct-by-construction**: every `argv.push` is paired with a `mask.push` so `mask.len() == argv.len()` structurally. Mask bit = `true` iff the token is a secret VALUE:
   - cli_name, subcommand name, every flag-NAME token, PinValue tokens, sentinels, `--spec`/`-` → `false`.
   - secret Text value (`:268`) → `true`.
   - secret positional value (`:302`) → `true`.
   - slot value token → `slot_subkey_is_secret(row.subkey)` (see step 2).
   - composite value token in `emit_one` → `flag_is_secret(flag) || node_type_is_secret(node)` (see step 3).
2. **`slot_editor.rs` — `SlotState::to_slot_argv_masked(&self) -> Vec<(String, bool)>`:** identical to `to_slot_argv` but pairs each token with its mask bit — `false` for the `"--slot"` token, `row.subkey.is_secret_bearing()` for the `"@N.subkey=value"` token. `to_slot_argv` becomes `to_slot_argv_masked().into_iter().map(|(t,_)| t).collect()` (thin wrapper — single source of the format string). Using `is_secret_bearing()` (slot-editor-local) is sound: v0.38.0's T3 pins `is_secret_bearing() == slot_subkey_is_secret()` for all 10 variants, so the slot mask gate equals the persistence/`secrets` gate.
3. **`emit_one` gains a `&mut Vec<bool>` param** (threaded from `assemble_argv_with_secret_mask`). Every `argv.push` inside `emit_one` pairs a `mask.push`: `false` for flag-name tokens and all non-composite value tokens (Number/Dropdown/Range/Timestamp/TaggedOrIndexed/Path are never secret here — secret Text/positional are handled before `emit_one`, secret non-Text/non-Composite flags are Boolean-suppressed), and `flag_is_secret(flag) || node_type_is_secret(node)` for the composite value token (`:389`). Import `crate::secrets::{flag_is_secret, node_type_is_secret}` into `invocation.rs`.
4. **`render_copy_command_masked(argv, mask, flavor) -> String`:** like `render_copy_command`, but for every `mask[i] == true` token emits a fixed `SECRET_MASK` placeholder **un-quoted** (it is a display sentinel, never run — M3), and shell-quotes the rest as today. `SECRET_MASK` is a single `pub const SECRET_MASK: &str = "••••"` (4×`\u{2022}`) defined once in `invocation.rs` (M2). `render_copy_command` stays for the real (reveal/copy) path.

### Wiring the three display surfaces (`main.rs`)

- **`:898`:** replace `assemble_argv(sch, sub, state)` with `assemble_argv_with_secret_mask(...)` → `(argv, mut mask)`. The tree-mode `--spec`/`-` append (`:917-920`) must push two `false` onto `mask` to stay aligned.
- **D1 `:932`/`:986`:** `preview = render_copy_command_masked(&argv, &mask, Posix)`. **Split the alias:** `:934` currently sets `argv_posix = preview.clone()` and `:994` copies `argv_posix` as the REAL command — so `argv_posix` MUST be recomputed as the REAL `render_copy_command(&argv, Posix)` (the masked `preview` can no longer alias it). `argv_windows` (`:933`, copy path) stays REAL.
- **D2 `:1007`/`:1015`/`:1023-1026`:** `pending_confirm_argv` grows the mask: `Option<(Vec<String>, Vec<bool>, Option<Vec<u8>>)>` (mirror the existing tuple at `main.rs:108`). The modal loop renders `if mask[i] { SECRET_MASK } else { tok }`. Run still spawns the REAL `argv`.
- **D3 `:459-465`:** add `pub mask: Vec<bool>` to `runner::RunResult` (`runner.rs:18-25`); the last-run display renders `render_copy_command_masked(&result.argv, &result.mask, Posix)`. **Population path (R0-r2 I1 — `RunResult` is built in the runner layer, which has no concept of the display mask):** option (a), post-construction assignment. `run_with_stdin`'s struct literal (`runner.rs:148-153`) initialises `mask: Vec::new()` (one added line; the runner stays mask-oblivious). `spawn_and_capture` (`main.rs:1110`, 2 call sites `:1009`/`:1031`) receives a `mask: Vec<bool>` parameter and, immediately after `run_with_stdin` returns `Ok(result)`, assigns `result.mask = mask` BEFORE storing in `app.last_run`. (Rejected: (b) a `MaskedRunResult` wrapper — needless; (c) a `mask` param on `run_with_stdin` — leaks the GUI display concern into the runner layer.) The mask is already computed at `:898` and stored in `pending_confirm_argv` for the confirm path; the no-confirm path at `:1009` passes it directly.
- **Copy buttons (`:952-959`):** keep copying the REAL command. When `mask.iter().any(|&m| m)`, suffix the labels to make the reveal explicit — `"Copy command (POSIX) — reveals secret"` / `"Copy command (Windows) — reveals secret"` (and/or a hover warning). This is the "deliberate, labeled" half of (d): the DEFAULT display is masked; the copy is opt-in and now tells the truth.

## Part A tests (TDD)

- **T-A1 (mask correctness, per-source):** for each secret source, a known secret value → `assemble_argv_with_secret_mask` returns `mask.len()==argv.len()`, the secret value token masked `true`, the flag-name + any co-present non-secret value masked `false`:
  - (a) secret Text flag (`bundle --passphrase`);
  - (b) secret slot row (`@0.phrase=<seed>` — and a watch-only `@0.xpub=<xpub>` row NOT masked, discriminating);
  - (c) secret positional (`ms combine <shares>`);
  - (d) secret composite flag (`--share`);
  - (e) value-dependent composite — `--from phrase=<seed>` masked `true`; `--from xpub=<xpub>` masked `false` (the discriminating node-secrecy case C1 named).
  - no-secret subcommand → mask all-false.
- **T-A2 (masked render hides every secret):** for a state populating ALL four sources at once, `render_copy_command_masked(argv, mask, Posix)` contains NONE of the known secret substrings and DOES contain `SECRET_MASK` and the flag names; the real `render_copy_command(argv, Posix)` still contains them (reveal path intact). Assert `SECRET_MASK` appears literally (un-quoted — M3).
- **T-A3 (confirm-gate lower bound, anti-split-brain):** for each populated-secret state in T-A1, `mask.iter().any(|&m| m)` is true AND `should_confirm_run(sub, state)` is true. **The load-bearing assertion is the DANGEROUS direction (R0-r2 M3):** assert as a per-case invariant over EVERY T-A1 vector that `mask.any() ⟹ should_confirm_run` — i.e. no state where a token is masked but the confirm modal would NOT fire (that would mean a secret renders masked in the preview yet the run proceeds without the secret-bearing-run warning). Make it an explicit assertion, not prose. DOCUMENT the one safe asymmetry in the OTHER direction: a secret Boolean `*-stdin` toggle makes `should_confirm_run` true but emits NO token (greyed since v0.37.0) → mask all-false — safe (nothing to leak), so the relation is `mask.any() ⟹ should_confirm_run`, NOT `⟺`.
- **T-A4 (kittest, D1 preview):** render the action bar (or the `Preview:` label) for a secret-bearing form; assert the label text contains `SECRET_MASK` and NOT the secret. If the action bar isn't cleanly harness-isolable, assert at the `render_copy_command_masked` seam (T-A2 covers the security property) and record which — mirror v0.38.0's harness-isolation discipline.
- Existing suite green; no schema change (no flag-name/secret-bit delta → no `schema_mirror` / manual / toolkit-pin impact).

## Ritual

CHANGELOG `[0.39.0]`; version bump (Cargo.toml + Cargo.lock + README self-pin `:42`); FOLLOWUPS resolve `run-confirm-and-preview-show-secrets-cleartext` (record the (d) decision + the four-source mask + three display surfaces). File a FOLLOWUP for the deferred tree-mode pipeline masking + the composite paste-warn parity (round-1 M1). No toolkit pin / schema_mirror / manual impact. SemVer **MINOR** (user-visible: secret values now masked in every on-screen command display).

## Deferred (→ v0.40.0, with rationale)

- **Item 3 — paste-warn wiring** (`paste-warn-modal-dead-code`, `paste-warn-live-wiring-untested`): independent files (`secret_widget.rs`/`main.rs` modal state) + the `SecretLineEdit::show` return-shape change touching every secret-widget call site (round-1 I2: `widget.rs:110`, `widget.rs:153`, `main.rs:830` — re-grep at that cycle). Round-1 I1 (CHANGELOG over-claims paste-warn "fires") lands there, where the wiring makes the claim TRUE; until then the claim stays a documented-but-not-wired item (the v0.31.1 partial self-correction already flags half). The specific stale lines to correct at v0.40.0 fold time are `CHANGELOG.md:1978-1980` (`cell_paste_warn_modal_trigger` "validates the paste-warn modal text and behavior on `SecretLineEdit` paste events" — only the predicate is tested) and `CHANGELOG.md:2196` (the non-goals paste-warn-modal reference) — both line numbers decay, re-grep at that cycle. v0.40.0 owns the honest re-word.
- **Tree-mode POSIX pipeline** (`main.rs:926-927`/`:992`, `tree_form::posix_pipeline_command`): the build-descriptor spec JSON is masked via a DIFFERENT mechanism (JSON-string redaction, not the argv mask). Build-descriptor's policy tree is over XPUBS (watch-only) — none of its node types are in `SECRET_NODE_TYPES` — so the exposure is nil/low; defer to a FOLLOWUP. **R0: confirm build-descriptor tree nodes cannot carry a secret-class value** (if they can, pull this back in).

## Non-goals

The stdin-emit alternative (option b — reopens v0.37.0); copying a masked (non-runnable) command (the copy is the deliberate reveal); OS-snapshot occlusion (`gui-os-snapshot-secret-occlusion`); the allocator-residue limit (`gui-secret-buffer-allocator-residue`).
