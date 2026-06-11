# R0 Review — GUI v0.40.0 (wire the paste-warn modal into SecretLineEdit) — ROUND 1

**Source SHA reviewed:** `62a4eb9` (= v0.39.0). All citations grep-verified against working tree.

**Verdict: 🟡 YELLOW — 0 Critical / 2 Important / 4 Minor.** Both Importants are SPEC-prose corrections — the egui Context-data bus design is sound and the egui-0.31.1 API usage is confirmed correct from source.

---

## Critical
None.

## Important

**I1 — The SPEC's call-site count is structurally wrong (3 direct `show` sites, not 7).**
The SPEC §Design says "`SecretLineEdit::show` is called from **7 sites**: `main.rs:669`, `main.rs:843`, and 5 archetype-form sites (`archetype_form.rs:169,187,193,199,213`)." Wrong. There are exactly **3 DIRECT `show` call sites**: `widget.rs:110` (scalar), `widget.rs:153` (repeating), `main.rs:843` (positional). `main.rs:669` and the 5 `archetype_form.rs` lines are `render_with_dispatch(...)` calls, NOT `show` calls — they reach `show` only transitively via `widget.rs:110/153`. The recon doc (`cycle-prep-recon-...:29`) correctly lists 3. The bus-design CORRECTNESS is unaffected (`show` is the single chokepoint), but the rationale's framing is false. **Fix:** state 3 direct `show` sites; archetype forms reach `show` transitively through `render_with_dispatch`→`widget.rs`, so the chokepoint covers them with no archetype-form change. Specify the count-pin test asserts exactly those 3 source locations.

**I2 — The `Event::Paste` visibility assumption is KNOWN now; promote from "verify at impl" to stated fact.**
The detection relies on `Event::Paste` remaining in `ui.input().events` after the TextEdit processes it. Confirmed from egui 0.31.1 source: `TextEdit` reads events via `ui.input(|i| i.filtered_events(&filter))` (builder.rs:918) — `filtered_events` (input_state/mod.rs:726-731) returns a CLONED `Vec<Event>`, it does not consume/remove from `i.events`. The only retain (`builder.rs:802`) purges `Event::Ime(_)` only — `Event::Paste` is untouched. So `show`'s subsequent `ui.input(|i| i.events.iter()...)` scan finds the paste intact. **Fix:** state this as confirmed (with the file citations) so no fallback path is built; keep a one-line "re-verify on egui bump" note. The Tests-section "if injecting a paste is infeasible, drive the fallback" clause should be reframed: the event WILL be visible; the only open question is kittest focus/injection tooling.

## Minor

**M1** — `main.rs:669` is mis-cited as a `show` site (it's `render_with_dispatch`). Folded by the I1 rewrite.

**M2** — `remove_temp<T: 'static + Default>` — the complete bound includes `Default` (not just `Clone+Send+Sync+'static`). `bool: Default` (=false) satisfies it; note it so a future non-Default signal type doesn't surprise.

**M3** — CHANGELOG citation wrong: the over-claim is at `CHANGELOG.md:1989-1991` (`cell_paste_warn_modal_trigger` "validates the paste-warn modal text and behavior on `SecretLineEdit` paste events" — false while dead). `:2196` is a Non-Goals note ("paste-warn modal copy mentions [the OS limit]") — honest, needs no correction. **Fix:** target `:1989-1991` (precise about what the cell tests) + the new `[0.40.0]` entry.

**M4** — Multi-row clarification: for a repeating secret widget (e.g. 2 `--share` rows) each row's `show` scans the same global `Event::Paste`, but only the recipient row's `response.changed()` is true, so no double-trigger. Add one sentence so a future auditor doesn't flag it.

---

## Confirmations
- ctx-data bus mechanics correct: `data_mut` + `insert_temp`/`remove_temp::<bool>(Id)` is the right egui-0.31 API; read-once timing (show during CentralPanel closure → post-panel `remove_temp` → set `pending_paste_warn` → modal) is single-frame-correct; `remove_temp` reads+clears so no leak into next frame.
- Detection attribution (`changed() && Paste≥threshold`) correctly excludes scenario (b) (paste into a non-secret field → no secret `show` changes → no flag) and multi-row double-trigger (M4). Accepted false-negative: pasting text IDENTICAL to current content → `changed()` false → no warn (benign).
- Two admitted uncovered surfaces (composite value fields + secret slot fields, both not via SecretLineEdit) correctly scoped as FOLLOWUP; the honest CHANGELOG wording is "fires on SecretLineEdit paste events" (scoped), which the M3 fix supports.
- SemVer MINOR correct; no schema_mirror / manual / toolkit-pin trigger (GUI-internal, no clap flag/secret-bit change).
