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

## All Values when `exponent` is `0x7FF`

