# Cycle-prep recon — GUI secret-exposure cluster

**Repo:** mnemonic-gui, master @ `54c13c3` (= tag `mnemonic-gui-v0.37.0`).
**Mode:** RECON ONLY. No implementation.
**Scope:** the 3 open secret-exposure FOLLOWUPS (4 slugs) in the
`audit-2026-06-10-backlog` index (`FOLLOWUPS.md:8-29`, status **open**).

## Citation-drift note (read first)

The prompt's line numbers are the **audit snapshot** and have decayed across
v0.33.0/v0.34.0/v0.35.0/v0.37.0 merges. The FOLLOWUPS.md index lines are
similarly stale. **Live** (re-grepped at `54c13c3`) citations used below:

| Claim | Prompt / FOLLOWUPS cite | LIVE @ 54c13c3 |
|---|---|---|
| secret Text → argv push | `invocation.rs:255-273` | `invocation.rs:255-272` (Text secret arm), + `:297-305` (secret positional arm) |
| preview label | `main.rs:986` / `main.rs:842-844` | `main.rs:986` ✓ |
| copy buttons | `main.rs:932/986+` | buttons `main.rs:952-984`; copy-action `:988-1004`; argv build `:898/932-934` |
| run path | `pending_confirm_argv → runner` | `main.rs:1005-1011`; run-confirm modal `:1014-1038`; runner `runner.rs:74/99` |
| slot value edit | `slot_editor.rs:219-236` | `slot_editor.rs:219-228` (edit), `:234-235` (remove) |
| paste-warn dead code | `secrets.rs:164-196` | `secrets.rs:164-196` ✓ (`PASTE_WARN_MODAL_TEXT` :167, `should_warn_on_paste` :194, `PASTE_WARN_THRESHOLD` :189) |

All four slugs are still **open**. None has been silently resolved.

---

## ITEM 1 — `run-confirm-and-preview-show-secrets-cleartext` [obs]

### Current state (evidence)

**Secrets enter argv** at two sites in `assemble_argv`
(`src/form/invocation.rs`):
- `:255-272` — secret **Text** flags: `w.as_string()` (a `Zeroizing<String>`)
  → `argv.push(flag.name)` + `argv.push(value.as_str().to_string())`. The real
  secret value goes into the argv vector as a plain `String`.
- `:297-305` — secret **positionals**: same, `argv.push(value.as_str()...)`.

**`render_copy_command`** (`src/form/invocation.rs:422-435`) does
**shell-quoting only** (POSIX `posix_quote`/shlex `:445`, Windows `cmd_quote`
`:477`). **No masking hook, no redaction parameter** — it faithfully
reproduces every argv token including the secret. Its doc says the output is
"for the user's eyes only … NEVER used to spawn the subprocess" (`:5-10`,
`:419-421`).

**Consumers of the assembled argv** (`src/main.rs`, the form action bar at
`:898-1038`):

| # | Consumer | Site | Needs real secret? | Currently leaks? |
|---|---|---|---|---|
| 1 | **Preview label** `ui.label("Preview: {preview}")` | `:986` (`preview` = `render_copy_command(&argv, Posix)` `:932`) | NO | **YES** — cleartext on screen, unconditional |
| 2 | **Copy command (POSIX)** button → `ctx.copy_text(argv_posix)` | btn `:952-957`, copy `:988-996` | depends (see below) | **YES** — real secret to clipboard |
| 3 | **Copy command (Windows)** → `ctx.copy_text(argv_windows)` | btn `:958-959`, copy `:997-999` | depends | **YES** |
| 4 | **Run** → `pending_confirm_argv = Some((argv, …))` then `spawn_and_capture` | `:1005-1011` | **YES** (real value must reach the binary) | not a *display* leak — but see #5 |
| 5 | **Run-confirm modal** renders argv token-by-token: `for tok in &argv { ui.monospace("  {tok}") }` | `:1023-1026` | NO (display) | **YES** — the confirm modal ALSO prints secrets cleartext |
| 6 | Copy spec JSON (tree mode only) | `:1000-1004` | n/a — spec JSON, no inline secret | NO |

So there are **THREE display/clipboard leaks** (preview #1, copy buttons #2/#3,
confirm modal #5), not just the two the FOLLOWUPS body names. The confirm
modal (#5) is the irony: the modal that exists *because* a secret is present
displays the secret cleartext.

The run path (#4) is the **only** consumer that genuinely needs the real
secret value in argv (it's `Command::args()` — `runner.rs:114-125`).

### The toolkit stdin alternative — DOES exist, with a one-channel caveat

The toolkit schema carries `--passphrase-stdin` / `--secret-stdin` /
`--bip38-passphrase-stdin` (`mnemonic.rs:276/511/643/853/863/1228`, etc.) and
`=-` / `@env:VAR` sentinel forms (`mnemonic.rs:777/1159/1263`). These let a
caller pass the flag NAME on the command line and feed the secret VALUE via
stdin — so a copy-command emitting `--passphrase-stdin` carries no inline
secret.

**The GUI runner already has a single stdin feed:** `run_with_stdin(argv,
Option<Vec<u8>>)` (`runner.rs:99-145`, added v0.32.0 for tree-mode `--spec -`).
`pending_confirm_argv` is `(Vec<String>, Option<Vec<u8>>)` (`main.rs:108`) —
the stdin slot is already plumbed through Run and the confirm modal.

**Caveat (load-bearing):** there is exactly **ONE** stdin channel per
invocation. Tree-mode already consumes it for `--spec -`. And these `*-stdin`
Boolean toggles are the **suppressed/greyed-out** family
(`boolean-stdin-secret-toggles-never-emit`, RESOLVED v0.37.0 = *grey out, do
NOT emit*; the emit-alternative was explicitly declared a **non-goal**). So
fix-option (b) directly reopens a decision the user JUST closed in v0.37.0 the
other way. A multi-secret subcommand (e.g. `--passphrase` + a secret slot)
cannot route every secret through one stdin channel.

### Fix options (genuine UX/security tradeoff — needs USER DECISION)

- **(a) Mask the secret in the PREVIEW + confirm-modal display only; keep the
  real argv for Run.** Add a `render_copy_command_masked` (or a `mask: bool`
  param) that replaces secret-flag VALUES with `****` — driven by
  `flag_is_secret` / the secret-widget set, which the assembler already knows.
  *Sub-decision for the copy buttons:* copy the **masked** form (not runnable —
  honest but surprising: "I copied the command and it doesn't work") OR copy
  the **real** form (runnable, but a clipboard leak — clipboards are read by
  other apps / sync to phones). Recommend: preview + confirm-modal masked
  always; copy buttons → **masked by default** + an explicit
  "Copy with secrets (clipboard exposure)" affordance (this is essentially
  option (d)). Lowest blast radius; the run path is untouched; no runner change.

- **(b) GUI emits the stdin/sentinel form** (`--passphrase-stdin` / `=-`) so
  neither preview nor copy carries the secret, and the runner feeds the value
  via the existing `run_with_stdin`. **Reopens the v0.37.0 grey-out decision**
  (those toggles were deliberately made non-emitting); needs the
  single-stdin-channel arbitration (which secret wins stdin when there are
  multiple); copy-command becomes runnable-without-secret ONLY if the user also
  pipes the value (so the *copied* command still isn't self-contained). High
  coupling, high re-litigation cost. **Not recommended as the cluster's fix.**

- **(c) Suppress the preview entirely when `should_confirm_run(sub, state)` is
  true** (`secrets.rs:200-236` already computes exactly "a secret is present").
  Cheapest. Cost: the user loses the preview for the very invocations where a
  dry-run sanity check is most valuable; copy buttons still leak unless also
  gated. Blunt.

- **(d) Mask preview (+ confirm modal) + a separate explicit
  "Copy (reveals secret)" action button.** = (a) with the copy-button
  sub-decision resolved toward an explicit opt-in. The masked preview is the
  default; revealing/copying the real command is a deliberate second click.
  Best security/usability balance; slightly more UI.

**Recommendation:** **(d)** (which subsumes (a)). Mask in `render_copy_command`
via a new masked variant keyed on the secret set the assembler already has;
default the copy buttons to masked with an explicit reveal/copy-real button;
leave Run untouched. Do NOT pursue (b) here — it re-opens v0.37.0.
**This item REQUIRES a user decision** ((a)/(c)/(d) and the copy-button
masked-vs-real sub-choice) before R0.

---

## ITEM 2 — `slot-secret-values-rendered-unmasked` [minor] (most self-contained)

### Current state (evidence)

`src/form/slot_editor.rs::render` `:201-261`. The per-row value edit
(`:219-228`) branches ONLY on `(SlotSubkey::Path, Some(hint))` (placeholder
hint) vs a `_` fallback `ui.text_edit_singleline(&mut row.value)` (`:226`).
**No branch on `row.subkey.is_secret_bearing()`** → secret-bearing subkeys
(Phrase / Seedqr / Entropy / Ms1 / Wif / Xprv — `is_secret_bearing` `:82-92`)
render in **plaintext**.

**Remove drops a plain String without zeroize:** `:234-235`
`state.rows.remove(i)` drops a `SlotRow` (`:97-102`) whose `value: String`
(`:100`) is a plain heap allocation — no `Zeroize` on the way out.

### State shape — `SecretLineEdit` is NOT directly reusable

`SlotRow.value` is a **plain `String`** (`:100`), and `SlotRow` derives
`Serialize`/`Deserialize`/`Clone` (`:97`). `SecretLineEdit`
(`secret_widget.rs:32-34`) owns a `Zeroizing<Vec<u8>>`, is **non-`Clone` by
design** (`:21-24`), and is **`#[serde(skip)]`** in `FormState` (the
never-persist-by-type invariant). Swapping `SlotRow.value`'s type to
`SecretLineEdit` would (a) break `SlotRow`'s derives and the
`SlotState`/`state.json` round-trip, and (b) require all `value` consumers
(`to_slot_argv` `:150-165`, `rows_sorted`, `persistable_rows`,
`detect_slot_index_gaps`) to change. So the realistic fix is NOT "store a
SecretLineEdit per row" — it's:
- **render-side:** when `row.subkey.is_secret_bearing()`, render the value
  with `egui::TextEdit::singleline(&mut row.value).password(true)` (mirrors
  `SecretLineEdit::show`'s `.password(true)` `:74`) instead of plain
  `text_edit_singleline`. Keep the `String` storage.
- **remove-side:** `row.value.zeroize()` before/at `state.rows.remove(i)` for
  secret-bearing rows (the `zeroize` crate is already a dep; `String:
  Zeroize`).
- (optional, matches Item 1 (d)) thread paste-warn here too — but that depends
  on Item 3.

### Persistence story — CONFIRMED: secret slot VALUES already never persist

`redact_for_persistence` (`persistence.rs:75`) filters slot rows at
`:105-111`: `.filter(|r| !SECRET_SLOT_SUBKEYS.contains(&r.subkey.as_str()))`.
So secret-bearing slot rows are dropped at persist — the v0.34.0 §5 recon claim
holds. **This item is render-side + transient-memory only**; there is **no
on-disk leak** and **no funds-safety dimension**.

### Recommendation

Straightforward, **no user decision required**:
`.password(true)` gated on `is_secret_bearing()` + zeroize-on-remove. The only
design micro-choice is whether secret slot values get the masked treatment but
keep the non-secret `Path` hint affordance — keep the existing `Path` arm,
add a secret arm ahead of the `_` fallback. **Best first/standalone cycle.**

---

## ITEM 3 — `paste-warn-modal-dead-code` [minor] + `paste-warn-live-wiring-untested` [minor]

### Current state (evidence)

`src/secrets.rs:164-196`:
- `PASTE_WARN_MODAL_TEXT` (`:167-181`), `PASTE_WARN_THRESHOLD = 8` (`:189`),
  `should_warn_on_paste(flag, len)` (`:194-196`, signature `(&FlagSchema,
  usize) -> bool`; body `flag_is_secret(flag) && paste_len >= THRESHOLD`).
- **Verified dead in `src/`:** grep for `should_warn_on_paste`,
  `PASTE_WARN_MODAL_TEXT` over `src/` returns ONLY their definition sites.
  Their only consumers are `tests/widget_secret.rs` + `tests/secrets.rs` (pure
  predicate assertions). `widget_secret.rs:18-24` itself documents that the
  "live modal-state on the app" check is **deferred** (= the
  `paste-warn-live-wiring-untested` slug).
- **`SecretLineEdit` does no paste detection:** `secret_widget.rs::show`
  `:68-84` renders `TextEdit::singleline(...).password(true)` and reacts only
  to `response.changed()`. No event inspection.
- **No app-state plumbing exists:** there is NO `pending_paste_warn` field on
  the app (`main.rs` has only `pending_confirm_argv` `:108`). The
  `widget_secret.rs:18` comment references `app.pending_paste_warn` — a field
  that does not exist. So wiring is greenfield, not a one-line hook.

### Prose/SPEC claims that say paste-warn is ACTIVE (the "lie")

- **CHANGELOG.md** has several entries asserting secret flags get paste-warn as
  a *live* mitigation: `:229`, `:255`, `:281` ("fires the paste-warn /
  run-confirm modals"), `:1969-1971` ("validates the paste-warn modal text and
  behavior on `SecretLineEdit` paste events" — overstated; the test is a pure
  predicate, no live paste event), `:2187` (manual-style: occlusion "paste-warn
  modal copy mentions this").
- **v0.31.1 CHANGELOG `:70` already SELF-CORRECTS:** "the modal predicate
  exists but is not wired live." So the codebase already half-acknowledges the
  dead code — the remaining prose at `:229/:255/:281` is the stale half.
- **README.md:** no paste mention (does not lie — good).
- SPEC bodies (`design/SPEC_gui_v0_31_1_*`, `v0_33_0_*`, `v0_34_0_*`) +
  phase reports reference `PASTE_WARN_MODAL_TEXT` byte-exactness, not live
  firing.

### Feasibility of WIRING (option a) — egui 0.31 supports it

Pinned `egui = "0.31"` (Cargo.toml:13-14; resolved 0.31.1). `egui::Event::Paste(String)`
exists (`egui-0.31.1/.../data/input.rs:388`) and `InputState.events:
Vec<Event>` is public (`input_state/mod.rs:225`). **Crucially, `TextEdit` does
NOT consume `Event::Paste` from the global `i.events` vec** — it processes
paste on a *filtered copy* and only retains-out IME events (`builder.rs:802`,
paste handled at `:947` without a `retain`). So `SecretLineEdit::show` CAN
inspect `ui.input(|i| i.events.iter().find_map(|e| match e {
Event::Paste(s) => Some(s.len()), _ => None }))` after the `TextEdit` call,
call `should_warn_on_paste`, and (option a) set a `pending_paste_warn` app flag
to drive a modal next frame (mirroring the `pending_confirm_argv` pattern at
`main.rs:1014-1038`). Wiring is **feasible** but is **not trivial** — it needs:
a new app field, a modal render block, and the widget→app signal path (the
widget currently has no app handle; the per-frame return shape would change).

### Recommendation: depends on Item 1's outcome

- If the cluster ships masking + run-confirm hardening (Item 1 (d)) and the
  team wants defense-in-depth on **paste specifically**, **WIRE it** (option a)
  — the predicate + threshold + modal text already exist and egui supports the
  detection; the marginal cost is the app-field + modal + widget-signal.
- If not, **REMOVE the dead code** (option b): drop `PASTE_WARN_MODAL_TEXT` /
  `should_warn_on_paste` / `PASTE_WARN_THRESHOLD` and **downgrade the
  CHANGELOG `:229/:255/:281` prose** from "fires the paste-warn modal" to the
  honest run-confirm-only mitigation. Cheapest; removes the lie.

**Net recommendation:** the run-confirm modal (Item 1) is the *substantive*
secret gate; paste-warn is a nice-to-have second tier. If Item 1 lands masking,
**wire** paste-warn alongside (the marginal work is small and the infra
overlaps). Otherwise **remove + downgrade prose** rather than carry a
documented-but-fake mitigation. Either way, the CHANGELOG over-claims must be
fixed — that is the part that is non-optional. **A user steer (wire vs remove)
is desirable but can be folded into the Item-1 decision** (wire-if-Item-1,
else-remove).

---

## Cross-cutting: independent vs coupled

- **Item 2 is FULLY INDEPENDENT.** Render-side masking + zeroize-on-remove in
  `slot_editor.rs`, no on-disk impact, no shared infra with 1 or 3, no user
  decision. Smallest, cleanest, can ship alone.
- **Item 1 and Item 3 are COUPLED** through the secret-display/modal infra and
  the wire-vs-remove fork: whether to wire paste-warn (Item 3a) is naturally
  decided *with* whether to harden the secret-display surface (Item 1 masking +
  confirm-modal masking). Both touch `secrets.rs` + `main.rs`'s modal region.
  Doing them together avoids two separate visits to the same code and lets the
  paste-warn decision ride the Item-1 masking decision.
- **Item 1 also touches `render_copy_command`** (invocation.rs) — a function
  with byte-exact round-trip tests (POSIX/Windows quoting). A masked variant
  must NOT alter the existing un-masked quoting path the tree-mode pipeline
  copy depends on (`tree_form::posix_pipeline_command` reuses `posix_quote`).
- All three are **GUI-local** — no sibling-repo flag surface change, no
  `schema_mirror` impact (flag-NAME parity only; these are render/masking/
  zeroize changes, not flag additions). No toolkit companion needed.

## Recommended cycle split

**Two cycles.**

1. **Cycle 1 — Item 2 (`slot-secret-values-rendered-unmasked`).** Standalone,
   no user decision, smallest blast radius. `.password(true)` gated on
   `is_secret_bearing()` + zeroize-on-remove in `slot_editor.rs`. TDD:
   characterization test that a secret-subkey row renders masked (kittest) +
   a zeroize-on-remove unit assertion. **Can go straight to R0** (the design is
   determined). Ship first to bank a clean win.

2. **Cycle 2 — Items 1 + 3 together (the display/modal hardening).**
   **BLOCKED on a user decision** for Item 1 ((a)/(c)/(d) + copy-button
   masked-vs-real) and the implied Item-3 fork (wire-if-masking-lands,
   else-remove + prose downgrade). Brainstorm-spec → R0 only AFTER the user
   picks. Folds the CHANGELOG over-claim fix in regardless.

(One combined cycle is possible but Item 2's independence + zero-decision
status argues for shipping it first rather than gating it behind the Item-1
decision.)

## Decisions the user must make before R0

1. **Item 1 — secret-display posture:** (a) mask preview/modal only,
   (c) suppress preview when a secret is present, or **(d) mask preview+modal +
   explicit "copy with secrets" button** (recommended). Plus the sub-choice:
   do the **copy buttons** copy the **masked** (honest, not runnable) or
   **real** (runnable, clipboard leak) command? Recommend masked-default +
   explicit reveal. (Option (b) stdin-emit is NOT recommended — it reopens the
   v0.37.0 grey-out decision and hits the single-stdin-channel limit.)
2. **Item 3 — paste-warn:** **wire** (option a — feasible on egui 0.31; needs a
   new app field + modal + widget signal) vs **remove** (option b — delete the
   dead symbols). Recommended: wire IFF Item 1 lands masking, else remove. The
   CHANGELOG `:229/:255/:281` over-claim fix is **non-optional** either way.

Item 2 needs **no** decision.
