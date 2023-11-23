#[repr(transparent)]
pub struct Base(isize);

impl Default for Base {
    fn default() -> Self {
        Self(1)
    }
}

pub const ADD_REF: isize = 1;
pub const RELEASE: isize = -1;

impl Base {
    pub unsafe fn update<const I: isize>(p: *mut Self) -> isize {
        let c = (*p).0 + I;
        (*p).0 = c;
        c
    }
}
