//! Copy-command shell-quoting tests (SPEC §6.6, Phase 2 R1 I-4).
//!
//! The render output is for display copy-paste only; it is NEVER re-parsed
//! or used to spawn the subprocess. POSIX uses `shlex::try_quote`; Windows
//! uses ArgvQuote (odd-backslash) encoding per `CommandLineToArgvW`
//! convention — see `src/form/invocation.rs::cmd_quote` for the full rules.
//! R2 C-1 / R3 I-1 fold.

use mnemonic_gui::form::invocation::{render_copy_command, ShellFlavor};

// ── POSIX ─────────────────────────────────────────────────────────────────

#[test]
fn posix_simple_argv_no_quoting() {
    let argv = vec!["mnemonic".into(), "bundle".into(), "--network".into(), "mainnet".into()];
    let s = render_copy_command(&argv, ShellFlavor::Posix);
    assert_eq!(s, "mnemonic bundle --network mainnet");
}

#[test]
fn posix_value_with_space_is_quoted() {
    let argv = vec![
        "mnemonic".into(),
        "export-wallet".into(),
        "--wallet-name".into(),
        "Vault Cold Storage".into(),
    ];
    let s = render_copy_command(&argv, ShellFlavor::Posix);
    // `shlex::try_quote` uses double quotes by default for safe strings.
    assert!(
        s.contains("'Vault Cold Storage'") || s.contains("\"Vault Cold Storage\""),
        "expected wallet-name to be quoted in: {}",
        s
    );
}

#[test]
fn posix_value_with_single_quote_is_quoted() {
    let argv = vec![
        "mnemonic".into(),
        "convert".into(),
        "--passphrase".into(),
        "it's a secret".into(),
    ];
    let s = render_copy_command(&argv, ShellFlavor::Posix);
    // shlex emits "it'\\''s a secret" or "\"it's a secret\""; either is
    // POSIX-safe. We only assert that the unquoted form is NOT present.
    assert!(
        !s.ends_with(" it's a secret"),
        "passphrase must not appear unquoted in: {}",
        s
    );
    // Round-trip: shlex::split should recover the original tokens.
    let parsed = shlex::split(&s).expect("shlex round-trip");
    assert_eq!(parsed, argv);
}

#[test]
fn posix_value_with_double_quote_is_quoted() {
    let argv = vec!["mnemonic".into(), "--name".into(), r#"a"b"#.into()];
    let s = render_copy_command(&argv, ShellFlavor::Posix);
    let parsed = shlex::split(&s).expect("shlex round-trip");
    assert_eq!(parsed, argv);
}

#[test]
fn posix_value_with_dollar_sign_is_quoted() {
    // Dollar sign would trigger shell variable expansion if unquoted.
    let argv = vec!["mnemonic".into(), "--passphrase".into(), "$HOME".into()];
    let s = render_copy_command(&argv, ShellFlavor::Posix);
    let parsed = shlex::split(&s).expect("shlex round-trip");
    assert_eq!(parsed, argv);
    // Defensive: the literal `$HOME` substring (unquoted) must not appear
    // as a bare word.
    assert!(
        !s.split_whitespace().any(|tok| tok == "$HOME"),
        "$HOME must not appear unquoted in: {}",
        s
    );
}

#[test]
fn posix_value_with_backtick_is_quoted() {
    // Backtick would trigger command substitution if unquoted.
    let argv = vec!["mnemonic".into(), "--passphrase".into(), "`id`".into()];
    let s = render_copy_command(&argv, ShellFlavor::Posix);
    let parsed = shlex::split(&s).expect("shlex round-trip");
    assert_eq!(parsed, argv);
}

// ── Windows cmd.exe ───────────────────────────────────────────────────────

#[test]
fn windows_simple_argv_double_quoted() {
    let argv = vec!["mnemonic".into(), "bundle".into(), "--network".into(), "mainnet".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert_eq!(s, "\"mnemonic\" ^\r\n  \"bundle\" ^\r\n  \"--network\" ^\r\n  \"mainnet\"");
}

#[test]
fn windows_embedded_double_quote_escaped_as_backslash_quote() {
    // ArgvQuote canonical rule: lone `"` (no preceding `\`) is encoded as
    // `\"` (n=0 → 2n+1 = 1 backslash + literal `"`). The `""` form is
    // NOT recognized by CommandLineToArgvW as a literal-`"` escape —
    // it parses as (close-quote, reopen-quote) and the literal `"` is
    // lost. R2 C-1 fold.
    let argv = vec!["mnemonic".into(), r#"a"b"#.into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert_eq!(s, "\"mnemonic\" ^\r\n  \"a\\\"b\"");
}

#[test]
fn windows_empty_string_renders_as_empty_quotes() {
    let argv = vec!["mnemonic".into(), "--ms1".into(), "".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert_eq!(s, "\"mnemonic\" ^\r\n  \"--ms1\" ^\r\n  \"\"");
}

#[test]
fn windows_line_continuation_separator_is_caret_crlf_indent() {
    // Pin the exact separator so future maintainers don't accidentally
    // change `\r\n` to `\n` (CRLF is correct for cmd.exe paste-targets).
    let argv = vec!["a".into(), "b".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert!(s.contains(" ^\r\n  "));
}

// R1 I-1 fold: Windows path ending in `\` must not consume the close-quote.
// `CommandLineToArgvW` treats `\` immediately before `"` as an escape, so
// `"C:\tmp\"` parses as the unclosed token `C:\tmp"`. The fix is to double
// any backslash run that precedes a `"` (interior or closing).

#[test]
fn windows_trailing_backslash_does_not_break_close_quote() {
    let argv = vec!["mnemonic".into(), "--output".into(), r"C:\tmp\".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    // Expected encoded form: "C:\tmp\\" — the single trailing backslash
    // is doubled so the close-quote is unambiguous.
    assert!(
        s.contains(r#""C:\tmp\\""#),
        "expected doubled trailing backslash in: {}",
        s
    );
}

#[test]
fn windows_interior_backslash_run_unchanged() {
    // Pure interior backslashes (not followed by `"`) pass through.
    let argv = vec!["a".into(), r"C:\Users\Alice\file.txt".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert!(s.contains(r#""C:\Users\Alice\file.txt""#), "interior backslashes mangled: {}", s);
}

#[test]
fn windows_backslash_immediately_before_embedded_quote_emits_three_backslashes() {
    // Input: 4 chars — a, `\`, `"`, b. Preceding `\` count n=1; emit
    // 2n+1 = 3 backslashes + literal `"`. R2 C-1 fold.
    let argv = vec!["literal".into(), "a\\\"b".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    // Expected encoded form: "a\\\"b" — 3 backslashes (the input's `\`
    // tripled per the 2n+1 rule), then the literal `"`, then `b`.
    assert!(
        s.contains("\"a\\\\\\\"b\""),
        "expected `2n+1 = 3` backslashes before `\"` in: {}",
        s
    );
}

#[test]
fn windows_double_backslash_before_quote_emits_five_backslashes() {
    // Input: 5 chars — a, `\`, `\`, `"`, b. Preceding `\` count n=2; emit
    // 2n+1 = 5 backslashes + literal `"`. R2 C-1 fold.
    let argv = vec!["literal".into(), "a\\\\\"b".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    // Expected: "a\\\\\\\"b" — 5 backslashes then `"` then `b`.
    assert!(
        s.contains("\"a\\\\\\\\\\\"b\""),
        "expected `2n+1 = 5` backslashes before `\"` in: {}",
        s
    );
}

/// Round-trip helper: parse `s` per `CommandLineToArgvW` rules and
/// return the argv vector. Used by R2 C-1 fold to verify our quoting
/// roundtrips correctly. Implements rules 1-3 from `cmd_quote`'s
/// doc-comment in reverse.
fn parse_cmdline_argv_w(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut argv: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            let mut n = 0usize;
            while i < chars.len() && chars[i] == '\\' {
                n += 1;
                i += 1;
            }
            if i < chars.len() && chars[i] == '"' {
                // n backslashes followed by `"`. Emit n/2 literal `\`.
                for _ in 0..(n / 2) {
                    cur.push('\\');
                }
                if n % 2 == 1 {
                    cur.push('"');
                } else {
                    in_quotes = !in_quotes;
                }
                i += 1;
            } else {
                for _ in 0..n {
                    cur.push('\\');
                }
            }
        } else if chars[i] == '"' {
            in_quotes = !in_quotes;
            i += 1;
        } else if chars[i].is_whitespace() && !in_quotes {
            if !cur.is_empty() {
                argv.push(std::mem::take(&mut cur));
            }
            i += 1;
        } else {
            cur.push(chars[i]);
            i += 1;
        }
    }
    if !cur.is_empty() {
        argv.push(cur);
    }
    argv
}

/// Round-trip a single argv element through `cmd_quote` →
/// `parse_cmdline_argv_w`; assert recovery.
fn assert_cmd_roundtrip(input: &str) {
    let argv_in = vec!["bin".to_string(), input.to_string()];
    // Render the way the real GUI does, but skip the multiline `^\r\n`
    // separator — collapse to plain spaces for parse-time.
    let rendered_parts: Vec<String> = argv_in
        .iter()
        .map(|s| {
            // Reuse the actual cmd_quote via render_copy_command on a
            // single-element vec, then strip the wrapping.
            render_copy_command(std::slice::from_ref(s), ShellFlavor::WindowsCmd)
        })
        .collect();
    let line = rendered_parts.join(" ");
    let parsed = parse_cmdline_argv_w(&line);
    assert_eq!(parsed, argv_in, "roundtrip broke for input {:?}", input);
}

#[test]
fn windows_roundtrip_all_embedded_quote_shapes() {
    // The GUI's assemble_argv layer omits empty Text/Path values per
    // SPEC §6.7, so empty argv elements are not a real GUI emission.
    // The roundtrip set below pins the real-world shapes that DO reach
    // render_copy_command.
    assert_cmd_roundtrip("simple");
    assert_cmd_roundtrip("with space");
    assert_cmd_roundtrip(r#"a"b"#); // lone `"`
    assert_cmd_roundtrip("a\\\"b"); // 1 `\` + `"`
    assert_cmd_roundtrip("a\\\\\"b"); // 2 `\` + `"`
    assert_cmd_roundtrip(r"C:\tmp\"); // trailing `\`
    assert_cmd_roundtrip(r"C:\Users\Alice\file.txt"); // interior `\`
}
