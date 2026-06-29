# Post-implementation whole-diff review — mnemonic-gui automated UI-functionality test harness

**Reviewer:** opus architect (independent, adversarial whole-system gate)
**Scope:** `feat/ui-harness-p0-spike` @ `c79936d` vs `master` @ `da47994` — the COMPLETE feature (P0–P6), including the **folded P4 (I4) formal review**.
**Date:** 2026-06-29
**Diff:** 11 files, +4137 / −15 — TESTS + docs ONLY, **zero `src/` change** (verified).

---

## VERDICT: **GREEN — PR-ready (0 Critical / 0 Important).**

The harness is internally coherent across all six phase files, the shared engine is used consistently (no helper misused across phases), the anti-tautology posture is uniform and explicit, the secret-hygiene net is leak-free on every surface I could reach, and the deterministic gate + I4 are CI-wired with real teeth. I verified GREEN empirically, not by re-reading the per-phase reports:

- Deterministic gate (`cargo test --jobs 2`, the 5 normally-run binaries): **all pass** — spike + i1 + i2(31) + i3(7) + sweep(2 run / 3 `#[ignore]`d), cargo exit 0, **0 compiler warnings**.
- **I4 live** against the installed pinned binaries (mnemonic 0.75.0 / md 0.11.3 / ms 0.13.2 / mk 0.11.2): **4/4 pass** (decode-address, md decode, ms decode, mk decode).
- **CI clippy gate** (`cargo clippy --all-targets -- -D warnings`, the real `build.yml` gate): **exit 0**.

Nothing below blocks the PR; the Minor/Nit items are polish.

---

## Critical
None.

## Important
None.

## Minor / Nit

1. **(PR-readiness) The design trail is untracked.** `design/SPEC_*`, `design/IMPLEMENTATION_PLAN_*`, the six `design/agent-reports/gui-ui-test-harness-{spec,plan,p1,p2,p3,p5}-r0-*` reports, and `design/GUI_TEST_HARNESS_CONSULT.md` are **untracked** on the branch (the committed diff is the 11 code/doc files only). The GUI repo *does* track `design/` (125 files), and the test headers + README cite these docs by path. Per repo convention they should be `git add`ed into the PR so the R0 audit trail and the deferred-P4-review note ship with the code. (This whole-diff review will be a 7th agent-report — stage it too.) Not a functional blocker.

2. **(Doc nit) "7 new test binaries" is 6.** The PR adds 6 top-level integration-test binaries (`spike_widget_drivers`, `ui_harness_i{1,2,3,4}_*`, `ui_harness_sweep`). `tests/ui_harness/mod.rs` is a *submodule* (`mod ui_harness;`-included), not an auto-discovered target, and `tests/ui_harness/README.md` is docs. Immaterial to the OOM math (peak is `-j`-bounded, not count-bounded), but the count in the task framing is off by one.

3. **(Hygiene nit, P4) I4's exit-≠0 message surfaces `stderr`.** For the `ms decode` cell, stdout (the recovered-phrase channel) is correctly withheld on every failure path; stderr is surfaced on a non-zero exit. This is sound here because (a) the vector is the *public* all-zero "abandon…about" vector, and (b) a decode that exits non-zero has no recovered phrase to have written. The doc is honest ("stderr carries only non-secret warnings/notes"). **Carry-forward:** if a future I4 cell ever feeds a *real*-secret-bearing input, stderr must also become coordinate-only. File-as-comment, not a fix.

4. **(Maintenance-coupling nit, I3) The `i3_classified_secret_partition_census` hard-pins `(value_bearing, narrowed_boolean, total) = (40, 24, 64)` + `secret_positionals == 5`.** This is now a *second* site (alongside `secret_taxonomy_pin.rs`) that must bump when the secret taxonomy changes. That redundancy is deliberate defense-in-depth (a new secret *kind* should loudly force the harness to gain a driver), so I'd keep it — just note it in the FOLLOWUP/README so the next secret-flag cycle updates both.

5. **(Optional CI belt-and-suspenders) No `--jobs` bound on the CI test step.** See the CI-readiness verdict — assessed as LOW risk and **no change required**; listed here only as the available mitigation if a future link-OOM flake appears.

---

## Shared-engine coherence (`tests/ui_harness/mod.rs`)

**Coherent. No phase misuses a shared helper in a way that contradicts another.** Specifics I checked:

- **`render_one_flag` (isolation) vs `render_whole_form` (whole-form) — the M1 omission is safe everywhere it's used.** `render_one_flag` deliberately omits the real form loop's three mode `continue`s (`--slot` SlotEditor handoff, build-descriptor tree-mode, build-descriptor archetype-mode). The **sweep** (which uses `render_one_flag` via `render_flag_harness`) does NOT misuse this on a build-descriptor sub: `sweep_candidate_bases` keeps every build-descriptor candidate **mode-free** (empty base only — no `tree`, no `--archetype` seeded), AND `prepared_eligible_base` hard-guards every flag with `is_render_suppressed(...)` before driving (skipping any flag the real form would `continue`). So in the only state the sweep ever drives a build-descriptor flag, the real form is in generic mode where it *also* suppresses nothing — `render_one_flag` is byte-faithful there. I2 (which DOES exercise tree/archetype/slot modes) correctly uses `render_whole_form`, which reproduces all three `continue`s. The two renderers never overlap on a state where the omission would matter.

- **`flag_is_secret` is the single shared predicate on both sides of the secret boundary.** The I1/sweep enumerator (`identity_flags`) *excludes* `flag_is_secret(flag)`; I3 *enumerates* `flag_is_secret(flag)`. Both key on the identical `mnemonic_gui::secrets::flag_is_secret`, which is also the predicate `render_with_dispatch` uses to route Text→`SecretLineEdit`/`secret_widgets` vs `state.values`. So a secret flag can never be driven down the values-routed I1 path (it would render a `PasswordInput`, not the `TextInput`/store the I1 drive+assert expect), and a non-secret flag can never be missed by I3. No drift, no double-counting, no gap — the partition is exact (`enumerator_excludes_secret_passphrase` + `i3_classified_secret_partition_census` pin both halves).

- **I1-isolation vs I2/sweep-whole-form cannot lie to each other.** I1's slice is all `conditional: None` subs (no gate can suppress), so the isolation render observes pure render→store→argv wiring. The sweep, on *conditional* subs, only asserts a round-trip when `effect_of(...) ∈ {Visible, Required}` both before AND after the drive (the post-drive re-check catches self-gating → `SelfGated`, deferred to I2). So the isolation render never asserts wiring on a flag whose whole-form gate state would differ. I2 owns the gate-interaction truth via `render_whole_form`. The boundary is clean.

- **Enumerator/classifier consistency** is itself gated (`enumerator_yields_only_identity_nonsecret_nonrepeating` cross-checks `identity_kind` vs the yielded kind for every flag of all four CLIs). `Injected::kind()` cross-checks against the flag's real kind inside `drive()` (`assert!(injected.kind().contains(&kind))`), so a mismatched injection panics rather than silently testing the wrong widget.

---

## P4 ruling (the folded formal review)

**PASS.** `ui_harness_i4_realcli.rs` is a faithful, deterministic, hygienic real-CLI oracle.

- **Determinism:** every input is a fixed public vector; every asserted field (`valid`/`script_type`/`networks`, `schema`/`tree.tag`, `entropy_hex`/`word_count`/`language`, `xpub`/`origin_fingerprint`/`chunks`) is a pure function of the input string — no RNG, no timestamp, no env in the asserted path. Verified live: 4/4 GREEN against the installed pins.
- **Genuinely tests the GUI's assembler, not a hand-rolled argv:** the `--json` Boolean is driven ON through the **real P1 harness widget** (`render_flag_harness` → `drive(Boolean(true))`), then `assemble_argv(schema, sub, h.state())` is the GUI's OWN assembler. A non-vacuity guard (`assert!(argv.contains("--json"))`) fails loudly if the drive no-ops — so a broken drive can't be mis-attributed to the CLI. (The 4 targets are all `conditional: None`, so no gate can suppress `--json` — stated and correct.)
- **Env-gating is EARLY-RETURN-SKIP, not `#[ignore]`:** `pinned_bin()` returns `None` on unset/blank → the cell prints a skip note and `return`s; a SET-but-bogus value deliberately does NOT skip (the runner errors loudly). Confirmed both directions: unset ⇒ clean skip; set ⇒ runs. Critically this means under `cargo test --workspace` with the pins installed (the CI shape) I4 actually **runs** — it is not silently inert.
- **Secret hygiene for `ms decode` (its `ms1` positional is `secret:true`):** the recovered-phrase channel is stdout, and stdout is **never echoed on any failure path** — the JSON parse-failure message is coordinate-only ("payload withheld — see coordinates"), and `result.stdout` is `.clone()`d only because `RunResult: Drop` zeroizes (E0509) and is passed straight to `serde_json::from_slice` (never `eprintln!`d). The secret positional is seeded into the real `secret_widgets["positional:ms1"]` store the assembler reads (the bin-private positional widget isn't reachable from the integration harness — an honest, documented hand-seed of *data*, while the `--json` flag under test is widget-driven). Only stderr is surfaced — acceptable (public vector; see Minor #3).

---

## CI-readiness verdict (incl. OOM assessment)

**READY.** The deterministic harness + I4 gate via `schema-mirror.yml:127-133` `cargo-test-full-suite` (`cargo test --workspace`, all four `*_BIN` set to bare names).

- **(a) All deterministic cells + I4 RUN there.** The four pins are `cargo install`ed in the preceding steps (`install-mnemonic-toolkit` → `mnemonic`, `install-{md,ms,mk}-cli` → `md`/`ms`/`mk`) into `~/.cargo/bin` (on `$PATH`), and the step sets `MNEMONIC_BIN=mnemonic` etc. So I4 resolves the bins and exercises them — **not** silently skipped. Verified the live equivalent passes 4/4.
- **(b) The proptest `#[ignore]` finders do NOT run in the gate.** `cargo test --workspace` skips `#[ignore]`d tests by default (they need `-- --ignored`). Confirmed: the sweep binary reports `2 passed; 3 ignored`. No proptest randomness, shrink, or `proptest-regressions` file (`failure_persistence: None`) ever enters CI.
- **(c) Linker-OOM risk — LOW; no mitigation required.** The local `--jobs 2` note is correctly scoped to high-core dev boxes: this machine is **24-core**, so a bare `cargo test` defaults to `-j 24` → up to 24 concurrent egui/kittest link jobs → the documented OOM on `argv_assembler_slot`. GitHub `ubuntu-latest` (public-repo standard: **4 vCPU / 16 GB**) defaults to `-j 4`, so **peak link parallelism is capped at 4** regardless of binary count. Peak memory is governed by that ceiling, not the total number of test binaries — and the repo ALREADY links ~58 egui test binaries through this same `cargo test --workspace` step on every master/PR run (it's the shipping gate). Adding **6** more binaries raises wall-clock, not peak RSS (≈4 × ~2.5 GB worst-case concurrent debug-links ≈ 10 GB < 16 GB). So this PR does not move the CI peak. **No change needed.** If a future link-OOM flake ever appears, the cheapest fix is a CI-only `cargo test --workspace --jobs 2` (or a repo `.cargo/config.toml [build] jobs`), but adding it now would only slow CI for no benefit.
- **(d) `proptest` dev-dep + Cargo.lock churn — inert for everything that matters.** The lock diff is **purely additive** (proptest + rand/rand_chacha/rand_core/rand_xorshift, rusty-fork, wait-timeout, unarray, quick-error, fnv) — **no production dep version changed** (no `-name`/`-version` lines). `proptest` is `[dev-dependencies]`, so it never enters the shipped binary's graph and `cargo install --locked` (which doesn't build dev-deps) is unaffected; the lock remains satisfiable. MSRV is fine: `rust-version = "1.88"` ≫ proptest 1.x's MSRV. `readme_pin_coherence` / `wire_shape_snapshot` parse only README install lines + `Cargo.toml [package].version` / `[dependencies] mnemonic-toolkit` — they do not read `[dev-dependencies]` or the lock graph, so the new dev-dep can't trip them. `schema_mirror` / `schema_mirror_secret_drift` / `secret_taxonomy_pin` / `gui_schema_conditional_drift` see no clap/schema/secret-taxonomy change (test-only addition).

---

## Whole-feature hygiene posture

**Self-custody-grade. No leak seam found across I1/I2/I3/I4 + the sweep.**

- **No real secrets anywhere.** Every secret fixture is a `FAKE_SECRET_FIXTURE_*` / `SWEEP_FIXTURE_*` / `FAKE_[A-Z0-9_]{4,20}` sentinel; the only "secrets" with real structure are the *public* watch-only `SURVIVING_XPUB` and the public ms/mk/md/address vectors.
- **Failure messages are coordinate-only on every secret surface.** I3's `leak_msg`/`coord` never embed the FormState, the AccessKit tree, the serialized blob, the argv, or the fixture value; secret reads go through `SecretLineEdit::as_string()` (`Zeroizing<String>`); `fixture_landed`/the surface checks return `bool` only. I4 withholds stdout. The P5 proptest I3 finder is likewise coordinate-only.
- **The net BITES (non-vacuous).** Per-flag `fixture_landed` proves the injection actually reached the store; `i3_negative_persist_check_bites` + `i3_negative_masked_preview_check_bites` are deliberate-leak cells proving `persist_leaks`/`masked_preview_leaks` would catch a real leak; `i3_tree_key_persist_then_redact` proves the redactor *discriminates* (xpub survives, private `key`/`keys` blanked) and the PRE-redaction serialize genuinely contains the fixtures (so the POST assertion isn't vacuous); `reached_argv_masked >= 35/40` proves the masking path is exercised, not bypassed.
- **All three persistence/leak surfaces covered + the disjointness argument is sound:** (1) `redact_for_persistence`→serialize, (2) `assemble_argv_with_secret_mask`→`render_copy_command_masked` (both shell flavors) with per-token `mask==true` correctness, (3) `--spec -` stdin held in the `Zeroize`+`Drop`-scrubbing `PendingConfirm` (proven scrubbed). A classified-secret *flag* routes to `secret_widgets`/`state.values`, never `state.tree`, so it is structurally incapable of reaching the tree-spec stdin (asserted), and the tree `key`/`keys` legs are covered separately.
- **`src/` correctly untouched.** The P5 sweep found **0 functional bugs** and **1 usability FOLLOWUP** (`gui-prefilled-default-text-appends-on-type`) — and that is **honest**, not vacuously green: the sweep drives 342 real round-trips (floor-gated `checked >= 80`, `subs_with_cover >= 40`, `n_subs == 61`), and the one finding is a genuine UX papercut (editable-prefilled default → typed-without-clearing concatenation) correctly triaged MINOR because the underlying render→store→argv **wiring is correct** (the `Text("")` empty-seed exposes that the cleared-then-typed value persists faithfully). The empty-seed is a harness-artifact fix, not a bug-masker: a real wiring break would still RED (it removes the prefill, it does not bypass the widget→store→argv path). No `src/` change was required and none was skipped. The FOLLOWUP entry is well-formed (Surfaced / Why-MINOR / Action / Status `open` Tier `ux` + Companion `gui-automated-ui-functionality-harness`).

---

### Bottom line
**GREEN. Ship the PR.** Deterministic gate + I4 live + the exact CI clippy gate all verified GREEN; shared engine is coherent; hygiene is leak-free; CI OOM risk is low and needs no mitigation; the only pre-PR housekeeping is staging the untracked `design/` artifacts (Minor #1) so the R0 audit trail travels with the code.
