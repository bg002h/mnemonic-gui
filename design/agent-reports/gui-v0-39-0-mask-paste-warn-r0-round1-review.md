# R0 review — GUI v0.39.0 (mask preview/modal/copy + wire paste-warn) — ROUND 1

**Status of this record:** RECONSTRUCTED FROM SESSION NOTES. The round-1
architect review was conducted on the COMBINED Part A + Part B spec
(`SPEC_gui_v0_39_0_mask_preview_paste_warn.md` @ source SHA `71c7ecd`), but
its verbatim agent output was lost to context compaction before it was
persisted to this directory — a violation of the CLAUDE.md persist-verbatim
convention (the exact failure mode that convention warns about). This file
records the findings faithfully from my working notes so the audit trail is
not a blank. The NEXT round (round 2), dispatched against the re-scoped
Part-A-only spec, is persisted verbatim. Because the spec is being materially
re-scoped between round 1 and round 2, round 2 is effectively a fresh review.

**Verdict:** 🔴 RED — 1 Critical / 2 Important.

---

## Critical

**C1 — Part A mask is INCOMPLETE: it misses every secret-bearing
`NodeValueComposite` value.** The spec set the mask "exactly at the two
secret-value `argv.push(value…)` sites" (secret Text `invocation.rs:268`,
secret positional `:303`). But secret values ALSO reach argv as cleartext
tokens through `emit_one` (`:380-390`), which pushes
`format!("{}={}", node, value)` for `NodeValueComposite` flags — both the
`secret:true` flag `--share` (falls through the secret branch at `:274-276`)
and the `secret:false` but value-dependent `--from phrase=<seed>` (not a
secret flag at all; reaches `emit_one` at `:285-287`, secret because the NODE
is secret-classed). The mask must be set wherever a secret VALUE token is
pushed, which includes the composite emit. Concrete option (a): set the mask
inside `emit_one` — thread a `&mut Vec<bool>` parallel to `&mut argv`, mark
the composite value-token `true` when the flag is secret-bearing **or** the
value is a secret-classed composite (`node_type_is_secret(node)`).
**T-A1 MUST add a secret-composite case** (both `--share` and `--from
phrase=…`).

---

## Important

**I1 — CHANGELOG honesty (Part B).** Part B's existing CHANGELOG entries
over-claim that the paste-warn modal "fires" / "validates … on paste events"
when `should_warn_on_paste` + `PASTE_WARN_MODAL_TEXT` are DEAD code (no `src/`
caller). Re-cite and correct the over-claiming lines (round-1 snapshot cited
≈`:238`/`:264`/`:290` and the longer-form claims ≈`:1979-1980`/`:2196`, plus
the v0.31.1 partial self-correction ≈`:79`) — these line numbers are
snapshots and MUST be re-grepped at fold time. After Part B wiring they
become TRUE; if Part B is deferred, the false claims must be corrected to
"documented-but-not-yet-wired" NOW rather than left as a live falsehood.

**I2 — enumerate ALL `SecretLineEdit::show` call sites (Part B).** The
`show` return-shape change (to signal an over-threshold paste upward) touches
every secret-widget call site. Round-1 enumerated three:
`widget.rs:110` (secret-Text scalar), `widget.rs:153` (secret-Text
repeating), `main.rs:830` (secret positional). Re-grep at fold time and pin
the count in a test so a future fourth site can't silently skip the signal.

---

## Minor

- **M1** — file a FOLLOWUP for composite paste-warn parity: paste-warn (Part
  B) wires into `SecretLineEdit`, but `NodeValueComposite` values are typed in
  a different widget; a paste of a seed into `--from`'s value field would not
  trigger the warn. Track the gap.
- **M2** — `SECRET_MASK` constant placement/visibility: define once
  (`invocation.rs` or `secrets.rs`) and reuse; don't inline the `••••`
  literal at each display site.
- **M3** — the masked render must NOT shell-quote the mask placeholder (it's
  a display sentinel, never run) — assert it appears literally.

---

## Recommendation (advisory, not a gate)

**Split into two cycles: v0.39.0 = Part A (masking) only; v0.40.0 = Part B
(paste-warn wiring).** C1 makes Part A non-trivial (the mask must now thread
through `emit_one` and cover every secret argv source), and Part A is the
higher-value, always-on-screen leak. Part B (paste-warn) is independent
(different files: `secret_widget.rs`/`main.rs` modal state) and carries the
`show`-return-shape change that touches every secret-widget call site.
