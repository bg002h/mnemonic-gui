INPUT_CHARSET = "0123456789()[],'/*abcdefgh@:$%{}IJKLMNOPQRSTUVWXYZ&+-.;<=>?!^_|~ijklmnopqrstuvwxyzABCDEFGH`#\"\\ "
CHECKSUM_CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
GENERATOR = [0xf5dee51989, 0xa9fdca3312, 0x1bab10e32d, 0x3706b1677a, 0x644d626ffd]
def polymod(c, val):
    c0 = c >> 35
    c = ((c & 0x7ffffffff) << 5) ^ val
    for i in range(5):
        if (c0 >> i) & 1: c ^= GENERATOR[i]
    return c
def checksum(s):
    c = 1; cls = 0; clscount = 0
    for ch in s:
        pos = INPUT_CHARSET.find(ch)
        c = polymod(c, pos & 31); cls = cls * 3 + (pos >> 5); clscount += 1
        if clscount == 3: c = polymod(c, cls); cls = 0; clscount = 0
    if clscount > 0: c = polymod(c, cls)
    for j in range(8): c = polymod(c, 0)
    c ^= 1
    return ''.join(CHECKSUM_CHARSET[(c >> (5 * (7 - j))) & 31] for j in range(8))
