# SPEC — mnemonic-gui v0.22.0 — toolkit-v0.41.0 pin catch-up + ms1 slot picker + pin-coherence guard

**Status:** draft (pre-R0). **Target:** `mnemonic-gui` **0.22.0** (SemVer MINOR).
**Provenance:** toolkit FOLLOWUP `gui-ms1-slot-subkey-pending-pin-bump` (`mnemonic-toolkit/design/FOLLOWUPS.md`); recon `mnemonic-toolkit/cycle-prep-recon-gui-ms1-slot-subkey-pending-pin-bump.md`; pre-SPEC architect design review (SOUND-WITH-CHANGES, 0C/3I — folded below); a measurement run of `schema_mirror` against the current released binaries.
**Source SHAs (re-grep at impl time):** GUI `ec9f00b` (master) + the `bundle-slot-ms1-gui` draft `d04bad9`; toolkit `d8d0170` (v0.41.0). Current released siblings: ms `0.7.0`, mk `0.7.0`, md `0.6.2`.
**Decisions (user):** include the pin-coherence recurrence guard; SemVer **MINOR → 0.22.0**.

---

## §1. Problem (measured, not assumed)

`mnemonic-gui` last shipped **v0.21.3** pinning `mnemonic-toolkit-v0.37.3`. The GUI's per-cycle discipline (visible across the entire CHANGELOG) is to bump `pinned-upstream.toml` + `Cargo.toml` git-dep + `Cargo.lock` **in lockstep** with each toolkit feature. That discipline broke after v0.21.3: schema edits for toolkit features v0.38.0–v0.41.0 (`addresses`, `ms-shares-split/combine`, the ms `split`/`combine`, mk `--xpub`, etc.) were committed to `src/schema/*.rs` on master **without** any version / pin / CHANGELOG bump (the K-of-N v0.40.0 GUI commit `ec9f00b` is the clearest instance — it touched ONLY `src/schema/{mnemonic,ms}.rs`).

**Consequence:** `schema_mirror` (set-equality, `tests/schema_mirror.rs:5`) is **RED in CI**, because CI installs the binaries at the STALE `pinned-upstream.toml` tags (mnemonic v0.38.0 / ms v0.5.0 / mk v0.6.0 / md v0.6.1) while the schemas are already ahead. Separately, the prepared ms1 draft (`bundle-slot-ms1-gui`: slot-editor `Ms1` picker + `SECRET_SLOT_SUBKEYS` snapshot += `"ms1"`) cannot compile until the Cargo lib pin reaches v0.41.0 (the `src/secrets.rs` const-assert).

**Measurement (GUI master + `+1.94.0`, `schema_mirror` vs CURRENT binaries mnemonic 0.41.0 / ms 0.7.0 / mk 0.7.0 / md 0.6.2):** ALL 21 schema_mirror tests **PASS** — **ZERO clap flag-NAME drift** on every subcommand the schemas list. The ONLY gap: **`md repair`** (in the md 0.6.2 binary, absent from `src/schema/md.rs` — 9 binary vs 8 schema). So the schemas are already current for flag NAMES + subcommand presence except `md repair`; the redness is **purely stale pins**. Dropdown value-enums (TEMPLATES, EXPORT_FORMATS, address-types, MS_SHARES_TO_SHAPES) verified clean against toolkit source.

## §2. Goal

Bring all pins current to the released toolchain, land the ms1 slot picker, add the `md repair` schema entry, fix the stale user-facing install/banner artifacts, and add a pin-coherence guard so the lockstep break cannot recur silently. Net effect: `schema_mirror` GREEN again — **this cycle is also a CI bug-fix**, not purely a feature. GUI `0.21.3` → **`0.22.0`**.

## §3. SECRET_NODE_TYPES is UNCHANGED v0.37.3→v0.41.0 → the draft snapshot is sufficient (architect Answer A)

`src/secrets.rs` has TWO compile-time `const _: () = assert!(secret_slice_eq(<toolkit re-export>, v0_3_canonical_fallback::SECRET_*))` guards (`:78-99`). Verified `mnemonic_toolkit::secret_taxonomy` at tag `mnemonic-toolkit-v0.37.3` vs master (v0.41.0):
- `SECRET_NODE_TYPES` (8): `phrase, entropy, xprv, wif, ms1, bip38, electrum-phrase, seedqr` — **byte-identical** at both tags.
- `SECRET_SLOT_SUBKEYS`: **5 → 6** (`+ms1` at index 3).

So the draft's `SECRET_SLOT_SUBKEYS`-only snapshot bump (already on the branch, `src/secrets.rs:67-68`) is **provably sufficient**: the `SECRET_NODE_TYPES` snapshot needs NO edit, and both const-asserts compile at the v0.41.0 pin. (No new `NodeType` was added by the silent-payment/nostr/addresses/ms-shares cycles — `addresses --address-type` is a free-text value_parser, not a NodeType.)

## §4. Edit list (the architect's 12 items, folded)

**Pins (the load-bearing fix):**
1. `Cargo.toml:42` — toolkit git-dep `tag = "mnemonic-toolkit-v0.37.3"` → `"mnemonic-toolkit-v0.41.0"`. Regen + stage `Cargo.lock` (`cargo update -p mnemonic-toolkit` or `cargo build`, per `feedback_phase_6_cargo_lock_stage_with_version_bump`). Compile-safe: the GUI's ENTIRE toolkit-lib use is `secret_taxonomy::{SECRET_SLOT_SUBKEYS, SECRET_NODE_TYPES}` (two `&[&str]` consts; no module move/rename at v0.41.0).
2. `Cargo.toml:3` — `version = "0.21.3"` → `"0.22.0"`.
3. `pinned-upstream.toml` — `[mnemonic].tag` v0.38.0→v0.41.0; `[ms].tag` v0.5.0→v0.7.0; `[mk].tag` v0.6.0→v0.7.0; `[md].tag` v0.6.1→v0.6.2 (+ re-word the stale `[md]` "bumped … to v0.4.3 … md 0.4.3" comment, ~`:32-36`).

**Schema gap:**
4. `src/schema/md.rs` — add the `md repair` `SubcommandSchema` (§5). This is the ONLY schema-body edit.

**ms1 draft (KEEP from `bundle-slot-ms1-gui`):**
5. `src/form/slot_editor.rs` — the `Ms1` picker variant (+ `ALL` + `as_str` + `is_secret_bearing`), already on the branch.
6. `src/secrets.rs:67-68` — `SECRET_SLOT_SUBKEYS` snapshot `["phrase","seedqr","entropy","ms1","xprv","wif"]`, already on the branch. **No `SECRET_NODE_TYPES` snapshot edit (§3).**

**User-facing completeness (architect I-1/I-2 — else this cycle re-stales artifacts):**
7. `README.md` (~`:42,50-53`) — bump the manual-install `cargo install --tag` block (currently wildly stale: mnemonic-toolkit v0.13.0 / md-cli v0.5.0 / ms-cli v0.2.1 / mk-cli v0.3.1 / gui v0.3.0) to the current set: mnemonic-gui v0.22.0, mnemonic-toolkit v0.41.0, md-cli v0.6.2, ms-cli v0.7.0, mk-cli v0.7.0 — so README:47's "match pinned-upstream.toml" claim is true again. (No `scripts/install.sh` exists in this repo.)
8. `pinned_version` banner strings: `src/schema/mnemonic.rs` ~`:3452` "mnemonic 0.38.0" → "mnemonic 0.41.0"; `src/schema/md.rs` ~`:532` "md 0.5.0" → "md 0.6.2"; `src/schema/mk.rs` ~`:476` "mk 0.6.0" → "mk 0.7.0". (`src/schema/ms.rs` ~`:529` "ms 0.7.0" already current — leave.)

**Recurrence guard (user decision: include):**
9. NEW `tests/pin_coherence.rs` (§6) — pure-logic assert that `Cargo.toml`'s toolkit git-dep `tag` == `pinned-upstream.toml [mnemonic].tag`. ~20 LOC, no binary/network.

**Docs + lockstep:**
10. `CHANGELOG.md` — new `[0.22.0]` entry (§9).
11. `mnemonic-toolkit/design/FOLLOWUPS.md` — flip `gui-ms1-slot-subkey-pending-pin-bump` to `resolved <gui-sha>` once landed (`feedback_per_phase_agents_forget_followup_status_flip`).

## §5. `md repair` SubcommandSchema (exact — architect Answer B)

md-cli v0.6.2 `repair` (`descriptor-mnemonic/crates/md-cli/src/cmd/repair.rs`): one positional `md1_strings: Vec<String>` (`required, num_args=1..`) + one flag `--json` (bool). Mirror the existing `inspect`/`decode` shape in `src/schema/md.rs`:

```rust
const REPAIR_FLAGS: &[FlagSchema] = &[FlagSchema {
    name: "--json",
    kind: FlagKind::Boolean,
    required: false,
    repeating: false,
    help: "Emit a single JSON envelope on stdout instead of the text-form report.",
    secret: false,
    default_value: None,
    global: false,
}];
const REPAIR_POSITIONALS: &[PositionalArgSchema] = &[PositionalArgSchema {
    name: "md1-strings",
    required: true,
    repeating: true,
    help: "One or more md1 strings to repair (BCH error-correction). `-` reads one per line from stdin. Chunked-form md1 only.",
}];
// appended to SUBCOMMANDS:
SubcommandSchema {
    name: "repair",
    human_name: "Repair (BCH error-correction)",
    flags: REPAIR_FLAGS,
    positional_args: REPAIR_POSITIONALS,
    allows_slots: false,
    conditional: None,
},
```
Re-grep the live `FlagSchema`/`SubcommandSchema`/`PositionalArgSchema` field set + an existing md entry at impl time and match it exactly (field names/order may differ from this sketch). `md repair` is the ONLY md gap (9 binary subcommands vs 8 schema; the other 8 unchanged v0.6.1→v0.6.2, which was the output-class-advisory PATCH with no md flag change).

## §6. Pin-coherence guard (architect Answer C)

The bug class — "schema updated, `pinned-upstream.toml`/Cargo pin NOT bumped, masked by a local-binary schema_mirror run" — has fired TWICE (K-of-N v0.40.0; this FOLLOWUP). The existing gates can't catch it: `schema_mirror`/`schema_mirror_secret_drift`/`gui_schema_conditional_drift` all run a LIVE binary via `*_BIN` (skipping when absent) and have NO knowledge of the declared pins. `tests/pin_coherence.rs` (pure-logic, no binary, no network):

```rust
//! Guards the bug class "Cargo toolkit pin and pinned-upstream.toml drift
//! apart" (CHANGELOG v0.22.0). The two MUST move in lockstep — pinned-upstream
//! line 20-21's own comment declares it; this promotes that prose to a gate.
#[test]
fn cargo_toolkit_pin_matches_pinned_upstream_mnemonic_tag() {
    let cargo = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let pinned = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/pinned-upstream.toml")).unwrap();
    // Cargo.toml: mnemonic-toolkit = { git = "...", tag = "mnemonic-toolkit-vX.Y.Z" }
    let cargo_tag = extract_tag_after("mnemonic-toolkit", &cargo);   // parse the tag = "..." on the mnemonic-toolkit dep line
    // pinned-upstream.toml: [mnemonic] ... tag = "mnemonic-toolkit-vX.Y.Z"
    let pinned_tag = extract_mnemonic_tag(&pinned);
    assert_eq!(cargo_tag, pinned_tag,
        "pin drift: Cargo.toml toolkit tag {cargo_tag:?} != pinned-upstream [mnemonic].tag {pinned_tag:?}; \
         bump both in lockstep (CHANGELOG v0.22.0 bug class)");
}
```
Implement the two small parsers inline (string-scan the `mnemonic-toolkit` dep line + the `[mnemonic]` table's `tag`). Keep it dependency-free (no `toml` crate needed; if `toml` is already a dev-dep, use it). Scope note: this guards only the two TOOLKIT pins agree — the three sibling pins rely on the standing paired-PR discipline + the live schema_mirror gate (acceptable).

## §7. Verification gate (cycle-final)

Build all four current binaries (mnemonic v0.41.0 from `mnemonic-toolkit` master; ms v0.7.0; md v0.6.2 `--features cli-compiler`; mk v0.7.0). Then, with `MNEMONIC_BIN`/`MS_BIN`/`MD_BIN`/`MK_BIN` set to those abs paths, on the cycle branch (Cargo lib pin already at v0.41.0):
- `cargo +1.94.0 test --workspace` GREEN — incl. `schema_mirror` (all per-CLI cells, now that pins == current binaries + `md repair` added), `schema_mirror_secret_drift`, `gui_schema_conditional_drift`, `xpub_search_schema_mirror`, the template-groups parity cells, `secret_taxonomy_pin`, and the NEW `pin_coherence`.
- The two `const _: () = assert!` supply-chain guards (`src/secrets.rs:78-99`) COMPILE at the v0.41.0 pin (the load-bearing proof the draft + §3 are correct).
- `cargo +1.94.0 clippy --all-targets -- -D warnings` clean. GUI builds (the toolkit pins 1.85 but the GUI needs ≥1.88 → use `+1.94.0`/`+stable`; the GUI's own CI uses `@stable`).
- **FIRST execution action (pre-impl sizing confirm):** re-run `schema_mirror` against the freshly-built current binaries to empirically re-confirm `md repair` is the sole flag-NAME delta before editing.

## §8. Phasing (mandatory opus R0 on SPEC + each phase + end-of-cycle; 0C/0I before code; re-dispatch after every fold; persist to `design/agent-reports/`)

Small cycle — two phases:
- **P1 — pins + ms1 draft + md repair + guard (the code).** Bump Cargo lib pin + `pinned-upstream.toml` tags; keep the draft (picker + snapshot); add `md repair` schema; add `tests/pin_coherence.rs`. Build the 4 current binaries; run the §7 gate to GREEN. (TDD: `pin_coherence` test written + failing against the pre-bump state, then passing after the lockstep bump; `md repair` confirmed by the schema_mirror `md` cell flipping green.)
- **P2 — version + docs.** `Cargo.toml` version 0.22.0; `pinned_version` banners; README install block; CHANGELOG `[0.22.0]`; flip the toolkit FOLLOWUP. Re-run the full gate.

## §9. SemVer + CHANGELOG

**MINOR → 0.22.0** (architect Answer D): the GUI CHANGELOG convention is new-subcommand-in-a-schema → MINOR (v0.20.0 silent-payment, v0.21.0 decode-address/verify-message). This adds the `md repair` schema entry AND the user-facing slot-editor `Ms1` picker. The `[0.22.0]` CHANGELOG entry (Keep-a-Changelog prose style, matching prior entries) must state: (a) lands the prepared ms1 slot-editor picker + `SECRET_SLOT_SUBKEYS` snapshot at toolkit pin v0.41.0; (b) catch-up pin bump across all four CLIs to current — **RESTORES `schema_mirror` green** (the pins had lagged the schemas since v0.21.3 — a CI bug-fix); (c) adds the `md repair` schema entry; (d) the new `pin_coherence` guard + names the "schema-ahead-of-pins, masked by local-binary run" bug class; (e) confirms `SECRET_NODE_TYPES` unchanged so only the `SECRET_SLOT_SUBKEYS` snapshot moved.

## §10. Citations (re-grep at impl time)

GUI `ec9f00b`: `Cargo.toml:3,42`; `pinned-upstream.toml` `[mnemonic/ms/mk/md].tag`; `src/secrets.rs:7,34,67-68,78-99`; `src/schema/md.rs` (SUBCOMMANDS + the `inspect`/`decode` shape to mirror, `pinned_version` ~`:532`); `src/schema/mnemonic.rs:3452`; `src/schema/mk.rs:476`; `src/form/slot_editor.rs` (draft picker); `tests/schema_mirror.rs:5,20-44,112` (set-equality, flag-NAME extractor); `README.md:42,47,50-53`; `CHANGELOG.md` (convention). Toolkit `d8d0170`: `crates/mnemonic-toolkit/src/secret_taxonomy.rs:76,111` (consts) vs tag `mnemonic-toolkit-v0.37.3:…/secret_taxonomy.rs`. md-cli v0.6.2: `descriptor-mnemonic/crates/md-cli/src/cmd/repair.rs`, `main.rs` subcommand list.
```
