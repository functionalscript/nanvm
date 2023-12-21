pub trait Cast<T> {
    fn cast(self) -> T;
}

impl Cast<u64> for u64 {
    fn cast(self) -> u64 {
        self
    }
}

// pointer

impl<T> Cast<*const T> for u64 {
    fn cast(self) -> *const T {
        self as *const T
    }
}

impl<T> Cast<u64> for *const T {
    fn cast(self) -> u64 {
        self as u64
    }
}

// bool

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
