//! Argv assembler + copy-command shell-quoting (SPEC §6).
//!
//! `assemble_argv` produces a `Vec<String>` from a (Schema, SubcommandSchema,
//! FormState) triple per the byte-exact SPEC §6.7 emission rules.
//! `render_copy_command` formats an argv vector for display-only copy-paste
//! under either POSIX or Windows shell flavors.
//!
//! The argv is passed directly to `std::process::Command::args()` (no shell
//! escaping needed); the copy-command output is for the user's eyes only
//! and is NEVER used to spawn the subprocess.

use crate::schema::{
    FlagKind, FlagSchema, FlagValue, Schema, SubcommandSchema, TaggedOrIndexedValue,
    TimestampValue,
};

/// Shell flavor for `render_copy_command`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellFlavor {
    /// POSIX shells (bash, zsh, fish). `shlex::try_quote` does the heavy
    /// lifting; multi-token argv joined with single spaces.
    Posix,
    /// Windows `cmd.exe`. Double-quote each arg; embedded `"` becomes `""`.
    /// Lines joined with ` ^\r\n  ` for shell-side line continuation.
    WindowsCmd,
}

/// Assemble the argv for `<cli> <subcommand> [flags...]`.
///
/// Invariants per SPEC §6:
///   1. `argv[0]` = `schema.cli_name`. No absolute path.
///   2. `argv[1]` = `subcommand.name`.
///   3. Flag emission order = `subcommand.flags` declaration order.
///   4. Repeating flags emit one argv pair per FormState entry in form-
///      state order (slot-index ascending is the SlotEditor's responsibility;
///      Phase 3 wires it).
///   5. Empty / false / absent values are NOT emitted.
pub fn assemble_argv(
    schema: &Schema,
    subcommand: &SubcommandSchema,
    state: &crate::schema::FormState,
) -> Vec<String> {
    let mut argv: Vec<String> = Vec::new();
    argv.push(schema.cli_name.to_string());
    argv.push(subcommand.name.to_string());

    for flag in subcommand.flags {
        if flag.repeating {
            for (_, value) in state.values.iter().filter(|(k, _)| k == flag.name) {
                emit_one(flag, value, &mut argv);
            }
        } else if let Some((_, value)) = state.values.iter().find(|(k, _)| k == flag.name) {
            emit_one(flag, value, &mut argv);
        }
    }

    argv
}

fn emit_one(flag: &FlagSchema, value: &FlagValue, argv: &mut Vec<String>) {
    match (&flag.kind, value) {
        (FlagKind::Text, FlagValue::Text(v)) => {
            if !v.is_empty() {
                argv.push(flag.name.to_string());
                argv.push(v.clone());
            }
        }
        (FlagKind::Number { .. }, FlagValue::Number(n)) => {
            argv.push(flag.name.to_string());
            argv.push(n.to_string());
        }
        (FlagKind::Dropdown(_), FlagValue::Dropdown(v)) => {
            if !v.is_empty() {
                argv.push(flag.name.to_string());
                argv.push(v.clone());
            }
        }
        (FlagKind::Boolean, FlagValue::Boolean(true)) => {
            argv.push(flag.name.to_string());
        }
        (FlagKind::Boolean, FlagValue::Boolean(false)) => {
            // Omitted.
        }
        (FlagKind::Range, FlagValue::Range(a, b)) => {
            argv.push(flag.name.to_string());
            argv.push(format!("{},{}", a, b));
        }
        (FlagKind::Timestamp, FlagValue::Timestamp(t)) => {
            argv.push(flag.name.to_string());
            argv.push(match t {
                TimestampValue::Now => "now".to_string(),
                TimestampValue::Unix(n) => n.to_string(),
            });
        }
        (
            FlagKind::NodeValueComposite(_),
            FlagValue::NodeValueComposite { node, value },
        ) => {
            // SPEC §6.7 R3 I-3 fold: empty value → omit (matches Text/Path
            // empty-value rule and avoids upstream's "value is empty"
            // rejection from `parse_from_input` at convert.rs:128-132).
            if !value.is_empty() {
                argv.push(flag.name.to_string());
                argv.push(format!("{}={}", node, value));
            }
        }
        (FlagKind::TaggedOrIndexed(_), FlagValue::TaggedOrIndexed(tv)) => {
            argv.push(flag.name.to_string());
            argv.push(match tv {
                TaggedOrIndexedValue::Tag(t) => t.clone(),
                TaggedOrIndexedValue::Indexed(n) => format!("@{}", n),
            });
        }
        (FlagKind::Path { stdio_sentinel }, FlagValue::Path(p)) => {
            if p.is_empty() {
                return;
            }
            if p == "-" && !stdio_sentinel {
                // Schema explicitly disallows stdin sentinel for this flag;
                // the GUI form should have rejected it at validation time.
                // Defensive guard: skip emission.
                return;
            }
            argv.push(flag.name.to_string());
            argv.push(p.clone());
        }
        // Type-shape mismatch: state carries the wrong FlagValue for this
        // flag's FlagKind. Phase 2 trusts the form widget to maintain the
        // invariant; we silently skip rather than panic. Phase 5+ may add
        // a debug assertion if useful.
        _ => {}
    }
}

/// Render a shell-quoted single-line command for display copy-paste.
/// Per SPEC §6.6 this output is for the user's eyes only — it is NEVER
/// re-parsed or used to spawn the subprocess.
pub fn render_copy_command(argv: &[String], flavor: ShellFlavor) -> String {
    match flavor {
        ShellFlavor::Posix => argv
            .iter()
            .map(|s| posix_quote(s))
            .collect::<Vec<_>>()
            .join(" "),
        ShellFlavor::WindowsCmd => argv
            .iter()
            .map(|s| cmd_quote(s))
            .collect::<Vec<_>>()
            .join(" ^\r\n  "),
    }
}

/// POSIX shell quoting. Wraps `shlex::try_quote` and falls back to a manual
/// single-quote encoding for the (rare) inputs `shlex` rejects (interior
/// NULs — not expected in clap argv but defended against).
fn posix_quote(s: &str) -> String {
    match shlex::try_quote(s) {
        Ok(cow) => cow.into_owned(),
        Err(_) => {
            // Fallback: encode each `'` as `'\''` and wrap in single quotes.
            let escaped = s.replace('\'', "'\\''");
            format!("'{}'", escaped)
        }
    }
}

/// Windows quoting compatible with `CommandLineToArgvW` parsing (which
/// `cmd.exe`, PowerShell, and the Windows C runtime all use under the
/// hood). Wraps the token in `"..."`; doubles embedded `"`; and — the
/// load-bearing rule absent from the naive cmd.exe doc — doubles any run
/// of backslashes that immediately precedes a `"` (including the closing
/// `"`), so a trailing `\` does not consume the close-quote. Without this,
/// any Windows path ending in `\` (e.g. `C:\tmp\`) renders as `"C:\tmp\"`
/// and the close-`"` is consumed as a literal, leaving the token unclosed.
/// R1 I-1 fold.
fn cmd_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            // Count the run of consecutive backslashes.
            let mut n = 1usize;
            while chars.peek() == Some(&'\\') {
                chars.next();
                n += 1;
            }
            // If a `"` or end-of-string follows, double the run; the
            // doubled backslashes are themselves literal AND the
            // subsequent `"` (whether interior or our close-`"`) is then
            // unambiguously a quote-delimiter (interior) or quote-close.
            let next_is_quote_or_end =
                matches!(chars.peek(), Some(&'"')) || chars.peek().is_none();
            let to_write = if next_is_quote_or_end { n * 2 } else { n };
            for _ in 0..to_write {
                out.push('\\');
            }
        } else if c == '"' {
            // Embedded literal `"` → `""` (cmd.exe-style; also accepted by
            // CommandLineToArgvW since the doubling means: close, reopen).
            out.push('"');
            out.push('"');
        } else {
            out.push(c);
        }
    }
    out.push('"');
    out
}
