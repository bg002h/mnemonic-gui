# SPEC — Wave-3 GUI wire-shape lane (W3-1 keystone + W3-3 mk-help mirror)

Repo: `/scratch/code/shibboleth/mnemonic-gui` (current HEAD `6df305d` on `master`, crate `0.48.1`).
Toolkit build-pin (load-bearing): `Cargo.toml:42` `mnemonic-toolkit` tag `mnemonic-toolkit-v0.70.0`.
All wire-shapes below were captured LIVE from a `mnemonic 0.70.0` binary (the pin), 2026-06-22.

Scope (orchestrator decision): KEYSTONE W3-1 (per-consumer `--json` wire-shape golden snapshots for
`import-wallet` + `xpub-search` ONLY) + a tiny W3-3 prose mirror (GUI mk schema help string).
GUI MINOR. Ships as PR → CI green (I verify before merge) → tag `mnemonic-gui-v0.49.0`. NO crates.io publish.
The toolkit FOLLOWUP flip lands on the toolkit side AFTER the GUI PR merges (NOT in this repo).

---

## 0. Semver + ship mechanism

- **mnemonic-gui MINOR → `0.49.0`** (new test surface + a new tests/ module + re-baselined fixtures + a
  schema-help prose edit). Bump `Cargo.toml:3` `version = "0.48.1"` → `"0.49.0"`.
  - Note: `Cargo.toml:62 version = "0.6"` is a *dependency* version (unrelated); do NOT touch it.
- **Version-site completeness (release ritual — all bumped in lockstep with `Cargo.toml:3`):**
  1. `Cargo.toml:3` `version = "0.48.1"` → `"0.49.0"`.
  2. **`README.md:42`** GUI self install-line `--tag mnemonic-gui-v0.48.1` → `--tag mnemonic-gui-v0.49.0`.
     This is **CI-GATED**: `tests/readme_pin_coherence.rs::readme_install_tags_match_pins` (L68) asserts the
     README `mnemonic-gui` `--tag` == `format!("mnemonic-gui-v{}", cargo_version())` (cargo_version reads
     `Cargo.toml [package].version`, L59-65). It is pure-logic (no binary) and runs under
     `cargo test --workspace` — i.e. the SAME `cargo-test-full-suite` schema-mirror.yml CI job AND the §5
     local check. Leaving README.md:42 at v0.48.1 RED-s that gate in CI and locally. The four SIBLING pins on
     README.md:50-53 (toolkit/md/ms/mk) are sourced from `pinned-upstream.toml` and are NOT bumped this lane
     (no pin change) — only the GUI self-tag moves. See `project_toolkit_release_ritual_version_sites`
     (version-site silent-drift class).
  3. **`CHANGELOG.md`** add a `## mnemonic-gui [0.49.0] — <date>` stanza at the top (above the `[0.48.1]`
     entry). This is **NOT CI-gated** (no test asserts a version stanza — the four `tests/` CHANGELOG hits are
     comment/doc text only, not assertions). It is a release-completeness item: every prior tagged release
     (`[0.48.1]`, `[0.48.0]`, `[0.47.0]`, …) carries a stanza; v0.49.0 must not be the first to skip one.
- **Ship:** GUI PR → all platform checks (build.yml) + schema-mirror.yml green → merge → tag `mnemonic-gui-v0.49.0`.
  PR+CI-before-tag ritual (GUI is NOT direct-FF). No sibling publish, no install.sh (GUI has none), no manual mirror.
- **No toolkit version change in this PR.** The toolkit FOLLOWUP Status flip is a separate 1-line toolkit
  `design/FOLLOWUPS.md` edit the orchestrator lands toolkit-side post-merge (§7).

---

## 1. W3-1 — current behavior (the gap being closed)

`tests/cli_envelope_smoke.rs` (59 lines, 5 cells) does LOOSE presence-checks over **stale vendored
v0.27.0 fixtures** via `include_str!` — it never runs the pinned binary and never asserts full shape:

- `import_wallet_json_envelope_parses_v0_27_x_shape` — only asserts top-level array + `schema_version=="1"`
  + `bundle.descriptor` present. Does NOT assert `roundtrip.status`.
- `xpub_search_path_of_xpub_match` — asserts `result=="match"` + `path` present.
- `xpub_search_path_of_xpub_no_match` — asserts `result=="no_match"` only.
- `xpub_search_account_of_descriptor` — asserts `matched_cosigners[0].account` present.
- `xpub_search_passphrase_of_xpub` — asserts `result=="match"`.

**Confirmed drift between the vendored fixtures and live v0.70.0** (this is exactly what the slug wants pinned):

| Surface | Vendored v0.27.0 fixture | LIVE v0.70.0 | Drift |
|---|---|---|---|
| `xpub-search path-of-xpub` no_match | `{...,"path":null,"template":null,"account":null,"target_xpub_canonical":...,"target_xpub_variant":null,"searched_count":140}` | `{"schema_version":"1","mode":"path-of-xpub","result":"no_match","target_xpub_canonical":...,"target_xpub_variant":null,"searched_count":140}` | **path/template/account keys OMITTED (not null) on no_match** |
| `xpub-search account-of-descriptor` no_match | (no vendored no_match cell asserted) | `{"schema_version":"1","mode":"account-of-descriptor","result":"no_match","cosigners_total":1,"searched_count_per_cosigner":140,"descriptor_shape":"literal_xpub","unspendable_internal_keys":[]}` | `matched_cosigners` OMITTED on no_match (present-on-match only) |
| `import-wallet --json` roundtrip | fixture HAS `status` but the smoke test never asserts it | `roundtrip` = `{byte_exact, diff, semantic_match, status}` | smoke test under-asserts `status` |

The fixtures are loose-equality only, so this drift is invisible today. Option (b) = upgrade to FULL
structural assertions keyed to the v0.70.0 pin, captured FROM the binary (NOT hand-written).

### Live-captured exit codes (load-bearing for test design)
- `path-of-xpub` MATCH → exit **0**; NO_MATCH → exit **4** (stderr "no match in searched set…").
- `account-of-descriptor` MATCH → exit **0**; NO_MATCH → exit **4**.
- `import-wallet --json` (coldcard/bsms) → exit **0**.
- **The golden test MUST NOT require exit 0 for no_match cells** — it asserts on stdout JSON; the
  `--json` envelope is emitted on stdout even when the process exits non-zero. (Mirror the toolkit's own
  contract: structured envelope on stdout, advisory on stderr.)

### export-wallet is DESCOPED (proven, not assumed)
`mnemonic export-wallet … --json` → `error: unexpected argument '--json' found` (exit 64). export-wallet has
**NO `--json` envelope**; its "wire-shape" is the per-`--format` wallet-file output (bitcoin-core/specter/
coldcard JSON files; bsms/jade/descriptor/green text). It is **out of scope for this lane** — note as a
follow-on (§8). Do NOT invent an export-wallet `--json` golden.

---

## 2. W3-1 — exact change (TDD, single implementer)

**Mechanism (matches the existing repo split):** LIVE-binary capture in CI (where the pinned v0.70.0
toolkit binary is installed) + a gate-skip when the binary is absent (so local `cargo test` without the
binary still PASSES). This mirrors `schema_mirror.rs`'s `resolve_bin()` / skip discipline. The vendored
fixtures are RE-BASELINED at v0.70.0 so the *local* (offline) reference is also current.

### 2.1 Binary resolution + skip helper
Reuse the exact pattern already in the repo. `tests/schema_mirror.rs:46-50` defines:
```rust
fn resolve_bin(cli_name: &str) -> String {
    let env_var = format!("{}_BIN", cli_name.to_ascii_uppercase().replace('-', "_"));
    std::env::var(&env_var).unwrap_or_else(|_| cli_name.to_string())
}
```
and the const-parity cells at `tests/schema_mirror.rs:608-620` show the canonical **skip-when-absent**
idiom (MNEMONIC_BIN unset AND `mnemonic --help` not on PATH → `eprintln!(…skip…); return;`).

In the NEW test module, add a local helper that returns `Option<String>` (the binary path) or `None`
(skip), so each cell early-returns on `None`:
```rust
/// `MNEMONIC_BIN` wins; else probe bare `mnemonic` on PATH. None => skip (no binary).
fn mnemonic_bin() -> Option<String> {
    if let Ok(b) = std::env::var("MNEMONIC_BIN") { return Some(b); }
    match std::process::Command::new("mnemonic").arg("--version").output() {
        Ok(_) => Some("mnemonic".into()),
        Err(_) => None,
    }
}
```
Rationale for NOT panicking: CI sets `MNEMONIC_BIN=mnemonic` for `cargo test --workspace`
(`.github/workflows/schema-mirror.yml:127-133`), so cells RUN in CI; dev laptops without the binary SKIP.

### 2.2 New test module: `tests/wire_shape_snapshot.rs`
Add a NEW file (do not overload `schema_mirror.rs`). Each cell:
1. `let Some(bin) = mnemonic_bin() else { return; };` (skip when absent).
2. Invoke the subcommand with `--json` on a canonical, no-real-funds input (abandon×11+about seed; vendored
   blobs).
3. `serde_json::from_slice::<serde_json::Value>(&out.stdout)` (NOT exit-code-gated).
4. Assert FULL structural shape: the exact top-level key SET, nested key sets, and the stable scalar values
   (mode, result, schema_version, searched_count, descriptor_shape, source_format, roundtrip.status). Use a
   recursive key-set extractor so a key ADD or REMOVE anywhere fails. Assert key PRESENCE/ABSENCE for the
   match-vs-no_match conditional keys (the drift class). Avoid asserting the byte content of
   `roundtrip.diff` (noisy, key-order-sensitive — see §2.4).

> **Keysets below are ILLUSTRATIVE, not the exhaustive expected set for deeply nested objects.** The cell
> table (§2.2) and §2.3 enumerate keysets for the TOP-level and the directly-named nested objects, but a
> fully-recursive extractor flattens EVERY nested object — including ones not hand-enumerated here. Example:
> the `wireshape_import_wallet_bsms_multisig` cell pins `bundle.multisig`'s keyset
> `{cosigner_count,cosigners,path_family,template,threshold}` but does NOT spell out the per-cosigner
> sub-object `bundle.multisig.cosigners[i]`, whose live v0.70.0 keyset is
> `{index,master_fingerprint,origin_path,xpub}`. **The captured golden (§2.3 — captured FROM the v0.70.0
> binary) is the source of truth for the FULL recursive key set**; the implementer MUST capture the expected
> sets from the binary's output, NOT hand-write them from this spec's prose (which would risk an incomplete
> expected set under a recursive extractor). Equivalently, an implementer who prefers explicit per-path
> assertions MAY scope the recursive extractor to only the named object paths each cell enumerates — but the
> capture-from-binary approach is preferred and self-completing. No behavior difference either way; this note
> is to prevent an implementer hand-coding an under-specified expected set.

**Cells to author (8 total):**

| Cell | Subcommand + input | Assert |
|---|---|---|
| `wireshape_path_of_xpub_match` | `xpub-search path-of-xpub --phrase-stdin --target-xpub <ABANDON_BIP84_ACCT0_XPUB> --json` | top key-set == `{schema_version,mode,result,path,template,account,target_xpub_canonical,target_xpub_variant,searched_count}`; `result=="match"`, `mode=="path-of-xpub"`, `path=="m/84'/0'/0'"`, `template=="bip84"`, `account==0` |
| `wireshape_path_of_xpub_no_match` | same flags, `--target-xpub <UNRELATED_XPUB>` | top key-set == `{schema_version,mode,result,target_xpub_canonical,target_xpub_variant,searched_count}`; **assert `path`/`template`/`account` ABSENT**; `result=="no_match"` |
| `wireshape_account_of_descriptor_match` | `xpub-search account-of-descriptor --phrase-stdin --descriptor <LITERAL_XPUB_DESC> --json` | top key-set == `{schema_version,mode,result,matched_cosigners,cosigners_total,searched_count_per_cosigner,descriptor_shape,unspendable_internal_keys}`; `matched_cosigners[0]` key-set == `{cosigner_index,path,template,account}`; `descriptor_shape=="literal_xpub"` |
| `wireshape_account_of_descriptor_no_match` | same, descriptor with an unrelated xpub | top key-set == `{schema_version,mode,result,cosigners_total,searched_count_per_cosigner,descriptor_shape,unspendable_internal_keys}`; **assert `matched_cosigners` ABSENT**; `result=="no_match"` |
| `wireshape_passphrase_of_xpub_match` | `xpub-search passphrase-of-xpub --phrase-stdin --passphrase <PP> --target-xpub <ABANDON+PP_BIP84_ACCT0_XPUB> --json` | top key-set == `{schema_version,mode,result,path,template,account,target_xpub_canonical,target_xpub_variant,searched_count}`; `result=="match"`, `mode=="passphrase-of-xpub"` |
| `wireshape_passphrase_of_xpub_no_match` | same, wrong passphrase or unrelated xpub | top key-set == `{schema_version,mode,result,target_xpub_canonical,target_xpub_variant,searched_count}`; **assert path/template/account ABSENT**; `result=="no_match"` |
| `wireshape_import_wallet_bsms_multisig` | `import-wallet --blob <BSMS_2OF3> --format bsms --json` | top-level is ARRAY len≥1; entry key-set == `{bundle,roundtrip,schema_version,source_format}`; `source_format=="bsms"`; `roundtrip` key-set == `{byte_exact,diff,semantic_match,status}` (**assert `status` present**); `bundle` key-set == `{account,descriptor,master_fingerprint,md1,mk1,mode,ms1,multisig,network,origin_path,origin_paths,privacy_preserving,schema_version,template}`; `bundle.multisig` key-set == `{cosigner_count,cosigners,path_family,template,threshold}` |
| `wireshape_import_wallet_coldcard_singlesig` | `import-wallet --blob <COLDCARD_JSON> --format coldcard --json` | top-level ARRAY len≥1; entry key-set == `{bundle,coldcard_source_metadata,roundtrip,schema_version,source_format}` (**note the source-format-specific key `coldcard_source_metadata`**); `source_format=="coldcard"`; `roundtrip` has `status`; `coldcard_source_metadata` key-set == `{bip_derivation,chain,dropped_fields,raw_account,xfp}` |

The 8 cells together pin: (a) both no_match key-omission drift classes; (b) the `roundtrip.status` key;
(c) the source-format-conditional metadata key (`coldcard_source_metadata` vs bitcoin-core's `source_metadata`
vs none for bsms); (d) the multisig sub-object shape.

### 2.3 Canonical inputs (no real funds — abandon×11+about test seed)
All confirmed reproducible against the v0.70.0 binary this session:
- SEED = `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
  (piped via `--phrase-stdin`; never on argv — §2.5 secret hygiene).
- `ABANDON_BIP84_ACCT0_XPUB` = `xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V`
  (live MATCH at `m/84'/0'/0'`, bip84, account 0).
- `UNRELATED_XPUB` = `xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV`
  (live NO_MATCH; this is the same xpub the stale vendored no_match fixture used).
- `LITERAL_XPUB_DESC` (match) = `wpkh([5436d724/84'/0'/0']<ABANDON_BIP84_ACCT0_XPUB>/<0;1>/*)` — live match,
  `matched_cosigners[0]={cosigner_index:0,path:"m/84'/0'/0'",template:"bip84",account:0}`.
- passphrase match (`passphrase-of-xpub`): **the vendored xpub is NOT reusable as a target — its passphrase
  is unknown and unverifiable.** The vendored value
  `tests/fixtures/v0_27_0_envelopes/passphrase_of_xpub.match.json` pins
  `xpub6CvHDtn5otAu9fjb7mPpbfizn2A31pQkwLsogfkHaMfsnoGVCwRidN6rZryTBE6G8b6MF152XgJSKiEBpgt3Jx7udU43auRCHB1hvJTRuBu`
  at `m/84'/0'/0'`/bip84 — but it is a v0.27.0 artifact with NO recorded passphrase, and it does not match
  under the common test passphrases (brute-forcing test/passphrase/abandon/BIP39/mnemonic/12345/password/…
  all yield no_match exit 4). **Do NOT attempt to reverse-engineer the vendored xpub's passphrase.** Instead
  FORWARD-DERIVE a fresh (passphrase, xpub) pair at impl time: CHOOSE a passphrase (e.g. a literal test
  string), run `xpub-search passphrase-of-xpub --phrase-stdin --passphrase <PP> --target-xpub <X>` to discover
  the xpub it yields for the abandon×11+about seed at the searched path, then PIN both the chosen passphrase
  and the captured match-xpub into the cell. (Equivalently: `mnemonic xpub ... --passphrase <PP>` to derive
  the BIP84 acct-0 xpub directly, then feed it as the target.) The no_match cell can reuse the SAME chosen
  passphrase with `UNRELATED_XPUB`, or the right passphrase with a wrong xpub.
- BSMS blob: copy `mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-multi-2of3.txt`
  into `tests/fixtures/wallet_import/` (or reuse the existing vendored multisig blob if structurally equal).
  Live shape confirmed structurally IDENTICAL to the existing vendored `envelope_v0_27_0.json` (zero key
  drift), so re-vendoring the BSMS blob is optional — but capture the golden from the binary regardless.
- COLDCARD blob: REUSE the already-vendored `tests/fixtures/coldcard_generic_bip84_mainnet.json`
  (no new fixture needed; live import confirmed `source_format=="coldcard"` + `coldcard_source_metadata`).

> CRITICAL CI-VERIFY (the keystone hazard): the goldens MUST be captured FROM the v0.70.0 binary, not
> hand-written and not copied from the stale v0.27.0 vendored fixtures. The CI `schema-mirror.yml` installs
> the toolkit via `cargo install --locked --git … --tag v0.70.0` (line 49-62) and runs `cargo test
> --workspace` with `MNEMONIC_BIN=mnemonic` (line 127-133). If a golden encodes the OLD shape (e.g.
> path/template/account:null on no_match), CI REDs against the v0.70.0 binary while a stale vendored-fixture
> assertion would have passed locally. Capture-from-binary is the whole point of option (b).

### 2.4 `roundtrip.diff` handling (anti-churn)
Live import-wallet (coldcard, bitcoin-core) returns `roundtrip.byte_exact==false` with a large multi-line
`diff` string whose content is sensitive to JSON key ordering. **Do NOT assert the byte content of
`roundtrip.diff`.** Assert only that `roundtrip` has the key SET `{byte_exact,diff,semantic_match,status}`,
`status` is a string, and `byte_exact`/`semantic_match` are bools. This pins the wire SHAPE (the slug's
intent) without forcing a re-vendor on every cosmetic toolkit serialization change.

### 2.5 Secret hygiene in the test
- Pass the seed via `--phrase-stdin` (stdin), NEVER `--phrase` on argv. The abandon×11+about seed holds no
  real funds, but the off-argv discipline is the standing first-class bar.
- Assert on **stdout JSON only**; ignore stderr (xpub-search emits an argv-leakage advisory + the no_match
  "no match in searched set" line on stderr — not part of the wire-shape).

### 2.6 Disposition of the OLD `cli_envelope_smoke.rs`
Two acceptable options (implementer's TDD call; recommend (A)):
- **(A) Replace:** delete `tests/cli_envelope_smoke.rs` and the now-stale vendored fixtures it `include_str!`s
  (`tests/fixtures/v0_27_0_envelopes/*` + `tests/fixtures/wallet_import/envelope_v0_27_0.json`), since the new
  live-capture module supersedes them. If deleted, also remove the SOURCE.md rows? — those fixtures are NOT
  in SOURCE.md (only coldcard + descriptor_builder are), so no SOURCE.md edit needed. Verify no OTHER test
  `include_str!`s them: grep before deleting.
- **(B) Re-baseline + keep:** re-capture each vendored fixture at v0.70.0 (so the offline reference is
  current) and upgrade the 5 smoke cells to full structural assertions over the re-baselined fixtures, while
  the new live cells assert against the binary. This keeps an offline path but doubles the maintenance.

**Recommendation: (A) Replace.** The live-capture module + skip-when-absent already covers offline (skips)
and CI (asserts vs pinned binary); keeping stale vendored fixtures invites exactly the drift this lane closes.
Before deleting, run: `grep -rn "v0_27_0_envelopes\|envelope_v0_27_0\|cli_envelope_smoke" tests/ src/` to
confirm no other consumer.

---

## 3. W3-3 — current behavior + exact change (GUI mk-help prose mirror)

`src/schema/mk.rs` carries the hand-maintained mk `vectors` schema. Two sites restate the WRONG claim
(mk-cli source `crates/mk-cli/src/cmd/vectors.rs:70-73` actually honors `--pretty` under `--out`):

- **Comment** `src/schema/mk.rs:272-273`:
  `// silently ignored when --out is set (not a clap conflicts_with).`
- **Help string** `src/schema/mk.rs:280`:
  `help: "Pretty-print JSON output. Silently ignored when --out is set.",`

**Exact edits (prose only):**
- L280 → `help: "Pretty-print JSON output. Also honored when --out is set (each per-fixture file is pretty-printed).",`
- L272-273 comment → reword to `// honored even when --out is set (each per-fixture file is pretty-printed; not a clap conflicts_with).`

Wording sourced from the toolkit manual-gui source-truth phrasing at
`mnemonic-toolkit/docs/manual-gui/src/70-mk/76-vectors.md:23-27` ("Source actually honors `--pretty` even
when `--out` is set: per … each per-fixture written file uses `serde_json::to_string_pretty`").

**Why this stays GREEN:** `schema_mirror.rs::mk_schema_flag_names_match_help_text` compares flag NAME SETS
only (`schema_flag_names` maps `f.name`; `extract_flag_names` regex-extracts `--<flag>` tokens). The `help:`
prose field is never compared. Verified: no flag added/removed/renamed → no schema_mirror drift.

**Out of scope for THIS GUI lane:** the mk-cli source reword + tag + crates.io publish, and the toolkit
manual/manual-gui prose edits, are SEPARATE repo cycles (the W3-3 slug's primary fix). This lane touches the
GUI schema help string ONLY (the orchestrator's "tiny W3-3 mirror"). Note the remaining sites as deferred (§8).

---

## 4. Exact files touched (this PR)

1. `tests/wire_shape_snapshot.rs` — NEW (the 8 live-capture golden cells + `mnemonic_bin()` helper).
2. `tests/cli_envelope_smoke.rs` — DELETE (superseded; option A) — or upgrade in place (option B).
3. `tests/fixtures/v0_27_0_envelopes/*` (6 files) — DELETE if option A (else re-baseline at v0.70.0).
4. `tests/fixtures/wallet_import/envelope_v0_27_0.json` — DELETE if option A (else re-baseline).
5. `tests/fixtures/wallet_import/bsms-2line-multi-2of3.txt` — NEW vendored BSMS blob (if not reusing an
   existing one); add a SOURCE.md provenance row (`tests/fixtures/SOURCE.md`) pinning `mnemonic-toolkit-v0.70.0`.
6. `tests/fixtures/coldcard_generic_bip84_mainnet.json` — REUSE (no change; already provenanced in SOURCE.md).
7. `src/schema/mk.rs` — 2 prose edits (L272-273 comment, L280 help string).
8. `Cargo.toml:3` — version `0.48.1` → `0.49.0`.
9. **`README.md:42`** — GUI self install-line `--tag mnemonic-gui-v0.48.1` → `--tag mnemonic-gui-v0.49.0`
   (CI-GATED by `tests/readme_pin_coherence.rs::readme_install_tags_match_pins`; must move in lockstep with
   `Cargo.toml:3` or `cargo test --workspace` RED-s in CI and locally — §0, §6). The sibling pins
   README.md:50-53 are NOT touched (no pin change).
10. **`CHANGELOG.md`** — add a `## mnemonic-gui [0.49.0] — <date>` stanza at the top (release-completeness;
   NOT CI-gated, but every prior tagged release carries one — §0).
11. `tests/fixtures/SOURCE.md` — add the BSMS-blob provenance row + (if option A) note the retired
   v0_27_0 envelope fixtures. Document the wire-shape re-vendor step (capture-from-binary at the toolkit pin).

Do NOT touch: `pinned-upstream.toml` / `Cargo.toml:42` toolkit pin (stays v0.70.0 — this lane keys to it,
does not bump it). No `src/` runtime change beyond the mk.rs prose. No CI-workflow edit.

---

## 5. Test / verification surface

- `cargo test --workspace` (locally WITHOUT the binary): the 8 new cells SKIP (binary-absent), the rest pass.
  Verifies the skip path doesn't false-RED.
- `MNEMONIC_BIN=$(which mnemonic) cargo test --test wire_shape_snapshot` with the v0.70.0 binary on PATH:
  all 8 cells RUN and PASS (this is the CI-equivalent local check — REQUIRED before declaring done; a
  v0.70.0 binary is present in this environment).
- `cargo clippy --all-targets -- -D warnings`: new test code must be clippy-clean (build.yml gate runs this
  over `--all-targets`, which INCLUDES tests).
- `cargo test --test schema_mirror`: still GREEN (W3-3 prose edit adds/removes no flag; W3-1 adds no flag).
- `cargo test --test readme_pin_coherence` (pure-logic, NO binary): must be GREEN AFTER the lockstep
  README.md:42 + Cargo.toml:3 bump (§0, §4). This is the version-site-drift gate — run it locally before PR;
  it RED-s if README.md:42 still says `mnemonic-gui-v0.48.1` while Cargo says `0.49.0`. (Also runs inside the
  `cargo test --workspace` full-suite above.)
- Negative/anti-stale check: temporarily hand-edit one golden to the OLD shape (path:null on no_match) and
  confirm the cell RED-s against the v0.70.0 binary — proves the test actually pins the new shape (then revert).

---

## 6. CI gates to verify (HOW — incl. CI-only gates)

The HARD-GATE discipline: name every gate that re-fires and HOW to verify, including CI-only gates a local
build can't reproduce.

1. **GUI `schema-mirror.yml` — `cargo-test-full-suite` (`cargo test --workspace`, `MNEMONIC_BIN=mnemonic`)**
   — fires on the PR (`pull_request: [master, release/**]`). This is the load-bearing gate: the 8 new
   wire-shape cells RUN HERE against the cargo-installed **v0.70.0** binary (installed by `install-mnemonic-toolkit`
   at lines 49-62 from `pinned-upstream.toml [mnemonic].tag`). **HOW to verify:** goldens were captured FROM
   the v0.70.0 binary (§2.3) → PASS. If captured from stale fixtures → REDs in CI while passing locally with a
   vendored-fixture assertion. This is the CI-ONLY hazard (the install-from-tag binary is the source of truth,
   not your laptop's). Pre-merge: the orchestrator re-runs this against the freshly-installed v0.70.0 binary.
2. **GUI `schema-mirror.yml` — `cargo-test-full-suite` — `readme_pin_coherence.rs::readme_install_tags_match_pins`**
   (the SAME `cargo test --workspace` job as gate 1, but this cell is pure-logic, NO binary). Asserts the
   README `mnemonic-gui` self `--tag` == `format!("mnemonic-gui-v{}", cargo_version())` (test L68/L75;
   `cargo_version` reads `Cargo.toml [package].version`, L59-65). **HOW to verify:** bump README.md:42 to
   `mnemonic-gui-v0.49.0` in lockstep with `Cargo.toml:3` (§0, §4) → PASS. If README stays at v0.48.1 →
   REDs in BOTH CI and the §5 local `cargo test --workspace`. This is the version-site silent-drift class
   (`project_toolkit_release_ritual_version_sites`); it is reproducible locally (no binary needed), so the
   §5 local run catches it pre-PR.
3. **GUI `schema-mirror.yml` — `cargo-test-schema-mirror`** — flag-NAME parity gate. UNAFFECTED (no flag
   add/remove/rename in W3-1 or W3-3). **HOW:** `cargo test --test schema_mirror` green; the mk help PROSE is
   not compared. PASS.
4. **GUI `schema-mirror.yml` — `smoke-gui-schema-mk` (+ mnemonic/md/ms)** — asserts `mk gui-schema` emits
   `{version>=1,cli:mk}`. mk gui-schema carries NO help text → prose edit invisible. **HOW:** unchanged. PASS.
5. **GUI `schema-mirror.yml` — `ci_workflow_snapshot` test** (`tests/schema_mirror.rs:163`) — asserts required
   step names present in `schema-mirror.yml`. We don't edit the workflow. **HOW:** PASS (no workflow change).
6. **GUI `build.yml` — clippy job** (`cargo clippy --all-targets -- -D warnings`, line 29-30) — fires on PR;
   the new test file is compiled under `--all-targets`. **HOW:** keep the test code clippy-clean (no unused
   imports, no needless clones, prefer `let Some(..) else { return }`). PASS.
7. **GUI `build.yml` — build/package matrix** (5 platforms) — fires on PR; a test-only + prose change does
   not affect the binary. **HOW:** PASS (build unaffected).
8. **GUI has NO fmt CI gate** (confirmed: only `build.yml` + `schema-mirror.yml`; grep for `cargo fmt`/
   `rustfmt` returns nothing). MEMORY: do NOT `cargo fmt` the GUI repo. No fmt gate to trip.
9. **CI-ONLY gates that do NOT fire (flagged to bound the blast radius):**
   - Toolkit `manual-gui.yml` `gui-schema-coverage` (the G1-B-revert class CI-only gate that clones the GUI
     at the pinned tag and demands FULL schema documentation) is PATH-FILTERED to toolkit `docs/manual-gui/**`
     + `manual-gui-v*` tags. It does NOT fire on a GUI-repo test/prose change and adds no GUI schema. NOT
     triggered. (It would only re-fire if a future toolkit pin bump in the toolkit's `manual-gui` pinned-upstream
     moved the GUI tag — a separate cycle.)
   - Toolkit `sibling-pin-check.yml` — scans toolkit workflow `--tag` lines vs install.sh canonical. We bump
     NO toolkit pin → does not fire. (Explicitly: do NOT opportunistically bump the toolkit's mk-cli pin to
     chase W3-3 — that would arm sibling-pin-check + manual-gui re-fires for zero benefit, since no toolkit
     gate reads help prose.)
   - Toolkit `manual.yml` flag-coverage lint — checks flag NAMES, not prose; the GUI repo has no docs/manual.
     N/A. No g6/mlock coupling (GUI has no mlock byte-anchor). No MSRV change.

---

## 7. FOLLOWUP flips (lands TOOLKIT-side, post-merge — NOT in this PR)

After the GUI PR merges, the orchestrator lands a 1-line edit in
`mnemonic-toolkit/design/FOLLOWUPS.md` (NOT in the GUI repo):

- Slug `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`, **Status line is L3458**
  (verified current; the slug header is L3452, Companion L3461). Flip the residual:
  - Current L3458: `Status: open — option (c) … shipped v0.34.3 … Residual = option (b): per-consumer --json
    wire-shape regression tests on the GUI side for high-traffic subcommands (xpub-search/import-wallet/
    export-wallet), v0.30+.`
  - New: mark option (b) **shipped** in `mnemonic-gui-v0.49.0` for `xpub-search` + `import-wallet` (live-capture
    golden snapshots keyed to the v0.70.0 toolkit pin, `tests/wire_shape_snapshot.rs`). Record that
    **export-wallet is descoped** (it has NO `--json` envelope — its wire-shape is per-`--format` file output;
    tracked as a follow-on, §8). If both covered surfaces are done and export-wallet is the only residual, flip
    Status to `resolved` with the export-wallet follow-on note, OR keep `open` narrowed to the export-wallet
    follow-on per the orchestrator's call. (The slug's Companion is `none`; no sibling FOLLOWUP to mirror.)

This is the only cross-repo touch. No toolkit code/CI change, no toolkit tag required for the flip (rides a
trivial doc commit toolkit-side).

---

## 8. Deferred / follow-on notes

1. **export-wallet wire-shape regression** — DESCOPED from this lane (no `--json` envelope). A follow-on
   could pin a representative subset of `export-wallet --format {bitcoin-core,specter,coldcard}` JSON FILE
   outputs (byte-exact goldens keyed to the toolkit pin), distinct from the envelope-shape pattern here.
   File/track as a new FOLLOWUP (or fold into the option-b residual note).
2. **W3-3 remaining sites (separate repo cycles, NOT this GUI lane):**
   - mk-cli source reword `crates/mk-cli/src/cmd/vectors.rs:22` + tag + crates.io publish (mk-cli PATCH;
     needed for the corrected `--help` to reach users).
   - toolkit `docs/manual/src/40-cli-reference/44-mk-cli.md:389` table row + `docs/manual-gui/src/70-mk/
     76-vectors.md:22-29,63` de-caveat + `docs/manual-gui/src/90-appendices/94-release-history.md:89-92` flip
     + the W3-3 slug `mnemonic-toolkit/design/FOLLOWUPS.md:2397-2405` Status flip + 2 missing companion
     FOLLOWUP entries (mk + gui). All doc-only, no toolkit bump, decoupled from the mk publish.
3. **Maintenance contract (document in SOURCE.md):** the live-capture goldens are valid ONLY at the v0.70.0
   pin. Every future toolkit pin bump in `Cargo.toml`/`pinned-upstream.toml` MUST refresh/re-verify these
   goldens in lockstep — this is the LEADING drift gate the slug wants (a feature, not a bug). The refresh step
   = re-run the 8 cells against the new binary; if a key-set changed, that's an intentional wire-shape
   evolution → update the assertion + bump the GUI in the same cycle.

---

## 9. Atomicity / ordering

Single GUI PR, single commit acceptable (no split-push hazard — the only CI-only gate that cares about atomic
pins, `sibling-pin-check.yml`, is toolkit-side and not touched here). Recommended in-PR order (TDD):
1. Add `tests/wire_shape_snapshot.rs` with cells that RED against current state (assert NEW v0.70.0 shape;
   they will fail only if run against an old binary — since CI installs v0.70.0 they pass; locally without a
   binary they skip). Capture goldens FROM the binary first.
2. Vendor the BSMS blob (if new) + add SOURCE.md row.
3. Delete (option A) the stale `cli_envelope_smoke.rs` + its fixtures, after grep-confirming no other consumer.
4. Apply the `src/schema/mk.rs` prose edits.
5. **Version-site bump (all in lockstep — §0, §4):** `Cargo.toml:3` `0.48.1` → `0.49.0`; `README.md:42` GUI
   self-tag `mnemonic-gui-v0.48.1` → `mnemonic-gui-v0.49.0` (CI-gated by `readme_pin_coherence.rs` — must
   move WITH Cargo.toml or the full suite RED-s); add a `CHANGELOG.md [0.49.0]` stanza at the top
   (release-completeness; not CI-gated). Do NOT touch the four sibling README pins (L50-53) or Cargo.toml:42
   toolkit pin.
6. Run the full local verification (§5) incl. `cargo test --test readme_pin_coherence` (catches a missed
   README bump without a binary) and the anti-stale negative check.
7. Open PR → orchestrator verifies CI (all build.yml platform + clippy + schema-mirror.yml jobs green,
   especially `cargo-test-full-suite` against the freshly-installed v0.70.0 binary — which also runs the
   binary-free `readme_pin_coherence` cell) → merge → tag `mnemonic-gui-v0.49.0`.
8. Post-merge (toolkit side, orchestrator): flip the W3-1 FOLLOWUP (§7).

The W3-3 prose edit and W3-1 test addition are file-disjoint and can land in the same commit; no ordering
dependency between them. The FOLLOWUP flip is strictly AFTER merge (status reflects shipped reality).
