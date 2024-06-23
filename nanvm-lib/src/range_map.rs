use std::ops::Range;

use crate::common::default::default;

pub struct Entry<T, Num>
where
    Num: PartialOrd,
{
    pub value: T,
    pub to: Num,
}

pub struct RangeMap<T, Num>
where
    Num: PartialOrd,
{
    pub list: Vec<Entry<T, Num>>,
}

pub trait Union<T = Self> {
    fn union(&self, other: &T) -> T;
}

// impl RangeMap<T, Num> where Num: PartialOrd {

// }

pub fn merge<Num>(_a: Range<Num>, _b: Range<Num>) {
    todo!()
}

pub fn range_merge<T, Num>(a: RangeMap<T, Num>, b: RangeMap<T, Num>) -> RangeMap<T, Num>
where
    T: Union,
    Num: PartialOrd,
{
    let list = merge_iter(a.list.into_iter(), b.list.into_iter());
    RangeMap { list }
}

pub fn merge_iter<T, Num>(
    mut a: impl Iterator<Item = Entry<T, Num>>,
    mut b: impl Iterator<Item = Entry<T, Num>>,
) -> Vec<Entry<T, Num>>
where
    T: Union,
    Num: PartialOrd,
{
    let mut res: Vec<Entry<T, Num>> = default();
    let mut next_a = a.next();
    let mut next_b = b.next();
    loop {
        match next_a {
            Some(value_a) => match next_b {
                Some(value_b) => {
                    let value = value_a.value.union(&value_b.value);
                    if value_a.to > value_b.to {
                        res.push(Entry {
                            value,
                            to: value_b.to,
                        });
                        next_a = Some(value_a);
                        next_b = b.next();
                    } else {
                        res.push(Entry {
                            value,
                            to: value_a.to,
                        });
                        next_a = a.next();
                        next_b = Some(value_b);
                    }
                }
                None => {
                    res.push(value_a);
                    next_a = a.next();
                }
            },
            None => match next_b {
                Some(value_b) => {
                    res.push(value_b);
                    next_b = b.next();
                }
                None => {
                    break;
                }
            },
        }
    }
    res
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    #[test]
    #[wasm_bindgen_test]
    fn test() {}
}
