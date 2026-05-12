//! mnemonic-gui — binary entry. Thin wrapper over `mnemonic_gui` library.
//!
//! Phase 4 wires tracing init + `--debug` flag. The eframe app loop is
//! still stubbed; Phase 5+ wires the form rendering.

use clap::Parser;
use tracing_subscriber::EnvFilter;

/// Cross-platform GUI overlay for the m-format constellation CLIs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Enable debug-level tracing output to stderr (default: WARN).
    /// `RUST_LOG=<filter>` env var overrides this when set.
    #[arg(long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.debug);
    tracing::debug!(target: "mnemonic_gui::main", "tracing initialized");
    // Phase 5+: eframe::run_native(...) goes here.
    let _ = mnemonic_gui::app::placeholder;
}

fn init_tracing(debug_flag: bool) {
    let default_level = if debug_flag { "debug" } else { "warn" };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
