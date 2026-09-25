Value variants (corpus.py, shared): suffixes '', '\n', '\r\n', '\r', '\n\n', '\r\n\r\n', '\n\r\n', '\r\r\n', '\n\r', '\r\r', '\n\n\n', '  ', ' \n', '  \r\n', '\t\n', '\t', '\nX'; prefixes '\n', '\r\n', ' ', '\t'; 146 channel cells.

| channel kind | measured terminator | cells |
|---|---|---|
| DashValue | '\r\n' | 41 |
| DashValue | lenient (null) | 10 |
| EnvRef | '' | 53 |
| FileFlag | '\n' | 2 |
| FileFlag | '\r\n' | 2 |
| InFile | '\r\n' | 8 |
| PosDash | '\r\n' | 5 |
| StdinToggle | '\r\n' | 22 |
| StdinToggle | lenient (null) | 3 |

Lenient cells (terminator null; every mismatch is argv-fails/channel-ok, 0 are both-ok-different): `mnemonic convert --from entropy=` DashValue; `mnemonic convert --from xprv=` DashValue; `mnemonic convert --from minikey=` DashValue; `mnemonic repair --ms1` DashValue; `mnemonic inspect --ms1` DashValue; `mnemonic derive-child --from xprv=` DashValue; `mnemonic convert --from wif=` DashValue; `mnemonic xpub-search path-of-xpub --ms1` StdinToggle(--ms1-stdin); `mnemonic xpub-search account-of-descriptor --ms1` StdinToggle(--ms1-stdin); `mnemonic convert --from bip38=` DashValue; `mnemonic slip39 split --from entropy=` DashValue; `mnemonic ms-shares split --from entropy=` DashValue; `mnemonic xpub-search passphrase-of-xpub --ms1` StdinToggle(--ms1-stdin).

`--flag=VALUE` byte-identical to `--flag VALUE` (R3 Nm13; None = not a value-form input): False 2, None 43, True 39; not exact: `ms derive --passphrase`; `ms hashlock --hashlock-phrase`.

Per-input CLI `@env:` value rule (R3 NI7; None = no working CLI `@env:`, the GUI treats the bytes as typed): None 31, verbatim 53.

With NO terminator (fold 1's delivery), a wrong output at exit 0/4 on 14 cells: `mnemonic addresses --passphrase` StdinToggle(--passphrase-stdin); `mnemonic bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --passphrase` StdinToggle(--passphrase-stdin); `mnemonic convert --bip38-passphrase` StdinToggle(--bip38-passphrase-stdin); `mnemonic restore --passphrase` StdinToggle(--passphrase-stdin); `mnemonic derive-child --passphrase` StdinToggle(--passphrase-stdin); `mnemonic silent-payment --passphrase` StdinToggle(--passphrase-stdin); `ms derive --passphrase` StdinToggle(--passphrase-stdin); `mnemonic verify-bundle --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search path-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search account-of-descriptor --passphrase` StdinToggle(--passphrase-stdin); `mnemonic xpub-search passphrase-of-xpub --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 split --passphrase` StdinToggle(--passphrase-stdin); `mnemonic slip39 combine --passphrase` StdinToggle(--passphrase-stdin).
