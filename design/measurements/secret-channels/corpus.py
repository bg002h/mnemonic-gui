"""ONE value corpus for every measuring and oracle script (R4 NI9): run_bytes.py derives terminators
and the per-input rule over it, run_plans.py's NI1/interim oracle legs run it, and
measure_reinterpret.py / the NC1 leg run CHANNEL_VARIANTS. Widening it widens every script at once."""

# endings appended to a value: every trailing sequence over {\r, \n} up to length 3 or 4 that
# matters, whitespace before a newline, a lone trailing tab, and an interior newline
SUFFIXES = ["", "\n", "\r\n", "\r", "\n\n", "\r\n\r\n", "\n\r\n", "\r\r\n", "\n\r", "\r\r", "\n\n\n",
            "  ", " \n", "  \r\n", "\t\n", "\t", "\nX"]
# prefixes: leading newline, CRLF, space and tab
PREFIXES = ["\n", "\r\n", " ", "\t"]
VARIANTS = [("suffix", e) for e in SUFFIXES] + [("prefix", p) for p in PREFIXES]


def apply(value, variant):
    kind, s = variant
    return value + s if kind == "suffix" else s + value


# spellings a CLI might conceivably treat as a channel when they arrive as a value (NI8). {V} is a
# variable name. measure_reinterpret.py probes each on argv; run_plans.py's NC1 leg puts each in
# the user's variable. None is expected to be DELIVERED by the GUI: the broad predicate refuses all
# that could plausibly be read as a channel (plan.looks_like_channel), whatever any CLI does.
CHANNEL_VARIANTS = ["-", " -", "- ", "-\n", "\t-", "-\r\n", " -",
                    "@env:{V}", " @env:{V}", "@env:{V} ", "\t@env:{V}", "\n@env:{V}", "@env:{V}\n", "@env:{V}\r\n",
                    "@ENV:{V}", "@Env:{V}", "@env: {V}", " @env:{V}", "​@env:{V}", "＠env:{V}", "@env{V}"]
