"""T7 echo helper (DESIGN §A9): stands in for a CLI. It parses its OWN argv
for the three private channel spellings and reports, as JSON on stdout, the
exact bytes it received on each:

  - a token holding `@env:NAME`   -> the variable's bytes
  - a token `/dev/fd/N`           -> the pipe's bytes, read to EOF
  - a token `-`, `<prefix>=-`, or `--X-stdin` -> stdin, read to EOF (once)

plus its argv and every MNEMONIC_GUI_* variable it inherited (the env-scrub
check). Bytes are hex so nothing is lost."""
import json, os, sys

out = {"argv": [a.encode("utf-8", "surrogateescape").hex() for a in sys.argv[1:]],
       "channels": [], "gui_env": sorted(k for k in os.environ if k.startswith("MNEMONIC_GUI_"))}
stdin_used = False
for i, tok in enumerate(sys.argv[1:], start=1):
    if "@env:" in tok:
        name = tok.split("@env:", 1)[1]
        val = os.environb.get(name.encode())
        out["channels"].append({"index": i, "how": "env", "bytes": None if val is None else val.hex()})
    elif tok.startswith("/dev/fd/"):
        with open(tok, "rb") as f:
            out["channels"].append({"index": i, "how": "fd", "bytes": f.read().hex()})
    elif tok == "-" or tok.endswith("=-") or (tok.startswith("--") and tok.endswith("-stdin")):
        if stdin_used:
            out["channels"].append({"index": i, "how": "stdin-again", "bytes": ""})
            continue
        stdin_used = True
        out["channels"].append({"index": i, "how": "stdin", "bytes": sys.stdin.buffer.read().hex()})
print(json.dumps(out))
