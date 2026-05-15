//! Phase G P1.4 — manual-gui v1.0 cycle: widget_help_icon kittest RED.
//!
//! Companion to the `manual-gui` cycle landing in `mnemonic-toolkit`
//! (`/scratch/code/shibboleth/mnemonic-toolkit/docs/manual-gui/`).
//! Per SPEC §2.4 (Option C help-icon placement) + §2.1 G7, every
//! Dropdown / NodeValueComposite flag gets a `?` button rendered to
//! the right of its label. Click emits
//! `ctx.open_url(OpenUrl::new_tab(manual_url_for_flag(tab, sub, flag)))`,
//! the URL pointing at the rendered HTML manual's stable anchor for
//! the (subcommand, flag) pair.
//!
//! This cell exercises the canonical example called out in SPEC §2.1
//! G7:
//!
//!     mnemonic convert --from   →
//!     https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from
//!
//! RED state at P1.4: `?` buttons do not yet exist in `form/widget.rs`
//! (P2.2 wires them). The harness boots, accesskit maps the tree, and
//! `query_by_label("?")` returns `None` — the first assertion fails
//! with a SPEC-pointing message. When P2.2 lands, the second
//! assertion's URL equality also exercises P2.1's helper module.
//!
//! Harness API per SPEC §3.0 G-P0.3 (egui_kittest 0.31):
//! `harness.ctx` is a `pub ctx: egui::Context` FIELD. But reading
//! emitted URLs via `harness.ctx.output(|o| ...)` does NOT work:
//! `Context::end_pass` (`egui-0.31.1/src/context.rs:2331`) drains
//! `viewport.output` via `std::mem::take` at the end of every frame.
//! By the time `harness.run()` returns, the live `ctx.viewport().output`
//! is reset to `Default::default()`. The drained PlatformOutput is
//! stashed inside the harness's `FullOutput`. The authoritative read
//! path for kittest tests is therefore:
//!
//!     harness.output().platform_output.commands
//!         .iter()
//!         .find_map(|cmd| match cmd {
//!             egui::OutputCommand::OpenUrl(u) => Some(u.url.clone()),
//!             _ => None,
//!         })
//!
//! Note also that `Context::open_url(...)` (`context.rs:1452-1453`)
//! writes to `PlatformOutput.commands: Vec<OutputCommand>` only;
//! the older `PlatformOutput.open_url: Option<OpenUrl>` field is
//! `#[deprecated]` (`data/output.rs:115`) and is NOT touched by the
//! modern API. The companion sanity probe
//! `cell_help_icon_read_path_sanity_probe` below empirically verifies
//! the chosen read path before the real RED-assert runs.
//!
//! G-P2.3 amendment (post-P1.4-LOCK, empirically driven): after
//! `button.click()`, the test uses `harness.step()` — NOT
//! `harness.run()`. `Harness::run()` (`egui_kittest-0.31.1/src/lib.rs:281`)
//! loops `step()` until repaint_delay != ZERO. A click on a stateful
//! egui button typically triggers a follow-up immediate repaint
//! (animation / hover state), so `run()` returns AFTER 2-3 frames.
//! `harness.output()` (`lib.rs:359`) returns the LAST frame's
//! FullOutput, and `viewport.output.commands` is per-frame (drained
//! at end_pass via `std::mem::take`). So `run()`'s extra no-op frames
//! after the click frame overwrite the OpenUrl command we wanted to
//! observe. `harness.step()` runs exactly one frame (the one that
//! processes the queued accesskit click action), capturing its output
//! before the buffer is overwritten. Empirically verified at G-P2.3:
//! step() → 1 OpenUrl with correct URL; run() → 3 steps, [] commands.
//!
//! FOLLOWUP: plan §2.1 G7 + §3.1 P1.4 snippets updated to the
//! `harness.output().platform_output.commands` form (P1.4 R0 fold v2
//! 2026-05-15) so SPEC and the test agree. G-P2.3 adds the step()
//! amendment as a third post-LOCK clarification.

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::widget;
use mnemonic_gui::schema::{self, FlagSchema, FormState};

fn flag_named(sub_name: &str, flag_name: &str) -> &'static FlagSchema {
    let sub = schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == sub_name)
        .unwrap_or_else(|| panic!("subcommand {sub_name} not in mnemonic schema"));
    sub.flags
        .iter()
        .find(|f| f.name == flag_name)
        .unwrap_or_else(|| {
            panic!("flag {flag_name} not in subcommand {sub_name}")
        })
}

/// Pre-flight sanity probe per P1.4 R0 C-1: verifies our chosen read
/// path (`harness.output().platform_output.commands`) actually
/// captures URLs emitted by `ctx.open_url(...)`. Without this, the
/// real-test's URL assertion below could be a false-RED that never
/// transitions to GREEN even after P2.2 wires the `?` button
/// correctly. egui 0.31's `Context::end_pass` drains
/// `viewport.output` via `std::mem::take` (`context.rs:2331`); the
/// drained PlatformOutput becomes part of `FullOutput` which the
/// harness stashes — so `harness.output().platform_output` is the
/// authoritative read path, NOT `harness.ctx.output(|o| ...)`.
#[test]
fn cell_help_icon_read_path_sanity_probe() {
    let mut harness = Harness::new_ui(|ui| {
        ui.ctx()
            .open_url(egui::OpenUrl::same_tab("https://example.invalid/probe"));
    });
    harness.run();
    let url = harness
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|cmd| match cmd {
            egui::OutputCommand::OpenUrl(open_url) => Some(open_url.url.clone()),
            _ => None,
        });
    assert_eq!(
        url.as_deref(),
        Some("https://example.invalid/probe"),
        "sanity probe: ctx.open_url(...) must surface via harness.output().platform_output.commands",
    );
}

#[test]
fn cell_help_icon_emits_open_url_for_mnemonic_convert_from() {
    let flag = flag_named("convert", "--from");
    let initial = FormState::default();

    let mut harness = Harness::new_ui_state(
        |ui, state| {
            widget::render_with_dispatch(ui, CliTab::Mnemonic, "convert", flag, state);
        },
        initial,
    );

    // Force one frame so accesskit can map the rendered tree.
    harness.run();

    // RED at P1.4: query_by_label returns None because the `?` button
    // doesn't exist in widget.rs yet. When P2.2 lands the per-NVC
    // affordance, the button is found and click triggers ctx.open_url.
    let button = harness.query_by_label("?");
    assert!(
        button.is_some(),
        "expected `?` help button next to `{}` label on `mnemonic convert` form \
         (SPEC §2.4 Option C: per-NodeValueComposite `?` affordance)",
        flag.name,
    );

    button.unwrap().click();
    // step() not run(): see module-level docstring for the
    // "viewport.output drained per frame" rationale (G-P2.3 amendment).
    harness.step();

    let url = harness
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|cmd| match cmd {
            egui::OutputCommand::OpenUrl(open_url) => Some(open_url.url.clone()),
            _ => None,
        });
    assert_eq!(
        url.as_deref(),
        Some("https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from"),
        "click on `?` should emit ctx.open_url to the SPEC §2.2 anchor for \
         mnemonic-convert-from (build-time-overridable via env var \
         MNEMONIC_GUI_MANUAL_BASE_URL per SPEC §2.4)",
    );
}
