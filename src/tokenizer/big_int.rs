use std::ops::Add;

struct BigInt {
    sign: i8,
    value: Vec<u64>
}

impl Add for BigInt {
    type Output = BigInt;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}