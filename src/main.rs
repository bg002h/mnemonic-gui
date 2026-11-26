//! mnemonic-gui — binary entry. Thin wrapper over `mnemonic_gui` library.
//!
//! Phase 4 wires tracing init + `--debug` flag. v0.1.1 wires the eframe
//! loop + a minimal interactive form (bundle subcommand as the canonical
//! demo). Tab-strip + per-subcommand wiring follows the Phase 6 R1 I-3
//! fold's data-layer foundation.
//!
//! P0 spike of the `gui_example_tutorial` cycle (SPEC §3.1(a)): the whole
//! application shell (`MnemonicGuiApp` — struct + impls + helpers) moved
//! VERBATIM into the gui-gated library module [`mnemonic_gui::app_window`]
//! so the tutorial harness can drive the real window headlessly. This file
//! is now only `fn main()` + the eframe bootstrap + tracing init.

use std::process::ExitCode;

use clap::Parser;
use eframe::egui;
use tracing_subscriber::EnvFilter;

use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::persistence;

/// The GUI's own exit statuses (`SPEC_constellation_exit_codes.md` rev 5,
/// R12): 0, 64, 32 (cannot start) or 101, plus the deliberate 130 when a
/// clean shutdown after a signal does not finish in time
/// (`app_window.rs`). The codes the CLIs it runs exit with are decoded in the
/// output panel, not here.
const EXIT_LEGEND: &str = "Exit status: 0 = closed normally. 32 = cannot start (no display, \
graphics initialisation failed). 64 = bad command line. 101 = internal error. 130 = \
stopped by a signal and the clean shutdown did not finish.";

/// Cross-platform GUI overlay for the m-format constellation CLIs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, after_help = EXIT_LEGEND)]
struct Cli {
    /// Enable debug-level tracing output to stderr (default: WARN).
    /// `RUST_LOG=<filter>` env var overrides this when set.
    #[arg(long)]
    debug: bool,
}

fn main() -> ExitCode {
    // Exit-code spec R12: a bad command line is 64 (clap's own 2 is a bit
    // under the constellation's scheme); `--help` / `--version` stay 0.
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // R8: `--help`/`--version` whose stdout cannot be written (a full
            // disk, a closed stdout) is an IO failure, 32; EPIPE (the reader
            // went away) keeps the code. A usage error stays 64 whether or
            // not its message reached stderr.
            let printed = e.print();
            let io_failed = printed.is_err_and(|err| err.kind() != std::io::ErrorKind::BrokenPipe);
            return ExitCode::from(if e.exit_code() != 0 {
                mnemonic_gui::exit_code::USAGE
            } else if io_failed {
                mnemonic_gui::exit_code::IO
            } else {
                mnemonic_gui::exit_code::OK
            });
        }
    };
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
    let run = eframe::run_native(
        "mnemonic-gui",
        native_options,
        Box::new(move |cc| Ok(Box::new(MnemonicGuiApp::new(cc, loaded_state, state_path)))),
    );
    match run {
        Ok(()) => ExitCode::SUCCESS,
        // R12: the window could not open (no display, graphics init failed):
        // 32, "cannot start", not Rust's generic 1.
        Err(e) => {
            eprintln!("mnemonic-gui: cannot start: {e}");
            ExitCode::from(mnemonic_gui::exit_code::IO)
        }
    }
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
