use std::ops::Range;

pub struct Entry<T, Num> where Num: PartialOrd {
    pub value: T,
    pub to: Num,
}

pub struct RangeMap<T, Num> where Num: PartialOrd {
    pub list: Vec<Entry<T, Num>>
}

pub trait Reduce<T> {
    fn reduce(self, other: T) -> Option<T>;
}

pub fn merge<Num>(a: Range<Num>, b: Range<Num>) {
    todo!()
}

pub fn range_merge<T, Num>(a: RangeMap<T, Num>, b: RangeMap<T, Num>) where Num: PartialOrd {
    todo!()
}

pub fn reduce_op<T, Num>(a: Entry<T, Num>, b: Entry<T, Num>) where Num: PartialOrd {
    todo!()
}