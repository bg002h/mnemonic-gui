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
use mnemonic_gui::form::invocation::{assemble_argv, render_copy_command, ShellFlavor};
use mnemonic_gui::form::slot_editor::{SlotState, SlotSubkey};
use mnemonic_gui::form::widget;
use mnemonic_gui::path_detect::Detected;
use mnemonic_gui::runner;
use mnemonic_gui::schema::{self, FlagValue, FormState};
use mnemonic_gui::secrets;
use std::collections::BTreeMap;

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

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 720.0])
            .with_title("mnemonic-gui"),
        ..Default::default()
    };
    eframe::run_native(
        "mnemonic-gui",
        native_options,
        Box::new(|cc| Ok(Box::new(MnemonicGuiApp::new(cc)))),
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
    /// Run-confirm modal state. None = no modal; Some(argv) = pending.
    pending_confirm_argv: Option<Vec<String>>,
}

impl MnemonicGuiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        let mut active_subcommand = BTreeMap::new();
        active_subcommand.insert(CliTab::Mnemonic, "bundle".to_string());
        active_subcommand.insert(CliTab::Md, "inspect".to_string());
        active_subcommand.insert(CliTab::Ms, "inspect".to_string());
        active_subcommand.insert(CliTab::Mk, "inspect".to_string());

        // Seed the bundle form with reasonable defaults for the screenshot
        // demo (concrete enough to show realistic flag rendering).
        let mut form_state = BTreeMap::new();
        form_state.insert(
            "mnemonic:bundle".into(),
            FormState::from_pairs(vec![
                ("--network", FlagValue::Dropdown("mainnet".into())),
                ("--template", FlagValue::Dropdown("bip84".into())),
                ("--account", FlagValue::Number(0)),
                ("--multisig-path-family", FlagValue::Dropdown("bip87".into())),
            ])
            .with_slots(SlotState {
                rows: vec![mnemonic_gui::form::slot_editor::SlotRow {
                    index: 0,
                    subkey: SlotSubkey::Xpub,
                    value: "".into(),
                }],
            }),
        );

        Self {
            app_state: AppState::detect_all(),
            active_subcommand,
            form_state,
            last_run: None,
            last_run_error: None,
            show_cmdline: true,
            show_stdout: true,
            show_stderr: true,
            pending_confirm_argv: None,
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

    fn form_key(tab: CliTab, sub: &str) -> String {
        format!("{}:{}", tab.bin_name(), sub)
    }
}

impl eframe::App for MnemonicGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            });
            if let Some(ref err) = self.last_run_error {
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), format!("subprocess error: {}", err));
            }
            if let Some(ref result) = self.last_run {
                if self.show_cmdline {
                    ui.label(format!(
                        "argv: {}",
                        render_copy_command(&result.argv, ShellFlavor::Posix)
                    ));
                }
                ui.label(format!(
                    "exit: {}",
                    result.exit_code.map(|n| n.to_string()).unwrap_or_else(|| "(killed)".into())
                ));
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
                    .map(|(_, v)| *v)
                    .unwrap_or(mnemonic_gui::schema::Visibility::Visible)
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Flag widgets.
                for flag in sub.flags {
                    if flag.name == "--slot" && sub.allows_slots {
                        continue; // SlotEditor handles below.
                    }
                    let v = visibility_of(flag.name);
                    if matches!(v, mnemonic_gui::schema::Visibility::Hidden) {
                        continue;
                    }
                    // v0.2 Phase B.1: render_with_dispatch handles both
                    // secret (SecretLineEdit via state.secret_widgets) and
                    // non-secret (FlagValue via state.values) paths,
                    // centralizing the get-or-default + write-back dance
                    // and the secret/non-secret dispatch in one place.
                    ui.add_enabled_ui(
                        !matches!(v, mnemonic_gui::schema::Visibility::Disabled),
                        |ui| {
                            widget::render_with_dispatch(ui, flag, state);
                        },
                    );
                }
                // SlotEditor.
                if sub.allows_slots {
                    ui.separator();
                    ui.label("Slot rows:");
                    mnemonic_gui::form::slot_editor::render(ui, &mut state.slots);
                }
                // Positional args.
                for (i, pos) in sub.positional_args.iter().enumerate() {
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
            });

            ui.separator();

            // Snapshot argv + secret-status BEFORE the action bar so we can
            // drop the `state` mutable borrow ahead of any `self`-touching
            // callback (Run / pending_confirm_argv).
            let argv = assemble_argv(sch, sub, state);
            let needs_confirm = secrets::should_confirm_run(sub, state);
            let preview = render_copy_command(&argv, ShellFlavor::Posix);
            let argv_windows = render_copy_command(&argv, ShellFlavor::WindowsCmd);
            let argv_posix = preview.clone();
            let _ = state; // explicit end-of-life for clarity

            let mut copy_posix = false;
            let mut copy_windows = false;
            let mut run_clicked = false;
            ui.horizontal(|ui| {
                if ui.button("Copy command (POSIX)").clicked() {
                    copy_posix = true;
                }
                if ui.button("Copy command (Windows)").clicked() {
                    copy_windows = true;
                }
                if ui.button("Run").clicked() {
                    run_clicked = true;
                }
            });
            ui.label(format!("Preview: {preview}"));

            if copy_posix {
                ctx.copy_text(argv_posix);
            }
            if copy_windows {
                ctx.copy_text(argv_windows);
            }
            if run_clicked {
                if needs_confirm {
                    self.pending_confirm_argv = Some(argv);
                } else {
                    spawn_and_capture(self, argv);
                }
            }
        });

        // ── Run-confirm modal ────────────────────────────────────────────
        if let Some(argv) = self.pending_confirm_argv.clone() {
            egui::Window::new("Confirm secret-bearing run")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(secrets::RUN_CONFIRM_MODAL_PREFIX);
                    ui.separator();
                    ui.label("Argv:");
                    for tok in &argv {
                        ui.monospace(format!("  {}", tok));
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            self.pending_confirm_argv = None;
                            spawn_and_capture(self, argv);
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_confirm_argv = None;
                        }
                    });
                });
        }
    }

    fn on_exit(&mut self) {
        tracing::info!("on_exit() called — clean shutdown via wayland close event");
        // SPEC §9: best-effort zeroize sweep on close.
        for state in self.form_state.values_mut() {
            secrets::zeroize_form_state(state);
        }
    }
}

fn spawn_and_capture(app: &mut MnemonicGuiApp, argv: Vec<String>) {
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
    match runner::run(argv) {
        Ok(result) => {
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
