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
```

|hi\lo   |00        |01        |10        |11       |
|--------|----------|----------|----------|---------|
|000     |str3      |bigint    |          |         |
|001     |str8      |bigint    |          |         |
|010     |str7p0    |bigint    |          |         |
|011     |str7p1    |bigint    |          |         |
|100     |          |          |          |number   |
|101     |          |          |          |boolean  |
|110     |str654210 |          |null      |undefined|
|111: ref|string_ptr|bigint_ptr|object_ptr|array_ptr|

```js
const typeofF = (hi, lo) => lo === 0b11
    ? ['number', 'boolean', ' undefined', 'object'][hi & 0b11]
    : ['string', 'bigint', 'object']

const isRef = (hi, lo) => hi === 0b111

const instanceofArray = (hi, lo) => hi === 0b111 & lo === 0b11
```

|hi\lo |000      |001   |010        |011        |100   |101    |110      |111      |
|------|---------|------|-----------|-----------|------|-------|---------|---------|
|00    |         |str7p0|+bigint    |-bigint    |      |       |         |number   |
|01    |str654210|str7p1|+bigint    |-bigint    |      |       |         |boolean  |
|10    |str3     |str8  |           |           |      |       |null     |undefined|
|11:ref|str_ref  |      |+bigint_ptr|-bigint_ptr|fn_ptr|efn_ptr|objec_ptr|array_ptr|

```js
const typeofFunc = (hi, lo) => lo === 0b111
    ? ['number', 'boolean', ' undefined', 'object'][hi]
    : ['string', 'bigint', 'function', 'object'][lo >> 1]

const isRef = (hi, lo) => hi === 0b11

const instanceOfObject = (hi, lo) => hi === 0b11 & (lo >> 2) === 0b1
const instanceOfFunction = (hi, lo) => hi === 0b11 & (lo >> 1) === 0b10
const instanceofArray = (hi, lo) => hi === 0b11 & lo === 0b111
```
