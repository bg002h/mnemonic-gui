## R0 Review — Wave-2 secret-hygiene GUI lane (G1/G2/G3/G4)

**Verdict: GREEN — 0 Critical / 0 Important / 5 Minor.** Gate PASSES; implementation may proceed.

**Reviewer:** opus architect (adversarial R0). **Repo:** mnemonic-gui @ `7ce777d4` (= current HEAD, = `mnemonic-gui-v0.48.0` tip; the literal tag object is `0f9aa46f` but the pinned commit SHA is HEAD — pin is valid). **Cross-repo:** mnemonic-toolkit @ `34d3a724` (= current HEAD; commit exists, verified). Every load-bearing path/line/claim below was re-grepped against these pinned SHAs.

---

### Critical
None.

### Important
None.

### Minor
1. **G2/G4 fat-finger asymmetry (coherence).** After G2, an xprv mis-pasted into `hex` (hash-kind node) / `w` (wrap node) is blanked on-disk but STILL flows verbatim through `to_spec_json` (tree_model.rs:457 emits `node.hex`; :476 emits `node.w`) into Copy-spec-JSON / posix-pipeline / `--spec -` stdin — the G4 surface, DEFERred. Same threat, asymmetric coverage (durable leg closed, transient clipboard leg open). G4's DEFER is well-reasoned; add one sentence to §1/§4 noting the residual.
2. **Decayed line citations (cosmetic).** §3.1 cites secrets.rs:194 for PASTE_WARN_MODAL_TEXT (actual :182). §3.2 cites secret_widget.rs:11-14 (caveat spans 11-15, anchors `gui-secret-buffer-allocator-residue` not the tree-key slug). §1.1 field lines ~94/~99 vs actual 97/100. All snapshot decay; re-grep at impl time. Core fn citations (714/675/695/176/258) are exact.
3. **G1-B ship-mechanics under-specified.** Verified toolkit `changelog-check.yml` fires ONLY on `mnemonic-toolkit-v*` and EXPLICITLY exempts `manual-gui-v*` (workflow L13-15) → the prompt's 'changelog-check fires on the tag' concern is correctly NOT a blocker. But the spec should say explicitly: G1-B needs NO toolkit CHANGELOG entry and NO `mnemonic-toolkit-v*` tag. Also resolve §2.3 item-3's ambiguous 'should re-check' of the `[manual-gui]` implied pins (scope IN or OUT).
4. **G1-B cross-repo ordering hazard.** The manual-gui CI lint clones the pinned GUI ref and runs `gui-schema` against it → GUI `v0.48.1` tag must EXIST before the toolkit manual PR's CI can pass. State the hard ordering (GUI tag first, then toolkit manual PR) in §2.3.
5. **T6 test reuse subtlety.** New cells go in persist_redaction_v0_34_0.rs by T5 (L187); the xprv vector (L201) + 64-hex digest (L208, `0000…0001`) are reusable, but that digest is a BLANKED *key-field* case in T5 (correct — key uses the allowlist), so T6 must place it in `hex`/`w` to show SURVIVAL (opposite assertion direction). §1.5 case-3 already says this; flagged so the implementer does not copy T5's direction.

---

### Verified-correct (load-bearing claims confirmed against source)

**G2 core mechanism — SOUND.**
- `blank_non_extended_public_keys` (tree_model.rs:714) visits `key` + `keys[i]` + `children` ONLY — `hex`/`w` never visited. Confirmed the residue exists.
- `redacted_for_persistence` (tree_model.rs:176) is reached by the on-disk save path (persistence.rs:145 → :250 `serde_json::to_string_pretty`); the doc-comment L171-175 asserts "Hashlock `hex` is deliberately NOT redacted" — must be amended (spec §1.2 does so). NO other on-disk serialization of TreeNode exists (grep confirmed; tree_form.rs:109 is the clipboard/G4 path, not state.json).
- `is_xprv_like` (tree_model.rs:675) = `rsplit(']')` + `is_char_boundary(4)` + bytes 1..4 == "prv". **Empirically proven** (compiled & ran): xprv/tprv → blank; 64-hex digests → survive; ALL miniscript wrappers (a/s/c/t/d/v/j/n/l/u + combos incl. "sv") → survive; NO misfire. A pure-hex digest can NEVER have bytes 1..4 == "prv" (p/r/v ∉ hex alphabet) — the survival is STRUCTURALLY guaranteed, not just sampled.
- Shape (B) rejection is correct: `is_extended_public_like` (tree_model.rs:695) returns false for hex/w content → would blank every legit digest/wrapper (data-loss). The T6 case-3 survive-the-digest cell genuinely distinguishes (A) from (B).
- SemVer PATCH correct: change is inside a PRIVATE fn; `redacted_for_persistence` signature unchanged; no clap/dropdown/schema_mirror surface; **no `impl Drop` on TreeNode/TreeState** (grep confirmed empty) → no pub-struct-Drop trap; no signature fan-out.
- §1.6 stale-citation finding is REAL and correctly reconciled: FOLLOWUP `tree-xprv-heuristic-only-covers-key-fields` (FOLLOWUPS.md:123) cites `blank_xprv_keys` — a fn that NO LONGER EXISTS (renamed to `blank_non_extended_public_keys` in the v0.34.0 allowlist inversion). Implementer copying the slug verbatim would patch a dead name. Spec's reconciled framing is accurate. The slug's own text ("extend the is_xprv_like sweep", "keep-hex-digests posture") confirms shape (A) is slug-intended.
- §1.3 scope decision (zeroize_keys hex/w OUT) is reviewable & defensible; one-line escalation path documented. I do NOT escalate it — the on-disk leg is the higher-value durable exposure; the in-RAM twin's residual is bounded and the allocator-residue class is already ratified-accepted.

**G1-A — RECONCILE correct.** Modal redaction shipped at v0.39.0 (CHANGELOG L81): SECRET_MASK="••••" (invocation.rs:137), assemble_argv_with_secret_mask (:152), render_copy_command_masked (:524); modal substitution at main.rs:1099-1100; PendingConfirm.mask + impl Zeroize (runner.rs:84) + impl Drop (:94). The FOLLOWUP's premise is now FALSE (cites main.rs:512-535 verbatim render + `grep redact` only persistence.rs — both decayed). Status still `open` (FOLLOWUPS.md:720) → genuine followup-status-discipline flip in the shipping commit. Correct.

**G1-B — LOCKSTEP correct.** Manual is factually stale: 14-secret-handling.md `:::danger` L79-114 claims v0.3.0 plaintext; 11-what-is-mnemonic-gui.md feature-2 L45-46 same. `pinned-upstream.toml [mnemonic-gui].tag = "mnemonic-gui-v0.3.0"`. Toolkit FOLLOWUP `…-manual-companion` (design/FOLLOWUPS.md:1005) Status open, lists exactly (i)/(ii)/(iii)/(iv) = the spec's 3 edits + pin bump + flip. All-land-together requirement correct.

**G1 co-lander — correctly DEFERred.** Genuinely open: runner.rs:199 injects ONLY `MNEMONIC_FORCE_TTY` — no `@env:` bag, no argv→sentinel rewrite (grep confirmed). Distinct defense layer (off-`/proc/cmdline`) from on-screen redaction. Fenced spec is execution-ready and REUSES the right precedent: SecretString (not raw Zeroizing — the v0.67.0 non-redacting-Debug-leak lesson is correctly invoked); scrub-env-bag-on-exit mirrors PendingConfirm's Drop; the existing pub PendingConfirm Drop means NO new pub-struct-Drop trap; kittest cell `cell_import_wallet_repeating_ms1_argv` (tests/kittest_import_wallet_form.rs:157) pins literal-pass-through and MUST flip IF the co-lander lands (it stays as-is for the 0.48.1 PATCH — correct, since co-lander is deferred).

**G3 — DOCUMENTED-CAVEAT correct.** egui owns undo Strings; only lever `clear_undoer()` drops (≠ scrubs) the VecDeque; egui has zero Zeroize → 'clean' path leaves freed-but-unscrubbed heap = the already-accepted allocator-residue class. Sibling rulings consistent. `TreeNode::zeroize_keys` (M9, v0.46.0) scrubs the model; the on-screen/undo gap is in-source at secret_widget.rs (caveat L11-15) + FOLLOWUPS.md:764. NO-BUMP doc-only correct; shipping a clear_undoer pass + RESOLVED flip would over-claim — spec correctly refuses.

**G4 — DEFER correct.** Disjointness verified: NODE_KIND_SPECS (nodes.rs:63, len==17 pinned at :121) ∩ SECRET_NODE_TYPES_ARGV = ∅; `TreeNode.kind` is always a NODE_KIND_SPECS kind. Compile-time trip-wire `const _: () = assert!(secret_slice_eq(...))` (secrets.rs:80, :91) fails the BUILD on a toolkit-pin secret-node-type addition → auto-re-arms the slug. `to_spec_json` (tree_model.rs:434) emits only key/keys (watch-only xpubs by contract), k/n (uints), hex (public digests), w (wrap) — no structural secret carrier. No live leak; deferred with auto-re-arm. Correct.

**Version/ship sites — gate-backed & complete.** `readme_pin_coherence.rs::readme_install_tags_match_pins` (L75) gates the GUI self-tag against `mnemonic-gui-v{Cargo.toml version}` → bumping Cargo.toml to 0.48.1 FORCES the README L42 bump (RED otherwise). `pin_coherence.rs` gates the toolkit pin (Cargo dep ↔ pinned-upstream `[mnemonic].tag`) — NOT the self-pin (the spec's 'verify' hedge is resolved: the self-pin gate is readme_pin_coherence). schema_mirror.rs has exactly 21 `#[test]` (matches §5). All cited hygiene-gate test files exist (secret_taxonomy_pin, schema_mirror_secret_drift, secret_mask_preview_v0_39_0, run_holder_zeroize, widget_secret_mask_cycle15g, secrets, spec_nodes_mirror, tree_round_trip, persist_redaction_v0_34_0). NO fmt CI gate (.github/workflows = build.yml + schema-mirror.yml only) — confirms 'do NOT cargo fmt the GUI'.

**SemVer rules — applied correctly.** G2 PATCH (private-fn redaction widening, no signature/wire/pub-field change, no Drop). No SecretString/ScrubbedXpriv/Zeroizing migration in G2 (those types are toolkit-only, absent from this repo — grep confirmed) → the constellation 'secret-type-migration = MINOR' rule correctly does NOT trigger for G2. The co-lander, IF opted in, is correctly scoped MINOR (0.49.0) with its own R0 sub-cycle (new env-channel feature surface).

---

### Disposition
Spec is implementation-ready. The 5 Minor items are advisory polish (1 coherence sentence, citation re-grep, 2 ship-mechanics clarifications, 1 test-direction note) — none blocks the gate. Recommend folding Minors 3 and 4 (G1-B changelog/tag clarification + cross-repo ordering) as cheap pre-impl edits since they touch the cross-repo PR sequencing; Minors 1/2/5 can be addressed inline during implementation. Per project convention, re-dispatch the architect after the fold (folds can introduce drift), then proceed to the single-subagent TDD implementation. Per-phase R0 + post-impl whole-diff review MUST run the FULL `cargo test` suite (secret-classification touches ripple into taxonomy/schema-drift tests outside G2's targeted file).