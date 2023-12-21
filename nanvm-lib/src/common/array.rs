use super::cast::Cast;

impl<T, const N: usize> Cast<Vec<T>> for [T; N] {
    /// Move the array into a vector.
    /// Compare to `.to_vec()`, the function doesn't require `Clone` trait.
    fn cast(self) -> Vec<T> {
        let mut result = Vec::with_capacity(N);
        for i in self {
            result.push(i);
        }
        result
    }
}
