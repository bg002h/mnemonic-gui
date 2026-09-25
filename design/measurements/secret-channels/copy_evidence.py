"""R2 NI4 / Nm9: run the Copy recipes of DESIGN §A7 in bash, zsh and fish, and compare with
argv carrying the exact bytes (fingerprints). The "typed" recipes read the user's typing from
the shell's stdin, exactly as a paste-and-type session would.  Writes copy_evidence.md."""
import os, shutil, subprocess
B = os.environ["BIN_DIR"].rstrip("/") + "/"
P = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
# `-` and `@env:…` are not here: a typed field holding them never reaches a TYPED-value recipe
# (C1 refuses `-` and resolves `@env:VAR`, §A3a), and argv-exact would itself re-interpret them.
VALUES = ["pad", "  pad  ", "\tpad", "pad\t", " a  b ", "back\\slash", "tab\tin", "a'b\"c", "*", "$HOME", "%s%d"]
CMD = f'{B}mnemonic restore --from phrase=@env:PHR --template bip84 --passphrase @env:MNEMONIC_GUI_S1'
RECIPES = {  # the typed-EnvRef row of §A7, per shell
    "bash": 'IFS= read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1; ' + CMD,
    "zsh": 'IFS= read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1; ' + CMD,
    "fish": 'read -s -x --delimiter \\n MNEMONIC_GUI_S1; ' + CMD,
}
OLD = {"bash": 'read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1; ' + CMD,
       "zsh": 'read -rs MNEMONIC_GUI_S1; export MNEMONIC_GUI_S1; ' + CMD,
       "fish": 'read -sx MNEMONIC_GUI_S1; ' + CMD}
env = dict(os.environ, PHR=P)


def fp(out):
    for l in out.splitlines():
        if "master fingerprint" in l:
            return l.split()[2]
    return "(error)"


def argv_exact(v):
    return fp(subprocess.run([B + "mnemonic", "restore", "--allow-argv-secret", "--from", "phrase=" + P,
                              "--template", "bip84", "--passphrase", v], capture_output=True, text=True).stdout)


def shell(sh, script, stdin):
    return fp(subprocess.run([sh, "-c", script], input=stdin, capture_output=True, text=True, env=env).stdout)


shells = [s for s in ("bash", "zsh", "fish") if shutil.which(s)]
rows = ["| typed passphrase | argv-exact | " + " | ".join(f"{s} (§A7 recipe)" for s in shells)
        + " | " + " | ".join(f"{s} (fold-2 recipe)" for s in shells) + " |",
        "|---|---|" + "---|" * (2 * len(shells))]
bad = 0
for v in VALUES:
    want = argv_exact(v)
    new = [shell(s, RECIPES[s], v + "\n") for s in shells]
    old = [shell(s, OLD[s], v + "\n") for s in shells]
    bad += sum(x != want for x in new)
    rows.append(f"| `{v!r}` | {want} | " + " | ".join(("" if x == want else "**") + x + ("" if x == want else "**") for x in new)
                + " | " + " | ".join(("" if x == want else "**") + x + ("" if x == want else "**") for x in old) + " |")

# typed stdin row: the user types into the CLI itself (value, Enter, Ctrl-D)
stdin_bad = 0
for v in VALUES:
    got = fp(subprocess.run([B + "mnemonic", "restore", "--from", "phrase=@env:PHR", "--template", "bip84",
                             "--passphrase-stdin"], input=v + "\n", capture_output=True, text=True, env=env).stdout)
    stdin_bad += got != argv_exact(v)

# Nm9: ms combine's share group. $VAR provenance: printf '%s\n' "$S1" "$S2" | ms combine -- -
SH = ["ms12ec7pqsf34k729gl6yrsfk9q244kzsx62s9krxwywxv4jav", "ms12ec7pp3txgcymeun2ws3tceqmggc83f2m3l2cj299w9l9vr"]
want = subprocess.run([B + "ms", "combine", "--allow-argv-secret", "--"] + SH, capture_output=True, text=True).stdout
genv = dict(os.environ, S1=SH[0], S2=SH[1])
grp = {}
for s in shells:
    g = subprocess.run([s, "-c", f"printf '%s\\n' \"$S1\" \"$S2\" | {B}ms combine -- -"], capture_output=True, text=True, env=genv).stdout
    grp[s] = g == want
typed_grp = subprocess.run([B + "ms", "combine", "--", "-"], input="\n".join(SH) + "\n", capture_output=True, text=True).stdout == want

with open("copy_evidence.md", "w") as f:
    f.write("\n".join(rows) + "\n\n")
    f.write(f"§A7 recipe mismatches: {bad} of {len(VALUES) * len(shells)}. "
            f"Typed stdin row (`--passphrase-stdin`, value + Enter): {stdin_bad} mismatches of {len(VALUES)}. "
            f"Share group, `printf '%s\\n' \"$S1\" \"$S2\" | ms combine -- -`: "
            + ", ".join(f"{s} {'==' if ok else '!='} argv" for s, ok in grp.items())
            + f"; typed (one share per line, Ctrl-D): {'==' if typed_grp else '!='} argv.\n")
print(open("copy_evidence.md").read())
