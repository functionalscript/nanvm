use std::marker::PhantomData;

use crate::{
    bit_subset64::BitSubset64,
    container::{Clean, Container},
    object::Object,
    string16::String16,
};

#[derive(Debug)]
#[repr(transparent)]
struct Value(u64);

// compatible with `f64`
const INFINITY: u64 = 0x7FF0_0000_0000_0000;
const NAN: u64 = 0x7FF8_0000_0000_0000;
const NEG_INFINITY: u64 = 0xFFF0_0000_0000_0000;

//

const EXTENSION_MASK: u64 = 0xFFF8_0000_0000_0000;

const fn extension(tag: u64, sup: u64) -> BitSubset64 {
    BitSubset64::from_tag_and_superposition(EXTENSION_MASK | tag, sup)
}

struct PtrSubset<T: Clean>(BitSubset64, PhantomData<T>);

impl<T: Clean> PtrSubset<T> {
    const fn new(tag: u64) -> Self {
        Self(
            extension(
                tag | 0x1_0000_0000_0000,
                0x0000_FFFF_FFFF_FFFF, // 48-bits. We can reduce it to 45-bits by removing the first alignment bits.
            ),
            PhantomData,
        )
    }
    fn update<const ADD: bool>(&self, v: u64) {
        let v = v & self.0.superposition();
        if v == 0 {
            return;
        }
        unsafe {
            Container::update::<ADD>(v as *mut Container<T>);
        }
    }
}

// an object pointer

const OBJ_PTR: PtrSubset<Object> = PtrSubset::new(0);

// a string index

const STR_INDEX: BitSubset64 = extension(0x2_0000_0000_0000, 0xFFFF_FFFF);

// a pointer to a string

const STR_PTR: PtrSubset<String16> = PtrSubset::new(STR_INDEX.tag);

// all strings

const STR: BitSubset64 = STR_INDEX.or_unchecked(STR_PTR.0);

// all pointers

const PTR: BitSubset64 = OBJ_PTR.0.or(STR_PTR.0);

// bool

const BOOL: BitSubset64 = extension(0, 1);

const FALSE: u64 = BOOL.tag;
const TRUE: u64 = BOOL.union();

// all extensions

const EXTENSION: BitSubset64 = PTR.or_unchecked(STR_INDEX).or_unchecked(BOOL);

fn update<const ADD: bool>(v: u64) {
    if PTR.has(v) {
        if STR_PTR.0.has(v) {
            STR_PTR.update::<ADD>(v);
        } else {
            OBJ_PTR.update::<ADD>(v);
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
    const _: () = const_assert(BOOL.has(EXTENSION_MASK));
    const _: () = const_assert(!BOOL.has(EXTENSION_MASK | 2));

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
