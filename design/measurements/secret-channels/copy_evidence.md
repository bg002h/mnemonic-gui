| typed passphrase | argv-exact | bash (§A7 recipe) | zsh (§A7 recipe) | fish (§A7 recipe) | bash (fold-2 recipe) | zsh (fold-2 recipe) | fish (fold-2 recipe) |
|---|---|---|---|---|---|---|---|
| `'pad'` | 8728f37c | 8728f37c | 8728f37c | 8728f37c | 8728f37c | 8728f37c | 8728f37c |
| `'  pad  '` | bc403662 | bc403662 | bc403662 | bc403662 | **8728f37c** | **8728f37c** | bc403662 |
| `'\tpad'` | 6e09a416 | 6e09a416 | 6e09a416 | 6e09a416 | **8728f37c** | **8728f37c** | 6e09a416 |
| `'pad\t'` | 665b5e3e | 665b5e3e | 665b5e3e | 665b5e3e | **8728f37c** | **8728f37c** | 665b5e3e |
| `' a  b '` | 4fecc1c4 | 4fecc1c4 | 4fecc1c4 | 4fecc1c4 | **e3ef7859** | **e3ef7859** | 4fecc1c4 |
| `'back\\slash'` | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 | a7cda7f9 |
| `'tab\tin'` | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d | 3c98ed1d |
| `'a\'b"c'` | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 | 7dd841d7 |
| `'*'` | 05469108 | 05469108 | 05469108 | 05469108 | 05469108 | 05469108 | 05469108 |
| `'$HOME'` | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 | 85f87f43 |
| `'%s%d'` | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 | 2def6c58 |

§A7 recipe mismatches: 0 of 33. Typed stdin row (`--passphrase-stdin`, value + Enter): 0 mismatches of 11. Share group, `printf '%s\n' "$S1" "$S2" | ms combine -- -`: bash == argv, zsh == argv, fish == argv; typed (one share per line, Ctrl-D): == argv.

Multi-line typed values (R3 Nm11): `'mid\nline'`: argv-exact 74db797b, `read` recipe bash **bdb4bfbc**, zsh **bdb4bfbc**, fish **bdb4bfbc**, typed stdin row ==; `'a\r\nb'`: argv-exact d5e2d0db, `read` recipe bash **e19f2f8f**, zsh **e19f2f8f**, fish **e19f2f8f**, typed stdin row ==. So Copy is disabled for a typed EnvRef-bound value holding CR or LF; the stdin row stays.
