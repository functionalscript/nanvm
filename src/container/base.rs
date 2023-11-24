#[repr(transparent)]
pub struct Base(isize);

impl Default for Base {
    fn default() -> Self {
        Self(1)
    }
}

pub enum Update {
    AddRef = 1,
    Release = -1,
}

impl Base {
    pub unsafe fn update(p: *mut Self, update: Update) -> isize {
        let c = (*p).0 + update as isize;
        (*p).0 = c;
        c
    }
}
