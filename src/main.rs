//! mnemonic-gui — binary entry. Thin wrapper over `mnemonic_gui` library.
//!
//! Phase 4 wires tracing init + `--debug` flag. v0.1.1 wires the eframe
//! loop + a minimal interactive form (bundle subcommand as the canonical
//! demo). Tab-strip + per-subcommand wiring follows the Phase 6 R1 I-3
//! fold's data-layer foundation.

use clap::Parser;
use eframe::egui;
use tracing_subscriber::EnvFilter;

use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::form::invocation::{
    assemble_argv_with_secret_mask, render_copy_command, render_copy_command_masked, ShellFlavor,
};
use mnemonic_gui::form::slot_editor::{SlotState, SlotSubkey};
use mnemonic_gui::form::widget;
use mnemonic_gui::help::url as help_url;
use mnemonic_gui::path_detect::Detected;
use mnemonic_gui::persistence::{self, PersistedState};
use mnemonic_gui::runner::{self, PendingConfirm};
use mnemonic_gui::schema::{self, FlagValue, FormState};
use mnemonic_gui::secrets;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Cross-platform GUI overlay for the m-format constellation CLIs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug-level tracing output to stderr (default: WARN).
    /// `RUST_LOG=<filter>` env var overrides this when set.
    #[arg(long)]
    debug: bool,
}

fn main() -> eframe::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.debug);
    tracing::debug!(target: "mnemonic_gui::main", "tracing initialized");

    // v0.35.0 Phase-8 wiring (SPEC Decision 1): resolve the state path
    // ONCE here and load BEFORE run_native — window geometry can only be
    // applied via the ViewportBuilder (post-hoc ViewportCommand resize
    // flickers). The resolved path (not a re-resolution) travels into the
    // app so load and save can never diverge; `None` → never persist.
    let state_path = persistence::default_state_path();
    let loaded_state = state_path.as_deref().and_then(persistence::load);
    let window_size = loaded_state
        .as_ref()
        .and_then(|s| s.window_size)
        .unwrap_or([920.0, 720.0]);
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(window_size)
        .with_title("mnemonic-gui");
    // Wayland: outer position is compositor-private, so persisted
    // window_position stays None there and with_position is a no-op.
    if let Some(pos) = loaded_state.as_ref().and_then(|s| s.window_position) {
        viewport = viewport.with_position(pos);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "mnemonic-gui",
        native_options,
        Box::new(move |cc| Ok(Box::new(MnemonicGuiApp::new(cc, loaded_state, state_path)))),
    )
}

fn init_tracing(debug_flag: bool) {
    // Default filter suppresses noisy wgpu swap-chain warnings that
    // occur during idle 1 Hz keepalive repaints. RUST_LOG env-var
    // overrides everything.
    let default_filter = if debug_flag {
        "debug"
    } else {
        "warn,wgpu_hal=error,wgpu_core=error,egui_wgpu=error,naga=error"
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

// cycle-15 Lane G: `PendingConfirm` was promoted from a bare tuple here to a
// `Zeroize + Drop` struct in the `runner` lib module (so the app-level holder
// scrubs whole on drop AND the `secrets::scrub_app_run_holders` exit-sweep seam
// can name its type). Imported below.

/// Top-level egui app. Holds AppState (per-CLI detect results + active
/// tab), per-(cli, subcommand) FormState, and the captured last-run
/// stdout/stderr/argv. Output panel toggles per SPEC §B.10.
struct MnemonicGuiApp {
    app_state: AppState,
    /// Active subcommand per tab. Keys are CliTab; values are subcommand
    /// names from the per-tab schema.
    active_subcommand: BTreeMap<CliTab, String>,
    /// FormState per "cli:subcommand" key. Phase 8 persists this.
    form_state: BTreeMap<String, FormState>,
    last_run: Option<runner::RunResult>,
    last_run_error: Option<String>,
    show_cmdline: bool,
    show_stdout: bool,
    show_stderr: bool,
    /// Run-confirm modal state. `None` = no modal; `Some(PendingConfirm {
    /// argv, mask, stdin })` = pending. v0.32.0: carries the tree-mode
    /// `--spec -` stdin bytes alongside (build-descriptor has no secret flags
    /// so tree runs don't confirm TODAY, but the pending state must not
    /// silently drop the pipe if that ever changes). cycle-15 Lane G: the
    /// payload is the `Zeroize + Drop` `runner::PendingConfirm` struct (was a
    /// bare tuple) so the held cleartext argv/stdin scrubs on drop.
    pending_confirm_argv: Option<PendingConfirm>,
    /// v0.40.0 (Item 3) — set when a secret widget reported an over-threshold
    /// paste this frame (via the `secret_widget::paste_warn_id()` ctx-data
    /// bus); drives the informational paste-warn modal. Cleared on Dismiss.
    pending_paste_warn: bool,
    /// v0.6.0 P4 — last-observed `--template` value per form key, used by
    /// the per-frame template-aware-seed hook in `update()`. None means
    /// "no template was selected last frame" (either the form is new or
    /// the user cleared --template). On every frame, if the current
    /// template differs from this value, the hook seeds template-specific
    /// defaults for any Unset/absent flags via
    /// `form::conditional::template_defaults_for`. Seed-on-empty
    /// discipline preserves user-typed values across template switches —
    /// no auto-clear, no destructive mutation, no undo affordance needed.
    last_template: BTreeMap<String, Option<String>>,
    /// v0.35.0 Phase-8 wiring: the state.json path resolved ONCE in
    /// `main()` (SPEC Decision 1 — `on_exit` saves to THIS stored path;
    /// re-resolving could diverge if the env mutated mid-run). `None` →
    /// never save.
    state_path: Option<PathBuf>,
    /// v0.36.0 — autosave timer (seeded `now` at construction: first
    /// autosave only after a full interval) + the content-compare cache
    /// for `save_if_changed` (debounce-by-content; cache updates only
    /// after a successful write).
    last_autosave: std::time::Instant,
    last_saved_snapshot: Option<String>,
    /// v0.35.0: last-known window geometry, snapshotted per frame in
    /// `update()` (`on_exit` has no egui::Context). Some-GUARDED — while
    /// minimized egui-winit reports both viewport rects as `None`, so the
    /// snapshot only overwrites these on `Some`. Seeded from the loaded
    /// state so a session that never yields a `Some` (e.g. position on
    /// Wayland) re-persists the prior values instead of dropping them.
    window_size: Option<[f32; 2]>,
    window_position: Option<[f32; 2]>,
}

impl MnemonicGuiApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        loaded: Option<PersistedState>,
        state_path: Option<PathBuf>,
    ) -> Self {
        // v0.2 Phase B.2: OS-snapshot occlusion. macOS:
        // NSWindowSharingType::None; Windows: WDA_EXCLUDEFROMCAPTURE;
        // Linux: no-op (no compositor API at v0.2 — documented at
        // FOLLOWUPS `gui-os-snapshot-secret-occlusion`). Applied
        // here in `new()` so the protection is active for the entire
        // session, including the secret-bearing paste-warn modal.
        {
            use raw_window_handle::HasWindowHandle;
            if let Ok(handle) = cc.window_handle() {
                mnemonic_gui::platform::apply_window_capture_protection(handle);
            } else {
                tracing::warn!(
                    "OS-snapshot occlusion: cc.window_handle() failed; \
                     protection NOT applied (snapshots may leak)"
                );
            }
        }

        // Wayland compositor liveness keepalive. egui's reactive paint loop
        // only wakes `update()` on input events, so an idle window can go
        // many seconds between Wayland surface commits — long enough that
        // KDE/KWin flags the client "Not Responding" in the title bar.
        // The egui-documented pattern is to call `request_repaint()` from
        // ANOTHER THREAD (inline calls within `update()` are no-ops because
        // update is itself the response to an existing repaint request).
        // 1 Hz is plenty to satisfy KWin's multi-second threshold; idle
        // CPU stays near zero because each woken frame does no real GPU
        // work when state is unchanged.
        // Wayland compositor keepalive. egui's reactive paint loop
        // doesn't tick when idle, so KDE/KWin marks idle clients
        // "Not Responding" in the title bar. A 1 Hz request_repaint() from
        // a background thread keeps the surface healthy; egui's
        // reactive mode skips actual GPU work on unchanged frames, so
        // idle CPU stays at ~0%. Note: this only works with the wgpu
        // renderer; egui_glow on Wayland silently drops cross-thread
        // wake events (see FOLLOWUPS `gui-glow-wayland-loop-broken`).
        let ctx_keepalive = cc.egui_ctx.clone();
        std::thread::Builder::new()
            .name("wayland-keepalive".into())
            .spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                ctx_keepalive.request_repaint();
            })
            .expect("spawn wayland-keepalive thread");

        // Graceful Ctrl-C / SIGTERM handler. Routes through
        // ViewportCommand::Close so the eframe shutdown path runs
        // (on_exit zeroize sweep, window-close confirmation if any).
        // If the event loop is unresponsive, escalates to process::exit
        // after a 3 s grace.
        //
        // Unix (signal-hook): SIGINT + SIGTERM via the iterator API.
        // signal-hook's iterator is gated on cfg(not(windows)).
        //
        // Windows (ctrlc, v0.2 Phase A.2 / SPEC §5): Console CtrlC
        // handler. SIGTERM has no Windows equivalent — Ctrl-C only.
        // Both blocks share the same shape: clone egui::Context, send
        // ViewportCommand::Close, then process::exit(130) fallback.
        #[cfg(unix)]
        {
            let ctx_sig = cc.egui_ctx.clone();
            std::thread::Builder::new()
                .name("signal-handler".into())
                .spawn(move || {
                    let mut signals = signal_hook::iterator::Signals::new([
                        signal_hook::consts::SIGINT,
                        signal_hook::consts::SIGTERM,
                    ])
                    .expect("install signal-hook handlers");
                    // Single-shot: handler body always exits the process,
                    // so the `for ... forever()` loop never iterates more
                    // than once. `if let Some` is semantically identical
                    // and satisfies `clippy::never_loop`. v0.2 Phase B.1
                    // pickup of the v0.1.x pre-existing finding (CI for
                    // v0.1 did not run clippy --all-targets; v0.2 does).
                    if let Some(sig) = signals.forever().next() {
                        tracing::info!("received signal {sig}; requesting clean shutdown");
                        ctx_sig.send_viewport_cmd(egui::ViewportCommand::Close);
                        std::thread::sleep(std::time::Duration::from_secs(3));
                        tracing::warn!("clean shutdown timed out; exiting via process::exit");
                        std::process::exit(130);
                    }
                })
                .expect("spawn signal-handler thread");
        }

        #[cfg(windows)]
        {
            let ctx_ctrlc = cc.egui_ctx.clone();
            ctrlc::set_handler(move || {
                tracing::info!("Ctrl-C received (Windows); requesting clean shutdown");
                ctx_ctrlc.send_viewport_cmd(egui::ViewportCommand::Close);
                std::thread::sleep(std::time::Duration::from_secs(3));
                tracing::warn!("clean shutdown timed out (Windows); exiting via process::exit");
                std::process::exit(130);
            })
            .expect("install ctrlc handler (Windows)");
        }

        // v0.35.0 Phase-8 wiring (SPEC Decision 2): restore the persisted
        // selections / form state / toggles / geometry. The default (no
        // state file) path is unchanged: restore_selections on a default
        // PersistedState reproduces exactly the pre-wiring hardcoded
        // selections, and the demo seed below fires for the absent key.
        let mut app_state = AppState::detect_all();
        // The 3 output-pane toggles default to `true` on a fresh start;
        // `PersistedState::default()` has them `false` (derive(Default) —
        // the serde default_show_true only covers MISSING fields at
        // deserialize), so they are taken from `loaded` BEFORE the
        // unwrap_or_default below.
        let (show_cmdline, show_stdout, show_stderr) = match &loaded {
            Some(s) => (s.show_cmdline, s.show_stdout, s.show_stderr),
            None => (true, true, true),
        };
        let loaded = loaded.unwrap_or_default();
        let (active_tab, active_subcommand) =
            persistence::restore_selections(&loaded, |t| app_state.tab_available(t));
        app_state.active_tab = active_tab;
        // Direct field move — FormState is not Clone; the loaded state is
        // owned. Stale flag names in restored values stay INERT (render +
        // argv are schema-driven; no prune-on-load).
        let mut form_state = loaded.form_state_per_subcommand;

        // Seed the bundle form with reasonable defaults for the screenshot
        // demo (concrete enough to show realistic flag rendering).
        //
        // v0.35.0 demo-seed merge rule (SPEC R0-r1 M2): the restored map
        // wins for keys it contains — the seed applies ONLY when the
        // "mnemonic:bundle" key is absent (never re-seed over a
        // user-emptied form).
        //
        // v0.16.0 P5: REMOVED the unconditional `--multisig-path-family =
        // bip87` seed. With the default `--template = bip84` (single-sig),
        // the new SPEC §6.10.7 rule "single-sig template disables
        // --multisig-path-family" would render the flag widget Disabled
        // anyway. Pre-v0.16.0 the seeded value emitted into argv and
        // triggered the CLI's PATH_FAMILY_WITHOUT_MULTISIG byte-exact
        // rejection (the motivating bug for this cycle). A future cycle
        // (`gui-default-form-state-template-aware-seed`) may re-introduce
        // a template-aware seed that only sets multisig defaults when the
        // user picks a multisig template.
        if !form_state.contains_key("mnemonic:bundle") {
            form_state.insert(
                "mnemonic:bundle".into(),
                FormState::from_pairs(vec![
                    ("--network", FlagValue::Dropdown("mainnet".into())),
                    ("--template", FlagValue::Dropdown("bip84".into())),
                    ("--account", FlagValue::Number(0)),
                ])
                .with_slots(SlotState {
                    rows: vec![mnemonic_gui::form::slot_editor::SlotRow {
                        index: 0,
                        subkey: SlotSubkey::Xpub,
                        value: "".into(),
                    }],
                }),
            );
        }

        Self {
            app_state,
            active_subcommand,
            form_state,
            last_run: None,
            last_run_error: None,
            show_cmdline,
            show_stdout,
            show_stderr,
            pending_confirm_argv: None,
            pending_paste_warn: false,
            last_template: BTreeMap::new(),
            state_path,
            last_autosave: std::time::Instant::now(),
            last_saved_snapshot: None,
            window_size: loaded.window_size,
            window_position: loaded.window_position,
        }
    }

    fn schema_for(&self, tab: CliTab) -> &'static schema::Schema {
        match tab {
            CliTab::Mnemonic => &schema::mnemonic::SCHEMA,
            CliTab::Md => &schema::md::SCHEMA,
            CliTab::Ms => &schema::ms::SCHEMA,
            CliTab::Mk => &schema::mk::SCHEMA,
        }
    }

    /// v0.36.0 — the single save-side `PersistedState` construction, used
    /// by BOTH the autosave timer and `on_exit`. BORROW-SIDE by signature
    /// (v0.35.0 R0-r1 I2): never `mem::take` the form map — the exit
    /// zeroize sweep iterates `self.form_state` after this runs.
    fn build_persisted_state(&self) -> PersistedState {
        PersistedState {
            schema_version: persistence::SCHEMA_VERSION,
            last_cli_tab: self.app_state.active_tab.bin_name().to_string(),
            last_subcommand_per_tab: self
                .active_subcommand
                .iter()
                .map(|(tab, sub)| (tab.bin_name().to_string(), sub.clone()))
                .collect(),
            window_size: self.window_size,
            window_position: self.window_position,
            show_cmdline: self.show_cmdline,
            show_stdout: self.show_stdout,
            show_stderr: self.show_stderr,
            form_state_per_subcommand: self
                .form_state
                .iter()
                .map(|(k, v)| (k.clone(), persistence::redact_for_persistence(v)))
                .collect(),
        }
    }

    fn form_key(tab: CliTab, sub: &str) -> String {
        format!("{}:{}", tab.bin_name(), sub)
    }
}

/// v0.36.0 — autosave cadence (matches eframe's own auto_save default;
/// the 1 Hz keepalive guarantees the timer is evaluated at least once
/// per second even when idle).
const AUTOSAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

impl eframe::App for MnemonicGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Per-frame geometry snapshot (v0.35.0 Phase-8 wiring) ────────
        // on_exit has no egui::Context, so window geometry is snapshotted
        // here every frame (cheap). Some-GUARDED (SPEC R0-r1 I3): while
        // the window is MINIMIZED egui-winit sets both viewport rects to
        // None, and the 1 Hz keepalive thread keeps frames firing — an
        // unconditional snapshot would overwrite the last good geometry
        // with None (minimize → quit would lose it). Only overwrite on
        // Some. Wayland: outer position is compositor-private →
        // outer_rect stays None there and window_position keeps its
        // seeded/prior value.
        ctx.input(|i| {
            let viewport = i.viewport();
            if let Some(inner) = viewport.inner_rect {
                self.window_size = Some([inner.size().x, inner.size().y]);
            }
            if let Some(outer) = viewport.outer_rect {
                self.window_position = Some([outer.min.x, outer.min.y]);
            }
        });

        // ── v0.36.0 autosave (after the geometry snapshot so the write
        // carries current-frame geometry). Change-gated: save_if_changed
        // serializes once and skips when the redacted content is
        // byte-identical to the last successful write. Timer resets
        // REGARDLESS of write-vs-skip (bounds serialization to once per
        // interval). The final save stays unconditional in on_exit.
        if self.last_autosave.elapsed() >= AUTOSAVE_INTERVAL {
            if let Some(path) = &self.state_path {
                let persisted = self.build_persisted_state();
                if let Err(e) =
                    persistence::save_if_changed(&persisted, path, &mut self.last_saved_snapshot)
                {
                    tracing::warn!("autosave to {} failed: {e}", path.display());
                }
            }
            self.last_autosave = std::time::Instant::now();
        }

        // ── Top tab strip ────────────────────────────────────────────────
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("mnemonic-gui");
                ui.separator();
                for tab in CliTab::ALL {
                    let available = self.app_state.tab_available(*tab);
                    let label = if available {
                        tab.bin_name().to_string()
                    } else {
                        format!("{} (not installed)", tab.bin_name())
                    };
                    let mut btn = egui::Button::new(label);
                    if !available {
                        btn = btn.fill(egui::Color32::from_gray(64));
                    }
                    let resp = ui.add_enabled(available, btn);
                    if !available {
                        resp.clone().on_hover_text(
                            mnemonic_gui::app::missing_binary_tooltip(*tab),
                        );
                    }
                    if resp.clicked() {
                        self.app_state.active_tab = *tab;
                    }
                    if *tab == self.app_state.active_tab {
                        ui.label("◀");
                    }
                }
            });
        });

        // ── Output panel (bottom) ────────────────────────────────────────
        egui::TopBottomPanel::bottom("output").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_cmdline, "show command-line");
                ui.checkbox(&mut self.show_stdout, "show stdout");
                ui.checkbox(&mut self.show_stderr, "show stderr");
                // v0.10.0 B.3 (D33): the v0.9.0 R7 action-bar
                // `--no-auto-repair` checkbox has been DROPPED in lockstep
                // with toolkit v5 schema emission. `--no-auto-repair` is
                // now a first-class per-subcommand FlagSchema entry
                // (`global: true`) declared in `src/schema/mnemonic.rs`;
                // users wire it via the standard form widget per
                // subcommand. See `runner::prepend_no_auto_repair` (also
                // deleted) and the related v0.10.0 cycle close.
            });
            if let Some(ref err) = self.last_run_error {
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), format!("subprocess error: {}", err));
            }
            if let Some(ref result) = self.last_run {
                if self.show_cmdline {
                    // v0.39.0 (Item 1, D3): the last-run command-line is masked
                    // too — `result.mask` is the assembly-time mask carried
                    // through `spawn_and_capture`.
                    ui.label(format!(
                        "argv: {}",
                        render_copy_command_masked(
                            &result.argv,
                            &result.mask,
                            ShellFlavor::Posix
                        )
                    ));
                }
                render_exit_badge(ui, result.exit_code);
                if self.show_stdout && !result.stdout.is_empty() {
                    ui.label("stdout:");
                    egui::ScrollArea::vertical()
                        .id_salt("stdout")
                        .max_height(180.0)
                        .show(ui, |ui| {
                            ui.monospace(String::from_utf8_lossy(&result.stdout));
                        });
                }
                if self.show_stderr && !result.stderr.is_empty() {
                    ui.label("stderr:");
                    egui::ScrollArea::vertical()
                        .id_salt("stderr")
                        .max_height(120.0)
                        .show(ui, |ui| {
                            ui.monospace(String::from_utf8_lossy(&result.stderr));
                        });
                }
            } else {
                ui.label("(no run yet)");
            }
        });

        // ── Central form ────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            let active_tab = self.app_state.active_tab;
            let sch = self.schema_for(active_tab);

            ui.horizontal(|ui| {
                ui.label("Pinned:");
                ui.monospace(sch.pinned_version);
                ui.separator();
                let active_sub = self
                    .active_subcommand
                    .get(&active_tab)
                    .cloned()
                    .unwrap_or_default();
                egui::ComboBox::from_label("subcommand")
                    .selected_text(&active_sub)
                    .show_ui(ui, |ui| {
                        for sub in sch.subcommands {
                            if ui
                                .selectable_label(active_sub == sub.name, sub.human_name)
                                .clicked()
                            {
                                self.active_subcommand
                                    .insert(active_tab, sub.name.to_string());
                            }
                        }
                    });
                // manual-gui v1.0 (G-P2.2 / §2.4): per-subcommand `?`
                // help-icon — one of the three §1.6 Option C affordance
                // classes. Per §2.4 paragraph 2 + render-site contract,
                // this button lives at the subcommand-selector site
                // (NOT inside widget::render — that scope is per-flag
                // only). Click opens manual_url_for_subcommand in a new
                // tab.
                if !active_sub.is_empty() {
                    let help_btn = egui::Button::new("?")
                        .small()
                        .fill(egui::Color32::from_gray(96));
                    if ui.add(help_btn).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(
                            help_url::manual_url_for_subcommand(active_tab, &active_sub),
                        ));
                    }
                }
            });
            ui.separator();

            let active_sub_name = self
                .active_subcommand
                .get(&active_tab)
                .cloned()
                .unwrap_or_default();
            let sub = match sch
                .subcommands
                .iter()
                .find(|s| s.name == active_sub_name)
            {
                Some(s) => s,
                None => return,
            };

            // Compute conditional visibility once per frame.
            let key = Self::form_key(active_tab, &active_sub_name);
            // v0.6.0 P4: snapshot last-frame's --template BEFORE the
            // `form_state` mutable borrow, so the post-render hook can
            // compare without a second `self.last_template` borrow conflict.
            let last_template_for_key = self.last_template.get(&key).cloned().flatten();
            let state = self
                .form_state
                .entry(key.clone())
                .or_default();
            let vis = sub
                .conditional
                .map(|f| f(state))
                .unwrap_or_default();
            let visibility_of = |name: &str| -> mnemonic_gui::schema::Visibility {
                vis.iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(mnemonic_gui::schema::Visibility::Visible)
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                // v0.32.0 (node-tree SPEC §2.2): the three-way mode
                // selector at the top of the build-descriptor form
                // (Generic / spec file · Archetype · Tree builder). The
                // selector mutates state THIS frame, so the mode bits are
                // computed after it (the conditional `vis` above lags one
                // frame on a mode click; argv assembly below re-runs the
                // conditional post-click, so emission is always correct).
                if sub.name == "build-descriptor" {
                    mnemonic_gui::form::tree_form::render_mode_selector(ui, state);
                }
                let tree_mode = sub.name == "build-descriptor"
                    && mnemonic_gui::form::tree_form::tree_enabled(state);

                // v0.31.0 (archetype forms SPEC §3): when build-descriptor's
                // --archetype holds a non-empty ARCHETYPE_SPECS id, the 9 param
                // flags + --spec are skipped in the generic loop by NAME-SET (a
                // `continue` — NOT Visibility::Hidden, which suppresses argv;
                // declared params must emit) and the schema-driven archetype
                // param form renders directly under the --archetype dropdown
                // (the SlotEditor bespoke-surface precedent below). The
                // mode-independent flags keep their generic widgets.
                // v0.32.0: tree mode trumps — the archetype form never
                // renders in tree mode (the §0 mode-aware render dispatch).
                let archetype_spec = if sub.name == "build-descriptor" && !tree_mode {
                    mnemonic_gui::form::archetype_form::active_archetype(state)
                } else {
                    None
                };

                // Flag widgets.
                for flag in sub.flags {
                    if flag.name == "--slot" && sub.allows_slots {
                        continue; // SlotEditor handles below.
                    }
                    // v0.32.0 (node-tree SPEC §0): in tree mode neither
                    // the --spec row nor --archetype (nor the 10
                    // requires=archetype flags) renders — the tree form
                    // replaces them. Argv suppression is the conditional's
                    // (Disabled/Hidden); this name-set continue is the
                    // RENDER half of the mode mutex.
                    if tree_mode
                        && mnemonic_gui::form::tree_form::suppressed_in_tree_mode(
                            flag.name,
                        )
                    {
                        continue;
                    }
                    if archetype_spec.is_some()
                        && mnemonic_gui::form::archetype_form::suppressed_in_archetype_mode(
                            flag.name,
                        )
                    {
                        continue; // archetype param form renders these.
                    }
                    let v = visibility_of(flag.name);
                    if matches!(v, mnemonic_gui::schema::Visibility::Hidden) {
                        continue;
                    }
                    // v0.7.0: extract disabled_options orthogonally from the
                    // primary first-rule-wins visibility, so a flag can
                    // simultaneously be (e.g.) Required AND have invalid
                    // Dropdown options greyed out. The Dropdown widget
                    // consumes `disabled_options` directly; the primary
                    // Visibility decorates the label + gates add_enabled_ui.
                    let disabled_options: Vec<String> = vis
                        .iter()
                        .filter(|(k, _)| *k == flag.name)
                        .filter_map(|(_, v)| match v {
                            mnemonic_gui::schema::Visibility::DisableOptions {
                                values,
                            } => Some(values.clone()),
                            _ => None,
                        })
                        .flatten()
                        .collect();
                    // v0.2 Phase B.1: render_with_dispatch handles both
                    // secret (SecretLineEdit via state.secret_widgets) and
                    // non-secret (FlagValue via state.values) paths,
                    // centralizing the get-or-default + write-back dance
                    // and the secret/non-secret dispatch in one place.
                    ui.add_enabled_ui(
                        !matches!(v, mnemonic_gui::schema::Visibility::Disabled),
                        |ui| {
                            widget::render_with_dispatch(
                                ui,
                                active_tab,
                                &active_sub_name,
                                flag,
                                state,
                                &disabled_options,
                            );
                        },
                    );
                    // v0.31.0: the archetype param form (summary line +
                    // schema-ordered params) renders directly under the
                    // --archetype dropdown's generic widget (SPEC §3).
                    if flag.name == "--archetype" {
                        if let Some(spec) = archetype_spec {
                            mnemonic_gui::form::archetype_form::render(
                                ui,
                                active_tab,
                                &active_sub_name,
                                spec,
                                state,
                            );
                            // v0.32.0 P3 (node-tree SPEC §3): the
                            // archetype-mode "Edit as tree…" button —
                            // lowers the current params via --emit-spec
                            // and switches to tree mode; failure stays
                            // here with the error label. Lib seam:
                            // main.rs only dispatches.
                            mnemonic_gui::form::tree_form::render_edit_as_tree(
                                ui, state, "mnemonic",
                            );
                        }
                    }
                }
                // SlotEditor + per-`--slot` `?` help-icon (G-P2.2 /
                // §2.4): `--slot` is a repeating-field flag that bypasses
                // widget::render via the early `continue` above, so its
                // `?` button lives here next to the "Slot rows:" label.
                // URL composes via `manual_url_for_flag(tab, sub,
                // "--slot")` — same anchor scheme as other repeating
                // flags, just hosted at the SlotEditor surface.
                if sub.allows_slots {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Slot rows:");
                        let help_btn = egui::Button::new("?")
                            .small()
                            .fill(egui::Color32::from_gray(96));
                        if ui.add(help_btn).clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(
                                help_url::manual_url_for_flag(
                                    active_tab,
                                    &active_sub_name,
                                    "--slot",
                                ),
                            ));
                        }
                    });
                    // v0.8.1 F3 — non-canonical-descriptor default-path
                    // hint threading. Snapshot --descriptor / --network /
                    // --account as owned values so the subsequent
                    // mutable borrow of state.slots in slot_editor::render
                    // doesn't conflict (FormState's accessors return
                    // borrows tied to &state).
                    //
                    // v0.20.0 plan R0 I5 + Phase 3 R0 I2 fold: --account is
                    // a u32 in the toolkit. If the FormState carries a
                    // negative or out-of-u32-range i64 (a stale value, or
                    // a future schema drift), SUPPRESS the banner + hint
                    // entirely rather than silently coercing to 0 (which
                    // would make the banner declare an account-0 path
                    // while the form widget shows the user's typed value;
                    // confusing + dishonest). Suppression is the more
                    // truthful semantic — the CLI will reject the negative
                    // value at run time anyway.
                    let descriptor: String =
                        state.text_value("--descriptor").unwrap_or("").to_string();
                    let network: String = state
                        .dropdown_value("--network")
                        .unwrap_or("mainnet")
                        .to_string();
                    // Treat absence of --account as the implicit default
                    // 0 (matches toolkit's BundleArgs::account default).
                    // Out-of-u32-range values → None → banner/hint
                    // suppressed.
                    let account_opt: Option<u32> = match state.number_value("--account") {
                        None => Some(0),
                        Some(n) => u32::try_from(n).ok(),
                    };
                    let path_hint_string = match account_opt {
                        Some(account)
                            if !descriptor.is_empty()
                                && mnemonic_gui::form::conditional::classify_descriptor_canonicity(
                                    &descriptor,
                                ) == mnemonic_gui::form::conditional::Canonicity::NonCanonical =>
                        {
                            let coin =
                                mnemonic_gui::form::conditional::coin_type_for_network(
                                    &network,
                                );
                            Some(format!("m/48'/{coin}'/{account}'/2'"))
                        }
                        _ => None,
                    };
                    let path_hint = path_hint_string.as_deref();
                    mnemonic_gui::form::slot_editor::render(
                        ui,
                        &mut state.slots,
                        path_hint,
                    );
                    // v0.8.1 F3 — non-canonical descriptor banner. Closes
                    // the v0.19.0 I2 perceptibility gap; mirrors toolkit's
                    // bundle.rs::emit_default_path_notice with literal
                    // "@N" paraphrase (defaulted_indices computation
                    // deferred per v0.20.0 plan R1 I1 fold).
                    if let Some(banner) = account_opt.and_then(|account| {
                        mnemonic_gui::form::conditional::descriptor_non_canonical_default_path_notice(
                            &descriptor,
                            &network,
                            account,
                        )
                    }) {
                        ui.colored_label(
                            egui::Color32::from_rgb(96, 160, 220), // info-blue
                            banner,
                        );
                    }
                    // v0.7.2 SPEC §6.6 rows 10/11 — GUI-internal
                    // template/slot_count mismatch warning (Option A
                    // pattern, mirrors row-8 contiguity warning inside
                    // the slot grid). Renders inline orange banner when
                    // the chosen --template is incompatible with the
                    // current slot_count; suggests both directions of
                    // fix. CLI rows 10/11 remain the authoritative gate.
                    let template = state.dropdown_value("--template");
                    let slot_count = state.slot_count();
                    if let Some(warning) =
                        mnemonic_gui::form::conditional::template_slot_count_warning(
                            template,
                            slot_count,
                        )
                    {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 165, 0),
                            warning,
                        );
                    }
                }
                // Positional args. v0.34.0 (audit I5): SECRET positionals
                // render as per-row SecretLineEdits stored under the
                // "positional:<name>" reserved key (type-level
                // never-persist; the assembler reads the same key —
                // state.positionals is a dead path for them). Direct
                // SecretLineEdit::show — do NOT route through the
                // FlagSchema-coupled render_with_dispatch.
                for (i, pos) in sub.positional_args.iter().enumerate() {
                    if pos.secret {
                        let key = format!("positional:{}", pos.name);
                        let rows = state
                            .secret_widgets
                            .entry(key)
                            .or_insert_with(|| {
                                vec![mnemonic_gui::form::secret_widget::SecretLineEdit::new()]
                            });
                        let label = format!(
                            "{} {}{}",
                            pos.name,
                            if pos.required { "*" } else { "" },
                            if pos.repeating { "..." } else { "" }
                        );
                        let mut remove: Option<usize> = None;
                        let n = rows.len();
                        for (ri, row) in rows.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                row.show(ui, &label, pos.help);
                                if pos.repeating && n > 1 && ui.button("✕").clicked() {
                                    remove = Some(ri);
                                }
                            });
                        }
                        if let Some(ri) = remove {
                            let mut removed = rows.remove(ri);
                            removed.zeroize();
                        }
                        if pos.repeating && ui.button(format!("+ add {}", pos.name)).clicked() {
                            rows.push(
                                mnemonic_gui::form::secret_widget::SecretLineEdit::new(),
                            );
                        }
                        continue;
                    }
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} {}{}",
                            pos.name,
                            if pos.required { "*" } else { "" },
                            if pos.repeating { "..." } else { "" }
                        ));
                        while state.positionals.len() <= i {
                            state.positionals.push(String::new());
                        }
                        ui.text_edit_singleline(&mut state.positionals[i]);
                    });
                }
                // v0.32.0 (node-tree SPEC §2.2): the tree form renders
                // INSTEAD of the archetype form + the --spec row in tree
                // mode, after the mode-independent flag widgets. main.rs
                // is dispatch-only — the form logic lives in the library
                // (`form::tree_form`, the archetype_form lib seam).
                if tree_mode {
                    mnemonic_gui::form::tree_form::render(ui, state, "mnemonic");
                }
            });

            ui.separator();

            // v0.6.0 P4 — template-aware default seed. Fires on transitions
            // (current template != last-frame's template). For each
            // template-specific default returned by `template_defaults_for`,
            // seeds into state.values ONLY when the flag isn't already set
            // (seed-on-empty discipline — preserves any user-typed value
            // across template switches; never overwrites, never clears).
            // The visibility gate handles the opposite direction
            // (single-sig template → Disabled threshold/path-family); this
            // hook handles the "fresh form ergonomics" direction.
            let current_template = state.dropdown_value("--template").map(String::from);
            let template_changed = current_template != last_template_for_key;
            if template_changed {
                if let Some(new_template) = &current_template {
                    for (name, default_value) in
                        mnemonic_gui::form::conditional::template_defaults_for(new_template)
                    {
                        if !state.has_value(name) {
                            state.values.push((name.to_string(), default_value));
                        }
                    }
                }
            }

            // Snapshot argv + secret-status BEFORE the action bar so we can
            // drop the `state` mutable borrow ahead of any `self`-touching
            // callback (Run / pending_confirm_argv).
            let (mut argv, mut mask) = assemble_argv_with_secret_mask(sch, sub, state);
            // v0.32.0 (node-tree SPEC §2.2 Run leg): in tree mode the host
            // appends `--spec -` (the conditional suppressed any stale
            // --spec/--archetype/param emission) and pipes the generated
            // spec JSON via stdin. Copy-spec + Run are completeness-gated
            // (spec_stdin/spec_copy are None while the tree is incomplete
            // — R0-r1 M4).
            let tree_mode = sub.name == "build-descriptor"
                && mnemonic_gui::form::tree_form::tree_enabled(state);
            let spec_stdin = if tree_mode {
                mnemonic_gui::form::tree_form::spec_stdin_bytes(state)
            } else {
                None
            };
            let spec_copy = if tree_mode {
                mnemonic_gui::form::tree_form::spec_json_pretty(state)
            } else {
                None
            };
            if tree_mode {
                argv.push("--spec".to_string());
                mask.push(false);
                argv.push("-".to_string());
                mask.push(false); // both tree-mode tokens are non-secret
            }
            // v0.32.0 P3 (SPEC §3): in tree mode the POSIX copy emits a
            // paste-runnable printf pipeline (the spec JSON piped to
            // `--spec -`) instead of the bare argv (which would hang on
            // terminal stdin). Completeness-gated like Copy spec JSON;
            // the Windows copy stays argv + the separate JSON copy.
            let posix_pipeline = if tree_mode {
                mnemonic_gui::form::tree_form::posix_pipeline_command(state, &argv)
            } else {
                None
            };
            let needs_confirm = secrets::should_confirm_run(sub, state);
            // v0.39.0 (Item 1): the on-screen Preview is MASKED; the copy
            // buttons + Run still use the REAL command (the deliberate-reveal
            // half of decision (d)). `argv_posix` must NOT alias `preview`
            // anymore — it is the real clipboard payload.
            let any_secret = mask.iter().any(|&m| m);
            let preview = render_copy_command_masked(&argv, &mask, ShellFlavor::Posix);
            let argv_windows = render_copy_command(&argv, ShellFlavor::WindowsCmd);
            let argv_posix = render_copy_command(&argv, ShellFlavor::Posix);
            let _ = state; // explicit end-of-life for clarity
            // v0.6.0 P4 — update last_template AFTER state borrow ends.
            // `template_changed` was computed inside the state-borrow scope;
            // the inserted value is the post-render `current_template`.
            if template_changed {
                self.last_template.insert(key.clone(), current_template);
            }

            let mut copy_posix = false;
            let mut copy_windows = false;
            let mut copy_spec = false;
            let mut run_clicked = false;
            ui.horizontal(|ui| {
                // v0.32.0 P3: in tree mode the POSIX copy is the printf
                // pipeline — completeness-gated (same gate as Copy spec
                // JSON; the pipeline embeds the spec JSON).
                // v0.39.0: when the command carries a secret value, the copy
                // reveals it (the Preview is masked) — label the button so the
                // reveal is a deliberate, informed click.
                let posix_label = if any_secret {
                    "Copy command (POSIX) — reveals secret"
                } else {
                    "Copy command (POSIX)"
                };
                let windows_label = if any_secret {
                    "Copy command (Windows) — reveals secret"
                } else {
                    "Copy command (Windows)"
                };
                let posix_enabled = !tree_mode || posix_pipeline.is_some();
                if ui
                    .add_enabled(posix_enabled, egui::Button::new(posix_label))
                    .clicked()
                {
                    copy_posix = true;
                }
                if ui.button(windows_label).clicked() {
                    copy_windows = true;
                }
                // v0.32.0 (node-tree SPEC §2.2 / brainstorm R0-r1 M4 +
                // P3): the WINDOWS argv-Copy output ends in `--spec -`,
                // which pasted bare into a shell hangs on terminal stdin —
                // annotate, and offer the spec JSON as its own
                // completeness-gated copy. (The POSIX copy is the
                // self-contained printf pipeline since P3.)
                if tree_mode {
                    ui.weak("(Windows copy: spec via stdin — use Copy spec JSON)");
                    if ui
                        .add_enabled(
                            spec_copy.is_some(),
                            egui::Button::new("Copy spec JSON"),
                        )
                        .clicked()
                    {
                        copy_spec = true;
                    }
                }
                // Run is completeness-gated in tree mode (the same gate as
                // Validate/Copy-spec — SPEC §1.2).
                let run_enabled = !tree_mode || spec_stdin.is_some();
                if ui.add_enabled(run_enabled, egui::Button::new("Run")).clicked() {
                    run_clicked = true;
                }
            });
            ui.label(format!("Preview: {preview}"));

            if copy_posix {
                // v0.32.0 P3: tree mode copies the printf pipeline (Some
                // is guaranteed by the button gate; the argv fallback is
                // the non-tree path).
                ctx.copy_text(match posix_pipeline {
                    Some(pipeline) => pipeline,
                    None => argv_posix,
                });
            }
            if copy_windows {
                ctx.copy_text(argv_windows);
            }
            if copy_spec {
                if let Some(spec) = &spec_copy {
                    ctx.copy_text(spec.clone());
                }
            }
            if run_clicked {
                if needs_confirm {
                    self.pending_confirm_argv = Some(PendingConfirm {
                        argv,
                        mask,
                        stdin: spec_stdin,
                    });
                } else {
                    spawn_and_capture(self, argv, mask, spec_stdin);
                }
            }
        });

        // ── v0.40.0 (Item 3): paste-warn read-once ───────────────────────
        // The CentralPanel form rendered above; any secret widget that took an
        // over-threshold paste raised the ctx-data bus flag. `remove_temp`
        // reads AND clears it (so a stale flag can't leak into next frame).
        if ctx.data_mut(|d| {
            d.remove_temp::<bool>(mnemonic_gui::form::secret_widget::paste_warn_id())
                .unwrap_or(false)
        }) {
            self.pending_paste_warn = true;
        }

        // ── Run-confirm modal ────────────────────────────────────────────
        // cycle-15 Lane G (§3.2 E0509-safe consume): `PendingConfirm` now impls
        // `Drop`, so we CANNOT field-destructure the cloned value (E0509 —
        // "cannot move out of type which implements the Drop trait"). Bind the
        // WHOLE struct by a single name (legal for a `Drop` type); the modal
        // reads `pending.argv`/`pending.mask` BY-REF (unchanged below); the Run
        // path passes owned `.clone()`s into the owned `spawn_and_capture`. The
        // cloned `pending` drops at the end of THIS scope on every path (Run /
        // Cancel / modal-still-open) → its `Drop` scrubs the per-frame transient
        // — the residue this cycle closes. DO NOT delete the `Drop` impl to
        // "fix" a borrow error: that silently reverts to the un-scrubbed shape.
        if let Some(pending) = self.pending_confirm_argv.clone() {
            egui::Window::new("Confirm secret-bearing run")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(secrets::RUN_CONFIRM_MODAL_PREFIX);
                    ui.separator();
                    ui.label("Argv:");
                    // v0.39.0 (Item 1): the modal that exists BECAUSE a secret
                    // is present must not print it cleartext — mask each secret
                    // value token (the real argv still spawns on Run).
                    debug_assert_eq!(
                        pending.argv.len(),
                        pending.mask.len(),
                        "confirm-modal: mask/argv length mismatch — a secret token may render cleartext"
                    );
                    for (i, tok) in pending.argv.iter().enumerate() {
                        let shown = if pending.mask.get(i).copied().unwrap_or(false) {
                            mnemonic_gui::form::invocation::SECRET_MASK
                        } else {
                            tok.as_str()
                        };
                        ui.monospace(format!("  {}", shown));
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            self.pending_confirm_argv = None; // old struct drops → scrub
                            spawn_and_capture(
                                self,
                                pending.argv.clone(),
                                pending.mask.clone(),
                                pending.stdin.clone(),
                            );
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_confirm_argv = None; // old struct drops → scrub
                        }
                    });
                });
            // `pending` (a full PendingConfirm) drops here on every path → its
            // Drop scrubs the per-frame transient clone.
        }

        // ── v0.40.0 (Item 3): paste-warn modal ───────────────────────────
        // Informational + non-blocking (the value is already in the field);
        // warns about clipboard managers / paste history. One-shot per
        // trigger — re-fires on the next over-threshold paste.
        if self.pending_paste_warn {
            egui::Window::new("Secret paste warning")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(secrets::PASTE_WARN_MODAL_TEXT);
                    ui.separator();
                    if ui.button("Dismiss").clicked() {
                        self.pending_paste_warn = false;
                    }
                });
        }
    }

    fn on_exit(&mut self) {
        tracing::info!("on_exit() called — clean shutdown via wayland close event");
        // ── v0.35.0 Phase-8 wiring: save state.json ──────────────────────
        // ORDER IS LOAD-BEARING (SPEC Decision 4 / R0-r1 I1): save FIRST,
        // then the zeroize sweep below — `zeroize_form_state` blanks every
        // STRING-BEARING value (Text/Dropdown/Path/composite + all slot
        // rows + positionals; Number/Bool/Unset untouched), not just
        // secrets, so sweeping first would persist a gutted form.
        //
        // Construction is BORROW-SIDE (R0-r1 I2 — do NOT mem::take):
        // `redact_for_persistence` returns owned without Clone and
        // `save()`'s internal re-redaction is idempotent, so
        // `self.form_state` stays intact for the sweep. A mem::take would
        // silently turn the sweep into a no-op (secrets never zeroized at
        // exit).
        if let Some(path) = &self.state_path {
            let persisted = self.build_persisted_state();
            if let Err(e) = persistence::save(&persisted, path) {
                tracing::warn!("failed to save state.json to {}: {e}", path.display());
            }
        }
        // SPEC §9: best-effort zeroize sweep on close. Runs AFTER the save
        // above — see the load-bearing order comment there.
        for state in self.form_state.values_mut() {
            secrets::zeroize_form_state(state);
        }
        // cycle-15 Lane G: app-level run holders sit OUTSIDE form_state, so the
        // sweep above never reaches them. Scrub them here for early RAM
        // overwrite at shutdown (matching SecretBuffer's drop-ahead rationale).
        // These holders are NOT serialized (no Serialize on RunResult/
        // PendingConfirm) → order vs the state.json save is moot.
        secrets::scrub_app_run_holders(&mut self.last_run, &mut self.pending_confirm_argv);
    }
}

/// Render the subprocess exit-code line in the output panel.
///
/// v0.9.0 (D26 cross-CLI parity): exit 5 means the toolkit auto-fired BCH
/// repair and the corrected card is on stdout (`ToolkitError::
/// RepairShortCircuit { exit_code: 5 }`). Render it as a green badge so
/// users see the success at a glance; all other exit codes (including 0)
/// render in the default label colour.
///
/// Cases:
/// - `Some(0)` → plain `exit: 0` label.
/// - `Some(5)` → green badge with explanatory message.
/// - `Some(n)` for `n != 0 && n != 5` → plain `exit: <n>` label
///   (subprocess error path; stderr already renders below).
/// - `None` → plain `exit: (killed)` label (signal / no exit code).
fn render_exit_badge(ui: &mut egui::Ui, exit_code: Option<i32>) {
    match exit_code {
        Some(5) => {
            // Saturated green chosen to match the project's existing colour
            // palette (warning amber at form/slot_editor.rs:233 uses (220,
            // 165, 0); error red at main.rs:304 uses (220, 60, 60)). Green
            // (60, 180, 75) sits in the same chroma band and stays readable
            // on egui's default dark/light themes.
            ui.colored_label(
                egui::Color32::from_rgb(60, 180, 75),
                "exit: 5 — Repair Applied (BCH auto-fire succeeded; \
                 corrected chunk on stdout)",
            );
        }
        Some(n) => {
            ui.label(format!("exit: {}", n));
        }
        None => {
            ui.label("exit: (killed)");
        }
    }
}

/// v0.32.0: `stdin` carries the tree-mode spec JSON for `--spec -` runs
/// (`None` for every other subcommand/mode — byte-identical behavior via
/// `run_with_stdin`'s `None` delegation path).
fn spawn_and_capture(
    app: &mut MnemonicGuiApp,
    argv: Vec<String>,
    mask: Vec<bool>,
    stdin: Option<Vec<u8>>,
) {
    if argv.is_empty() {
        return;
    }
    // SPEC §B.8 class 1: detect-missing-binary path — surface a friendly
    // error rather than crashing.
    let bin = &argv[0];
    if !matches!(
        mnemonic_gui::path_detect::detect(bin),
        Detected::Found(_)
    ) {
        app.last_run = None;
        app.last_run_error = Some(format!(
            "`{}` not found on $PATH",
            bin
        ));
        return;
    }
    // v0.10.0 B.3 (D33): `--no-auto-repair` is now a first-class
    // FlagSchema entry per subcommand (toolkit v5 `global: true`); the
    // form's argv assembly handles it like any other Boolean flag. The
    // prior `runner::prepend_no_auto_repair` helper has been deleted.
    match runner::run_with_stdin(argv, stdin) {
        Ok(mut result) => {
            // v0.39.0 (R0-r2 I1): the runner layer is mask-oblivious; attach
            // the assembly-time display mask here, before storing.
            result.mask = mask;
            app.last_run = Some(result);
            app.last_run_error = None;
        }
        Err(e) => {
            app.last_run = None;
            app.last_run_error = Some(e.to_string());
        }
    }
}

// `default_value_for_flag` migrated to `widget::default_flag_value_for`
// in v0.2 Phase B.1 (centralized for use by `render_with_dispatch`).
