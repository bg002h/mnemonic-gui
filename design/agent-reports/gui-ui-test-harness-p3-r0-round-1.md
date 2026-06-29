# P3 (I3 classified-secret never-persist/never-leak regression net) — R0 Round 1

**Reviewer:** opus architect (mandatory per-phase R0 gate; secret-hygiene = first-class bar)
**Scope:** `tests/ui_harness_i3_secret_nopersist.rs` (678 lines, TESTS-ONLY) on
`feat/ui-harness-p0-spike` @ `c984f7d` (master untouched @ `da47994`).
**Authoritative:** SPEC §5 I3 (corrected); Plan P3; `src/secrets.rs`, `src/persistence.rs`,
`src/form/invocation.rs`, `src/form/tree_form.rs`, `src/form/tree_model.rs`, `src/form/widget.rs`,
`src/runner.rs`. Verified against source + live runs (incl. a throwaway reach-probe, since removed).

---

## VERDICT: GREEN — 0 Critical / 0 Important

This is not a rubber-stamp. The net is genuinely non-vacuous, the negative cells bite, the redact
walk is the real one, the discrimination is proven (xpub survives, private-shaped blanks), and the
harness itself is hygiene-clean (coordinate-only failures, FAKE sentinels, Zeroizing-respecting
reads). Three NITs below, none blocking. Proceed to P4.

---

## Critical
None.

## Important
None.

---

## Hygiene ruling — PASS (the harness never risks leaking a secret)

Grepped every `assert*`/`format!`/`panic!`/`println!`/`eprintln!`/`dbg!`/`{:?}` in the file:

- **Every failure message is coordinate-only.** `leak_msg`/`coord` (`:99-111`) emit `tab/sub/flag`
  + a static "payload withheld" tail; nothing else. The remaining asserts print only integer counts
  (`:274,295,372,375,437`) or static strings (tree/negative cells). No assert formats `FormState`,
  the AccessKit tree, the serialized blob, the argv, or the fixture value. `h.state()` is read but
  never printed.
- **FAKE sentinels only.** All injected values are `FAKE_SECRET_FIXTURE_*` (`:120-122,446-451`).
  `TREE_KEY_FIXTURE`/`TREE_KEYS_FIXTURE` are non-xpub-shaped FAKE keys; `SURVIVING_XPUB` is a public
  watch-only xpub (correct — public, survives by design). No real key anywhere.
- **`Zeroizing` respected.** `fixture_landed` reads the secret-Text store via
  `w.as_string().as_str()` (`:182`) — `SecretLineEdit::as_string` returns `Zeroizing<String>`
  (`src/form/secret_widget.rs:123`); no plaintext long-lived `String` copy is retained. The only
  transient `String::from_utf8_lossy` copies (`:556,591`) are of FAKE fixtures, for a `.contains`
  comparison — acceptable (FAKE, not a real secret).
- **The scrub seam is real.** `i3_tree_secret_key_off_preview_and_spec_stdin_scrubbed` builds a
  `PendingConfirm` and calls `.zeroize()` (`src/runner.rs:84-92`) then asserts no fixture residue —
  matches the live holder scrub.

---

## Per-item verification

### 1. Non-vacuity (the crux)
- **(a) `fixture_landed` (`:180-194`)** confirms the inject reached the production store — reads
  `secret_widgets` (the Zeroizing `as_string()` path) for Text and `state.values`
  `NodeValueComposite{value}`/`Text` for the composite. The DRIVE-NOOP assert (`:322-327`) RED-flags
  any injection that no-op'd into the void. The sweep passed with `covered == 40`, so all 40 actually
  landed — not vacuously green.
- **(b) Surfaces genuinely exercised.** I instrumented a throwaway probe replicating the drive: **all
  40 value-bearing secrets reach argv-with-mask (NOT_REACHED=0)** — so the `reached_argv_masked >= 35`
  floor (`:375-379`) is *conservative*; in reality the `all_masked` classification check (`:347-353`)
  is non-vacuously exercised for all 40. The **persist** surface is type-level-always-green for the 39
  Text secrets (`secret_widgets` is `#[serde(skip)]` and `redact_for_persistence` reconstructs it
  empty), but **genuinely discriminating** for the NodeValueComposite secret (seed-xor-combine
  `--share`, node `phrase` ∈ `SECRET_NODE_TYPES_ARGV` → dropped at `persistence.rs:101-105`). The
  **spec-stdin** assert for flag secrets (`:363-367`) is an honest *structural disjointness* statement
  (a flag routes to `secret_widgets`/`values`, never `state.tree`; `spec_stdin_bytes` reads only
  `state.tree`) — the report's "weak for flags" framing is honest, and the **TREE path is the real
  spec-stdin test** (`i3_tree_secret_key_off_preview_and_spec_stdin_scrubbed` asserts the stdin
  *carries* the key, off-preview, and scrubbed).
- **(c) Negatives bite.** `i3_negative_persist_check_bites` (`:604-654`): control (a
  `secret_widgets`-held fixture does NOT persist) + **Leak A** (a fixture forced into `state.values`
  under a non-secret name SURVIVES redaction → `persist_leaks` MUST be `true`, proving the check is
  not vacuous) + **Leak B** (a pre-redaction tree serialize contains the key, proving
  `redact_for_persistence` is the load-bearing step). `i3_negative_masked_preview_check_bites`
  (`:656-678`): an unmasked token renders cleartext (check fires); a `mask=true` token is redacted.
  All FAKE fixtures; no real secret is ever persisted by a negative cell.

### 2. Persist surface uses the REAL redact walk
`persist_leaks` (`:204-208`) runs `redact_for_persistence` → `serde_json::to_string` (not bare
`serde(skip)`). `i3_tree_key_persist_then_redact` (`:485-520`) proves redaction is LOAD-BEARING:
PRE-redaction serialize contains BOTH `key` and `keys` fixtures (`:491-496`); POST-redaction both are
absent (`:500-509`); the watch-only **xpub SURVIVES** (`:513-519`). Traced to
`persistence.rs:142-145` → `tree_model.rs:180-191 redacted_for_persistence` →
`:718-743 blank_non_extended_public_keys` (positive `is_extended_public_like` allowlist, `:699-708`).
The redactor discriminates — non-vacuous.

### 3. Masked-argv surface
`masked_preview_leaks` (`:214-219`) renders BOTH `ShellFlavor::Posix` + `WindowsCmd` via
`render_copy_command_masked` (`invocation.rs:524-545`, substitutes `SECRET_MASK`). `argv_mask_status`
(`:225-241`) asserts every fixture-bearing raw-argv token carries `mask == true`. The "35/40" claim
is honest as a floor — empirically 40/40 reach, so no fixture-bearing token is silently uncovered;
the (currently-zero) "conditionally suppressed" remainder would be a no-leak no-op (not emitted ⇒
cannot leak), and persist + structural spec-stdin still cover those.

### 4. Harness hygiene
See the Hygiene ruling above — PASS.

### 5. Enumeration completeness
`classified_secret_flags` (`:76-86`) field-extracts via `flag_is_secret` (`secrets.rs:151-153`) over
all 4 schemas — the SAME predicate `render_with_dispatch` routes on (`widget.rs:89,175`). The census
(`:247-299`) asserts every classified secret is `Text | NodeValueComposite | Boolean` (a future secret
`Number`/`Path`/`Dropdown` trips it → forces a driver, logged not silently uncovered). Counts pinned:
**(40 value-bearing, 24 Boolean, 64 total)** + **5 secret positionals** — all verified live.

### 6. Narrowings — all LEGITIMATE
- **(a) 24 Boolean `*-stdin` toggles** — LEGITIMATE. Rendered as an always-disabled checkbox with no
  value buffer (`widget.rs:175-189`); the assembler `continue`s them (`invocation.rs:315-317`). The
  cell pins the only residual (a stale persisted checkbox): name-net membership asserted, a forced
  `Boolean(true)` dropped by `redact_for_persistence`, and never emitted to argv (`:386-438`).
- **(b) 5 secret positionals** — LEGITIMATE. Their widget render is **bin-private** (`main.rs:825-878`,
  inside the egui loop) — genuinely unreachable from the integration harness. Persist is type-level
  (`secret_widgets["positional:<name>"]` serde-skip + unconditional positional drop at
  `persistence.rs:130`) and the DRIVE/emit/serialize legs are covered by `persist_redaction_v0_34_0`
  (9 green). Count pinned at 5 (`:288-298`).
- **(c) seed-xor-combine `--share` Composite → `state.values`** — LEGITIMATE and ACCURATE. Confirmed in
  the schema: TWO `Text` `--share` (slip39-combine `mnemonic.rs:1779`, ms-shares-combine `:1969` →
  `secret_widgets`) + ONE `NodeValueComposite` `--share` (seed-xor-combine `:2074`, node `phrase` →
  `state.values` via `render_repeating`). The sweep drives both paths (`fixture_landed` checks both
  stores); the composite is the genuinely-discriminating persist case (node `phrase` blanked) and its
  argv token is `mask=true` (flag is secret-bearing).

### 7. Gates (all GREEN)
- `cargo test --test ui_harness_i3_secret_nopersist --jobs 2` → **7/7**.
- P0 `spike_widget_drivers` **6/6**; P1 `ui_harness_i1_roundtrip` **10/10**; P2
  `ui_harness_i2_conditional` **31/31**.
- `persist_redaction_v0_34_0` **9/9**; `secret_taxonomy_pin` **9/9**; `schema_mirror_secret_drift`
  **1/1**; `repeating_secret_rows` **8/8**.
- `cargo clippy --all-targets -D warnings` (forced re-check on the I3 target) → **clean**.
- Broad `cargo test --jobs 2` → all green, **0 FAILED**.
- `git diff master..` is **TESTS-ONLY** (no `src/` change); no broad `cargo fmt`. Worktree left clean.

---

## Minor / Nit (non-blocking — optional fold)
- **N1 (comment-accuracy):** the `reached_argv_masked >= 35` framing implies a "conditionally
  suppressed remainder"; empirically all 40 reach (NOT_REACHED=0 today). The floor is sound and the
  slack is good forward-defense — but the comment slightly overstates current suppression. Optional:
  note "all 40 reach today; the 5-slack tolerates future conditional drift."
- **N2 (transparency):** for the 39 Text secrets the surface-1 (persist) assertion is type-level
  always-green (`secret_widgets` serde-skip); its regression value (catching a future re-route of a
  secret Text into `state.values`) is carried by negative Leak A + the composite + the tree cell. The
  `persist_leaks` doc already states this honestly — fine to leave; a one-line in-cell note would make
  the type-level-vs-runtime split explicit.
- **N3:** `i3_negative_persist_check_bites` Leak B asserts only the PRE-redaction serialize contains
  the key; the POST-redaction `persist_leaks == false` for the same tree is proven in
  `i3_tree_key_persist_then_redact`. The load-bearing demonstration is complete but split across two
  cells — acceptable.

---

## Bottom line
**GREEN, 0C/0I.** The I3 classified-secret regression net is non-vacuous, discriminating, and
hygiene-clean against a first-class secret bar; all named gates pass; the diff is tests-only and the
worktree is clean. Cleared to proceed to P4.
