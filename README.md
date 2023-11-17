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

## Not a float (NaF)

|prefix            |           |
|------------------|-----------|
|0111_1111_1111_0  |Infinity   |
|0111_1111_1111_1  |NaN        |
|1111_1111_1111_0  |-Infinity  |
|1111_1111_1111_1  |NaF        |

### Types of NaFs

```rust
const fn some<const M: u64, const E: u64>(n: u64) -> bool {
    n & M == E
}

const fn all<const M: u64>(n: u64) -> bool {
    some::<M, M>(n)
}

const NAF: u64 = 0xFFF8_0000_0000_0000;

const fn is_naf(v: u64) -> bool { all::<NAF>(v) }
const fn is_number(v: u64) -> bool { !is_naf(v) }

const NAF_PTR: u64 = NAF | 0x4_0000_0000_0000;

const NAF_NULL: u64 = NAF_PTR;

const fn is_ptr(v: u64) -> bool { all::<NAF_PTR>(v) }

const NAF_STR: u64 = NAF | 0x2_0000_0000_0000;

const fn is_str(v: u64) -> bool { all::<NAF_STR>(v) }

const NAF_STR_PTR: u64 = NAF_PTR | NAF_STR;

const fn is_str_ptr(v: u64) -> bool { all::<NAF_STR_PTR>(v) }

const fn is_index_str(v: u64) -> bool { some::<NAF_STR_PTR, NAF_STR>(v) }

const fn is_bool(v: u64) -> bool { some::<NAF_STR_PTR, NAF>(v) }
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
