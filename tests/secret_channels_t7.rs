//! T7 — runner echo test (DESIGN §A9; portable, no CLIs). The REAL runner
//! (`runner::run_plan`) executes each plan against a helper that parses its
//! own argv for `@env:NAME`, `/dev/fd/N` and `-`/`--X-stdin` and prints the
//! bytes it received on each. They must equal each source's target plus
//! terminator, every run must finish within a timeout (a leaked pipe write
//! end hangs the helper), the child must inherit NO `MNEMONIC_GUI_*` variable
//! beyond the plan's own (env hygiene, §A4.4), and the interim path's argv
//! must carry each resolved target at its own flag.
//!
//! Runner-side mutations this catches: bindings swapped, stdin written from
//! the wrong binding, the terminator dropped, the env scrub dropped, the
//! write end left open (T5).

mod secret_channels_common;

use std::sync::mpsc;
use std::time::Duration;

use mnemonic_gui::form::channels::{
    channel_table, materialize, plan_sources, policy, ChannelKind, PlanOptions, RunPlan,
    SourceValue,
};
use secret_channels_common::{env_of, shape_assembled, shapes, Shape};

fn helper() -> String {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/secret_channels/echo_helper.py"
    )
    .to_string()
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// Run `plan` with `python3 <helper>` in place of the CLI, with a timeout.
fn echo(plan: &RunPlan) -> serde_json::Value {
    let mut p = plan.clone();
    p.argv[0] = "python3".into();
    p.argv.insert(1, helper());
    p.mask.insert(1, false);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = mnemonic_gui::runner::run_plan(&p)
            .map(|r| (r.exit_code, r.stdout.clone(), r.stderr.clone()));
        let _ = tx.send(r);
    });
    let (code, out, err) = rx
        .recv_timeout(Duration::from_secs(30))
        .expect("T7: the run did not finish — a pipe write end leaked into the child, or stdin never closed")
        .expect("spawn");
    assert_eq!(
        code,
        Some(0),
        "helper failed: {}",
        String::from_utf8_lossy(&err)
    );
    serde_json::from_slice(&out).expect("helper JSON")
}

/// Distinct sentinels per source, each ending in `suffix` (target bytes).
fn sentinel_values(sh: &Shape, suffix: &str) -> Vec<Vec<String>> {
    sh.sources
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (0..s.values.len())
                .map(|k| format!("S{i}x{k}{suffix}"))
                .collect()
        })
        .collect()
}

fn planned_for(
    sh: &Shape,
    values: &[Vec<String>],
    os: &str,
) -> Option<(RunPlan, Vec<Vec<String>>)> {
    let srcs: Vec<_> = sh
        .sources
        .iter()
        .zip(values)
        .map(|(s, v)| {
            let mut p = s.plan_source();
            p.value = if s.is_group() {
                SourceValue::group(v.clone())
            } else {
                SourceValue::one(v[0].clone())
            };
            p
        })
        .collect();
    let planned = plan_sources(
        &srcs,
        channel_table(),
        policy(),
        os,
        &env_of(&[]),
        PlanOptions::default(),
    )
    .ok()?;
    Some((
        materialize(&shape_assembled(sh, values), &planned),
        values.to_vec(),
    ))
}

#[test]
fn t7_each_channel_delivers_its_own_sources_target_plus_terminator() {
    // A decoy the runner must scrub (§A4.4): an inherited MNEMONIC_GUI_* var.
    // SAFETY-free: nextest runs each test in its own process.
    std::env::set_var("MNEMONIC_GUI_S0", "INHERITED-DECOY");
    std::env::set_var("MNEMONIC_GUI_LEAK", "INHERITED-DECOY");
    let mut checked = 0;
    // endings a target may have (NI9 refuses a trailing CR/LF): interior
    // newline, edge spaces, a tab
    for suffix in ["", "  ", "\tz", "\nX"] {
        for sh in shapes() {
            let values = sentinel_values(sh, suffix);
            let Some((rp, values)) = planned_for(sh, &values, "linux") else {
                continue;
            };
            let got = echo(&rp);
            let chans = got["channels"].as_array().unwrap();
            // the plan's own env names, and nothing inherited
            let mut want_env: Vec<String> = rp.env.iter().map(|(k, _)| k.clone()).collect();
            want_env.sort();
            let got_env: Vec<String> = got["gui_env"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect();
            assert_eq!(
                got_env, want_env,
                "{}: the child saw MNEMONIC_GUI_* beyond the plan's",
                sh.name
            );
            for (i, bv) in rp.bindings.iter().enumerate() {
                let b = &bv.binding;
                let s = &sh.sources[i];
                let target = if s.is_group() {
                    values[i].join("\n")
                } else {
                    values[i][0].clone()
                };
                let mut want = match b.kind {
                    ChannelKind::StdinMulti => target.into_bytes(),
                    _ => format!("{target}{}", b.terminator).into_bytes(),
                };
                if b.kind == ChannelKind::InFile && s.is_group() {
                    want.push(b'\n');
                }
                // the helper's sys.argv[k] is the plan's argv[k] (argv[0]
                // became `python3 <helper>`)
                let at = bv.argv_index.expect("a private binding names a token");
                let at = match b.kind {
                    ChannelKind::FileFlag | ChannelKind::InFile => at + 1,
                    _ => at,
                };
                let entry = chans
                    .iter()
                    .find(|c| c["index"].as_u64() == Some(at as u64))
                    .unwrap_or_else(|| {
                        panic!("{} src{i}: no channel at argv {at}: {got}", sh.name)
                    });
                let how = entry["how"].as_str().unwrap();
                let expect_how = match b.kind {
                    ChannelKind::EnvRef => "env",
                    k if k.is_fd() => "fd",
                    _ => "stdin",
                };
                assert_eq!(how, expect_how, "{} src{i}", sh.name);
                let bytes = unhex(entry["bytes"].as_str().expect("the variable is set"));
                assert_eq!(
                    String::from_utf8_lossy(&bytes),
                    String::from_utf8_lossy(&want),
                    "{} src{i} ({:?}, suffix {suffix:?}): wrong bytes delivered",
                    sh.name,
                    b.kind
                );
                checked += 1;
            }
            assert!(
                !chans.iter().any(|c| c["how"] == "stdin-again"),
                "{}: two stdin readers",
                sh.name
            );
        }
    }
    assert!(checked > 200, "only {checked} bindings checked");
}

#[test]
fn t7_interim_argv_carries_each_resolved_target_at_its_own_flag() {
    let mut checked = 0;
    for sh in shapes() {
        let values = sentinel_values(sh, "  ");
        let Some((rp, values)) = planned_for(sh, &values, "macos") else {
            continue;
        };
        let got = echo(&rp);
        let argv: Vec<String> = got["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| String::from_utf8(unhex(h.as_str().unwrap())).unwrap())
            .collect();
        // helper argv[k] is the plan's argv[k + 1]
        assert_eq!(
            argv[sh.sub.len()],
            "--allow-argv-secret",
            "{}: opt-in after the subcommand",
            sh.name
        );
        for (i, s) in sh.sources.iter().enumerate() {
            match s.form.as_str() {
                "node" | "value" => {
                    let flag = s.flag.as_ref().unwrap();
                    let tok = format!("{}{}", s.prefix, values[i][0]);
                    let pos = argv
                        .iter()
                        .position(|t| *t == tok)
                        .unwrap_or_else(|| panic!("{} src{i}: {tok:?} not on argv", sh.name));
                    assert_eq!(&argv[pos - 1], flag, "{} src{i}: not at its flag", sh.name);
                }
                _ => {
                    for v in &values[i] {
                        let pos = argv.iter().position(|t| t == v).unwrap();
                        assert!(
                            argv[..pos].contains(&"--".to_string()),
                            "{} src{i}: positional before --",
                            sh.name
                        );
                    }
                }
            }
            checked += 1;
        }
        assert!(
            got["channels"].as_array().unwrap().is_empty(),
            "{}: interim used a channel",
            sh.name
        );
    }
    assert!(checked > 40);
}
