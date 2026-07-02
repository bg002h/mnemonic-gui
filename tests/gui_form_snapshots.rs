//! P1 — the PERMANENT 61-form snapshot suite (visual-screenshot track; plan
//! `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` P1, spec §4/§5;
//! ratifications in `agent-reports/visual-track-p0-r0-review.md`).
//!
//! Renders every GUI subcommand form (mnemonic 32 + md 10 + ms 10 + mk 9 =
//! 61) over the canonical blank fixture `render_fixture(tab, sub)` through
//! the SHARED extended render path (`ui_harness::render_whole_form` +
//! `render_positionals` + `render_action_bar` — the same composition the
//! faithfulness gate anchors) and compares each against the committed corpus
//! `tests/snapshots/forms/<tab>-<sub>.png` at kittest's DEFAULT dify
//! threshold 0.6.
//!
//! **Sizing (P0-R0 RATIFIED §1):** `fit_contents()` at `pixels_per_point =
//! 2.0` — the viewport is sized to content, so viewport clipping is
//! impossible by construction; the per-form Run-node tree assertion (the
//! LAST rendered widget) enforces the no-clipping bar per form.
//!
//! **Env gate:** `GUI_SNAPSHOTS=1` EARLY-RETURN-SKIP (loud `eprintln!`
//! marker) — plain `cargo test` runs on machines with no pinned software
//! rasterizer. The `build.yml` `snapshots` job is the enforcing consumer:
//! lavapipe-Vulkan (`mesa-vulkan-drivers` + `WGPU_BACKEND=vulkan`),
//! `--nocapture` (observability), plus the ran-at-all census (61 ×
//! `.new.png` — kittest writes `.new.png` on PASSING comparisons too).
//!
//! **Adapter guard (P0-R0 A1):** under `GUI_SNAPSHOTS=1` the selected wgpu
//! adapter MUST be `device_type == Cpu` — UNCONDITIONALLY (this enforces the
//! software-rasterizer corpus-provenance posture for CI AND local
//! regeneration) — and `backend` must match `WGPU_BACKEND` IFF that env var
//! is set (a hard `Vulkan` assert would break the sanctioned Plan-C
//! llvmpipe-GL local-regen path). NEVER match on adapter `name` / `driver`:
//! lavapipe self-reports its name as "llvmpipe (…)", and the GL arm's
//! `driver` field is EMPTY.
//!
//! **Corpus regeneration (spec §5 M4):** any software rasterizer works —
//! locally `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1
//! UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots` is the proven
//! path; the pinned CI job arbitrates the result at the 0.6 threshold.
//!
//! **Size ≠ identity (P0-R0 A5):** PNG byte-SIZE is NOT an identity signal —
//! in the spike's CI cross-backend pair, 4 files differed by ±1 B while the
//! 61-file TOTALS were identical (sub-LSB pixel drift compresses to equal
//! sizes). Only sha256 hashes / the threshold gate mean anything.
//!
//! **Render ceiling (loud-fail):** wgpu's default `max_texture_dimension_2d`
//! (8192 px) caps `fit_contents()` at ≈190 flag rows (≈43 physical px/row @
//! ppp 2.0; today's tallest form, `mnemonic verify-bundle`, is 29 rows /
//! 1253 px — 6.5× headroom). A future form crossing the ceiling FAILS the
//! render (snapshot error → test failure); it never silently clips.
//!
//! **Secret hygiene (spec §4):** the fixture is the blank `render_fixture`
//! state — no secret value is ever injected; secret widgets render
//! masked/empty. The always-run `secret_flags_never_carry_a_default_value`
//! test below (I3) enforces that no schema default can pre-fill secret
//! material into the on-load form — and therefore into the PNG corpus.

use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions};

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::fixtures::render_fixture;
use mnemonic_gui::form::render_emit;
use mnemonic_gui::schema::{FormState, SubcommandSchema};

mod ui_harness;

/// The committed corpus directory (leg-2's `verify-figures-gui` manual gate
/// byte-copies from here).
const SNAPSHOT_DIR: &str = "tests/snapshots/forms";
/// Crisp-print scale (spec §4; P0-R0 ratified): fixed pixels_per_point for
/// every snapshot.
const PIXELS_PER_POINT: f32 = 2.0;

/// The whole-form snapshot harness: the SHARED extended render path (flag
/// grid + positionals + `[ Run ]`) over the canonical `render_fixture` state,
/// at `PIXELS_PER_POINT`, viewport shrunk-to-content (`fit_contents()`).
fn snapshot_harness(tab: CliTab, sub: &'static SubcommandSchema) -> Harness<'static, FormState> {
    let base = render_fixture(tab, sub.name);
    let mut harness = Harness::builder()
        .with_pixels_per_point(PIXELS_PER_POINT)
        .build_ui_state(
            move |ui, state: &mut FormState| {
                ui_harness::render_whole_form(ui, tab, sub, state);
                ui_harness::render_positionals(ui, sub, state);
                ui_harness::render_action_bar(ui);
            },
            base,
        );
    harness.fit_contents();
    harness
}

/// Map a `WGPU_BACKEND` env spelling to the single `wgpu::Backend` it selects
/// (the `wgpu::Backends::from_comma_list` spellings). `None` for
/// multi-backend / unknown spellings — the backend assert is skipped there
/// (the `device_type == Cpu` assert never is).
fn expected_backend(env: &str) -> Option<eframe::wgpu::Backend> {
    match env.to_ascii_lowercase().as_str() {
        "vulkan" | "vk" => Some(eframe::wgpu::Backend::Vulkan),
        "gl" | "opengl" | "gles" => Some(eframe::wgpu::Backend::Gl),
        "metal" | "mtl" => Some(eframe::wgpu::Backend::Metal),
        "dx12" | "d3d12" => Some(eframe::wgpu::Backend::Dx12),
        "webgpu" => Some(eframe::wgpu::Backend::BrowserWebGpu),
        _ => None,
    }
}

#[test]
fn gui_form_snapshots_all_61() {
    if std::env::var("GUI_SNAPSHOTS").as_deref() != Ok("1") {
        eprintln!(
            "SNAPSHOTS-SKIP: GUI_SNAPSHOTS != 1 — gui_form_snapshots skipped \
             (no pinned software rasterizer in this env; the build.yml \
             `snapshots` job is the enforcing consumer)"
        );
        return;
    }

    // ── the A1 adapter guard (same selection path as the harness below:
    //    kittest's default setup prefers CPU adapters, honors WGPU_BACKEND) ──
    let info = {
        let render_state =
            egui_kittest::wgpu::create_render_state(egui_kittest::wgpu::default_wgpu_setup());
        render_state.adapter.get_info()
    };
    eprintln!("SNAPSHOTS-ADAPTER: {info:?}");
    assert_eq!(
        info.device_type,
        eframe::wgpu::DeviceType::Cpu,
        "GUI_SNAPSHOTS renders MUST come from a software rasterizer \
         (device_type Cpu) — got {info:?}. Use lavapipe (mesa-vulkan-drivers \
         + WGPU_BACKEND=vulkan) or llvmpipe (WGPU_BACKEND=gl \
         LIBGL_ALWAYS_SOFTWARE=1)."
    );
    if let Ok(env) = std::env::var("WGPU_BACKEND") {
        match expected_backend(&env) {
            Some(expected) => assert_eq!(
                info.backend, expected,
                "adapter backend does not honor WGPU_BACKEND={env} — got {info:?}"
            ),
            None => eprintln!(
                "SNAPSHOTS-ADAPTER: WGPU_BACKEND={env} is not a single-backend \
                 spelling; backend assert skipped (device_type asserted above)"
            ),
        }
    }

    // ── census: the enumeration must cover exactly the 61 forms ──
    let all = render_emit::all_forms();
    assert_eq!(
        all.len(),
        61,
        "expected exactly 61 GUI subcommand forms (mnemonic 32 + md 10 + ms 10 + mk 9)"
    );

    // Default threshold 0.6 (kittest's dify default — the fleet-proven gate).
    let opts = SnapshotOptions::new().output_path(SNAPSHOT_DIR);
    let mut failures: Vec<String> = Vec::new();
    let mut rendered = 0usize;
    for (tab, sub) in &all {
        let name = format!("{}-{}", tab.bin_name(), sub.name);
        let mut harness = snapshot_harness(*tab, sub);
        // The no-clipping bar (P0-R0 §1): the `[ Run ]` bar is the LAST
        // rendered widget — its node present in the settled tree proves the
        // form rendered to the bottom, for every form.
        assert!(
            harness.query_all_by_label("Run").next().is_some(),
            "{name}: Run button node missing from the AccessKit tree"
        );
        // Accumulate instead of panicking per-form: a partial drift still
        // renders + diffs the WHOLE corpus, so the failure-path artifact
        // upload carries every `.diff.png`, not just the first (spec §5
        // remediation-ladder forensics). Unlike the P0 spike, a MISSING
        // baseline is a hard failure here — the corpus is committed.
        if let Err(err) = harness.try_snapshot_options(&name, &opts) {
            failures.push(format!("{name}: {err}"));
        }
        rendered += 1;
    }
    assert_eq!(rendered, 61, "census: every one of the 61 forms must render");
    assert!(
        failures.is_empty(),
        "form-snapshot failures ({} of 61):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn secret_flags_never_carry_a_default_value() {
    // I3 (spec §4) — ALWAYS-RUN (schema truth needs no rasterizer, so this is
    // deliberately NOT behind the GUI_SNAPSHOTS gate): a secret flag carrying
    // a clap-declared default would PRE-FILL secret material into the on-load
    // form — and therefore into the committed PNG corpus. Enforce
    // `flag.secret == true ⇒ flag.default_value.is_none()` over the static
    // schema tables of all 4 tabs.
    let mut violations: Vec<String> = Vec::new();
    let mut subs = 0usize;
    let mut secret_flags = 0usize;
    for tab in CliTab::ALL.iter().copied() {
        for sub in render_emit::schema_for(tab).subcommands {
            subs += 1;
            for flag in sub.flags {
                if flag.secret {
                    secret_flags += 1;
                    if flag.default_value.is_some() {
                        violations.push(format!(
                            "{}/{} {}: secret flag carries default_value={:?}",
                            tab.bin_name(),
                            sub.name,
                            flag.name,
                            flag.default_value
                        ));
                    }
                }
            }
        }
    }
    assert_eq!(subs, 61, "census: the hygiene sweep must cover all 61 subcommands");
    assert!(
        secret_flags > 0,
        "vacuous sweep: no secret flags found in any schema table — schema drift?"
    );
    assert!(
        violations.is_empty(),
        "secret-flag default_value hygiene violations ({}):\n{}",
        violations.len(),
        violations.join("\n")
    );
}
