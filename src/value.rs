use std::marker::PhantomData;

use crate::{
    container::{Clean, Container},
    object::Object,
    string16::String16,
    bit_subset64::BitSubset64,
};

#[derive(Debug)]
#[repr(transparent)]
struct Value(u64);

// compatible with `f64`
const INFINITY: u64 = 0x7FF0_0000_0000_0000;
const NAN: u64 = 0x7FF8_0000_0000_0000;
const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;

const EXTENSION: BitSubset64 = BitSubset64::from_tag(0xFFF8_0000_0000_0000);

struct PtrSubset<T: Clean>(BitSubset64, PhantomData<T>);

impl<T: Clean> PtrSubset<T> {
    const fn new(s: BitSubset64) -> Self {
        Self(s, PhantomData)
    }
    fn update<const ADD: bool>(&self, v: u64) {
        if v == NULL {
            return;
        }
        unsafe {
            Container::update::<ADD>((v & self.0.superposition()) as *mut Container<T>);
        }
    }
}

const PTR: PtrSubset<Object> =
    PtrSubset::new(BitSubset64::from_tag(EXTENSION.mask | 0x2_0000_0000_0000));

const NULL: u64 = PTR.0.tag;

const STR: BitSubset64 = BitSubset64::from_tag(EXTENSION.mask | 0x4_0000_0000_0000);

const STR_PTR: PtrSubset<String16> = PtrSubset::new(STR.and(PTR.0));

const FALSE: u64 = EXTENSION.mask;
const TRUE: u64 = EXTENSION.mask | 1;

const BOOL: BitSubset64 = BitSubset64::from_tag_and_union(TRUE & FALSE, TRUE | FALSE);

fn update<const ADD: bool>(v: u64) {
    if PTR.0.has(v) {
        if STR_PTR.0.has(v) {
            STR_PTR.update::<ADD>(v);
        } else {
            PTR.update::<ADD>(v);
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        update::<true>(self.0);
        Self(self.0)
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        update::<false>(self.0);
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::const_assert::const_assert;

    const _: () = const_assert(BOOL.has(FALSE));
    const _: () = const_assert(BOOL.has(TRUE));
    const _: () = const_assert(!BOOL.has(0));
    const _: () = const_assert(!BOOL.has(NAN));
    const _: () = const_assert(BOOL.has(EXTENSION.mask));
    const _: () = const_assert(!BOOL.has(EXTENSION.mask | 2));

    #[test]
    fn test_nan() {
        assert_eq!(f64::INFINITY.to_bits(), INFINITY);
        assert_ne!(f64::NAN, f64::NAN);
        assert_eq!(f64::NAN.to_bits(), NAN);
        assert_eq!(f64::NEG_INFINITY.to_bits(), NEG_INFINITY);
    }

    #[test]
    fn test_unsized() {
        let _x: Rc<[u8]> = Rc::new([1, 3]);
        // let _y: Rc<(u8, [u8])> = Rc::new((5, [1, 3]));
        // let r = Vec::default();
        // let n = 4 + 4;
        // let _y: Rc<[u8]> = Rc::new([5; n]);
    }
}
