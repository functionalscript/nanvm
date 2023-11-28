# Unknown

|S|E, 11 bit    |FM|FV, 51 bit|
|-|-------------|--|----------|
|0|111_1111_1111|0 |+Inf      |
|0|111_1111_1111|1 |NaN       |
|1|111_1111_1111|0 |-Inf      |
|1|111_1111_1111|1 |Ext       |

## Bitwise not number

We need it to make sure that the default value is `undefined` (`void 0`).

|S|E, 11 bit    |FM|FV, 51 bit|
|-|-------------|--|----------|
|0|000_0000_0000|0 |Ext       |
|0|000_0000_0000|1 |1.. NaN   |
|1|000_0000_0000|0 |1.. -Inf  |
|1|000_0000_0000|1 |1.. Ext   |

## Extension

|S|E            | |type|48 bit        |
|-|-------------|-|----|--------------|
|0|000_0000_0000|0|000 |0.. undefined |
| |             | |001 |0.. null      |
| |             | |010 |boolean       |
| |             | |011 |              |
| |             | |100 |object        |
| |             | |101 |array         |
| |             | |110 |string        |
| |             | |111 |bigint        |

```rust
const fn is_number(u: u64) -> bool {
    u > 0x0007_FFFF_FFFF_FFFF
}
enum Unpacked {
    Undefined,
    Null,
    Boolean(bool),
    Object(Object),
    Array(Array),
    String(String),
    BigInt(BigInt),
}
const fn is_ptr(u: u64) -> bool {
    // 0b1111_1111_1111_1100_...
    // 0b0000_0000_0000_0100_...
    u & 0xFFFC_0000_0000_0000 == 0x0004_0000_0000_0000
}
```

## Pointer Types

We can reduce a pointer size to 45 bits. In this case we can extend pointer types from 4 to 32. Some future possible values:

1. object
2. array
3. string
4. function
5. bigint
6. Uint8Array
7. external function
8. Date
9. Symbol for Iterator, AsyncIterator
10. Promise
11. Map
12. Set
13. RegExp
14. ArrayBuffer
15. Buffer
