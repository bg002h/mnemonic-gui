//! T6 — what the user sees (DESIGN §A7, §A9), and the form→source mapping.
//!
//! 1. **Form mapping:** every shape, filled into a real `FormState` for its
//!    real subcommand, is planned by the GUI's own `channels::plan` exactly as
//!    the pure planner plans the shape's sources (same refusal, or the same
//!    channel per source with the same payload bytes). The GUI's flag order
//!    may differ from the prototype's argv order (bundle, verify-bundle,
//!    silent-payment), which renumbers `MNEMONIC_GUI_S<i>` only.
//! 2. **Copy (§A7):** never a secret value or `--allow-argv-secret`; the
//!    spellings of the §A7 table; disabled for a refused plan, for a typed
//!    EnvRef-bound value holding CR/LF, and on Windows for a stdin binding.
//! 3. **Shell leg:** the GUI's OWN Copy text, run in bash, zsh and fish with
//!    the real `mnemonic`, equals argv-exact for the 11 typed passphrases
//!    (needs `MNEMONIC_BIN`; the one Linux job runs it).
//! 4. **The window:** Preview, binding lines and the confirm dialog contain no
//!    sentinel and do contain each binding with its provenance; a refusal
//!    disables Run and says why.

mod secret_channels_common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use mnemonic_gui::app::{AppState, CliTab};
use mnemonic_gui::app_window::MnemonicGuiApp;
use mnemonic_gui::form::channels::copy::{
    copy_commands, CopyState, MULTILINE_TOOLTIP, WINDOWS_STDIN_TOOLTIP,
};
use mnemonic_gui::form::channels::{
    self, channel_table, plan_sources, policy, EnvRule, PlanOptions, SourceValue,
};
use mnemonic_gui::form::secret_widget::SecretLineEdit;
use mnemonic_gui::form::slot_editor::{SlotRow, SlotState, SlotSubkey};
use mnemonic_gui::path_detect::Detected;
use mnemonic_gui::schema::{self, FlagKind, FlagValue, FormState, Schema, SubcommandSchema};
use secret_channels_common::{env_of, shapes, Shape, PLATFORMS};

fn schema_of(cli: &str) -> &'static Schema {
    match cli {
        "mnemonic" => &schema::mnemonic::SCHEMA,
        "ms" => &schema::ms::SCHEMA,
        "md" => &schema::md::SCHEMA,
        _ => &schema::mk::SCHEMA,
    }
}

fn sub_of(schema: &'static Schema, name: &str) -> &'static SubcommandSchema {
    schema
        .subcommands
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("{name}"))
}

fn subkey(s: &str) -> SlotSubkey {
    match s {
        "phrase" => SlotSubkey::Phrase,
        "seedqr" => SlotSubkey::Seedqr,
        "entropy" => SlotSubkey::Entropy,
        "ms1" => SlotSubkey::Ms1,
        "xprv" => SlotSubkey::Xprv,
        "wif" => SlotSubkey::Wif,
        other => panic!("subkey {other}"),
    }
}

/// A shape filled into a real form: the public `pre` flags as values, each
/// source in its widget (secret Text rows, slot rows, composites, node Text,
/// secret positional rows).
fn form_state(
    sh: &Shape,
    values: &[Vec<String>],
) -> (&'static Schema, &'static SubcommandSchema, FormState) {
    let schema = schema_of(&sh.cli);
    let sub = sub_of(schema, &sh.sub.join("-"));
    let mut st = FormState::default();
    let mut k = 0;
    while k < sh.pre.len() {
        let name = sh.pre[k].clone();
        let flag = sub
            .flags
            .iter()
            .find(|f| f.name == name)
            .unwrap_or_else(|| panic!("{} {name}", sh.name));
        let mut vals = Vec::new();
        k += 1;
        while k < sh.pre.len() && !sh.pre[k].starts_with("--") {
            vals.push(sh.pre[k].clone());
            k += 1;
        }
        if vals.is_empty() {
            st.values.push((name.clone(), FlagValue::Boolean(true)));
        }
        for v in vals {
            let fv = match &flag.kind {
                FlagKind::Dropdown(_) => FlagValue::Dropdown(v),
                FlagKind::Number { .. } => FlagValue::Number(v.parse().unwrap()),
                FlagKind::Path { .. } => FlagValue::Path(v),
                _ => FlagValue::Text(v),
            };
            st.values.push((name.clone(), fv));
        }
    }
    let mut slots = SlotState::default();
    for (s, v) in sh.sources.iter().zip(values) {
        match s.form.as_str() {
            "node" if s.flag.as_deref() == Some("--slot") => {
                let (idx, sk) = s
                    .prefix
                    .trim_start_matches('@')
                    .trim_end_matches('=')
                    .split_once('.')
                    .unwrap();
                slots.rows.push(SlotRow {
                    index: idx.parse().unwrap(),
                    subkey: subkey(sk),
                    value: v[0].clone(),
                });
            }
            "node" => {
                let flag = s.flag.clone().unwrap();
                let node = s.prefix.trim_end_matches('=').to_string();
                let kind = &sub.flags.iter().find(|f| f.name == flag).unwrap().kind;
                let fv = match kind {
                    FlagKind::NodeValueComposite(_) => FlagValue::NodeValueComposite {
                        node,
                        value: v[0].clone(),
                    },
                    _ => FlagValue::Text(format!("{}{}", s.prefix, v[0])),
                };
                st.values.push((flag, fv));
            }
            "value" => st
                .secret_widgets
                .entry(s.flag.clone().unwrap())
                .or_default()
                .push(SecretLineEdit::from_text(&v[0])),
            _ => {
                let pos = sub
                    .positional_args
                    .iter()
                    .find(|p| p.secret)
                    .expect("secret positional");
                let rows = st
                    .secret_widgets
                    .entry(format!("positional:{}", pos.name))
                    .or_default();
                for x in v {
                    rows.push(SecretLineEdit::from_text(x));
                }
            }
        }
    }
    st.slots = slots;
    (schema, sub, st)
}

fn values_of(sh: &Shape) -> Vec<Vec<String>> {
    sh.sources.iter().map(|s| s.values.clone()).collect()
}

type Row = (String, String, String, Option<String>, String);

fn rows_of(bindings: &[channels::Binding], resolved: &[SourceValue]) -> Vec<Row> {
    let mut v: Vec<Row> = bindings
        .iter()
        .zip(resolved)
        .map(|(b, r)| {
            let payload = match r {
                SourceValue::Group(xs) => {
                    xs.iter().map(|x| x.as_str()).collect::<Vec<_>>().join("\n")
                }
                SourceValue::One(x) => x.to_string(),
            };
            (
                b.key.clone(),
                format!("{:?}", b.kind),
                b.terminator.clone(),
                b.flag.clone(),
                payload,
            )
        })
        .collect();
    v.sort();
    v
}

#[test]
fn t6_every_shape_through_the_real_form_plans_as_the_pure_planner() {
    let none = env_of(&[]);
    let mut n = 0;
    let mut order_drift: Vec<String> = Vec::new();
    for sh in shapes() {
        let (schema, sub, st) = form_state(sh, &values_of(sh));
        // the form's sources are exactly the shape's
        let asm = channels::assemble(schema, sub, &st);
        // F-694: in the FORM's order, not sorted. The planner is order-dependent
        // by design (§A4.3 "first source in argv order"), so shapes.py must list
        // sources exactly as the form assembles them or the pure plan (T8, §A5)
        // describes a plan the GUI never makes.
        let got_keys: Vec<&str> = asm.sites.iter().map(|s| s.key.as_str()).collect();
        let want_keys: Vec<&str> = sh.sources.iter().map(|s| s.key.as_str()).collect();
        if got_keys != want_keys {
            order_drift.push(format!(
                "{}: form {got_keys:?}, shapes.py {want_keys:?}",
                sh.name
            ));
            continue;
        }
        for os in PLATFORMS {
            let pure = plan_sources(
                &sh.plan_sources(),
                channel_table(),
                policy(),
                os,
                &none,
                PlanOptions::default(),
            );
            let gui = channels::plan(schema, sub, &st, &none, os);
            match (pure, gui) {
                (Err(a), Err(b)) => assert_eq!(a.code, b.code, "{} {os}", sh.name),
                (Ok(a), Ok(b)) => {
                    let gb: Vec<channels::Binding> =
                        b.bindings.iter().map(|x| x.binding.clone()).collect();
                    // the GUI plan's resolved values are its sources' (typed here)
                    let gsrc = asm.sources();
                    let gres: Vec<SourceValue> = gsrc.into_iter().map(|s| s.value).collect();
                    assert_eq!(
                        rows_of(&a.bindings, &a.resolved),
                        rows_of(&gb, &gres),
                        "{} {os}",
                        sh.name
                    );
                    for bv in &b.bindings {
                        if let Some(e) = &bv.binding.env {
                            assert_eq!(
                                e,
                                &format!("MNEMONIC_GUI_S{}", bv.binding.source),
                                "{}: env name follows the GUI's own source index",
                                sh.name
                            );
                        }
                    }
                    // no secret byte on the private path's argv
                    if os == "linux" {
                        for s in &sh.sources {
                            for v in &s.values {
                                assert!(
                                    !b.argv.iter().any(|t| t.contains(v.as_str())),
                                    "{}: secret on argv",
                                    sh.name
                                );
                            }
                        }
                    }
                }
                (a, b) => panic!(
                    "{} {os}: pure {:?} vs gui {:?}",
                    sh.name,
                    a.map(|_| ()).map_err(|e| e.code),
                    b.map(|_| ()).map_err(|e| e.code)
                ),
            }
            n += 1;
        }
    }
    assert!(
        order_drift.is_empty(),
        "shapes.py lists sources in a different order from the form (the planner is \
         order-dependent, so the pure plan would not be the GUI's):\n{}",
        order_drift.join("\n")
    );
    assert_eq!(n, shapes().len() * 3);
}

// ── Copy (§A7) ────────────────────────────────────────────────────────────

const P: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PWSENT: &str = "PASSWORDsentinel9";

fn restore_state(from: &str, pw: &str) -> FormState {
    let mut st = FormState::from_pairs(vec![
        ("--from", FlagValue::Text(from.into())),
        ("--template", FlagValue::Dropdown("bip84".into())),
    ]);
    st.secret_widgets
        .insert("--passphrase".into(), vec![SecretLineEdit::from_text(pw)]);
    st
}

fn restore() -> (&'static Schema, &'static SubcommandSchema) {
    (
        &schema::mnemonic::SCHEMA,
        sub_of(&schema::mnemonic::SCHEMA, "restore"),
    )
}

fn ready(c: &CopyState) -> &str {
    c.text().unwrap_or_else(|| panic!("copy disabled: {c:?}"))
}

#[test]
fn t6_copy_typed_values_never_appear_and_use_the_a7_spellings() {
    let (sc, sub) = restore();
    let st = restore_state(&format!("phrase={P}"), PWSENT);
    let (posix, windows) = copy_commands(sc, sub, &st, &env_of(&[]));
    let t = ready(&posix);
    assert!(
        !t.contains(PWSENT) && !t.contains("abandon"),
        "a secret in Copy: {t}"
    );
    assert!(!t.contains("--allow-argv-secret"), "{t}");
    assert!(t.contains("# --from phrase=: IFS= read -rs MNEMONIC_GUI_S0; export MNEMONIC_GUI_S0   (fish: read -s -x --delimiter \\n MNEMONIC_GUI_S0)"), "{t}");
    assert!(
        t.contains("# stdin (--passphrase): type it, then Enter, then Ctrl-D"),
        "{t}"
    );
    assert!(
        t.lines().last().unwrap().starts_with("mnemonic restore "),
        "{t}"
    );
    assert!(
        t.contains("phrase=@env:MNEMONIC_GUI_S0") && t.contains("--passphrase-stdin"),
        "{t}"
    );
    // Windows cannot feed a pipe
    assert_eq!(windows, CopyState::Disabled(WINDOWS_STDIN_TOOLTIP.into()));
}

#[test]
fn t6_copy_env_provenance_spells_the_users_own_variable_or_printf() {
    let (sc, sub) = restore();
    let st = restore_state("phrase=@env:MY_SEED", "@env:MY_PW");
    let env = env_of(&[("MY_SEED", P), ("MY_PW", PWSENT)]);
    let (posix, _) = copy_commands(sc, sub, &st, &env);
    let t = ready(&posix);
    assert!(!t.contains(PWSENT) && !t.contains("abandon"), "{t}");
    assert!(
        t.contains("phrase=@env:MY_SEED"),
        "EnvRef + $VAR: the user's own @env: {t}"
    );
    // F-694: the expected spelling of the stdin-bound `$MY_PW` is DERIVED from
    // the measured table, not hand-kept (§A0 item 3). Under a verbatim (or no)
    // CLI `@env:` rule, `printf '%s<T>'` is exact; under any other rule (0.105.1:
    // strip-one-trailing-newline) Copy defers to the CLI's own `@env:MY_PW`.
    let last = t.lines().last().unwrap();
    let rule = channel_table()["mnemonic restore --passphrase"].cli_env_rule;
    if matches!(rule, None | Some(EnvRule::Verbatim)) {
        assert!(
            last.starts_with("printf '%s\\r\\n' \"$MY_PW\" | mnemonic restore "),
            "{rule:?}: {t}"
        );
    } else {
        assert!(
            last.starts_with("mnemonic restore ")
                && last.contains("--passphrase @env:MY_PW")
                && !t.contains("printf"),
            "{rule:?}: {t}"
        );
    }
}

#[test]
fn t6_copy_share_group_and_pipe_fd_recipes() {
    let ms = &schema::ms::SCHEMA;
    let mut st = FormState::default();
    st.secret_widgets.insert(
        "positional:shares".into(),
        vec![
            SecretLineEdit::from_text("ms12aaa"),
            SecretLineEdit::from_text("ms12bbb"),
        ],
    );
    let (posix, _) = copy_commands(ms, sub_of(ms, "combine"), &st, &env_of(&[]));
    let t = ready(&posix);
    assert!(
        t.contains("# stdin: paste each share on its own line, then Ctrl-D")
            && t.ends_with("ms combine -- -"),
        "{t}"
    );
    let mut st = FormState::default();
    st.secret_widgets.insert(
        "positional:shares".into(),
        vec![
            SecretLineEdit::from_text("@env:S1"),
            SecretLineEdit::from_text("@env:S2"),
        ],
    );
    let (posix, _) = copy_commands(
        ms,
        sub_of(ms, "combine"),
        &st,
        &env_of(&[("S1", "ms12aaa"), ("S2", "ms12bbb")]),
    );
    assert!(
        ready(&posix).ends_with("printf '%s\\n' \"$S1\" \"$S2\" | ms combine -- -"),
        "{posix:?}"
    );
    // ms verify phrase+ms1: the ms1 goes over a pipe fd → `--in <FILE>`
    let mut st = FormState::default();
    st.secret_widgets
        .insert("--phrase".into(), vec![SecretLineEdit::from_text(P)]);
    st.secret_widgets.insert(
        "positional:ms1".into(),
        vec![SecretLineEdit::from_text(
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        )],
    );
    let (posix, _) = copy_commands(ms, sub_of(ms, "verify"), &st, &env_of(&[]));
    let t = ready(&posix);
    assert!(
        t.contains("--in <FILE>") && t.contains("# FILE: a file holding the ms1 card"),
        "{t}"
    );
    assert!(
        !t.contains("ms10entr") && !t.contains("abandon") && !t.contains("/dev/fd/"),
        "{t}"
    );
}

#[test]
fn t6_copy_disabled_for_multiline_typed_envref_and_refusals() {
    let (sc, sub) = restore();
    // a typed phrase spanning lines rides EnvRef → Copy disabled (R3 Nm11)
    let st = restore_state("phrase=abandon\nabandon", PWSENT);
    let (posix, _) = copy_commands(sc, sub, &st, &env_of(&[]));
    assert_eq!(posix, CopyState::Disabled(MULTILINE_TOOLTIP.into()));
    // a refused plan disables both, with the refusal as the tooltip
    let st = restore_state(&format!("phrase={P}"), "-");
    let (posix, windows) = copy_commands(sc, sub, &st, &env_of(&[]));
    for c in [posix, windows] {
        match c {
            CopyState::Disabled(t) => assert!(t.contains("C1-dash"), "{t}"),
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn t6_windows_copy_for_env_only_bindings_uses_rem_set_lines() {
    let sh = shapes()
        .iter()
        .find(|s| s.name == "bundle wsh-multi 2 slots")
        .unwrap();
    let (schema, sub, st) = form_state(sh, &values_of(sh));
    let (_, windows) = copy_commands(schema, sub, &st, &env_of(&[]));
    let t = ready(&windows);
    assert!(
        t.contains("REM --slot @N.phrase=: set MNEMONIC_GUI_S0=<the value>"),
        "{t}"
    );
    assert!(!t.contains("abandon") && !t.contains("zoo"), "{t}");
}

// ── Shell leg ─────────────────────────────────────────────────────────────

const VALUES: [&str; 11] = [
    "pad",
    "  pad  ",
    "\tpad",
    "pad\t",
    " a  b ",
    "back\\slash",
    "tab\tin",
    "a'b\"c",
    "*",
    "$HOME",
    "%s%d",
];

fn sh_run(shell: &str, script: &str, stdin: &str, env: &[(&str, &str)]) -> String {
    use std::io::Write;
    let path = format!(
        "{}:{}",
        secret_channels_common::bin_dir().display(),
        std::env::var("PATH").unwrap()
    );
    let mut cmd = std::process::Command::new(shell);
    cmd.arg("-c").arg(script).env("PATH", path);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut ch = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("{shell} is required for the T6 shell leg: {e}"));
    ch.stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    String::from_utf8_lossy(&ch.wait_with_output().unwrap().stdout).into_owned()
}

fn argv_exact(argv: &[&str]) -> String {
    let b = secret_channels_common::bin_dir();
    let o = std::process::Command::new(b.join(argv[0]))
        .args(&argv[1..])
        .output()
        .unwrap();
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// The §A7 recipes the GUI prints, EXECUTED: the typed stdin row
/// (`restore`, passphrase typed), the `$VAR` recipe (`restore`, passphrase
/// from `$MY_PW`: a printf pipeline or the CLI's own `@env:`, per the measured
/// rule), and the typed-EnvRef `read` recipe (`xpub-search path-of-xpub`, where
/// a clean typed passphrase rides `MNEMONIC_GUI_S1`; an edge-whitespace one
/// takes stdin instead) — each against argv-exact.
#[test]
fn t6_shell_leg_printed_recipes_equal_argv_exact_in_bash_zsh_fish() {
    let (sc, sub) = restore();
    let mut bad = Vec::new();
    let mut read_legs = 0;
    for v in VALUES {
        let want = argv_exact(&[
            "mnemonic",
            "restore",
            "--allow-argv-secret",
            "--from",
            &format!("phrase={P}"),
            "--template",
            "bip84",
            "--passphrase",
            v,
        ]);
        assert!(want.contains("master fingerprint"), "{want}");
        // typed stdin row
        let (posix, _) = copy_commands(
            sc,
            sub,
            &restore_state("phrase=@env:PHR", v),
            &env_of(&[("PHR", P)]),
        );
        let line = ready(&posix).lines().last().unwrap().to_string();
        // $VAR printf pipeline
        let (posix2, _) = copy_commands(
            sc,
            sub,
            &restore_state("phrase=@env:PHR", "@env:MY_PW"),
            &env_of(&[("PHR", P), ("MY_PW", v)]),
        );
        let line2 = ready(&posix2).lines().last().unwrap().to_string();
        for shell in ["bash", "zsh", "fish"] {
            if sh_run(shell, &line, &format!("{v}\n"), &[("PHR", P)]) != want {
                bad.push(format!("{shell} typed stdin row {v:?}"));
            }
            if sh_run(shell, &line2, "", &[("PHR", P), ("MY_PW", v)]) != want {
                bad.push(format!("{shell} $VAR recipe {v:?}"));
            }
        }
        // typed-EnvRef `read` recipe
        let xpub = argv_exact(&[
            "mnemonic",
            "convert",
            "--allow-argv-secret",
            "--from",
            &format!("phrase={P}"),
            "--to",
            "xpub",
            "--template",
            "bip84",
            "--passphrase",
            v,
        ]);
        let xpub = xpub
            .lines()
            .find_map(|l| l.strip_prefix("xpub: "))
            .unwrap_or_else(|| panic!("{xpub}"))
            .trim()
            .to_string();
        let want3 = argv_exact(&[
            "mnemonic",
            "xpub-search",
            "path-of-xpub",
            "--allow-argv-secret",
            "--target-xpub",
            &xpub,
            "--phrase",
            P,
            "--passphrase",
            v,
        ]);
        assert!(want3.contains("match: m/84'/0'/0'"), "{want3}");
        let xs = &schema::mnemonic::SCHEMA;
        let mut st = FormState::from_pairs(vec![("--target-xpub", FlagValue::Text(xpub.clone()))]);
        st.secret_widgets.insert(
            "--phrase".into(),
            vec![SecretLineEdit::from_text("@env:PHR")],
        );
        st.secret_widgets
            .insert("--passphrase".into(), vec![SecretLineEdit::from_text(v)]);
        let (posix3, _) = copy_commands(
            xs,
            sub_of(xs, "xpub-search-path-of-xpub"),
            &st,
            &env_of(&[("PHR", P)]),
        );
        let t3 = ready(&posix3).to_string();
        // F-694: which recipe applies is DERIVED from the plan Copy printed. On
        // 0.105.1 the passphrase's EnvRef has no exact terminator (strip-one), so
        // a value with edge whitespace is kept off it (§A3c) and takes stdin, and
        // the phrase moves to the user's own `@env:PHR`. A clean value still rides
        // `MNEMONIC_GUI_S1` and exercises the `read` recipe.
        let Some(comment) = t3.lines().find(|l| l.contains("read -rs MNEMONIC_GUI_S1")) else {
            assert!(
                t3.contains("--phrase @env:PHR") && t3.contains("--passphrase-stdin"),
                "neither the read recipe nor the stdin row: {t3}"
            );
            let cmd3 = t3.lines().last().unwrap();
            for shell in ["bash", "zsh", "fish"] {
                if sh_run(shell, cmd3, &format!("{v}\n"), &[("PHR", P)]) != want3 {
                    bad.push(format!("{shell} xpub-search stdin row {v:?}"));
                }
            }
            continue;
        };
        read_legs += 1;
        let posix_read = comment
            .split_once(": ")
            .unwrap()
            .1
            .split("   (fish: ")
            .next()
            .unwrap();
        let fish_read = comment
            .split("(fish: ")
            .nth(1)
            .unwrap()
            .trim_end_matches(')');
        let cmd3 = t3.lines().last().unwrap();
        for shell in ["bash", "zsh", "fish"] {
            let recipe = if shell == "fish" {
                fish_read
            } else {
                posix_read
            };
            let got = sh_run(
                shell,
                &format!("{recipe}; {cmd3}"),
                &format!("{v}\n"),
                &[("PHR", P)],
            );
            if got != want3 {
                bad.push(format!("{shell} read recipe {v:?}"));
            }
        }
    }
    assert!(bad.is_empty(), "§A7 recipe mismatches: {bad:?}");
    // Non-vacuity: the typed-EnvRef `read` recipe must still run for some value.
    assert!(
        read_legs > 0,
        "the read recipe never ran: no value rode MNEMONIC_GUI_S1"
    );
}

// ── The window ────────────────────────────────────────────────────────────

fn app_with(sub: &str, st: FormState) -> Harness<'static, MnemonicGuiApp> {
    let found = |p: &str| Detected::Found(PathBuf::from(p));
    let app_state = AppState {
        mnemonic_detect: found("/pinned/mnemonic"),
        md_detect: found("/pinned/md"),
        ms_detect: found("/pinned/ms"),
        mk_detect: found("/pinned/mk"),
        active_tab: CliTab::Mnemonic,
    };
    let mut app = MnemonicGuiApp::new_headless(app_state, None, None);
    app.active_subcommand
        .insert(CliTab::Mnemonic, sub.to_string());
    app.form_state.insert(format!("mnemonic:{sub}"), st);
    Harness::builder()
        .with_size(egui::Vec2::new(1400.0, 2400.0))
        .with_max_steps(64)
        .build_state(|ctx, app: &mut MnemonicGuiApp| app.ui(ctx), app)
}

fn labels<S>(h: &Harness<'static, S>) -> Vec<String> {
    h.query_all(egui_kittest::kittest::By::new())
        .filter_map(|n| n.label().or_else(|| n.value()))
        .collect()
}

#[test]
#[cfg(target_os = "linux")]
fn t6_preview_and_confirm_show_bindings_with_provenance_and_no_secret() {
    let mut h = app_with("restore", restore_state(&format!("phrase={P}"), PWSENT));
    h.run();
    let all = labels(&h);
    let joined = all.join("\n");
    let preview = all
        .iter()
        .find(|l| l.starts_with("Preview: "))
        .unwrap_or_else(|| panic!("{joined}"));
    assert!(
        preview.contains("phrase=@env:MNEMONIC_GUI_S0") && preview.contains("--passphrase-stdin"),
        "{preview}"
    );
    assert!(
        joined.contains("--from phrase= ← env MNEMONIC_GUI_S0 (typed)"),
        "{joined}"
    );
    assert!(
        joined.contains("--passphrase ← stdin via --passphrase-stdin + '\\r\\n' (typed)"),
        "{joined}"
    );
    assert!(!preview.contains(PWSENT) && !preview.contains("abandon"));
    h.get_by_label("Run").click();
    h.run();
    let all = labels(&h).join("\n");
    assert!(
        all.contains("This invocation sends these secrets privately to mnemonic:"),
        "{all}"
    );
    assert!(
        all.contains("--passphrase ← stdin via --passphrase-stdin + '\\r\\n' (typed)"),
        "{all}"
    );
    for l in labels(&h) {
        // the secret widgets hold the values; no other text may
        if l.contains(PWSENT) || l.contains("abandon abandon") {
            assert!(
                !l.contains("Preview") && !l.contains("invocation") && !l.contains('←'),
                "a secret in {l:?}"
            );
        }
    }
}

#[test]
fn t6_env_provenance_is_shown_and_a_refusal_disables_run() {
    // provenance line for a resolved variable (every OS: the Preview line)
    std::env::set_var("T6_MY_PW", PWSENT);
    let mut h = app_with(
        "restore",
        restore_state(&format!("phrase={P}"), "@env:T6_MY_PW"),
    );
    h.run();
    let all = labels(&h).join("\n");
    assert!(all.contains("(value of $T6_MY_PW)"), "{all}");
    assert!(
        !all.contains(PWSENT),
        "the resolved value must not be shown"
    );
    // a refused plan: Run disabled, the refusal named
    let mut h = app_with("restore", restore_state(&format!("phrase={P}"), "-"));
    h.run();
    let all = labels(&h).join("\n");
    assert!(
        all.contains("Run refused — C1-dash: mnemonic restore --passphrase"),
        "{all}"
    );
    let run = h.get_by_label("Run");
    assert!(
        run.is_disabled(),
        "Run must be disabled while the plan is refused"
    );
    let _ = BTreeMap::<u8, u8>::new();
}
