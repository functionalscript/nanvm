use crate::common::bit_subset64::{BitSubset64, Cast};

use super::{bitset::BOOL, extension::Extension};

impl Cast<u64> for bool {
    #[inline(always)]
    fn cast(self) -> u64 {
        self as u64
    }
}

impl Cast<bool> for u64 {
    #[inline(always)]
    fn cast(self) -> bool {
        self != 0
    }
}

impl Extension for bool {
    const SUBSET: BitSubset64<bool> = BOOL;
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[wasm_bindgen_test]
    fn test_bool_to_bitset() {
        let x: u64 = true.into();
        assert_eq!(x, 1u64);
        let x: u64 = false.into();
        assert_eq!(x, 0);
    }
}
