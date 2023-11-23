# NaNVM

## Float64

|Sign, 1|Exponent, 11|Fraction, 52    |Value   |
|-------|------------|----------------|--------|
|0      |000         |0_0000_0000_0000|number  |
|       |...         |                |number  |
|       |7FF         |0_0000_0000_0000|+Inf    |
|       |            |...             |reserved|
|       |            |8_0000_0000_0000|NaN     |
|       |            |...             |reserved|
|1      |000         |0_0000_0000_0000|number  |
|       |...         |                |number  |
|       |7FF         |0_0000_0000_0000|-Inf    |
|       |            |...             |reserved|

## Value

`exponent == 0x7FF` is used for special values (53 bits):

|Special value, 53 bits|Value   |# of values  |
|----------------------|--------|-------------|
|00_0000_0000_0000     |+Inf    |     1       |
|...                   |reserved|2^51-1       |
|08_0000_0000_0000     |Nan     |     1       |
|...                   |reserver|2^51-1       |
|10_0000_0000_0000     |-Inf    |     1       |
|...                   |reserved|2^52-1       |

## Pointers

`45` bits = `48` bits - `3` bits of alignment.

We need, at least, two types of pointers:
- `&string`,
- `&object`.

`null` is a separate value, so our pointers have `2^45 - 1` values.

## Array Index

`2^32 - 1` values.

## Bool

`2` values.

## Number Extension

|prefix            |           |
|------------------|-----------|
|0111_1111_1111_0  |Infinity   |
|0111_1111_1111_1  |NaN        |
|1111_1111_1111_0  |-Infinity  |
|1111_1111_1111_1  |Extension  |

### Extension Types

```rust
const EXTENSION_MASK: u64 = 0xFFF8_0000_0000_0000;

const PTR_MASK: u64 = EXTENSION | 0x4_0000_0000_0000;
const NULL: u64 = PTR_MASK;

const STR_MASK: u64 = EXTENSION | 0x2_0000_0000_0000;

const STR_PTR_MASK: u64 = PTR_MASK | STR_MASK;

const FALSE: u64 = EXTENSION;
const TRUE: u64 = FALSE | 1;
```

## String

```rust
type String16 = Rc<[u16]>;
```

## Object

```rust
type Object = Rc<[Value]>
```

## Array

```rust
type Array = Rc<[Value]>;
```
