# SPEC — GUI v0.37.0: grey out the suppressed `*-stdin` secret toggles (the dead checkbox isn't a lie)

**Cycle:** mnemonic-gui v0.37.0 (MINOR) · **Source SHA:** `55e5cd2` (= v0.36.0) · **Resolves:** `boolean-stdin-secret-toggles-never-emit` (FOLLOWUPS.md) — **user decision: grey them out** (not emit; the GUI runner has no stdin channel).

## Problem (verified)

A `*-stdin` secret toggle — `flag_is_secret(flag) && FlagKind::Boolean` (all `repeating: false`) — renders as a live `ui.checkbox` (`widget.rs:570`, via the generic scalar `render()` path) but its checked state NEVER reaches argv: `assemble_argv`'s secret branch `continue`s for non-Text/non-Composite secrets (`invocation.rs:255-273`). 24 sites / 6 names (`--passphrase-stdin`, `--secret-stdin`, `--decrypt-password-stdin`, `--bip38-passphrase-stdin`, `--phrase-stdin`, `--ms1-stdin`). A user can check a control that does nothing — the dead checkbox is a lie.

## Design — one early branch in `render_with_dispatch`, predicate = the assembler's suppression set

In `src/form/widget.rs::render_with_dispatch`, AFTER the secret-Text block and BEFORE the repeating/scalar paths, add:

```rust
// vX: a secret Boolean is a `*-stdin` toggle the assembler SUPPRESSES
// (the GUI runner has no stdin channel to feed it — invocation.rs's
// secret-branch `continue`). Render it DISABLED so the dead control
// isn't a lie. This predicate is EXACTLY the assembler's suppressed
// set (flag_is_secret && Boolean → no emit), so render and emit cannot
// drift. No state.values writeback (returns early) — the flag stays
// absent from argv, matching the suppression.
if crate::secrets::flag_is_secret(flag) && matches!(flag.kind, FlagKind::Boolean) {
    ui.horizontal(|ui| {
        let mut unchecked = false;
        // Label the checkbox ITSELF (R0-r1 M2) so kittest get_by_label
        // targets the CheckBox node (the tree_form.rs:543 is_disabled()
        // precedent), not a separate Label.
        ui.add_enabled(false, egui::Checkbox::new(&mut unchecked, flag.name))
            .on_hover_text(
                "stdin toggles can't be driven from the GUI (no stdin channel) — \
                 type the value in the inline field, or run the CLI directly",
            );
        render_help_icon(ui, tab, subcommand, flag);
    });
    return;
}
```

- **Predicate-mirror is load-bearing (R0-r1 verified EXACT):** the full census of `flag_is_secret==true` across all 4 schemas partitions cleanly into Text (inline secrets → secret_widgets block), one NodeValueComposite (`--share`), and exactly 6 Booleans (the `*-stdin` toggles) — NO secret `Number/Path/Dropdown/Range/Timestamp`. So `secret && !Text && !Composite` ≡ the 6 Booleans ≡ the assembler's `else→continue` set; Boolean-only grey-out neither over- nor under-greys. T1's converse-closure makes a future secret-non-Boolean trip RED.
- **No value writeback:** returning early skips the scalar path's `state.values.push` — the flag never gets a `Boolean(false)` entry; clean, and the assembler would suppress it anyway.
- Non-secret Booleans (`--privacy-preserving`, `--reveal-secret`, `--emit-spec`, the `now` timestamp toggle, etc.) are untouched (they're `flag_is_secret == false`).
- Placement before the `repeating` check is safe (all these toggles are `repeating: false`); gating on `Boolean` makes it unambiguous regardless.

## Tests (TDD)

- **T1 (predicate==suppression invariant, pure logic):** for every flag across all 4 `schema::{mnemonic,md,ms,mk}::SCHEMA` subcommands, assert `(flag_is_secret(flag) && kind==Boolean)` ⟺ a checked instance produces NO argv entry from `assemble_argv` (FormState with the flag `Boolean(true)`, assert absent). **CONVERSE-CLOSURE (R0-r1 M1):** also assert NO flag is `flag_is_secret && !Text && !NodeValueComposite && !Boolean` — a future secret `Path`/`Number` would be assembler-suppressed but NOT greyed (a live-but-dead control), so it must trip RED here rather than ship silently.
- **T2 (live disabled, kittest):** render `bundle` (carries `--passphrase-stdin`) via egui_kittest; assert `harness.get_by_label("--passphrase-stdin").is_disabled()` — the checkbox is labeled with `flag.name` (§Design) so this targets the CheckBox node directly (the `tree_form.rs:543` precedent; kittest 0.1.0 exposes `is_disabled()` via Deref).
- Existing suite green (no schema change → schema_mirror/drift gates unaffected; the assembler behavior is byte-identical — only the widget's interactability changes).

## Ritual

- CHANGELOG `[0.37.0]`; version bump (Cargo.toml + Cargo.lock + README self-pin); FOLLOWUPS resolve `boolean-stdin-secret-toggles-never-emit` recording the user's grey-out decision (+ that the predicate mirrors the assembler so the two can't drift).
- No toolkit pin change; no schema_mirror impact (no flag-name/secret-bit change); no manual/companion.
- SemVer MINOR (user-visible: 24 checkboxes become disabled-with-tooltip).

## Non-goals

Emitting the toggles (the rejected alternative — needs a `runner.rs` stdin-feed story); any schema/secret-classification change; the inline-value secret widgets (already masked).
