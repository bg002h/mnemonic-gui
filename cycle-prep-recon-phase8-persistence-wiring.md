# cycle-prep recon — Phase-8 persistence wiring (state.json load/save)

- **Repo:** mnemonic-gui, branch `master` @ `1a1615abf8880691867134cd7aae35825c28b21e` (= tag `mnemonic-gui-v0.34.0`), in sync with `origin/master`.
- **Design source:** §10 + Phase 8 of `/home/bcg/.claude/plans/declarative-tumbling-shell.md` (converged plan; R1 I-4 / R2 I-2 / R3 I-1/I-3 folds).
- **Scope:** recon only. Maps the gap between the converged Phase-8 persistence design and what exists at v0.34.0, for a wiring spec.
- **Headline:** the persistence MODULE is feature-complete (every §10 field already exists in `PersistedState`; redaction hardened v0.31.1→v0.34.0; 19 tests). The gap is 100% WIRING: `persistence::{save, load, default_state_path}` have **zero callers in src/** — `MnemonicGuiApp::new(cc)` never loads, no save hook exists, and window geometry is never captured. eframe's own persistence feature is OFF (deliberately consistent with §10's hand-rolled design).

---

## 1. What `src/persistence.rs` already provides (213 lines — complete vs §10)

Signatures (all `pub`, module exported at `src/lib.rs:11`):

| API | Where | Notes |
|---|---|---|
| `pub const SCHEMA_VERSION: u32 = 1` | `src/persistence.rs:34` | bump on incompatible on-disk shape change |
| `pub struct PersistedState` | `src/persistence.rs:44-59` | `Default + Debug + Serialize + Deserialize`; **NOT Clone** (FormState dropped Clone when SecretLineEdit arrived — :38-43) |
| `pub fn redact_for_persistence(&FormState) -> FormState` | `src/persistence.rs:74-144` | 4 drop classes (below) |
| `pub fn redact_persisted_state(&PersistedState) -> PersistedState` | `src/persistence.rs:149-166` | maps redaction over every form entry |
| `pub fn save(&PersistedState, &Path) -> io::Result<()>` | `src/persistence.rs:176-187` | creates parent dirs; **redacts internally** + **stamps `SCHEMA_VERSION` unconditionally** (R1 I-1 fold, :171-183 — callers cannot write a stale version) |
| `pub fn load(&Path) -> Option<PersistedState>` | `src/persistence.rs:193-205` | version mismatch → rename to `.bak` + `None`; missing file / malformed JSON → `None` (no `.bak`) |
| `pub fn default_state_path() -> Option<PathBuf>` | `src/persistence.rs:210-213` | exactly §10: `directories::ProjectDirs::from("org","mnemonic-gui","mnemonic-gui").config_dir()/state.json`; `None` → "don't persist" |

**PersistedState fields vs §10's list — NONE MISSING:**

| §10 field | PersistedState | Line |
|---|---|---|
| `last_cli_tab` | `String` | :47 |
| `last_subcommand_per_tab` | `BTreeMap<String, String>` | :48 |
| `window_size` / `window_position` | `Option<[f32; 2]>` each | :49-50 |
| `show_cmdline` / `show_stdout` / `show_stderr` | `bool`, `#[serde(default = "default_show_true")]` | :51-56 |
| per-subcommand non-secret form values + non-secret slot rows | `form_state_per_subcommand: BTreeMap<String, FormState>`, key `"<cli>:<subcommand>"` | :57-58 |

**Redaction (`redact_for_persistence`, :74-144) — the never-persist enforcement, ALREADY built:**
1. flag names in `SECRET_FLAG_NAMES` (:81);
2. any flag name `secret: true` anywhere across the 4 schemas — `secrets::schema_secret_flag_names()` name-level net, v0.31.1 (:90);
3. `NodeValueComposite` with `node ∈ SECRET_NODE_TYPES` (:94-97);
4. slot rows with `subkey ∈ SECRET_SLOT_SUBKEYS` (:104-110).
Plus: `positionals: Vec::new()` unconditional drop-ALL belt, v0.34.0 audit I5 (:117-123); `secret_widgets` fresh-empty (type-level `#[serde(skip)]`, :124-128); `tree` mapped through `TreeState::redacted_for_persistence` (:135-138); `edit_as_tree_error: None` (:139-142). The `SECRET_*` constants are re-exported from `mnemonic_toolkit::secret_taxonomy` (`src/secrets.rs:34`) with a frozen-snapshot supply-chain drift assert (`src/secrets.rs:38-94`).

**SCHEMA_VERSION semantics:** `save` stamps `SCHEMA_VERSION` regardless of input (:182-183); `load` compares and on mismatch does `fs::rename(path, path.with_extension("json.bak"))` then returns `None` (:199-203) — caller writes a fresh default. Matches §10 exactly. NOTE the asymmetry: **malformed JSON returns `None` WITHOUT a `.bak` rename** (:195-198) — the corrupt file sits until the next save overwrites it (decision point §6.4).

**Test coverage:**
- `tests/persistence.rs` (428 lines, 12 cells): round-trip (cell_1, cell_7), never-persist audit reading SECRET_* dynamically (cell_2, cell_8), watch-only survival (cell_3), version-mismatch → `.bak` (cell_4), missing/malformed → None (cell_5, cell_6), redaction idempotence (cell_9), parent-dir creation (cell_10), version-stamp (cell_11), `secret_widgets` both-directions (line 376).
- `tests/persist_redaction_v0_34_0.rs` (7 cells, t1-t5): positional drop/route/census + tree extended-public allowlist.
- Plus `tests/tree_round_trip.rs` redaction cell drives the v0.34.0 walk.

## 2. App lifecycle today (`src/main.rs`)

- **`MnemonicGuiApp::new(cc)` (main.rs:102-250):** uses `cc` only for `window_handle()` (capture protection, :109-119) and `egui_ctx` clones (keepalive thread :139-146, signal handlers :161-200). **Ignores `cc.storage`** (which is `None` anyway — §4). Hardcoded defaults: `active_subcommand` seeded `{Mnemonic→bundle, Md/Ms/Mk→inspect}` (:202-206); demo `mnemonic:bundle` FormState seed (:221-236); `show_cmdline/show_stdout/show_stderr = true` (:244-246).
- **App struct (main.rs:71-99):** `app_state: AppState` (holds `active_tab: CliTab` — the `last_cli_tab` analog; `src/app.rs:51-57`), `active_subcommand: BTreeMap<CliTab, String>` (the `last_subcommand_per_tab` analog — keyed by `CliTab`, NOT `String`), `form_state: BTreeMap<String, FormState>` (key `"cli:sub"` via `form_key`, main.rs:261-263 — **byte-identical shape to `PersistedState.form_state_per_subcommand`**), the 3 output-pane toggles (:80-82 — they DO exist, as runtime-only fields wired to checkboxes at :303-305), plus transients (`last_run`, `last_run_error`, `pending_confirm_argv`, `last_template`).
- **`run_native` (main.rs:38-48):** fixed `ViewportBuilder::with_inner_size([920.0, 720.0])`; no `with_position`; no `persistence_path` / `persist_window` (inert without the feature anyway).
- **Hooks:** `eframe::App::update` (:267) + `fn on_exit(&mut self)` (:900-906 — the wgpu-backend no-arg signature; runs the `secrets::zeroize_form_state` sweep). **No `save()` override, no `auto_save_interval`, no `persist_egui_memory`.** `on_exit` has NO `egui::Context` access → window geometry cannot be read there; it must be snapshotted per-frame in `update()` (via `ctx.input(|i| i.viewport().inner_rect/outer_rect)`).
- **Window geometry:** lives nowhere in app state today. eframe owns it; the only handle is the per-frame `ViewportInfo`.
- **`CliTab` (src/app.rs:17-34):** has `bin_name()` but **no inverse parse** (`from_bin_name`) — restore of `last_cli_tab: String` needs one (small lib addition).

## 3. Form-state shape

- **Per-(cli, subcommand)**, not global: `form_state: BTreeMap<String, FormState>` keyed `"<cli>:<subcommand>"` (main.rs:77, :261-263), lazily `or_default()`ed per frame (:416-419). Round-trip is therefore a **direct field move** — `PersistedState.form_state_per_subcommand` has the identical key scheme (persistence.rs:57-58). No re-shaping needed.
- **FormState serde posture (src/schema/mod.rs:291-343):** `values: Vec<(String, FlagValue)>` + `slots` serialize plainly; `positionals` serializes but is force-emptied at persist; `secret_widgets` is `#[serde(skip)]` (:320-322) — **deserialization default-constructs it empty, so secrets NEVER round-trip, by type**; `tree: Option<TreeState>` is `#[serde(default)]` (:332 — the missing-field migration leg); `edit_as_tree_error` `#[serde(skip)]` (:341).
- **v0.31.1+ implication for restore:** secrets live ONLY in `secret_widgets` (incl. `"positional:<name>"` rows, v0.34.0) → a restored session comes back with all secret fields blank and all watch-only fields populated. No migration story needed; `FormState::clone` doesn't exist, so restore is by-value move out of the loaded `PersistedState` (fine — load produces owned data).
- **Restore quirk (benign, spec should note):** `last_template` (main.rs:98) is not persisted; on first frame after restore the template-aware seed hook (:740-752) sees `template_changed == true` and seeds defaults — but only into UNSET flags (seed-on-empty), so restored values are never overwritten.

## 4. eframe specifics

- **Version:** `eframe = { version = "0.31", default-features = false, features = ["wgpu", "default_fonts", "wayland", "x11", "accesskit"] }` (Cargo.toml:13); lockfile resolves **eframe v0.31.1**. `egui`/`egui_kittest` 0.31. `directories = "5"` (Cargo.toml:17) already a direct dep.
- **eframe's built-in persistence is OFF:** the `persistence` feature is not in the list; `cargo tree -i eframe -e features` shows exactly the 5 features above; **`ron` is absent from Cargo.lock** (grep count 0). Consequences: `cc.storage == None`, `App::save` would never fire, `NativeOptions::persist_window`/`persistence_path` are inert. This is CONSISTENT with §10 (hand-rolled `state.json`); enabling the feature would add ron + a second parallel state file (egui memory) — recommend staying hand-rolled (§6.3).
- **Idiomatic hooks if we DID enable it:** `App::save(&mut self, &mut dyn Storage)` + `auto_save_interval()` (default 30 s) + `persist_egui_memory()`. With hand-rolled state these are unavailable as triggers (no storage) — the save trigger must be `on_exit` (+ optional own debounce in `update()`).

## 5. Tree state — confirmed inside the redact path

`redact_for_persistence` maps `state.tree` through `TreeState::redacted_for_persistence` (persistence.rs:135-138 → `src/form/tree_model.rs:176`), which since v0.34.0 (audit I6) drives the positive allowlist `blank_non_extended_public_keys` (tree_model.rs:695 — keep ONLY origin-stripped SLIP-132 extended-public matches). Transients `diagnostics`/`validate_ok` are `#[serde(skip)]` inside TreeState (tree_model.rs:53-64). Tested: `persist_redaction_v0_34_0.rs::t5` (14-row table) + the `tree_round_trip` redaction cell (surplus-children leg). Nothing to add for wiring.

## 6. Risks / decisions the spec must settle

1. **Load-at-startup ordering vs window geometry.** `window_size`/`window_position` must be applied to `ViewportBuilder` BEFORE `run_native` — so `load()` must happen in `main()` (main.rs:33-49), with the loaded `PersistedState` passed into `MnemonicGuiApp::new` (closure capture). Loading inside `new(cc)` is too late for initial geometry (would need `ViewportCommand::InnerSize` post-hoc — flicker).
2. **Geometry capture path.** `on_exit` has no ctx → snapshot `ctx.input(|i| i.viewport().inner_rect / outer_rect)` into app fields every frame (cheap), save from those at exit. **Wayland caveat:** outer position is not exposed (compositor-private) → `window_position` will be `None` on Wayland and `with_position` is a no-op there; fields are already `Option` — document, don't fight it.
3. **eframe-native vs hand-rolled geometry: recommend hand-rolled** (§10 as designed; keeps one state file, keeps the redaction audit the single chokepoint, avoids adding the `persistence`/ron dep surface).
4. **Stale-state robustness on restore (pinned schema changed):**
   - *Stale flag names in `values`:* INERT today — both render (main.rs:462 iterates `sub.flags`) and argv (`invocation.rs:126-162` iterates `subcommand.flags`) are schema-driven; entries for renamed/removed flags are never read. There is **no prune-on-load**; decide prune (hygiene, smaller state.json) vs leave-inert (zero-risk; recommend leave-inert + note). Caveat: a restored now-`DisableOptions` dropdown value WILL emit (invocation.rs:152-156 documented residual; CLI gates) — pre-existing, not new.
   - *Stale `last_subcommand_per_tab` entry:* an unknown subcommand renders an empty central panel (main.rs:401-408 `None => return`) — validate on restore against `schema_for(tab).subcommands`, else fall back to the current hardcoded defaults.
   - *Stale `last_cli_tab`:* parse via the new `CliTab::from_bin_name`; unknown → `CliTab::Mnemonic`. Consider also `tab_available()` (restored tab's binary may have been uninstalled) — recommend restore-anyway (tabs render disabled-but-selected today is impossible since clicks are gated; simplest: restore only if available, else default).
   - *Unknown `FlagValue` tag:* whole-file parse failure → `None` (state silently discarded, no `.bak`); per-value unknown tags degrade to `Unset` per the `#[serde(other)]` fallback (FOLLOWUPS `gui-flag-value-unset-serde-other-externally-tagged-dependency`, FOLLOWUPS.md:538 area — open, not a blocker). Decision: also `.bak` on malformed parse (symmetric with version mismatch) so a corrupt file is preserved for diagnosis rather than overwritten. Cheap; recommend yes.
5. **Save cadence.** `on_exit`-only loses state on crash/SIGKILL (the SIGINT/SIGTERM handlers DO route through `ViewportCommand::Close` → `on_exit` runs, main.rs:161-200 — clean). Recommend `on_exit`-only for the wiring cycle (debounced autosave = follow-up); order vs zeroize sweep: save FIRST, then zeroize (irrelevant for correctness — `save` redacts and `secret_widgets` never serialize — but state the order).
6. **Test seam.** `default_state_path()` is hardwired to ProjectDirs; integration tests must not touch the real config dir. The wiring needs a path-injection seam (env var `MNEMONIC_GUI_STATE_PATH` override inside `default_state_path`, or a `state_path: Option<PathBuf>` field on the app) — spec must pick one (env var is the smallest and testable from `tests/`).
7. **Migration non-issue confirmed:** save/load have never had callers (`FOLLOWUPS.md:26`; persistence.rs:118-123 comment "no state.json exists in the wild — save/load have never had callers") → no in-the-wild state.json, SCHEMA_VERSION stays 1, no migration code.
8. **Output-pane toggles:** exist (main.rs:80-82, checkboxes :303-305) — pure field-mapping, no new UI.

## 7. FOLLOWUPS + audit residuals — nothing blocks

- **`persistence-unwired-redaction-never-runs` [obs]** (FOLLOWUPS.md:26) — the target entry: "save, load, redact_persisted_state, default_state_path have no callers in src/; new(cc) ignores cc.storage; no save() override". This cycle resolves it.
- **Blockers explicitly CLEARED:** `positional-secrets-not-redacted-at-persist` (I5, resolved v0.34.0, FOLLOWUPS.md:96) and `tree-wif-hex-privkey-in-key-fields-unredacted` (I6, resolved v0.34.0, FOLLOWUPS.md:99-104) both state "**Phase-8 persistence wiring is now UNBLOCKED on this count**"; I4 secret flips cleared in v0.33.0.
- **Remaining persist-adjacent minors — none block wiring:**
  - `slot-secret-values-rendered-unmasked` [minor] (FOLLOWUPS.md:24) — RENDER-side only; secret slot VALUES already never persist (redaction class 4, persistence.rs:104-110).
  - `run-confirm-and-preview-show-secrets-cleartext` [obs] (FOLLOWUPS.md:27) — display-side; orthogonal to on-disk.
  - `gui-flag-value-unset-serde-other-externally-tagged-dependency` — restore-robustness hardening; wiring makes it LIVE-relevant for the first time (worth a spec cross-cite, not a gate).
  - `paste-warn-*` minors, `gui-os-snapshot-secret-occlusion` — unrelated surfaces.

## Recommended spec scope

- **Scope:** wiring-only cycle. (a) `CliTab::from_bin_name` + restore-validation helpers (lib, unit-testable); (b) `main()` loads via `default_state_path`+`load` before `run_native`, seeds `ViewportBuilder` size/position, passes state into `new()`; (c) `new()` maps the 6 field groups onto the app (tab w/ availability fallback, per-tab subcommand w/ schema validation, form_state direct move, 3 toggles); (d) per-frame geometry snapshot in `update()`; (e) `on_exit` saves (before zeroize) via `save()`; (f) `.bak`-on-malformed symmetry (tiny `load` change + test); (g) `MNEMONIC_GUI_STATE_PATH` test seam. NO redaction changes, NO schema changes, NO new deps.
- **SemVer:** **MINOR** (v0.35.0) — new user-visible behavior (session restore + a state file on disk), zero CLI-schema delta → no `schema_mirror`/pin impact; no toolkit companion needed.
- **Phases:** P1 lib helpers + load/restore mapping (TDD: restore-validation cells, stale-tab/stale-subcommand fallbacks, `.bak`-on-malformed); P2 main.rs wiring + geometry snapshot + on_exit save (integration round-trip via the env-var seam; kittest optional for toggle restore); P3 docs (README/CHANGELOG; note the Wayland position caveat + "delete state.json to reset") + flip `persistence-unwired-redaction-never-runs` to resolved. R0 gate per CLAUDE.md before any code.
