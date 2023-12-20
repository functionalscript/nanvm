pub trait Cast<T> {
    fn cast(self) -> T;
}

impl Cast<u64> for u64 {
    fn cast(self) -> u64 {
        self
    }
}
