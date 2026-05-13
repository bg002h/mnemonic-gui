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
    /// Windows `cmd.exe` + `CommandLineToArgvW`. Double-quote each arg;
    /// embedded `"` is encoded as `\"` per the `ArgvQuote` odd-backslash
    /// rule (see `cmd_quote` doc-comment for full rules — `""` is NOT a
    /// valid literal-`"` escape under `CommandLineToArgvW`). Lines joined
    /// with ` ^\r\n  ` for shell-side line continuation.
    /// R2 C-1 / R3 I-1 fold.
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
        // SPEC §6.4: when allows_slots == true, the `--slot` flag is
        // emitted from SlotState (not from `values`), in slot-index
        // ascending order. The schema still carries a `--slot` FlagSchema
        // entry so the schema-mirror flag-name test sees it.
        if flag.name == "--slot" && subcommand.allows_slots {
            for token in state.slots.to_slot_argv() {
                argv.push(token);
            }
            continue;
        }
        // SPEC §3 / v0.2 Phase B.1: secret-flag branch. For secret-class
        // flags, the buffer lives in `state.secret_widgets[flag.name]`
        // (a `SecretLineEdit` owning a `Zeroizing<Vec<u8>>`), NOT in
        // `state.values`. Wrap the extracted `String` in `Zeroizing::new`
        // per R1 N-1 fold — the transient is one-call-scoped but heap-
        // allocated, so the wrap engages `Zeroizing::Drop` for best-
        // effort zeroing past the argv emission. The `state.values`
        // lookup is bypassed for secret flags entirely.
        if crate::secrets::flag_is_secret(flag) {
            if let Some(widget) = state.secret_widgets.get(flag.name) {
                if !widget.is_empty() {
                    let value = zeroize::Zeroizing::new(widget.as_string());
                    argv.push(flag.name.to_string());
                    argv.push(value.as_str().to_string());
                }
            }
            continue;
        }
        if flag.repeating {
            for (_, value) in state.values.iter().filter(|(k, _)| k == flag.name) {
                emit_one(flag, value, &mut argv);
            }
        } else if let Some((_, value)) = state.values.iter().find(|(k, _)| k == flag.name) {
            emit_one(flag, value, &mut argv);
        }
    }

    // Positional args (Phase 6) — emit at the end of argv in form-state
    // order, skipping empty strings (SPEC §6.7 parity).
    for pos in &state.positionals {
        if !pos.is_empty() {
            argv.push(pos.clone());
        }
    }

    argv
}

fn emit_one(flag: &FlagSchema, value: &FlagValue, argv: &mut Vec<String>) {
    match (&flag.kind, value) {
        (FlagKind::Text, FlagValue::Text(v))
            if !v.is_empty() => {
                argv.push(flag.name.to_string());
                argv.push(v.clone());
            }
        (FlagKind::Number { .. }, FlagValue::Number(n)) => {
            argv.push(flag.name.to_string());
            argv.push(n.to_string());
        }
        (FlagKind::Dropdown(_), FlagValue::Dropdown(v))
            if !v.is_empty() => {
                argv.push(flag.name.to_string());
                argv.push(v.clone());
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
        )
            // SPEC §6.7 R3 I-3 fold: empty value → omit (matches Text/Path
            // empty-value rule and avoids upstream's "value is empty"
            // rejection from `parse_from_input` at convert.rs:128-132).
            if !value.is_empty() => {
                argv.push(flag.name.to_string());
                argv.push(format!("{}={}", node, value));
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

/// Windows quoting compatible with `CommandLineToArgvW` (the universal
/// Win32 parser used by cmd.exe → CreateProcess → target-binary startup).
/// Implements Daniel Colascione's canonical `ArgvQuote` rules from the
/// Microsoft "Everyone quotes command line arguments the wrong way" post:
///
/// 1. Wrap the token in `"…"`.
/// 2. For each run of `n` consecutive `\` followed by a literal `"`
///    (interior): emit `2n+1` `\` + `"`. The odd-count rule produces a
///    literal `"` and preserves in-quotes mode.
/// 3. For each run of `n` consecutive `\` at end-of-string (before the
///    close-`"`): emit `2n` `\`. The even-count rule produces `n`
///    literal `\` and toggles in-quotes mode (closing the wrapper).
/// 4. For each interior run of `n` consecutive `\` (not followed by a
///    `"`): emit `n` `\` (pass-through).
/// 5. A bare `"` (no preceding `\`) is encoded as `\"` (n=0 → 1 `\`).
///
/// R1 I-1 / R2 C-1 fold: prior implementations used `""` for embedded
/// `"`, which `CommandLineToArgvW` does NOT recognize as a literal-`"`
/// escape — it parses as (close-quote, reopen-quote) and the literal `"`
/// is lost from the resulting argv. The odd-backslash rule is the only
/// universal encoding.
fn cmd_quote(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            let mut n = 0usize;
            while i < chars.len() && chars[i] == '\\' {
                n += 1;
                i += 1;
            }
            if i == chars.len() {
                // 2b: end of input; double the run so close-`"` is
                // unambiguously the quote-close, not the escape target.
                for _ in 0..(n * 2) {
                    out.push('\\');
                }
            } else if chars[i] == '"' {
                // 2a: emit 2n+1 backslashes + the literal `"`.
                for _ in 0..(n * 2 + 1) {
                    out.push('\\');
                }
                out.push('"');
                i += 1;
            } else {
                // 2c: interior; pass through.
                for _ in 0..n {
                    out.push('\\');
                }
            }
        } else if chars[i] == '"' {
            // Rule 3: lone literal `"` → `\"`.
            out.push('\\');
            out.push('"');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out.push('"');
    out
}
