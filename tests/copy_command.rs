//! Copy-command shell-quoting tests (SPEC §6.6, Phase 2 R1 I-4).
//!
//! The render output is for display copy-paste only; it is NEVER re-parsed
//! or used to spawn the subprocess. Posix uses `shlex::try_quote`; Windows
//! uses double-quote-doubling per cmd.exe conventions.

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
fn windows_embedded_double_quote_is_doubled() {
    // cmd.exe convention: `"` inside `"…"` is escaped as `""`.
    let argv = vec!["mnemonic".into(), r#"a"b"#.into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert_eq!(s, "\"mnemonic\" ^\r\n  \"a\"\"b\"");
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
fn windows_backslash_immediately_before_embedded_quote_is_doubled() {
    // Input contains exactly: a, `\`, `"`, b. The `\` IS immediately
    // followed by `"`, so it must be doubled to disambiguate from the
    // `\"` escape sequence CommandLineToArgvW would otherwise interpret.
    // Expected encoded form: "a\\""b" — `\\` (doubled) then `""` (literal).
    let argv = vec!["literal".into(), "a\\\"b".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert!(
        s.contains(r#""a\\""b""#),
        "expected `\\` before `\"` to be doubled in: {}",
        s
    );
}

#[test]
fn windows_double_backslash_before_quote_is_doubled_to_four() {
    // Input: a, `\`, `\`, `"`, b. Two backslashes immediately before `"`.
    // Both must be doubled → 4 backslashes, then literal `""`.
    let argv = vec!["literal".into(), "a\\\\\"b".into()];
    let s = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert!(
        s.contains(r#""a\\\\""b""#),
        "expected 2 backslashes before `\"` to become 4 in: {}",
        s
    );
}
