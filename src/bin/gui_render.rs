//! `gui-render` — headless emit of the GUI's structural form renders (P2).
//!
//! Builds + runs under `--no-default-features` (the egui-free form model
//! only; no eframe/egui/wgpu/winit). Two modes:
//!
//! ```text
//! gui-render --form <tab> <subcommand>   # one form's render to stdout
//! gui-render --emit-all <dir>            # write <tab>-<sub>.gui for ALL forms
//! ```
//!
//! `<tab>` is one of `mnemonic | md | ms | mk`. The render itself + its
//! determinism / secret-masking discipline live in the library
//! (`mnemonic_gui::form::render_emit`); this binary is a thin CLI wrapper so
//! the same code path feeds the P3 faithfulness test.

use std::path::Path;
use std::process::ExitCode;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::render_emit;

const USAGE: &str = "\
usage:
  gui-render --form <tab> <subcommand>   print one form's render to stdout
  gui-render --emit-all <dir>            write <tab>-<sub>.gui for every form

  <tab> = mnemonic | md | ms | mk";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--form") => run_form(args.get(2), args.get(3)),
        Some("--emit-all") => run_emit_all(args.get(2)),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run_form(tab_arg: Option<&String>, sub_arg: Option<&String>) -> ExitCode {
    let (Some(tab_s), Some(sub_s)) = (tab_arg, sub_arg) else {
        eprintln!("error: --form needs <tab> <subcommand>\n\n{USAGE}");
        return ExitCode::FAILURE;
    };
    let Some(tab) = CliTab::from_bin_name(tab_s) else {
        eprintln!("error: unknown tab '{tab_s}' (expected mnemonic | md | ms | mk)");
        return ExitCode::FAILURE;
    };
    let schema = render_emit::schema_for(tab);
    let Some(sub) = schema.subcommands.iter().find(|s| s.name == sub_s.as_str()) else {
        eprintln!("error: unknown subcommand '{sub_s}' for tab '{tab_s}'");
        return ExitCode::FAILURE;
    };
    // render_form already terminates with a trailing LF — `print!`, not
    // `println!`, keeps the byte output identical to the `--emit-all` files.
    print!("{}", render_emit::render_form(tab, sub));
    ExitCode::SUCCESS
}

fn run_emit_all(dir_arg: Option<&String>) -> ExitCode {
    let Some(dir) = dir_arg else {
        eprintln!("error: --emit-all needs <dir>\n\n{USAGE}");
        return ExitCode::FAILURE;
    };
    match render_emit::emit_all(Path::new(dir)) {
        Ok(n) => {
            eprintln!("wrote {n} form renders to {dir}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: --emit-all failed: {e}");
            ExitCode::FAILURE
        }
    }
}
