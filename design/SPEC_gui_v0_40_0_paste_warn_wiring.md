# SPEC — GUI v0.40.0: wire the paste-warn modal into SecretLineEdit (Item 3)

**Cycle:** mnemonic-gui v0.40.0 (MINOR) · **Source SHA:** `62a4eb9` (= v0.39.0) · **Recon:** `cycle-prep-recon-paste-warn-wiring-item3.md`.
**User decision:** Item 3 = **wire the paste-warn modal** (make the documented mitigation real).
**Resolves:** `paste-warn-modal-dead-code`, `paste-warn-live-wiring-untested`. The LAST item of the GUI secret-exposure cluster (Item 2 = v0.38.0 slot mask; Item 1 = v0.39.0 on-screen masking).

## Problem (verified @ `62a4eb9`)

`should_warn_on_paste(&FlagSchema, usize)` (`src/secrets.rs:194`), `PASTE_WARN_MODAL_TEXT` (`:167`), and `PASTE_WARN_THRESHOLD = 8` (`:189`) are DEAD — grep confirms ZERO `src/` callers. `SecretLineEdit::show` (`src/form/secret_widget.rs:68`) does no paste detection; no `pending_paste_warn` app field exists. The CHANGELOG over-claims the modal "fires" (`CHANGELOG.md:1978-1980`, `:2196`) when the affordance does not exist (the v0.31.1 entry already self-corrects half).

## Design — an egui Context-data signal bus (no `show` signature change)

`SecretLineEdit::show` has exactly **3 DIRECT call sites** (R0-r1 I1): `src/form/widget.rs:110` (secret-Text scalar), `src/form/widget.rs:153` (secret-Text repeating), `src/main.rs:843` (secret positional). Other render paths — `src/main.rs:669` (the main form loop) and the 5 archetype sites (`src/form/archetype_form.rs:169,187,193,199,213`) — are `render_with_dispatch(...)` calls, NOT `show` calls; they reach `show` only TRANSITIVELY through `widget.rs:110/153` when the flag is a secret Text. Threading a return value up through `render_with_dispatch` (`widget.rs:81`, returns `()`) AND `archetype_form::render` (`:89`, no app access) would touch all of them and still risk a forgotten path. Instead, a per-frame **egui Context-data flag** is set inside `show` and read once in `update()`. Because `show` is the single chokepoint for ALL secret-Text input (the 3 direct sites + every transitive archetype path through them), the bus covers everything with no signature change.

1. **Detect in `SecretLineEdit::show` (`secret_widget.rs:68-84`):** after the existing `let response = ui.add(TextEdit…password(true))` (`:74`), read the frame's paste events:
   ```rust
   let pasted_len = ui.input(|i| i.events.iter().find_map(|e| match e {
       egui::Event::Paste(s) => Some(s.chars().count()),
       _ => None,
   }));
   if response.changed() {
       // existing buffer-update block …
       if let Some(len) = pasted_len {
           if len >= crate::secrets::PASTE_WARN_THRESHOLD {
               ui.ctx().data_mut(|d| d.insert_temp(paste_warn_id(), true));
           }
       }
   }
   ```
   **`Event::Paste` IS visible to this scan (R0-r1 I2 — confirmed from egui 0.31.1 source, not an open assumption):** `TextEdit` reads events via `ui.input(|i| i.filtered_events(&filter))` (egui `builder.rs:918`); `filtered_events` (`input_state/mod.rs:726-731`) returns a CLONED `Vec<Event>` — it does NOT remove events from `i.events`. The only `events.retain` (`builder.rs:802`) purges `Event::Ime(_)` exclusively; `Event::Paste` is untouched. So after the TextEdit call the paste event remains in `ui.input().events` for `show` to find. (Re-verify on any egui major bump.)
   **Attribution is via `response.changed()`** — a paste only changes the FOCUSED field's buffer, so only the field that actually received the paste sets the flag (a global paste event with no focused-secret-field recipient changes nothing → no false-positive). For a repeating secret widget (e.g. 2 `--share` rows) each row's `show` scans the same global `Event::Paste`, but only the recipient row's `response.changed()` is true → no double-trigger (R0-r1 M4). Accepted benign false-negative: pasting text IDENTICAL to the field's current content → `changed()` false → no warn. The flag-is-secret half of `should_warn_on_paste` is guaranteed structurally — `SecretLineEdit::show` renders ONLY for secret fields — so `show` checks the length half directly. `paste_warn_id()` = a module const `egui::Id::new("gui_secret_paste_warn_pending")`.
2. **App state + read-once (`main.rs`):** add `pending_paste_warn: bool` to `MnemonicGuiApp` (`:93`-struct; init `false` at `:314`-ish). In `update()`, AFTER the CentralPanel form render returns (the `show` calls have run) and BEFORE the modal section, read+clear the bus flag once:
   ```rust
   if ctx.data_mut(|d| d.remove_temp::<bool>(paste_warn_id()).unwrap_or(false)) {
       self.pending_paste_warn = true;
   }
   ```
   (`remove_temp::<T: 'static + Default>` both reads and clears — `bool: Default` (=`false`) satisfies the bound (R0-r1 M2) — so a stale flag can't persist into the next frame.)
3. **Modal (`main.rs`, mirror `pending_confirm_argv` at `:1037`):** when `self.pending_paste_warn`, render an INFORMATIONAL modal (`egui::Window` "Secret paste warning", non-blocking — the value is already in the field) showing `secrets::PASTE_WARN_MODAL_TEXT` with a single **Dismiss/OK** button that sets `self.pending_paste_warn = false`. One-shot per trigger (re-fires on the next over-threshold paste).
4. **CHANGELOG honesty (R0-r1 M3 — correct citation):** the over-claim is at `CHANGELOG.md:1989-1991` — `cell_paste_warn_modal_trigger` "validates the paste-warn modal text and behavior on `SecretLineEdit` paste events" (false while the modal is dead; the cell tests only the predicate). Re-word it to precisely describe what is tested (the OLD cell = predicate; the NEW v0.40.0 `tests/…` = live wiring) so it is true post-wiring. `CHANGELOG.md:2196` is a Non-Goals note ("paste-warn modal copy mentions [the OS-snapshot limit]") — honest, NO correction needed. Re-grep both at write time (line numbers decay).

## Tests (TDD)

- **T-B1 (predicate, EXISTS):** keep the `should_warn_on_paste` unit cells (`tests/widget_secret.rs`). No change.
- **T-B2 (live wiring, kittest — the missing `paste-warn-live-wiring-untested` check):** render a `SecretLineEdit` in a `Harness`; focus its field, inject an over-threshold paste via `harness.input_mut().events.push(egui::Event::Paste("x".repeat(THRESHOLD).into()))`, `run()` a frame, and assert the bus flag is set (`harness.ctx().data_mut(|d| d.get_temp::<bool>(paste_warn_id())) == Some(true)`). Negative: a paste of length `< THRESHOLD` → flag NOT set. **Verify RED:** without the `show` detection block the flag is never set (mirror v0.38/v0.39 scratch-revert discipline). The `Event::Paste` visibility is CONFIRMED (Design §1) — so any infeasibility is a kittest FOCUS/INJECTION-tooling issue (the widget must hold focus for the TextEdit to register the paste + fire `changed()`), NOT an event-visibility one; if the tooling blocks it, drive the post-paste condition directly (focused widget + the event present) and RECORD which path was used.
- **T-B3 (app modal, pure-logic where possible):** setting the bus flag → `update()` read-once sets `pending_paste_warn` → modal renders `PASTE_WARN_MODAL_TEXT` → Dismiss clears it. If the full `update()` isn't harness-isolable, assert the read-once+clear logic and the modal text at the seam, and record.
- **T-B4 (chokepoint pin, R0-r1 I1):** because the signal is a ctx-data bus set INSIDE `show` (not a per-call-site return), `show` is the single chokepoint. Pin the 3 direct `SecretLineEdit::show` call sites — assert a grep of `src/` finds exactly those 3 source locations (`widget.rs:110`, `widget.rs:153`, `main.rs:843`) calling `.show(` on a `SecretLineEdit`, so a future 4th direct site is a deliberate decision (it's still covered by the bus, but the count tripwire documents the surface). Add a one-line comment at `show` that the bus is the single chokepoint so a refactor doesn't reintroduce per-site wiring.
- Full suite green; no schema change.

## Ritual

CHANGELOG `[0.40.0]` (+ the honesty correction to the OLD entries); version bump (Cargo.toml + Cargo.lock + README self-pin `:42`); FOLLOWUPS resolve `paste-warn-modal-dead-code` + `paste-warn-live-wiring-untested` (record the wire decision + the ctx-bus design). No toolkit pin / schema_mirror / manual impact (no flag-name/secret-bit change; the modal is GUI-internal). SemVer **MINOR** (user-visible: a paste-warn modal now fires).

## Non-goals / deferred

- **Composite paste-warn parity** (`composite-paste-warn-parity`, filed v0.39.0): `NodeValueComposite` value fields don't use `SecretLineEdit`, so this wiring does NOT cover a paste into `--from phrase=<seed>`'s value box. The v0.39.0 masking already covers their DISPLAY; this cycle covers the SecretLineEdit paste affordance only. Extend OR keep documented — R0's call.
- **Secret SLOT-field paste-warn** (NEW, surfaced this cycle): v0.38.0 renders secret slot values via a raw `egui::TextEdit::singleline(&mut row.value).password(true)` (`slot_editor.rs`), NOT `SecretLineEdit` — so a paste into a `@N.phrase=` slot box also won't fire the warn. The ctx-bus design makes future coverage cheap (the slot render could set the same `paste_warn_id()` flag with the same `changed() + Event::Paste` check), but pulling it in widens the cycle. File a FOLLOWUP `slot-field-paste-warn-uncovered` (paired with `composite-paste-warn-parity` — both are "secret input not via SecretLineEdit") OR include it — R0's call on scope.
- OS clipboard-history / paste-manager mitigation (the modal only WARNS; `PASTE_WARN_MODAL_TEXT` already states the limit); the allocator-residue limit.
