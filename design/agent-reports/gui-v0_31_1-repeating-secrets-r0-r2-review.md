# R0 review — SPEC_gui_v0_31_1_repeating_secrets — round 2

**Verdict: YELLOW** (0C / 1I new — all ten round-1 folds correctly applied; one §3 disclosure gap round 1 did not catch)

## Round-1 fold verification (all RESOLVED)
- **C1 — RESOLVED.** Folded §2 traced against the real assemble_argv: visibility gates precede unchanged; the NodeValueComposite fall-through lands in the generic repeating values loop + emit_one's composite arm — exactly what the seed-xor cell asserts. `continue` placement right (Text arm continues unconditionally — load-bearing for the dead-path cell). Widget gate at `widget.rs:76` is literally `flag_is_secret && matches!(kind, Text)` — identical to §2's assumption.
- **C1 Boolean sub-trace — CONFIRMED.** Booleans render generically as checkboxes (fail the Text gate); today's assembler eats them; the folded `else { continue }` preserves byte-identical no-emit; no Boolean secret is repeating (census: all 17 sites repeating:false).
- **C2 — RESOLVED.** has_value pinned with per-row any; interplay with the required-seed (blank --share row → has_value false) correct for should_confirm_run + the required gate; the faithful negative migration target verified.
- **I1-I4, M1-M4 — RESOLVED** (all citations re-verified exact; the I4 lists checked line-by-line).

## Critical
None.

## Important
**I-NEW1 — §3's name-based union sweeps in MORE than disclosed, incl. a cross-CLI collision onto three NON-secret flags whose persisted values silently stop persisting — and one §3 sentence is false.** Full census also includes Text secrets `--decrypt-password` ×2, `--secret` ×2, `--digits` ×1, ms.rs `--hex` ×3 / `--phrase` ×4 — inert (already widget-routed) EXCEPT `--phrase`, which collides with three `secret: false` Text flags on mnemonic.rs xpub-search subcommands ("Master BIP-39 phrase (inline)", `mnemonic.rs:2280-2286/:2442-2448/:2712-2718`). Those are values-routed and **persist master phrases to state.json in plaintext TODAY** (no redaction class catches them). Under §3 they silently stop persisting — strictly safety-POSITIVE (closes a live leak), but: (a) disclose it (the same class I3 demanded); (b) §3's "Nothing writes secrets into values after this SPEC" is FALSE as worded (xpub-search --phrase keeps writing seed material into values; the union only catches it at persist via the lucky collision); (c) FOLLOWUP for the underlying census bug (xpub-search --phrase plausibly should be secret:true — flips its widget to SecretLineEdit; out of this PATCH).

## Minor
**m1 —** §4 cites the assembler rewrite at `:243`; the branch starts at `:238`. Cosmetic.

## Empirical probes run
Full reads (invocation.rs, widget.rs, mod.rs:280-360, secrets.rs, persistence.rs, SPEC, r1 report); secret:true census across all 4 schemas (awk extraction, hits manually confirmed); has_value call-site sweep; test-fn line verification; --share shapes; git local == dabbdfe.
