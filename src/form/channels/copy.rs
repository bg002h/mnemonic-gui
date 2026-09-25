//! DESIGN §A7 — the Copy command.
//!
//! Copy never contains a secret value and never synthesises an unmeasured
//! spelling. It is generated from the PRIVATE-channel plan on every OS
//! ([`super::data::Policy::for_copy`]), so a pasted command behaves like
//! Linux's Run. Each binding gets one comment line:
//!
//! | binding | provenance | POSIX |
//! |---|---|---|
//! | `EnvRef` | typed | `@env:MNEMONIC_GUI_S<i>` + `# IFS= read -rs …; export …` |
//! | `EnvRef` | `$VAR` | the user's own `@env:VAR` |
//! | stdin | typed | the plan's spelling + `# stdin: type it, then Enter, then Ctrl-D` |
//! | stdin | `$VAR` | `printf '%s<T>' "$VAR" \| <command>` (T = the cell's terminator) |
//! | pipe fd | any | `--in <FILE>` + `# FILE: a file holding …` |
//! | `StdinMulti` | typed | `-- -` + `# stdin: paste each share on its own line, then Ctrl-D` |
//! | `StdinMulti` | `$S1`… | `printf '%s\n' "$S1" "$S2" \| <command>` |
//!
//! Disabled: when the plan is refused (the tooltip is the refusal); for a
//! typed EnvRef-bound value holding CR or LF (a shell `read` takes one line);
//! on Windows for any stdin-bound binding.

use super::{
    channel_table, plan_with, policy, BindingView, ChannelKind, ChannelTable, EnvRule, Policy,
    Provenance, RunPlan,
};
use crate::form::invocation::{cmd_quote, posix_quote, render_copy_command, ShellFlavor};
use crate::schema::{FormState, Schema, SubcommandSchema};

/// Copy is disabled for a typed EnvRef-bound value holding CR or LF (R3 Nm11).
pub const MULTILINE_TOOLTIP: &str =
    "this value spans lines; use Run, or paste it into the command's own stdin prompt";

/// The Windows Copy cannot feed a pipe (§A7).
pub const WINDOWS_STDIN_TOOLTIP: &str = "needs a pipe; use the POSIX copy";

/// One Copy button's state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopyState {
    /// The text the button copies.
    Ready(String),
    /// The button is disabled; the string is its tooltip.
    Disabled(String),
}

impl CopyState {
    pub fn text(&self) -> Option<&str> {
        match self {
            CopyState::Ready(t) => Some(t),
            CopyState::Disabled(_) => None,
        }
    }
}

/// Both Copy buttons, under the committed table and policy.
pub fn copy_commands(
    schema: &Schema,
    sub: &SubcommandSchema,
    state: &FormState,
    user_env: &dyn Fn(&str) -> Option<String>,
) -> (CopyState, CopyState) {
    copy_commands_with(schema, sub, state, user_env, channel_table(), policy())
}

/// [`copy_commands`] under an explicit table and policy.
pub fn copy_commands_with(
    schema: &Schema,
    sub: &SubcommandSchema,
    state: &FormState,
    user_env: &dyn Fn(&str) -> Option<String>,
    table: &ChannelTable,
    policy: &Policy,
) -> (CopyState, CopyState) {
    let copy_policy = policy.for_copy();
    let plan = match plan_with(
        schema,
        sub,
        state,
        user_env,
        super::data::COPY_OS,
        table,
        &copy_policy,
    ) {
        Ok(p) => p,
        Err(r) => {
            let t = format!("Copy refused — {r}");
            return (CopyState::Disabled(t.clone()), CopyState::Disabled(t));
        }
    };
    (
        render(&plan, table, ShellFlavor::Posix),
        render(&plan, table, ShellFlavor::WindowsCmd),
    )
}

/// The input's CLI `@env:` rule is one under which `printf '%s'` of the raw
/// variable delivers the target (verbatim, or no working CLI `@env:`).
fn printf_is_exact(table: &ChannelTable, key: &str) -> bool {
    matches!(
        table.get(key).map(|e| e.cli_env_rule),
        Some(None) | Some(Some(EnvRule::Verbatim))
    )
}

fn has_envref_cell(table: &ChannelTable, key: &str) -> bool {
    table
        .get(key)
        .is_some_and(|e| e.channels.iter().any(|c| c.kind == ChannelKind::EnvRef))
}

/// The source's own flag (the last word of its key, when it is a flag).
fn source_flag(b: &BindingView) -> Option<String> {
    b.binding
        .key
        .rsplit(' ')
        .next()
        .filter(|w| w.starts_with("--"))
        .map(|w| w.to_string())
}

fn printf_escape(term: &str) -> String {
    term.replace('\r', "\\r").replace('\n', "\\n")
}

fn var_names(b: &BindingView) -> Option<Vec<String>> {
    b.provenance
        .iter()
        .map(|p| match p {
            Provenance::Env(n) => Some(n.clone()),
            Provenance::Typed => None,
        })
        .collect()
}

fn render(plan: &RunPlan, table: &ChannelTable, flavor: ShellFlavor) -> CopyState {
    let posix = flavor == ShellFlavor::Posix;
    if !plan.has_secrets() {
        return CopyState::Ready(render_copy_command(&plan.argv, flavor));
    }
    // Tokens as (text, quote?) — `<FILE>` placeholders stay bare.
    // One entry per planned argv token; a token may become several.
    let mut toks: Vec<Vec<(String, bool)>> =
        plan.argv.iter().map(|t| vec![(t.clone(), true)]).collect();
    let mut comments: Vec<String> = Vec::new();
    let mut pipe_prefix: Option<String> = None;
    let mut files = 0usize;
    let c = if posix { "#" } else { "REM" };
    for b in &plan.bindings {
        let label = b.binding.source_label();
        let kind = b.binding.kind;
        let at = b.argv_index;
        match kind {
            ChannelKind::EnvRef => {
                let name = b.binding.env.clone().unwrap_or_default();
                match var_names(b).as_deref() {
                    Some([var]) => {
                        if let Some(i) = at {
                            toks[i][0].0 = toks[i][0]
                                .0
                                .replace(&format!("@env:{name}"), &format!("@env:{var}"));
                        }
                        comments.push(format!("{c} {label}: read by the CLI from ${var}"));
                    }
                    _ => {
                        let multiline = plan
                            .env
                            .iter()
                            .find(|(k, _)| *k == name)
                            .is_some_and(|(_, v)| v.contains('\r') || v.contains('\n'));
                        if multiline {
                            return CopyState::Disabled(MULTILINE_TOOLTIP.into());
                        }
                        if posix {
                            comments.push(format!(
                                "# {label}: IFS= read -rs {name}; export {name}   \
                                 (fish: read -s -x --delimiter \\n {name})"
                            ));
                        } else {
                            comments.push(format!("REM {label}: set {name}=<the value>"));
                        }
                    }
                }
            }
            ChannelKind::StdinToggle | ChannelKind::DashValue | ChannelKind::PosDash => {
                if !posix {
                    return CopyState::Disabled(WINDOWS_STDIN_TOOLTIP.into());
                }
                match var_names(b).as_deref() {
                    Some([var]) if printf_is_exact(table, &b.binding.key) => {
                        pipe_prefix = Some(format!(
                            "printf '%s{}' \"${var}\" | ",
                            printf_escape(&b.binding.terminator)
                        ));
                        comments.push(format!("# {label}: stdin from ${var}"));
                    }
                    Some([var]) if has_envref_cell(table, &b.binding.key) => {
                        // The CLI's own `@env:VAR` applies its own rule.
                        let Some(i) = at else {
                            return CopyState::Disabled("no Copy spelling for this input".into());
                        };
                        match kind {
                            ChannelKind::StdinToggle => {
                                let Some(flag) = source_flag(b) else {
                                    return CopyState::Disabled(
                                        "no Copy spelling for this input".into(),
                                    );
                                };
                                toks[i] = vec![(flag, true), (format!("@env:{var}"), true)];
                            }
                            _ => {
                                let t = toks[i][0].0.clone();
                                toks[i][0].0 = format!("{}@env:{var}", &t[..t.len() - 1]);
                            }
                        }
                        comments.push(format!("# {label}: read by the CLI from ${var}"));
                    }
                    Some(_) => {
                        return CopyState::Disabled("no exact Copy spelling for this input".into());
                    }
                    None => {
                        comments.push(format!(
                            "# stdin ({label}): type it, then Enter, then Ctrl-D"
                        ));
                    }
                }
            }
            ChannelKind::StdinMulti => {
                if !posix {
                    return CopyState::Disabled(WINDOWS_STDIN_TOOLTIP.into());
                }
                match var_names(b) {
                    Some(vars) => {
                        let args: Vec<String> = vars.iter().map(|v| format!("\"${v}\"")).collect();
                        pipe_prefix = Some(format!("printf '%s\\n' {} | ", args.join(" ")));
                        comments.push(format!(
                            "# {label}: stdin from {}",
                            vars.iter()
                                .map(|v| format!("${v}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    None => comments
                        .push("# stdin: paste each share on its own line, then Ctrl-D".to_string()),
                }
            }
            ChannelKind::FileFlag | ChannelKind::InFile => {
                files += 1;
                let ph = if files == 1 {
                    "<FILE>".to_string()
                } else {
                    format!("<FILE{files}>")
                };
                if let Some(i) = at {
                    // the token after the flag is `/dev/fd/N`
                    if i + 1 < toks.len() && toks[i + 1][0].0.starts_with("/dev/fd/") {
                        toks[i + 1] = vec![(ph.clone(), false)];
                    }
                }
                let what = if b.binding.key.ends_with("<ms1>") || b.binding.key.ends_with("--ms1") {
                    "the ms1 card".to_string()
                } else {
                    format!("the {label} value")
                };
                comments.push(format!(
                    "{c} {}: a file holding {what}",
                    ph.trim_start_matches('<').trim_end_matches('>')
                ));
            }
            ChannelKind::Argv => {
                // The Copy plan is private by construction (Policy::for_copy).
                return CopyState::Disabled("Copy has no private plan here".into());
            }
        }
    }
    let quoted: Vec<String> = toks
        .iter()
        .flatten()
        .map(|(t, q)| match (q, posix) {
            (false, _) => t.clone(),
            (true, true) => posix_quote(t),
            (true, false) => cmd_quote(t),
        })
        .collect();
    let command = if posix {
        quoted.join(" ")
    } else {
        quoted.join(" ^\r\n  ")
    };
    let nl = if posix { "\n" } else { "\r\n" };
    let mut out = comments.join(nl);
    out.push_str(nl);
    if let Some(p) = pipe_prefix {
        out.push_str(&p);
    }
    out.push_str(&command);
    CopyState::Ready(out)
}
