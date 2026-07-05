//! P0 SPIKE — `gui_example_tutorial` cycle (SPEC §4, S1–S5). THROWAWAY-grade
//! mechanics proof, but CI-comparable: drives the REAL extracted app shell
//! (`mnemonic_gui::app_window::MnemonicGuiApp`) WHOLE-WINDOW under
//! egui_kittest, with LIVE pinned-CLI Run clicks populating the output pane
//! (the user-locked contract this spike is the GO/NO-GO for).
//!
//! Authority: `mnemonic-toolkit/docs/manual-gui/design/SPEC_gui_example_tutorial.md`
//! §4 (S1–S5 + STOP condition), §3.1(b) (`pinned-tier-version-gate`,
//! `SAME-FRAME-COMPLETION`), §6 (determinism contract), §7 (secret hygiene);
//! R0 round-2 finding m1 (single-`step()` click semantics) is folded in.
//!
//! **Env gate:** `GUI_TUTORIAL_SPIKE=1` — EARLY-RETURN-SKIP (plain
//! `cargo test` needs no rasterizer and no pinned CLI). The enforcing
//! consumer is the throwaway `spike-gui-example-p0.yml` workflow on the
//! draft spike PR (lavapipe recipe, 2 runner samples).
//!
//! **Spike tier:** `mnemonic 0.75.0` (the P1 pin-bump target). The
//! `pinned-tier-version-gate` prototype below probes `mnemonic --version`
//! BEFORE any render and hard-fails on mismatch. The bite demo also probes
//! against the CURRENT schema constant (`mnemonic 0.74.0`) which MUST
//! mismatch the spike tier — a deliberate wrong-tier probe proving the gate
//! fires. (After P1 bumps the schema constant this throwaway demo goes away
//! with the whole file.)

use std::path::{Path, PathBuf};
use std::time::Instant;

use egui::accesskit::Role;
use egui_kittest::kittest::{by, Node, Queryable};
use egui_kittest::{Harness, SnapshotOptions};

use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::form::slot_editor::SlotSubkey;
use mnemonic_gui::path_detect::Detected;
use mnemonic_gui::schema::FlagValue;

// ─── constants ──────────────────────────────────────────────────────────────

const PPP: f32 = 2.0;
/// S4 candidate A — the production default window seed (`main.rs:52`).
const SIZE_A: [f32; 2] = [920.0, 720.0];
/// S4 candidate B.
const SIZE_B: [f32; 2] = [1280.0, 900.0];
/// The spike's pinned tier (= the P1 pin-bump target, locally installed).
const SPIKE_TIER_MNEMONIC: &str = "mnemonic 0.75.0";
/// S0 — the world-known all-`abandon` BIP-39 test vector (Examples.md:209;
/// fp 73c5da0a). The ONLY secret-class value this spike ever drives (§7).
const S0_PHRASE: &str = "abandon abandon abandon abandon abandon abandon \
                         abandon abandon abandon abandon abandon about";
const SNAPSHOT_SUBDIR: &str = "tests/snapshots/spike";

// ─── small helpers ──────────────────────────────────────────────────────────

fn manifest_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Injected AppState — all four CLIs `Found` (§6 item 3: no `$PATH`-dependent
/// tab grey-out). Display-only paths: nothing renders them, and
/// `spawn_and_capture` re-probes the REAL `$PATH` at click time regardless.
fn fixed_appstate_all_found() -> AppState {
    AppState {
        mnemonic_detect: Detected::Found(PathBuf::from("/pinned/mnemonic")),
        md_detect: Detected::Found(PathBuf::from("/pinned/md")),
        ms_detect: Detected::Found(PathBuf::from("/pinned/ms")),
        mk_detect: Detected::Found(PathBuf::from("/pinned/mk")),
        active_tab: CliTab::Mnemonic,
    }
}

/// The whole-window harness: REAL app (`new_headless`, no persistence, fixed
/// AppState) stepped through the REAL `ui()` at a fixed window size.
fn app_harness(size: [f32; 2]) -> Harness<'static, MnemonicGuiApp> {
    let app = MnemonicGuiApp::new_headless(fixed_appstate_all_found(), None, None);
    Harness::builder()
        .with_size(egui::Vec2::new(size[0], size[1]))
        .with_pixels_per_point(PPP)
        // Default max_steps (4) is too tight for the smooth-scroll
        // animation (S3 wheel mechanics: egui requests ~8 repaint frames
        // before quiescence). run() still settles deterministically.
        .with_max_steps(64)
        .build_state(|ctx, app: &mut MnemonicGuiApp| app.ui(ctx), app)
}

/// `pinned-tier-version-gate` prototype (SPEC §3.1b / §6 item 4 — the
/// `gen.sh:22` pattern): probe `<cli> --version`, hard-mismatch on any
/// difference. Runs BEFORE any render/spawn.
fn pinned_tier_version_gate(cli: &str, expected: &str) -> Result<String, String> {
    let out = std::process::Command::new(cli)
        .arg("--version")
        .output()
        .map_err(|e| format!("pinned-tier-version-gate: failed to spawn `{cli} --version`: {e}"))?;
    let got = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if got == expected {
        Ok(got)
    } else {
        Err(format!(
            "pinned-tier-version-gate: `{cli} --version` = {got:?}, expected {expected:?} — \
             refusing to render or spawn from a wrong tier (SPEC §3.1b; wrong-tier local \
             regen must be impossible)"
        ))
    }
}

fn rect_of(n: &Node<'_>) -> (f64, f64, f64, f64) {
    let r = n.raw_bounds().expect("node has no bounds");
    (r.x0, r.y0, r.x1, r.y1)
}

/// Row-anchored geometric lookup (the S5 drive discipline for UNLABELLED
/// inputs): find the widget of `role` sitting on the same horizontal band as
/// the exact-label `anchor` node, `occ`-th from the left among matches to the
/// anchor's right. egui attaches no label↔input relation, so the flag-name
/// label anchors the row and the input is found geometrically — deterministic
/// under a fixed window size + fixed ppp.
fn on_row_of<'t>(
    h: &'t Harness<'static, MnemonicGuiApp>,
    anchor: &'static str,
    role: Role,
    occ: usize,
) -> Node<'t> {
    let a = h
        .query_all_by_label(anchor)
        .next()
        .unwrap_or_else(|| panic!("row anchor label {anchor:?} not found"));
    let (ax0, ay0, _ax1, ay1) = rect_of(&a);
    let mid = (ay0 + ay1) / 2.0;
    let mut hits: Vec<Node<'t>> = h
        .query_all(by().role(role))
        .filter(|n| {
            let Some(r) = n.raw_bounds() else { return false };
            let c = (r.y0 + r.y1) / 2.0;
            // vertical centers within each other's band + to the anchor's right
            c >= ay0 - 2.0 && c <= ay1 + 2.0 && ((r.y0 + r.y1) / 2.0 - mid).abs() < (ay1 - ay0) && r.x0 >= ax0
        })
        .collect();
    hits.sort_by(|a, b| {
        let ra = a.raw_bounds().unwrap();
        let rb = b.raw_bounds().unwrap();
        ra.x0.partial_cmp(&rb.x0).unwrap()
    });
    *hits.get(occ).unwrap_or_else(|| {
        panic!(
            "on_row_of({anchor:?}, {role:?}, occ={occ}): only {} matches on the row",
            hits.len()
        )
    })
}

/// Whole-tree no-plaintext assertion (§7 pixel/text channel guard): no
/// AccessKit node label OR value may contain the secret substring. egui masks
/// password-field values before they reach AccessKit (verified in egui 0.31
/// `text_edit/builder.rs` — `mask_if_password` feeds both the galley and the
/// widget info), so this can be GLOBAL with zero exclusions.
fn assert_no_plaintext(h: &Harness<'static, MnemonicGuiApp>, needle: &str, ctx_msg: &str) {
    let needle_owned = needle.to_string();
    let hits: Vec<String> = h
        .query_all_by(move |n: &Node<'_>| {
            n.label().map(|l| l.contains(&needle_owned)).unwrap_or(false)
                || n.value().map(|v| v.contains(&needle_owned)).unwrap_or(false)
        })
        .map(|n| format!("{:?} {:?}/{:?}", n.role(), n.label(), n.value()))
        .collect();
    assert!(
        hits.is_empty(),
        "no-plaintext violation ({ctx_msg}): {} node(s) expose the secret: {hits:#?}",
        hits.len()
    );
}

/// Close the (single) open egui popup. Ratified popup-close discipline for
/// AccessKit-driven option clicks: an AccessKit `Action::Click` has no
/// pointer position, so egui's `clicked_elsewhere()` close path never fires
/// and the popup would linger into the NEXT shot (caught visually in the
/// first spike render). egui closes any open popup on Escape
/// (`popup.rs:453` in egui 0.31), which is pointer-free and deterministic.
fn close_popup(h: &mut Harness<'static, MnemonicGuiApp>) {
    h.press_key(egui::Key::Escape);
    h.run();
}

/// Select a subcommand through the REAL ComboBox POPUP (S2/S5 exit
/// criterion): click the combo (unique by role+label — `from_label` labels
/// it "subcommand"), settle (the popup renders on its own egui layer), click
/// the option row by its human_name, settle, Escape-close the popup.
fn combo_select_subcommand(
    h: &mut Harness<'static, MnemonicGuiApp>,
    human_name: &'static str,
    expect_name: &str,
) {
    h.get_by_role_and_label(Role::ComboBox, "subcommand").click();
    h.run();
    h.get_by_label(human_name).click();
    h.run();
    close_popup(h);
    let active = h
        .state()
        .active_subcommand
        .get(&CliTab::Mnemonic)
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        active, expect_name,
        "combo popup drive must land the subcommand selection"
    );
}

/// SAME-FRAME-COMPLETION tripwire (SPEC §3.1b / §6.5, m1 single-step
/// semantics): deliver the ALREADY-QUEUED click in exactly ONE
/// `harness.step()`, then assert `last_run` landed BEFORE any further
/// stepping. Any future async runner fails HERE, by name — not as a
/// confusing corpus-wide pixel diff.
fn step_once_and_assert_same_frame_completion(
    h: &mut Harness<'static, MnemonicGuiApp>,
    what: &str,
) {
    h.step(); // exactly ONE frame: the click frame
    assert!(
        h.state().last_run.is_some(),
        "SAME-FRAME-COMPLETION violated ({what}): runner must complete in the Run-click \
         frame — populated-pane contract, SPEC §6.5; any async-runner change is a \
         USER-decision downgrade"
    );
}

/// Per-shot log row for the S4 budget table.
struct ShotLog {
    name: String,
    render_ms: u128,
    png_bytes: u64,
}

fn take_snapshot(
    h: &mut Harness<'static, MnemonicGuiApp>,
    name: &str,
    opts: &SnapshotOptions,
    logs: &mut Vec<ShotLog>,
    failures: &mut Vec<String>,
) {
    let t = Instant::now();
    let res = h.try_snapshot_options(name, opts);
    let render_ms = t.elapsed().as_millis();
    if let Err(e) = res {
        failures.push(format!("{name}: {e}"));
    }
    // Prefer the committed baseline size; fall back to the fresh .new.png.
    let dir = manifest_path(SNAPSHOT_SUBDIR);
    let png_bytes = std::fs::metadata(dir.join(format!("{name}.png")))
        .or_else(|_| std::fs::metadata(dir.join(format!("{name}.new.png"))))
        .map(|m| m.len())
        .unwrap_or(0);
    logs.push(ShotLog {
        name: name.to_string(),
        render_ms,
        png_bytes,
    });
}

/// Render the settled frame and return the raw RGBA bytes (in-process
/// byte-identity evidence: two independent harnesses driven identically must
/// produce IDENTICAL buffers).
fn raw_rgba(h: &mut Harness<'static, MnemonicGuiApp>) -> Vec<u8> {
    h.render().expect("render").into_raw()
}

// ─── the spike ──────────────────────────────────────────────────────────────

#[test]
fn gui_example_p0_spike() {
    if std::env::var("GUI_TUTORIAL_SPIKE").as_deref() != Ok("1") {
        eprintln!(
            "SPIKE-SKIP: GUI_TUTORIAL_SPIKE != 1 — gui_example_p0_spike skipped \
             (needs a software rasterizer + the pinned `mnemonic` on $PATH)"
        );
        return;
    }

    // ── adapter guard (the A1 pattern from gui_form_snapshots.rs) ──
    let info = {
        let render_state =
            egui_kittest::wgpu::create_render_state(egui_kittest::wgpu::default_wgpu_setup());
        render_state.adapter.get_info()
    };
    eprintln!("SPIKE-ADAPTER: {info:?}");
    assert_eq!(
        info.device_type,
        eframe::wgpu::DeviceType::Cpu,
        "spike renders MUST come from a software rasterizer — got {info:?}"
    );

    // ── pinned-tier-version-gate — BEFORE any render (SPEC §3.1b) ──
    // Bite demo: the CURRENT schema constant is the v0.74.0 tier while the
    // spike tier is v0.75.0 — probing the real binary against the schema
    // constant MUST fail. This is exactly the wrong-tier scenario the gate
    // exists for (an honest-looking `Pinned:` label over a wrong binary).
    let schema_tier = mnemonic_gui::schema::mnemonic::SCHEMA.pinned_version;
    assert_ne!(
        schema_tier, SPIKE_TIER_MNEMONIC,
        "bite demo precondition: the schema constant ({schema_tier}) must differ from \
         the spike tier until P1 bumps the pin"
    );
    let bite = pinned_tier_version_gate("mnemonic", schema_tier);
    assert!(
        bite.is_err(),
        "pinned-tier-version-gate BITE DEMO failed to fire: probing the local binary \
         against the stale schema tier {schema_tier:?} must mismatch"
    );
    eprintln!("SPIKE-GATE-BITE (deliberate wrong-tier probe): {}", bite.unwrap_err());
    // The real gate: the spike tier must be what's on $PATH — hard-fail else.
    match pinned_tier_version_gate("mnemonic", SPIKE_TIER_MNEMONIC) {
        Ok(v) => eprintln!("SPIKE-GATE-OK: {v}"),
        Err(e) => panic!("{e}"),
    }

    // ── cwd pin to the fixture dir (SPEC §6 item 6) ──
    let fixture_dir = manifest_path("tests/spike_fixtures");
    std::env::set_current_dir(&fixture_dir).expect("pin cwd to fixture dir");
    let descriptor = std::fs::read_to_string("policy.desc")
        .expect("read policy.desc")
        .trim()
        .to_string();

    let opts = SnapshotOptions::new().output_path(manifest_path(SNAPSHOT_SUBDIR));
    let mut logs: Vec<ShotLog> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    s1_whole_window(&opts, &mut logs, &mut failures);
    s2i_export_wallet_live_click(&opts, &mut logs, &mut failures);
    s2ii_j1_secret_modal(&opts, &mut logs, &mut failures);
    s2iii_refusal_probe(&opts, &mut logs, &mut failures);
    let chunks = s5_chain_bundle_json(&descriptor);
    s3_s4_restore_scroll_and_sizing(&chunks, &opts, &mut logs, &mut failures);

    // ── S4 budget table ──
    eprintln!("SPIKE-SHOT-TABLE: name render_ms png_bytes");
    let mut total: u64 = 0;
    for l in &logs {
        eprintln!("SPIKE-SHOT: {} {} {}", l.name, l.render_ms, l.png_bytes);
        total += l.png_bytes;
    }
    eprintln!(
        "SPIKE-SHOT-TOTAL: {} shots, {} bytes ({:.2} MiB); 51-shot projection at the \
         mean: {:.2} MiB",
        logs.len(),
        total,
        total as f64 / (1024.0 * 1024.0),
        (total as f64 / logs.len() as f64) * 51.0 / (1024.0 * 1024.0),
    );

    assert!(
        failures.is_empty(),
        "spike snapshot failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ─── S1 — whole-window render determinism ───────────────────────────────────

fn s1_whole_window(opts: &SnapshotOptions, logs: &mut Vec<ShotLog>, failures: &mut Vec<String>) {
    let mut h = app_harness(SIZE_A);

    // The whole window is genuinely there: tab strip, pinned line, combo,
    // output panel placeholder, action bar.
    for tab in ["mnemonic", "md", "ms", "mk"] {
        assert!(
            h.query_all_by_label(tab).next().is_some(),
            "tab strip: {tab} button missing"
        );
    }
    assert!(h.query_all_by_label("Pinned:").next().is_some(), "Pinned: line missing");
    assert!(
        h.query_all(by().role(Role::ComboBox).label("subcommand")).next().is_some(),
        "subcommand ComboBox missing"
    );
    assert!(
        h.query_all_by_label("(no run yet)").next().is_some(),
        "output panel placeholder missing"
    );
    assert!(h.query_all_by_label("Run").next().is_some(), "Run button missing");

    // Fresh-app demo seed is the deterministic baseline (SPEC §6 item 3):
    // mnemonic:bundle pre-filled + one EMPTY Xpub slot row.
    let seed = &h.state().form_state["mnemonic:bundle"];
    assert_eq!(seed.slots.rows.len(), 1, "demo seed: one slot row");
    assert_eq!(seed.slots.rows[0].subkey, SlotSubkey::Xpub, "demo seed: Xpub subkey");
    assert_eq!(seed.slots.rows[0].value, "", "demo seed: empty slot value");

    take_snapshot(&mut h, "s1-freshapp-920x720", opts, logs, failures);

    // In-process byte-identity: an independent, identically-constructed
    // harness must render the IDENTICAL RGBA buffer.
    let px1 = raw_rgba(&mut h);
    let mut h2 = app_harness(SIZE_A);
    let px2 = raw_rgba(&mut h2);
    assert_eq!(px1, px2, "S1: two same-env whole-window renders must be byte-identical");
    eprintln!("SPIKE-S1: whole-window render byte-identical across two harnesses ({} bytes)", px1.len());

    // S4 datapoint: the same fresh app at candidate B.
    let mut hb = app_harness(SIZE_B);
    take_snapshot(&mut hb, "s1-freshapp-1280x900", opts, logs, failures);
}

// ─── S2(i) — non-secret live Run click (export-wallet, template mode) ──────

/// Public single-sig xpub (from the tracked vault fixture / Examples.md —
/// the S0 bip84 account-0 xpub, world-known test-vector material).
const SLOT_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// SPIKE FINDING (reported; drives the pilot shape): the spec's S2(i) named
/// "export-wallet with `--descriptor-file`", but (a) export-wallet has NO
/// `--descriptor-file` flag (that's bundle/verify-bundle — recon-C cite
/// 851 is VERIFY_BUNDLE_FLAGS), and (b) the export-wallet `--descriptor`
/// TEXT arm is UNREACHABLE in the production GUI: `render_with_dispatch`
/// materializes `--template = Dropdown(opts[0] = "bip44")` on first render
/// (no schema default ⇒ `is_at_default` false ⇒ it EMITS), and
/// `conditional::export_wallet` then Disables `--descriptor` (mutex) with
/// no way to un-set the template (TEMPLATES has no "" option). The
/// J2/J3/J4 canonicalise/BSMS steps ride that arm — P1 must resolve
/// (GUI-side unset option or conditional refinement) before those chapters.
/// The S2 mechanics pilot therefore uses the REACHABLE template+slot mode.
fn s2i_export_wallet_live_click(
    opts: &SnapshotOptions,
    logs: &mut Vec<ShotLog>,
    failures: &mut Vec<String>,
) {
    let drive = |h: &mut Harness<'static, MnemonicGuiApp>, snap: bool,
                 opts: &SnapshotOptions,
                 logs: &mut Vec<ShotLog>,
                 failures: &mut Vec<String>| {
        // ComboBox POPUP drive — explicit S2 exit criterion (R0 M2i). The
        // popup renders on a separate egui layer inside the whole window.
        h.get_by_role_and_label(Role::ComboBox, "subcommand").click();
        h.run();
        assert!(
            h.query_all_by_label("Export Wallet (watch-only)").next().is_some(),
            "combo popup must be open with the export-wallet option visible"
        );
        if snap {
            take_snapshot(h, "s2-combo-popup-920x720", opts, logs, failures);
        }
        h.get_by_label("Export Wallet (watch-only)").click();
        h.run();
        close_popup(h);
        assert_eq!(
            h.state().active_subcommand[&CliTab::Mnemonic], "export-wallet",
            "popup option click must select the subcommand"
        );

        // --template = bip84 via the (unlabelled) flag dropdown popup —
        // row-anchored lookup, option row by exact label.
        on_row_of(h, "--template", Role::ComboBox, 0).click();
        h.run();
        h.get_by_label("bip84").click();
        h.run();
        close_popup(h);

        // One cosigner slot: `+ Add slot` then type the PUBLIC xpub into the
        // row's value editor (plain TextInput — xpub is NOT in
        // SECRET_SLOT_SUBKEYS, masked-by-construction taxonomy check below).
        h.get_by_label("+ Add slot").click();
        h.run();
        on_row_of(h, "@", Role::TextInput, 0).type_text(SLOT_XPUB);
        h.run();
        h.run(); // settle write-back (buffer lands at frame end)

        // --format = descriptor (explicit dropdown drive).
        on_row_of(h, "--format", Role::ComboBox, 0).click();
        h.run();
        h.get_by_label("descriptor").click();
        h.run();
        close_popup(h);

        // Live Run click — m1 single-step semantics.
        assert!(h.state().last_run.is_none(), "pre-click: no run yet");
        h.get_by_label("Run").click();
        assert!(
            h.state().last_run.is_none(),
            "click queued but not yet stepped: last_run must still be None \
             (proves the same-frame assertion is not vacuous)"
        );
        step_once_and_assert_same_frame_completion(h, "export-wallet run");
        h.run(); // settle for the shot

        let run = h.state().last_run.as_ref().expect("last_run");
        assert_eq!(run.argv[0], "mnemonic", "argv echo must be the bare CLI name");
        assert_eq!(
            run.exit_code,
            Some(0),
            "export-wallet must exit 0; argv={:?} stderr={}",
            run.argv,
            String::from_utf8_lossy(&run.stderr)
        );
        // The slot token spawns in argv and is NOT display-masked (xpub is a
        // public subkey — the SECRET_SLOT_SUBKEYS taxonomy boundary).
        let slot_idx = run
            .argv
            .iter()
            .position(|t| t.starts_with("@0.xpub="))
            .expect("xpub slot token in argv");
        assert!(!run.mask[slot_idx], "public xpub slot token must NOT be masked");
        let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
        assert!(
            stdout.starts_with("wpkh([00000000/84'/0'/0']xpub6CatWdiZiodm"),
            "canonical bip84 watch-only descriptor expected on stdout, got: {}",
            &stdout[..stdout.len().min(60)]
        );
        if snap {
            take_snapshot(h, "s2-exportwallet-run-920x720", opts, logs, failures);
        }
        stdout
    };

    let mut h1 = app_harness(SIZE_A);
    let out1 = drive(&mut h1, true, opts, logs, failures);
    let px1 = raw_rgba(&mut h1);

    // Double-run determinism (S2 exit): independent harness, same drive →
    // identical RunResult bytes AND identical rendered pixels.
    let mut h2 = app_harness(SIZE_A);
    let out2 = drive(&mut h2, false, opts, logs, failures);
    let px2 = raw_rgba(&mut h2);
    assert_eq!(out1, out2, "S2(i): double-run stdout must be byte-identical");
    assert_eq!(px1, px2, "S2(i): double-run rendered pixels must be byte-identical");
    eprintln!("SPIKE-S2i: live-click populated pane deterministic (stdout {} B)", out1.len());

    // S4 datapoint: the same populated-pane state at candidate B.
    let mut hb = app_harness(SIZE_B);
    let _ = drive(&mut hb, false, opts, logs, failures);
    take_snapshot(&mut hb, "s4-exportwallet-run-1280x900", opts, logs, failures);
}

// ─── S2(iii) — refusal-step probe: non-zero exit badge + stderr pane ───────

/// SPIKE FINDING #2 demo doubling as the J3/J4 refusal-step mechanics pilot:
/// `conditional::bundle` Disables `--template` for `--descriptor` but NOT
/// for `--descriptor-file`, so the demo-seeded `--template=bip84` EMITS
/// alongside a typed `--descriptor-file` and the CLI refuses (exit 2,
/// "mutually exclusive") — the exact `bundle --descriptor-file` shape of
/// the J2/J3 engrave steps. The refusal renders as a REAL non-zero exit
/// badge + stderr block in the pane — the mechanics every refusal step in
/// the tutorial needs (real non-zero exits, SPEC §12.2).
fn s2iii_refusal_probe(
    opts: &SnapshotOptions,
    logs: &mut Vec<ShotLog>,
    failures: &mut Vec<String>,
) {
    let mut h = app_harness(SIZE_A);
    // Fresh app is on mnemonic:bundle with the demo seed (template bip84).
    on_row_of(&h, "--descriptor-file", Role::TextInput, 0).type_text("policy.desc");
    h.run();
    h.run();
    assert!(h.state().last_run.is_none(), "pre-click: no run yet");
    h.get_by_label("Run").click();
    step_once_and_assert_same_frame_completion(&mut h, "bundle refusal (conflict shape)");
    h.run();
    {
        let run = h.state().last_run.as_ref().expect("last_run");
        assert_eq!(
            run.exit_code,
            Some(2),
            "descriptor-file + seeded template must be refused by the CLI; argv={:?}",
            run.argv
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains("mutually exclusive"),
            "refusal diagnostic expected on stderr: {stderr}"
        );
    }
    take_snapshot(&mut h, "s2-bundle-refusal-920x720", opts, logs, failures);
    eprintln!(
        "SPIKE-S2iii: refusal-step mechanics GREEN (real exit-2 badge + stderr pane); \
         finding: --descriptor-file lacks the template mutex GUI-side"
    );
}

// ─── S2(ii) — secret path: demo-seed flip, masked entry, modal, live run ────

fn s2ii_j1_secret_modal(
    opts: &SnapshotOptions,
    logs: &mut Vec<ShotLog>,
    failures: &mut Vec<String>,
) {
    let mut h = app_harness(SIZE_A);

    // S5 (R0 M2ii): start from the DEMO SEED — flip the seeded Xpub slot
    // row's subkey to `phrase` through the real subkey ComboBox popup. The
    // "@" label anchors the slot row (unique in the bundle form).
    assert_eq!(
        h.state().form_state["mnemonic:bundle"].slots.rows[0].subkey,
        SlotSubkey::Xpub,
        "baseline: the demo-seeded row is an Xpub row"
    );
    on_row_of(&h, "@", Role::ComboBox, 0).click();
    h.run();
    h.get_by_label("phrase").click();
    h.run();
    close_popup(&mut h);
    assert_eq!(
        h.state().form_state["mnemonic:bundle"].slots.rows[0].subkey,
        SlotSubkey::Phrase,
        "S5: the seeded row's subkey must flip to phrase via the popup drive"
    );

    // The row's value editor is now a PASSWORD field (masked-by-construction,
    // SECRET_SLOT_SUBKEYS taxonomy). Type S0 into it.
    on_row_of(&h, "@", Role::PasswordInput, 0).type_text(S0_PHRASE);
    h.run();
    h.run();
    assert_eq!(
        h.state().form_state["mnemonic:bundle"].slots.rows[0].value,
        S0_PHRASE,
        "typed phrase must land in the slot row value"
    );

    // §7 guards: the masked Preview carries the mask sentinel, and NO node in
    // the whole tree exposes the phrase (egui masks password values before
    // AccessKit, so this is global).
    let preview = h
        .query_all_by_label_contains("Preview:")
        .next()
        .expect("Preview label");
    let ptext = preview.label().or_else(|| preview.value()).unwrap_or_default();
    assert!(ptext.contains("••••"), "masked Preview must carry the •••• sentinel: {ptext}");
    assert_no_plaintext(&h, S0_PHRASE, "filled form, pre-run");
    assert_no_plaintext(&h, "abandon", "filled form, pre-run (word probe)");

    // Run click #1 → the CONFIRM MODAL must appear in the SAME frame (the
    // pending flag is set in the CentralPanel closure; the modal renders
    // later in the same update pass). Execution is deferred: last_run stays
    // None until the modal's own Run.
    h.get_by_label("Run").click();
    h.step(); // ONE frame: click + modal render
    assert!(
        h.state().last_run.is_none(),
        "secret path: the first Run click must DEFER to the modal, not spawn"
    );
    h.run(); // settle (modal open)
    assert!(
        h.query_all_by_label("Confirm secret-bearing run").next().is_some(),
        "confirm modal must be visible"
    );

    // Label-collision demo (S5 lookup discipline): with the modal open there
    // are exactly TWO `Run` buttons — the action bar's and the modal's. An
    // unscoped get_by_label would be ambiguous; the ratified discipline is
    // window-subtree scoping.
    let runs: Vec<Node<'_>> = h.query_all(by().role(Role::Button).label("Run")).collect();
    assert_eq!(
        runs.len(),
        2,
        "expected exactly 2 Run buttons with the modal open (action bar + modal)"
    );

    // §7(b): the modal's own token list renders masked — global tree probe
    // again, THEN the modal shot.
    assert_no_plaintext(&h, "abandon", "confirm modal open");
    take_snapshot(&mut h, "s2-j1-modal-920x720", opts, logs, failures);

    // Modal-Run click: scope the query to the modal WINDOW subtree.
    let modal_run = modal_scoped_run_button(&h);
    modal_run.click();
    step_once_and_assert_same_frame_completion(&mut h, "J1 bundle (modal path)");
    h.run(); // settle for the shot

    {
        let run = h.state().last_run.as_ref().expect("last_run");
        assert_eq!(run.argv[0], "mnemonic");
        assert_eq!(run.exit_code, Some(0), "J1 bundle must exit 0");
        // The REAL argv carries the cleartext slot token (that is what spawns) —
        // and its mask bit must be set so every display channel renders ••••.
        let slot_idx = run
            .argv
            .iter()
            .position(|t| t.starts_with("@0.phrase="))
            .expect("slot token in argv");
        assert!(run.mask[slot_idx], "slot token must be display-masked");
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            stdout.starts_with("# ms1 (entropy, BCH-checksummed)"),
            "J1 card set expected on stdout, got: {}",
            &stdout[..stdout.len().min(60)]
        );
    }
    // Post-run: pane shows masked argv + real cards; still no plaintext
    // phrase anywhere in the tree.
    assert_no_plaintext(&h, "abandon", "populated pane, post-run");
    take_snapshot(&mut h, "s2-j1-run-920x720", opts, logs, failures);
    eprintln!("SPIKE-S2ii: secret two-click modal path GREEN (masked at every channel)");
}

/// Find the modal window node and scope the Run-button query to its subtree
/// (the ratified label-collision discipline). Falls back loudly if the
/// window node shape ever changes.
fn modal_scoped_run_button<'t>(h: &'t Harness<'static, MnemonicGuiApp>) -> Node<'t> {
    let window = h
        .query_all(by().role(Role::Window).label("Confirm secret-bearing run"))
        .next()
        .unwrap_or_else(|| {
            panic!(
                "modal Window node not found by role+title; tree head: {:#?}",
                h.node()
            )
        });
    window
        .query_all(by().role(Role::Button).label("Run"))
        .next()
        .expect("Run button inside the modal subtree")
}

// ─── S5 chaining leg — GUI-driven `bundle --descriptor … --json` run ────────

/// Drive a real `--json` bundle run through the window (the `shots: 0`
/// production path of SPEC §3.1b/M1) and parse the md1 chunks out of the
/// captured RunResult — the J5 chaining mechanic. Also the Boolean
/// (checkbox) drive datapoint.
///
/// NOTE (spike finding, reported): this uses the `--descriptor` TEXT field,
/// not `--descriptor-file` — `conditional::bundle` Disables `--template`
/// only for `--descriptor`, so the demo-seeded `--template=bip84` is
/// argv-suppressed. With `--descriptor-file` the template WOULD emit and the
/// CLI rejects the pair (exit 2) — a conditional-coverage gap the tutorial
/// manifest must route around (or P1 fixes GUI-side).
fn s5_chain_bundle_json(descriptor: &str) -> Vec<String> {
    let mut h = app_harness(SIZE_A);
    // Fresh app is already on mnemonic:bundle (demo seed).
    on_row_of(&h, "--descriptor", Role::TextInput, 0).type_text(descriptor);
    h.run();
    h.run();
    on_row_of(&h, "--json", Role::CheckBox, 0).click();
    h.run();

    h.get_by_label("Run").click();
    step_once_and_assert_same_frame_completion(&mut h, "bundle --json (chaining leg)");
    h.run();

    let run = h.state().last_run.as_ref().expect("last_run");
    assert_eq!(run.exit_code, Some(0), "bundle --json must exit 0");
    let v: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("bundle --json stdout must parse");
    let chunks: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|c| c.as_str().expect("chunk").to_string())
        .collect();
    assert!(
        chunks.len() >= 10,
        "vault bundle must yield >=10 md1 chunks for the S3 tall form (got {})",
        chunks.len()
    );
    eprintln!("SPIKE-S5-CHAIN: GUI-run bundle --json yielded {} md1 chunks", chunks.len());
    chunks
}

// ─── S3 + S4 — scroll mechanics on the tall restore form + sizing ──────────

fn s3_s4_restore_scroll_and_sizing(
    chunks: &[String],
    opts: &SnapshotOptions,
    logs: &mut Vec<ShotLog>,
    failures: &mut Vec<String>,
) {
    let drive = |size: [f32; 2]| -> Harness<'static, MnemonicGuiApp> {
        let mut h = app_harness(size);
        combo_select_subcommand(
            &mut h,
            "Restore (re-derive a wallet export from a source)",
            "restore",
        );
        // Two REAL driven rows (S5 exit): the md1 header's own `+ add`
        // button (row-anchored — other repeating flags have their own),
        // then type into the newly-appeared empty row input, block-bounded
        // between the --md1 header and the --cosigner header.
        for chunk in chunks.iter().take(2) {
            md1_add_row(&mut h);
            md1_type_into_last_empty_row(&mut h, chunk);
        }
        {
            let st = h
                .state_mut()
                .form_state
                .get_mut("mnemonic:restore")
                .expect("restore form state");
            let driven: Vec<&(String, FlagValue)> =
                st.values.iter().filter(|(k, _)| k == "--md1").collect();
            assert_eq!(driven.len(), 2, "two widget-driven md1 rows");
            for (i, (_, v)) in driven.iter().enumerate() {
                assert_eq!(
                    v,
                    &FlagValue::Text(chunks[i].clone()),
                    "driven md1 row {i} must round-trip the typed chunk"
                );
            }
            // Sizing prep (declared state-seed, NOT a drive claim): rows
            // 3..=10 seeded directly so the form is journey-tall.
            for chunk in chunks.iter().skip(2).take(8) {
                st.values
                    .push(("--md1".to_string(), FlagValue::Text(chunk.clone())));
            }
        }
        h.run();
        h
    };

    // ── S3 at SIZE_A: top shot, wheel scroll, scrolled shot ──
    let mut h = drive(SIZE_A);

    // Driven-field visibility contract prototype (SPEC §5.4, machine-checkable):
    // every WIDGET-DRIVEN field must intersect the viewport in at least one
    // captured offset — here the two driven md1 rows at the TOP offset.
    {
        let rows = md1_row_inputs(&h);
        for (i, chunk) in chunks.iter().take(2).enumerate() {
            let r = rows
                .iter()
                .find(|n| n.value().as_deref() == Some(chunk.as_str()))
                .unwrap_or_else(|| panic!("driven md1 row {i} not found by value"))
                .raw_bounds()
                .unwrap();
            assert!(
                r.y1 > 0.0 && r.y0 < SIZE_A[1] as f64,
                "driven md1 row {i} must intersect the viewport in the top shot \
                 (rect y0={} y1={})",
                r.y0,
                r.y1
            );
        }
        eprintln!("SPIKE-S3: driven-field viewport-visibility assertion GREEN (top offset)");
    }

    take_snapshot(&mut h, "s3-restore-top-920x720", opts, logs, failures);

    let y_before = rect_of(&h.get_by_label("--md1")).1;
    wheel_scroll_form(&mut h, -520.0);
    let y_after = rect_of(&h.get_by_label("--md1")).1;
    assert!(
        y_after < y_before - 100.0,
        "MouseWheel scroll must move the form content up (before y={y_before}, after y={y_after})"
    );
    eprintln!("SPIKE-S3: wheel scroll moved --md1 header y {y_before} -> {y_after}");
    take_snapshot(&mut h, "s3-restore-scrolled-920x720", opts, logs, failures);
    let px1 = raw_rgba(&mut h);

    // Driven-field visibility contract prototype (SPEC §5.4): each driven
    // md1 row must intersect the viewport in at least one captured offset.
    // (At the top offset the rows are visible; assert against the scrolled
    // state's node bounds recorded above.)

    // Reproducibility: identical drive + identical wheel sequence in a fresh
    // harness → identical pixels.
    let mut h2 = drive(SIZE_A);
    wheel_scroll_form(&mut h2, -520.0);
    let px2 = raw_rgba(&mut h2);
    assert_eq!(
        px1, px2,
        "S3: scrolled render must be byte-identical across independent harnesses"
    );
    eprintln!("SPIKE-S3: scroll state pixel-reproducible (mechanism ii — injected MouseWheel)");

    // ── S4 at SIZE_B: same filled form ──
    let mut hb = drive(SIZE_B);
    take_snapshot(&mut hb, "s4-restore-top-1280x900", opts, logs, failures);
    // Measure whether the tall form still overflows at B: compare the last
    // md1 row's bottom edge against the viewport height.
    let rows: Vec<Node<'_>> = md1_row_inputs(&hb);
    let last_bottom = rows.last().map(|n| n.raw_bounds().unwrap().y1).unwrap_or(0.0);
    eprintln!(
        "SPIKE-S4: at 1280x900 the last md1 row bottom edge sits at y={last_bottom:.0} \
         (viewport height {}); >viewport means scrolling still required",
        SIZE_B[1]
    );
}

/// Click the `+ add` button that belongs to the `--md1` header row.
fn md1_add_row(h: &mut Harness<'static, MnemonicGuiApp>) {
    let header = h.get_by_label("--md1");
    let (hx0, hy0, _hx1, hy1) = rect_of(&header);
    let adds: Vec<Node<'_>> = h
        .query_all(by().role(Role::Button).label("+ add"))
        .filter(|n| {
            let Some(r) = n.raw_bounds() else { return false };
            let c = (r.y0 + r.y1) / 2.0;
            c >= hy0 - 2.0 && c <= hy1 + 2.0 && r.x0 >= hx0
        })
        .collect();
    assert_eq!(
        adds.len(),
        1,
        "exactly one `+ add` on the --md1 header row (row-anchored disambiguation; \
         the form has several repeating flags each with its own `+ add`)"
    );
    adds[0].click();
    h.run();
}

/// The md1 row TextInputs: block-bounded between the `--md1` header and the
/// `--cosigner` header (the next flag's label), sorted top-to-bottom.
fn md1_row_inputs<'t>(h: &'t Harness<'static, MnemonicGuiApp>) -> Vec<Node<'t>> {
    let top = rect_of(&h.get_by_label("--md1")).3; // header bottom
    let bottom = rect_of(&h.get_by_label("--cosigner")).1; // next header top
    let mut rows: Vec<Node<'t>> = h
        .query_all(by().role(Role::TextInput))
        .filter(|n| {
            let Some(r) = n.raw_bounds() else { return false };
            let c = (r.y0 + r.y1) / 2.0;
            c > top && c < bottom
        })
        .collect();
    rows.sort_by(|a, b| {
        a.raw_bounds()
            .unwrap()
            .y0
            .partial_cmp(&b.raw_bounds().unwrap().y0)
            .unwrap()
    });
    rows
}

/// Type into the last (newly-added, empty) md1 row input.
fn md1_type_into_last_empty_row(h: &mut Harness<'static, MnemonicGuiApp>, text: &str) {
    {
        let rows = md1_row_inputs(h);
        let target = rows
            .iter()
            .rev()
            .find(|n| n.value().unwrap_or_default().is_empty())
            .expect("an empty md1 row input after + add");
        target.type_text(text);
    }
    h.run();
    h.run();
}

/// S3 mechanism (ii): park the pointer over the central form area and inject
/// a MouseWheel event (egui routes scroll to the hovered ScrollArea), then
/// run to quiescence (smooth-scroll animation settles deterministically).
/// Mechanism (i) — AccessKit scroll actions — is DEAD in egui 0.31: the
/// crate handles no ScrollIntoView/ScrollUp/SetScrollOffset requests
/// (grep-verified), so (ii) is the primary and the vertical_scroll_offset
/// seam stays the guaranteed fallback (iii), unused.
fn wheel_scroll_form(h: &mut Harness<'static, MnemonicGuiApp>, delta_y: f32) {
    // Hover a point inside the form's scroll region: the --md1 header center.
    let (x0, y0, x1, y1) = rect_of(&h.get_by_label("--md1"));
    let pos = egui::pos2(((x0 + x1) / 2.0) as f32, ((y0 + y1) / 2.0) as f32);
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, delta_y),
        modifiers: egui::Modifiers::default(),
    });
    h.run();
}
