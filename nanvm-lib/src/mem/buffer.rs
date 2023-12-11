use core::ops::Range;

pub trait Buffer {
    fn items_mut(&mut self) -> &mut [u8];
    unsafe fn range(&mut self) -> Range<*mut u8> {
        self.items_mut().as_mut_ptr_range()
    }
}

impl Buffer for &mut [u8] {
    #[inline(always)]
    fn items_mut(&mut self) -> &mut [u8] {
        self
    }
}
