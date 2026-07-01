//! P0 SPIKE — visual-screenshot track feasibility on REAL GitHub runners.
//! **THROWAWAY: deleted in P1** (plan
//! `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` — the promoted suite
//! replaces this file; the rasterizer recipe is absorbed into `build.yml`).
//!
//! What this proves (spec §3, driven by `.github/workflows/spike-snapshots.yml`):
//!   (i)   kittest wgpu+snapshot renders headlessly on `ubuntu-latest` with a
//!         software rasterizer (Plan A: lavapipe-Vulkan via `mesa-vulkan-drivers`;
//!         Plan C fallback/extra-data: llvmpipe-GL);
//!   (ii)  two runs in the SAME job are byte-identical (the workflow sha256s
//!         the `.new.png` sets of two consecutive in-job runs);
//!   (iii) re-runs on ≥3 distinct runner instances all pass kittest's default
//!         0.6 dify threshold against the committed run-1 corpus.
//! Plus the M2 measurements the plan's sizing decision depends on: the
//! per-form flag-row ranking, `fit_contents()` vs fixed-800×600 on the TALLEST
//! form (the no-clipping bar), per-PNG dimensions/sizes, full-61 render time.
//!
//! **Env gate:** `GUI_SNAPSHOTS=1` EARLY-RETURN-SKIP — the schema-mirror CI job
//! runs `cargo test --workspace` on a runner with NO rasterizer; this test must
//! skip (loudly) there and on dev machines.
//!
//! **Run-1 semantics:** with no committed baseline PNG, a comparison reports
//! `missing-baseline` (tolerated, PNG still written as `{name}.new.png` for the
//! artifact upload); once the corpus is committed, any threshold diff FAILS.
//!
//! **Secret hygiene:** the fixture is `render_fixture` (blank
//! `FormState::default()`); no secret value is ever injected; secret widgets
//! render masked. The PNGs depict the canonical on-load screen only.
//!
//! The extended-render helpers (`render_one_positional` / `render_positionals`
//! / `render_action_bar`) are COPIED from `tests/gui_render_faithfulness.rs`
//! for the spike; P1 promotes the originals into `tests/ui_harness/mod.rs`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotError, SnapshotOptions};

use mnemonic_gui::app::CliTab;
use mnemonic_gui::form::fixtures::render_fixture;
use mnemonic_gui::form::render_emit::{self, Presence};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::schema::{FormState, PositionalArgSchema, SubcommandSchema};

mod ui_harness;

const SNAPSHOT_DIR: &str = "tests/snapshots/spike";
/// Crisp-print scale (spec §4): fixed pixels_per_point for every snapshot.
const PIXELS_PER_POINT: f32 = 2.0;

// ─── extended render path (COPIED from tests/gui_render_faithfulness.rs —
//     P1 promotes the originals into ui_harness; the spike must not touch the
//     shipped GREEN test file) ─────────────────────────────────────────────────

/// Render ONE positional through the production widget path — a byte-mirror of
/// `src/main.rs:832`'s positional loop body (minus the multi-row +/✕ chrome).
fn render_one_positional(
    ui: &mut egui::Ui,
    pos: &'static PositionalArgSchema,
    idx: usize,
    state: &mut FormState,
) {
    let label = format!(
        "{} {}{}",
        pos.name,
        if pos.required { "*" } else { "" },
        if pos.repeating { "..." } else { "" }
    );
    if pos.secret {
        let key = format!("positional:{}", pos.name);
        let rows = state
            .secret_widgets
            .entry(key)
            .or_insert_with(|| vec![SecretLineEdit::new()]);
        for row in rows.iter_mut() {
            ui.horizontal(|ui| {
                row.show(ui, &label, pos.help);
            });
        }
        return;
    }
    ui.horizontal(|ui| {
        ui.label(&label);
        while state.positionals.len() <= idx {
            state.positionals.push(String::new());
        }
        ui.text_edit_singleline(&mut state.positionals[idx]);
    });
}

/// Render every positional (extends `render_whole_form`, which renders none).
fn render_positionals(
    ui: &mut egui::Ui,
    sub: &'static SubcommandSchema,
    state: &mut FormState,
) {
    for (i, pos) in sub.positional_args.iter().enumerate() {
        render_one_positional(ui, pos, i, state);
    }
}

/// Render the `[ Run ]` action bar — a mirror of `src/main.rs:1023`.
fn render_action_bar(ui: &mut egui::Ui) {
    ui.add_enabled(true, egui::Button::new("Run"));
}

/// The whole-form snapshot harness: flag grid + positionals + `[ Run ]` over
/// the canonical `render_fixture` state, at `PIXELS_PER_POINT`. When `fit` is
/// true the viewport is shrunk-to-content (`fit_contents()`); otherwise it
/// stays at kittest's default 800×600 logical (the clipping comparison arm).
fn snapshot_harness(
    tab: CliTab,
    sub: &'static SubcommandSchema,
    fit: bool,
) -> Harness<'static, FormState> {
    let base = render_fixture(tab, sub.name);
    let mut harness = Harness::builder()
        .with_pixels_per_point(PIXELS_PER_POINT)
        .build_ui_state(
            move |ui, state: &mut FormState| {
                ui_harness::render_whole_form(ui, tab, sub, state);
                render_positionals(ui, sub, state);
                render_action_bar(ui);
            },
            base,
        );
    if fit {
        harness.fit_contents();
    }
    harness
}

// ─── PNG inspection (no `image` dev-dep needed — IHDR is fixed-offset) ───────

/// `(width, height)` out of a PNG's IHDR chunk.
fn png_dims(path: &Path) -> Option<(u32, u32)> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((w, h))
}

/// The freshly-rendered PNG for `name`: `.new.png` in compare mode, `.png`
/// under `UPDATE_SNAPSHOTS` (where kittest removes the `.new`).
fn rendered_png(name: &str) -> Option<PathBuf> {
    let new = PathBuf::from(SNAPSHOT_DIR).join(format!("{name}.new.png"));
    if new.exists() {
        return Some(new);
    }
    let committed = PathBuf::from(SNAPSHOT_DIR).join(format!("{name}.png"));
    committed.exists().then_some(committed)
}

/// Snapshot `harness` as `name`, tolerating ONLY the run-1 missing-baseline
/// case; any other error (threshold diff, size mismatch, render error) is
/// pushed to `failures`. Returns the rendered PNG's `(dims, bytes)`.
fn snapshot_one(
    harness: &mut Harness<'static, FormState>,
    name: &str,
    opts: &SnapshotOptions,
    failures: &mut Vec<String>,
) -> ((u32, u32), u64) {
    let baseline = PathBuf::from(SNAPSHOT_DIR).join(format!("{name}.png"));
    let baseline_existed = baseline.exists();
    let status = match harness.try_snapshot_options(name, opts) {
        Ok(()) => "match".to_string(),
        Err(SnapshotError::OpenSnapshot { .. }) if !baseline_existed => {
            "missing-baseline (run-1: corpus not committed yet)".to_string()
        }
        Err(err) => {
            failures.push(format!("{name}: {err}"));
            format!("FAIL: {err}")
        }
    };
    let (dims, bytes) = rendered_png(name)
        .map(|p| {
            (
                png_dims(&p).unwrap_or((0, 0)),
                std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0),
            )
        })
        .unwrap_or(((0, 0), 0));
    eprintln!(
        "SPIKE-PNG: {name} {}x{} {bytes}B status={status}",
        dims.0, dims.1
    );
    (dims, bytes)
}

// ─── the spike ───────────────────────────────────────────────────────────────

#[test]
fn p0_spike_snapshot_all_61_forms() {
    if std::env::var("GUI_SNAPSHOTS").as_deref() != Ok("1") {
        eprintln!(
            "SPIKE-SKIP: GUI_SNAPSHOTS != 1 — spike_form_snapshots skipped \
             (no pinned rasterizer in this env; the spike runs only in the \
             spike-snapshots workflow)"
        );
        return;
    }

    // The wgpu adapter the harness will select (same selection path: kittest's
    // default setup prefers CPU adapters, honors WGPU_BACKEND).
    let render_state =
        egui_kittest::wgpu::create_render_state(egui_kittest::wgpu::default_wgpu_setup());
    eprintln!("SPIKE-ADAPTER: {:?}", render_state.adapter.get_info());
    drop(render_state);

    let all = render_emit::all_forms();
    assert_eq!(
        all.len(),
        61,
        "expected exactly 61 GUI subcommand forms (mnemonic 32 + md 10 + ms 10 + mk 9)"
    );

    // ── the schema-derived flag-row ranking (rows = Present flags +
    //    positionals + the Run bar) ──
    let mut ranking: Vec<(usize, String)> = all
        .iter()
        .map(|(tab, sub)| {
            let proj = render_emit::project_form(*tab, sub);
            let rows = proj
                .flags
                .iter()
                .filter(|f| matches!(f.presence, Presence::Present))
                .count()
                + proj.positionals.len()
                + 1;
            (rows, format!("{}-{}", tab.bin_name(), sub.name))
        })
        .collect();
    ranking.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    eprintln!("SPIKE-RANKING (schema rows: Present flags + positionals + Run):");
    for (rows, name) in &ranking {
        eprintln!("SPIKE-RANKING-ROW: {rows:3} {name}");
    }

    // ── render + snapshot ALL 61 forms, fit_contents @ ppp 2.0 ──
    let opts = SnapshotOptions::new().output_path(SNAPSHOT_DIR);
    let mut failures: Vec<String> = Vec::new();
    let mut heights: Vec<(u32, u32, String)> = Vec::new(); // (h, w, name)
    let mut corpus_bytes: u64 = 0;
    let mut rendered = 0usize;
    let started = Instant::now();
    for (tab, sub) in &all {
        let name = format!("{}-{}", tab.bin_name(), sub.name);
        let mut harness = snapshot_harness(*tab, sub, true);
        // No-clipping tree evidence: the LAST rendered widget (the Run bar)
        // must exist in the settled tree for every form.
        assert!(
            harness.query_all_by_label("Run").next().is_some(),
            "{name}: Run button node missing from the AccessKit tree"
        );
        let ((w, h), bytes) = snapshot_one(&mut harness, &name, &opts, &mut failures);
        heights.push((h, w, name));
        corpus_bytes += bytes;
        rendered += 1;
    }
    let elapsed = started.elapsed();
    eprintln!("SPIKE-TIME: full-61 fit_contents render+snapshot took {elapsed:?}");
    eprintln!("SPIKE-CORPUS: 61 PNGs, total {corpus_bytes} bytes");

    heights.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.2.cmp(&b.2)));
    eprintln!("SPIKE-HEIGHT-RANKING (measured PNG px @ ppp 2.0, top 10):");
    for (h, w, name) in heights.iter().take(10) {
        eprintln!("SPIKE-HEIGHT-ROW: {h:5}px x{w:5}px {name}");
    }

    // ── the sizing comparison: the TALLEST form at kittest's default
    //    fixed 800×600 logical viewport (I4 — would a fixed frame clip?) ──
    let (tallest_h, tallest_w, tallest_name) = heights[0].clone();
    let (tab, sub) = all
        .iter()
        .find(|(tab, sub)| format!("{}-{}", tab.bin_name(), sub.name) == tallest_name)
        .expect("tallest form must be in the enumeration");
    let mut fixed = snapshot_harness(*tab, sub, false);
    let ((fw, fh), _) = snapshot_one(&mut fixed, "tallest-fixed-800x600", &opts, &mut failures);
    let clipped = tallest_h > fh;
    eprintln!(
        "SPIKE-TALLEST: {tallest_name} fit_contents={tallest_w}x{tallest_h}px vs \
         fixed-800x600={fw}x{fh}px -> fixed frame would clip: {clipped}"
    );

    assert_eq!(rendered, 61, "census: every one of the 61 forms must render");
    assert!(
        failures.is_empty(),
        "spike snapshot failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
