## 64-bit Enum

```
0_111_1111__1111_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 | -Inf
1_111_1111__1111_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 | +Inf
0_111_1111__1111_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0001 | NaN

Rest = 2^(64 - 11) - 3 = 2^53 - 3, 3 values: -Inf, +Inf, NaN.

Additional constants (4): undefined, null, true, false

Identifiers: 
    52 letters: 26: `a..z` + 26: `A..Z`
    2 symbols: `$`, `_`
    10 digits: `0..9`
    total: 64 = 6 bits

String:
    String8 x6 : 8 x 6 bit = 48 bit

    String7 x7 : 7 x 7 bit = 49 bit
    String6 x7 : 6 x 7 bit = 42 bit
    String5 x7 : 5 x 7 bit = 35 bit
    String4 x7 : 4 x 7 bit = 28 bit

    String3: 3 x 16 bit = 48 bit
    String2: 2 x 16 bit = 32 bit
    String1: 1 x 16 bit = 16 bit

    String0.

    StringU32: 32 bit, See 'array index'.

    2^48 +
    2^49 + 2^42 + 2^35 + 2^28 +
    2^48 + 2^32 + 2^16 +
    2^32 +
    1 =
    2^49 +
    2^48 + 2^48 +
    2^42 +
    2^35 +
    2^32 +
    2^28 +
    2^16 +
    2^0 =
    2^50 + 2^42 + 2^35 + 2^32 + 2^28 + 2^16 + 2^0 < 2^51

BigInt33:
```
