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

// ── G2 (bug hunt 2026-11-25): cmd.exe, not only CommandLineToArgvW ─────────
//
// `cmd_quote` produces text for `CommandLineToArgvW`; the Windows Copy is
// pasted into `cmd.exe`, which parses it first. `cmd.exe` toggles its quote
// state on EVERY `"` (it does not know `\"`), acts on `& | < > ( )` outside
// quotes, expands `%NAME%` even inside quotes, `!NAME!` under delayed
// expansion, and ends the line at CR/LF. The Windows Copy is therefore
// disabled for a token holding `"`, `%`, `!`, CR or LF; anything else passed
// through `cmd_quote` is inert to cmd.exe.

use mnemonic_gui::form::channels::copy::{copy_commands, CopyState};
use mnemonic_gui::form::invocation::{windows_cmd_copy_blocker, WINDOWS_CMD_UNSAFE_TOOLTIP};
use mnemonic_gui::schema::{self as gui_schema, FlagValue, FormState};

/// `cmd.exe`'s phase-2 view of a pasted line: the characters it would act on
/// outside its quoted regions (quote state toggled by every `"`, `^` escapes
/// the next char), plus any `%` or `!` anywhere (expanded inside quotes too).
fn cmd_active_chars(line: &str) -> Vec<char> {
    let mut out = Vec::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => in_q = !in_q,
            '%' | '!' => out.push(c),
            '^' if !in_q => {
                chars.next(); // escaped (the `^\r\n` continuation, too)
            }
            '&' | '|' | '<' | '>' | '(' | ')' if !in_q => out.push(c),
            _ => {}
        }
    }
    out
}

#[test]
fn g2_hostile_quote_value_shows_cmd_would_run_it_and_is_blocked() {
    let hostile = r#"x" & calc.exe & rem ""#;
    let argv = vec!["mnemonic".to_string(), "--label".to_string(), hostile.to_string()];
    // The bug: cmd_quote's output leaves `&` outside cmd.exe's quotes.
    let rendered = render_copy_command(&argv, ShellFlavor::WindowsCmd);
    assert!(cmd_active_chars(&rendered).contains(&'&'), "{rendered}");
    // The fix: the Windows Copy refuses such a token.
    assert_eq!(
        windows_cmd_copy_blocker(argv.iter().map(String::as_str)),
        Some(WINDOWS_CMD_UNSAFE_TOOLTIP)
    );
}

#[test]
fn g2_percent_bang_and_newlines_are_blocked() {
    for v in ["%USERPROFILE%", "a!b!", "50%", "line1\nline2", "a\rb", "\""] {
        assert!(windows_cmd_copy_blocker([v]).is_some(), "{v:?} must block");
    }
}

#[test]
fn g2_every_admitted_token_is_inert_to_cmd_and_round_trips() {
    // Shell metacharacters WITHOUT `"`/`%`/`!` stay inside cmd's quotes.
    let admitted = [
        "simple",
        "with space",
        "a & b | c < d > e (f) ^g",
        r"C:\tmp\",
        r"C:\Users\Alice\file (1).txt",
        "wsh(sortedmulti(2,[aabbccdd/48'/0'/0'/2']xpub/<0;1>/*))#abcd1234",
        "@env:MNEMONIC_GUI_S0",
    ];
    for v in admitted {
        assert!(windows_cmd_copy_blocker([v]).is_none(), "{v:?} must be admitted");
        let argv = vec!["mnemonic".to_string(), "--x".to_string(), v.to_string()];
        let line = render_copy_command(&argv, ShellFlavor::WindowsCmd);
        assert_eq!(cmd_active_chars(&line), Vec::<char>::new(), "{v:?} → {line:?}");
        let flat = line.replace(" ^\r\n  ", " ");
        assert_eq!(parse_cmdline_argv_w(&flat), argv, "{v:?}");
    }
}

#[test]
fn g2_windows_copy_button_disabled_for_a_cmd_unsafe_field_posix_unaffected() {
    let restore = gui_schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "restore")
        .unwrap();
    let none = |_: &str| None;
    let st = FormState::from_pairs(vec![(
        "--from",
        FlagValue::Text(r#"xpub=x" & calc.exe & rem ""#.into()),
    )]);
    let (posix, windows) = copy_commands(&gui_schema::mnemonic::SCHEMA, restore, &st, &none);
    assert!(matches!(posix, CopyState::Ready(_)), "{posix:?}");
    assert_eq!(windows, CopyState::Disabled(WINDOWS_CMD_UNSAFE_TOOLTIP.into()));
    // control: an ordinary value still copies on Windows
    let st = FormState::from_pairs(vec![("--from", FlagValue::Text("xpub=xpubABC".into()))]);
    let (_, windows) = copy_commands(&gui_schema::mnemonic::SCHEMA, restore, &st, &none);
    assert!(matches!(windows, CopyState::Ready(_)), "{windows:?}");
}

#[test]
fn g2_windows_copy_disabled_when_a_secret_plan_carries_a_cmd_unsafe_token() {
    // The secret-bearing branch of the Copy renderer is gated too: the ms1
    // card goes by `@env:` (an EnvRef binding, which the Windows Copy can
    // spell), and a PUBLIC field beside it holds a `"` that cmd would act on.
    let restore = gui_schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "restore")
        .unwrap();
    let env = |k: &str| {
        (k == "SEED").then(|| "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_string())
    };
    let with_output = |out: &str| {
        FormState::from_pairs(vec![
            ("--from", FlagValue::Text("ms1=@env:SEED".into())),
            ("--output", FlagValue::Path(out.into())),
        ])
    };
    // control: the secret branch is reached and copies on Windows
    let (_, windows) =
        copy_commands(&gui_schema::mnemonic::SCHEMA, restore, &with_output("out.txt"), &env);
    let CopyState::Ready(t) = &windows else {
        panic!("precondition: an EnvRef-only plan copies on Windows: {windows:?}")
    };
    assert!(t.contains("@env:SEED") && t.contains("out.txt"), "{t:?}");
    let (posix, windows) = copy_commands(
        &gui_schema::mnemonic::SCHEMA,
        restore,
        &with_output(r#"o" & calc.exe & rem ""#),
        &env,
    );
    assert!(matches!(posix, CopyState::Ready(_)), "{posix:?}");
    assert_eq!(windows, CopyState::Disabled(WINDOWS_CMD_UNSAFE_TOOLTIP.into()));
}

/// Review F-M1 / r2-m1: a secret-bearing Copy (either flavour) starts with a
/// comment saying it needs the pinned CLI version or newer; a Copy with no
/// secret does not carry it.
#[test]
fn fm1_secret_copy_starts_with_the_version_warning() {
    use mnemonic_gui::cli_version::pinned_version_for;
    use mnemonic_gui::form::channels::copy::copy_version_warning;
    let restore = gui_schema::mnemonic::SCHEMA
        .subcommands
        .iter()
        .find(|s| s.name == "restore")
        .unwrap();
    let env = |k: &str| {
        (k == "SEED").then(|| "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_string())
    };
    let pinned = pinned_version_for("mnemonic").unwrap();
    let st = FormState::from_pairs(vec![("--from", FlagValue::Text("ms1=@env:SEED".into()))]);
    let (posix, windows) = copy_commands(&gui_schema::mnemonic::SCHEMA, restore, &st, &env);
    for (state, c) in [(&posix, "#"), (&windows, "REM")] {
        let CopyState::Ready(t) = state else {
            panic!("precondition: ready {state:?}")
        };
        let first = t.lines().next().unwrap();
        assert_eq!(first, copy_version_warning(c, pinned), "{t:?}");
        assert!(
            first.contains(&format!("Needs {pinned} or newer")),
            "{first}"
        );
        let last = t.lines().last().unwrap();
        assert!(
            !last.starts_with(c),
            "the command stays after the comments: {t:?}"
        );
    }
    let none = |_: &str| None;
    let st = FormState::from_pairs(vec![("--from", FlagValue::Text("xpub=xpubABC".into()))]);
    let (posix, _) = copy_commands(&gui_schema::mnemonic::SCHEMA, restore, &st, &none);
    assert!(!posix.text().unwrap().contains("Needs "), "{posix:?}");
}
