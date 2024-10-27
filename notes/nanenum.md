```rust
/**
 * Sign bit: 1 bit
 * Exponent: 11 bits
 * Significand precision: 53 bits (52 explicitly stored)
 *
 * NaN: exponent all ones 111_1111_1111
 *
 * 53 bits.
 *
 * From this 53 bits there are 3 reserved values:
 * - +Inf: 0_111_1111_1111_0...,
 * - -Inf: 1_111_1111_1111_0...,
 * - NaN.
 *
 * Number of states: 2^53-3.
 */
struct NanEnum(u64)

impl NanEnum {

}
```
