use crate::{
    bit_subset64::BitSubset64,
    container::Clean,
    object::Object,
    ptr_subset::{PtrSubset, PTR_SUBSET_SUPERPOSITION},
    string16::String16, number,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct Value(u64);

const EXTENSION: BitSubset64 = BitSubset64::from_tag(0xFFF8_0000_0000_0000);

const EXTENSION_SPLIT: (BitSubset64, BitSubset64) = EXTENSION.split(0x0004_0000_0000_0000);

const BOOL: BitSubset64 = EXTENSION_SPLIT.0;
const PTR: BitSubset64 = EXTENSION_SPLIT.1;

const PTR_SPLIT: (BitSubset64, BitSubset64) = PTR.split(0x0002_0000_0000_0000);

const STRING: PtrSubset<String16> = PTR_SPLIT.0.ptr_subset();
const STRING_TAG: u64 = STRING.subset().tag;
const OBJECT: PtrSubset<Object> = PTR_SPLIT.1.ptr_subset();
const OBJECT_TAG: u64 = OBJECT.subset().tag;

const FALSE: u64 = BOOL.tag;
const TRUE: u64 = BOOL.tag | 1;

fn update<const ADD: bool>(v: u64) {
    if !PTR.has(v) {
        return;
    }
    let p = v & PTR_SUBSET_SUPERPOSITION;
    if p == 0 {
        return;
    }
    if STRING.subset().has(v) {
        STRING.update::<ADD>(p);
    } else {
        OBJECT.update::<ADD>(p);
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

impl Value {
    fn from_number(n: f64) -> Self {
        let n = n.to_bits();
        assert!(number::is_valid(n));
        Self(n)
    }
    fn from_bool(b: bool) -> Self {
        Self(if b { TRUE } else { FALSE })
    }
    const fn get_number(&self) -> Option<f64> {
        if EXTENSION.has(self.0) {
            return None
        }
        Some(self.0 as f64)
    }
    const fn get_bool(&self) -> Option<bool> {
        if !BOOL.has(self.0) {
            return None
        }
        Some(self.0 != FALSE)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::{const_assert::const_assert, number::NAN};

    const _: () = const_assert(BOOL.has(FALSE));
    const _: () = const_assert(BOOL.has(TRUE));
    const _: () = const_assert(!BOOL.has(0));
    const _: () = const_assert(!BOOL.has(NAN));
    const _: () = const_assert(BOOL.has(EXTENSION.mask));

    #[test]
    fn test_unsized() {
        let _x: Rc<[u8]> = Rc::new([1, 3]);
        // let _y: Rc<(u8, [u8])> = Rc::new((5, [1, 3]));
        // let r = Vec::default();
        // let n = 4 + 4;
        // let _y: Rc<[u8]> = Rc::new([5; n]);
    }
}
