Value variants (corpus.py, shared): suffixes '', '\n', '\r\n', '\r', '\n\n', '\r\n\r\n', '\n\r\n', '\r\r\n', '\n\r', '\r\r', '\n\n\n', '  ', ' \n', '  \r\n', '\t\n', '\t', '\nX'; prefixes '\n', '\r\n', ' ', '\t'; 167 channel cells.

| channel kind | measured terminator | cells |
|---|---|---|
| DashValue | '\r\n' | 58 |
| DashValue | lenient (null) | 10 |
| EnvRef | '' | 41 |
| EnvRef | lenient (null) | 16 |
| FileFlag | '\n' | 2 |
| FileFlag | '\r\n' | 2 |
| InFile | '\r\n' | 8 |
| PosDash | '\r\n' | 5 |
| StdinToggle | '\r\n' | 22 |
| StdinToggle | lenient (null) | 3 |

Lenient cells (terminator null; every mismatch is argv-fails/channel-ok, 108 are both-ok-different): `mnemonic addresses --passphrase` EnvRef; `mnemonic bundle --passphrase` EnvRef; `mnemonic convert --from entropy=` DashValue; `mnemonic convert --from xprv=` DashValue; `mnemonic convert --from minikey=` DashValue; `mnemonic convert --passphrase` EnvRef; `mnemonic convert --bip38-passphrase` EnvRef; `mnemonic restore --passphrase` EnvRef; `mnemonic derive-child --passphrase` EnvRef; `mnemonic silent-payment --passphrase` EnvRef; `mnemonic repair --ms1` DashValue; `mnemonic inspect --ms1` DashValue; `ms derive --passphrase` EnvRef; `mnemonic derive-child --from xprv=` DashValue; `mnemonic convert --from wif=` DashValue; `mnemonic verify-bundle --passphrase` EnvRef; `mnemonic electrum-decrypt --decrypt-password` EnvRef; `mnemonic xpub-search path-of-xpub --ms1` StdinToggle(--ms1-stdin); `mnemonic xpub-search path-of-xpub --passphrase` EnvRef; `mnemonic xpub-search account-of-descriptor --ms1` StdinToggle(--ms1-stdin); `mnemonic xpub-search account-of-descriptor --passphrase` EnvRef; `mnemonic xpub-search passphrase-of-xpub --passphrase` EnvRef; `mnemonic slip39 split --passphrase` EnvRef; `mnemonic slip39 combine --passphrase` EnvRef; `mnemonic convert --from bip38=` DashValue; `mnemonic slip39 split --from entropy=` DashValue; `mnemonic ms-shares split --from entropy=` DashValue; `mnemonic xpub-search passphrase-of-xpub --ms1` StdinToggle(--ms1-stdin); `mnemonic import-wallet --decrypt-password` EnvRef.

`--flag=VALUE` byte-identical to `--flag VALUE` (R3 Nm13; None = not a value-form input): False 1, None 43, True 40; not exact: `ms hashlock --hashlock-phrase`.

Per-input CLI `@env:` value rule (R3 NI7; None = no working CLI `@env:`, the GUI treats the bytes as typed): None 27, strip-one-trailing-newline 16, verbatim 41.

With NO terminator (fold 1's delivery), a wrong output at exit 0/4 on 28 cells: `mnemonic addresses --passphrase` DashValue; `mnemonic addresses --passphrase` StdinToggle(--passphrase-stdin); `mnemonic bundle --passphrase` DashValue; `mnemonic bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --passphrase` DashValue; `mnemonic convert --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --bip38-passphrase` DashValue; `mnemonic convert --bip38-passphrase` StdinToggle(--bip38-passphrase-stdin); `mnemonic restore --passphrase` DashValue; `mnemonic restore --passphrase` StdinToggle(--passphrase-stdin); `mnemonic derive-child --passphrase` DashValue; `mnemonic derive-child --passphrase` StdinToggle(--passphrase-stdin); `mnemonic silent-payment --passphrase` DashValue; `mnemonic silent-payment --passphrase` StdinToggle(--passphrase-stdin); `ms derive --passphrase` DashValue; `ms derive --passphrase` StdinToggle(--passphrase-stdin); `mnemonic verify-bundle --passphrase` DashValue; `mnemonic verify-bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search path-of-xpub --passphrase` DashValue; `mnemonic xpub-search path-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search account-of-descriptor --passphrase` DashValue; `mnemonic xpub-search account-of-descriptor --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search passphrase-of-xpub --passphrase` DashValue; `mnemonic xpub-search passphrase-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 split --passphrase` DashValue; `mnemonic slip39 split --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 combine --passphrase` DashValue; `mnemonic slip39 combine --passphrase` StdinToggle(--passphrase-stdin).
