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
    pub unsafe fn update(&mut self, update: Update) -> isize {
        let c = self.0 + update as isize;
        self.0 = c;
        c
    }
}
