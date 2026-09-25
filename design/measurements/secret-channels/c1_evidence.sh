#!/bin/bash
# C1 evidence: what a user-typed `-` / `@env:NAME` in a secret field yields under each reading,
# against the release binaries (which predate the F-687 CLI change).
# restore --from phrase=<abandon…about> --template bip84; the user's intended passphrase is "hunter2".
set -u
M=${BIN_DIR:?set BIN_DIR}/mnemonic
P="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
fp() { grep -o 'master fingerprint: [0-9a-f]*' | awk '{print $3}'; }
A=--allow-argv-secret
echo "intended (argv 'hunter2')                                      : $($M restore $A --from "phrase=$P" --template bip84 --passphrase hunter2 2>/dev/null | fp)"
echo "no passphrase                                                  : $($M restore $A --from "phrase=$P" --template bip84 2>/dev/null | fp)"
echo "today's pass-through: --passphrase @env:MY_PW (MY_PW=hunter2)    : $(MY_PW=hunter2 $M restore $A --from "phrase=$P" --template bip84 --passphrase @env:MY_PW 2>/dev/null | fp)"
echo "reading 1, typed text is the bytes: S1='@env:MY_PW' via @env:S1   : $(MY_PW=hunter2 MNEMONIC_GUI_S1='@env:MY_PW' $M restore $A --from "phrase=$P" --template bip84 --passphrase @env:MNEMONIC_GUI_S1 2>/dev/null | fp)"
echo "reading 1, typed text is the bytes: '@env:MY_PW' via -stdin       : $(printf '@env:MY_PW' | MY_PW=hunter2 $M restore $A --from "phrase=$P" --template bip84 --passphrase-stdin 2>/dev/null | fp)"
echo "reading 2, sentinel passed through: --passphrase -  (stdin=hunter2): $(printf 'hunter2' | $M restore $A --from "phrase=$P" --template bip84 --passphrase - 2>/dev/null | fp)"
echo "  = the literal passphrase '-' on argv                              : $($M restore $A --from "phrase=$P" --template bip84 --passphrase - 2>/dev/null | fp)"
echo "DESIGN (GUI resolves \$MY_PW itself), bytes via @env:MNEMONIC_GUI_S1 : $(MNEMONIC_GUI_S1=hunter2 $M restore $A --from "phrase=$P" --template bip84 --passphrase @env:MNEMONIC_GUI_S1 2>/dev/null | fp)"
echo "DESIGN (GUI resolves \$MY_PW itself), bytes via --passphrase-stdin   : $(printf 'hunter2' | $M restore $A --from "phrase=$P" --template bip84 --passphrase-stdin 2>/dev/null | fp)"
S=${BIN_DIR}/mnemonic
echo "reading 2 on silent-payment: --passphrase @env:MY_PW (MY_PW=hunter2) vs argv hunter2:"
a=$($S silent-payment $A --secret "$P" --passphrase hunter2 2>/dev/null | grep -m1 address)
b=$(MY_PW=hunter2 $S silent-payment $A --secret "$P" --passphrase @env:MY_PW 2>/dev/null | grep -m1 address)
[ "$a" = "$b" ] && echo "  same" || echo "  DIFFERENT: the @env: text is taken literally"
echo "Copy spelling for a stdin-bound value from \$MY_PW (R1 Nm7), MY_PW=\$'hunter2\\n' (trailing newline):"
echo "  argv with the exact bytes 'hunter2\\n'                          : $($M restore $A --from "phrase=$P" --template bip84 --passphrase $'hunter2\n' 2>/dev/null | fp)"
for sh in bash zsh fish; do
  if command -v $sh >/dev/null; then
    out=$(MY_PW=$'hunter2\n' PHR="$P" M="$M" $sh -c 'printf '"'"'%s\r\n'"'"' "$MY_PW" | $M restore --from phrase=@env:PHR --template bip84 --passphrase-stdin' 2>/dev/null | fp)
    printf "  %-5s printf '%%s\\\\r\\\\n' \"\$MY_PW\" | … --passphrase-stdin         : %s\n" "$sh" "$out"
  fi
done
echo "R2 NC1: MY_PW='@env:OTHER', OTHER=hunter2 (\$OTHER's wallet is ca2c62d2):"
echo "  CLI's own --passphrase @env:MY_PW (resolves once)                 : $(MY_PW='@env:OTHER' OTHER=hunter2 $M restore $A --from "phrase=$P" --template bip84 --passphrase @env:MY_PW 2>/dev/null | fp)"
echo "  GUI Linux plan: resolved bytes over --passphrase-stdin + \\r\\n      : $(printf '@env:OTHER\r\n' | OTHER=hunter2 $M restore $A --from "phrase=$P" --template bip84 --passphrase-stdin 2>/dev/null | fp)"
echo "  UNGUARDED interim: resolved bytes on argv (re-resolved by the CLI) : $(OTHER=hunter2 $M restore $A --from "phrase=$P" --template bip84 --passphrase '@env:OTHER' 2>/dev/null | fp)"
echo "  GUI, fold 5: refused value-looks-like-a-channel on EVERY path (plan.py; test_plan.py NI8 legs)"
