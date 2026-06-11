# cycle-prep recon — 2026-06-11 — paste-warn wiring (Item 3, v0.40.0)

**Origin/master SHA at recon time:** `62a4eb9` (= v0.39.0)
**Local branch:** `master`
**Sync state:** up-to-date (v0.39.0 just pushed + tagged)
**Untracked:** the cycle-prep/SPEC/agent-report files for v0.38/v0.39.

Slugs: `paste-warn-modal-dead-code`, `paste-warn-live-wiring-untested`. The LAST item of the GUI secret-exposure cluster (Item 2 = v0.38.0, Item 1 = v0.39.0). Expect drift in the snapshot line numbers.

---

## Per-slug verification

### `paste-warn-modal-dead-code`
- **WHAT:** `PASTE_WARN_MODAL_TEXT` + `should_warn_on_paste` are never called in `src/`; `SecretLineEdit::show` does no paste detection; the affordance described in prose/SPEC does not exist.
- **Citations:**
  - `src/secrets.rs:164-196` — **DRIFTED:** `PASTE_WARN_MODAL_TEXT` is at `secrets.rs:167`, `should_warn_on_paste` at `secrets.rs:194`, `PASTE_WARN_THRESHOLD = 8` at `:189`. Both `pub`; **confirmed ZERO `src/` callers** (`grep should_warn_on_paste|PASTE_WARN_MODAL_TEXT src/` → only the defs). **ACCURATE claim.**
  - `SecretLineEdit::show` at `src/form/secret_widget.rs:68` — signature `pub fn show(&mut self, ui, label, help)` returns `()`; body adds a `TextEdit::singleline(...).password(true)` (`:74`), no paste inspection. **ACCURATE.**
  - No `pending_paste_warn` field on `MnemonicGuiApp` (grep → none). **ACCURATE.**

### `paste-warn-live-wiring-untested`
- **WHAT:** `tests/widget_secret.rs` asserts only the pure predicate + buffer/zeroize transitions; its own doc defers the live check.
- **Citations:** `tests/widget_secret.rs:18-24,42-71` — re-verify exact lines at SPEC time; the doc-comment defers the live paste check. **(predicate-only, ACCURATE in spirit.)**

---

## Design facts for the SPEC

- **3 `SecretLineEdit::show` call sites** (re-grepped @ 62a4eb9): `src/form/widget.rs:110` (secret-Text scalar `rows[0].show(ui, flag.name, flag.help)`), `src/form/widget.rs:153` (secret-Text repeating `w.show(ui,"","")`), `src/main.rs:843` (secret positional `row.show(ui,&label,pos.help)`). Round-1 I2 cited `main.rs:830` → **DRIFTED-by-13 → :843.** Pin the count (3) in a test so a 4th site can't silently skip the signal.
- **Signal threading:** `show` returns `()` today. Change to return a small response (e.g. `bool paste_over_threshold` or `SecretLineEditResponse { changed, paste_over_threshold }`). Detect via `ui.input(|i| i.events.iter().find_map(|e| if let egui::Event::Paste(s)=e { Some(s.chars().count()) } else { None }))` — egui 0.31 leaves `Event::Paste` in the global event vec (verify it's not consumed by the TextEdit). The flag-is-secret half of `should_warn_on_paste` is guaranteed (SecretLineEdit only renders for secret fields), so `show` checks `paste_len >= PASTE_WARN_THRESHOLD`; where a `&FlagSchema` is in scope (widget.rs sites) the call site MAY use `should_warn_on_paste(flag, len)` to keep the predicate live.
- **App modal:** add `pending_paste_warn: bool` to `MnemonicGuiApp` (mirror `pending_confirm_argv` pattern, main.rs:108/314). Set when any secret widget reports an over-threshold paste this frame; render an informational modal using `PASTE_WARN_MODAL_TEXT` with a Dismiss/OK that clears it. Non-blocking (the value is already in the field).
- **CHANGELOG honesty (carried from v0.39.0 round-1 I1):** correct the over-claim lines `CHANGELOG.md:1978-1980` (`cell_paste_warn_modal_trigger` "validates the paste-warn modal text and behavior on SecretLineEdit paste events") + `:2196` (non-goals paste-warn reference). After wiring they become TRUE — verify wording matches the wired behavior. Re-grep (line numbers decay).
- **Composite parity gap:** FOLLOWUP `composite-paste-warn-parity` (filed v0.39.0) — paste into a `NodeValueComposite` value field won't trigger the warn (composites don't use `SecretLineEdit`). Either extend or document at this cycle.

---

## Recommended scope
Single cycle, v0.40.0 (MINOR — user-visible affordance: a paste-warn modal now fires). No clap flag / secret-bit / schema_mirror / manual / toolkit-pin change (GUI-internal). Tests: T-B1 (predicate, exists), T-B2 (live wiring via kittest `Event::Paste` injection — the missing live check), T-B3 (app modal pending_paste_warn set/clear), + the 3-call-site count pin. R0 gate before impl.
