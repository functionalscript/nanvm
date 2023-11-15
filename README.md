# NaNVM

## Embedded String Size

| Size | Bits | Total                     | Total bits |
|------|------|---------------------------|------------|
| 1x16 | 16   | 0x1_0000                  |  16        |
| 2x16 | 32   | 0x1_0001_0000             | >32        |
| 4x12 | 48   | 0x1_0001_0001_0000        | >48        |
| 5x10 | 50   | 0x5_0001_0001_0000        | >50        |
| 6x8  | 48   | 0x6_0001_0001_0000        | >50        |
| 7x7  | 49   | 0x8_0001_0001_0000        | >52        |
| 8x6  | 48   | 0x9_0001_0001_0000        | >52        |
| 9x5  | 45   | 0x9_2001_0001_0000        | >52        |
| 10x5 | 50   | 0xD_2001_0001_0000        | >52        |

## Pointer Size

2^48/2^3 = 2^45 = 0x2000_0000_0000

### Pointer Types

- string
- bigint
- object
  - object
  - Array
  - UInt8Array
  - ...

## Constants

- +Inf,Nan,-Inf,true,false,undefined: 6

## Embedded BigInt

## Embedded Index String
