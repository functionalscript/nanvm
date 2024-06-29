use crate::common::default::default;

pub struct Entry<Num, T>
where
    Num: PartialOrd,
{
    pub key: Num,
    pub value: T,
}

pub struct RangeMap<Num, T>
where
    Num: PartialOrd,
{
    pub list: Vec<Entry<Num, T>>,
}

pub trait Union {
    fn union(self, other: Self) -> Self;
}

impl<Num, T> RangeMap<Num, T>
where
    Num: PartialOrd,
{
    pub fn get(&self, key: Num) -> Option<&T> {
        let len = self.list.len() as i32;
        let mut b = 0;
        let mut e = len - 1;
        loop {
            if b >= len {
                return None;
            }
            if e < b {
                return Some(&self.list.get(b as usize).unwrap().value);
            }
            let mid = b + ((e - b) >> 1);
            if key <= self.list.get(mid as usize).unwrap().key {
                e = mid - 1;
            } else {
                b = mid + 1;
            }
        }
    }
}

pub fn merge<Num, T>(a: RangeMap<Num, T>, b: RangeMap<Num, T>) -> RangeMap<Num, T>
where
    T: Union,
    T: Clone,
    Num: PartialOrd,
{
    let list = merge_iter(a.list.into_iter(), b.list.into_iter());
    RangeMap { list }
}

pub fn merge_iter<Num, T>(
    mut a: impl Iterator<Item = Entry<Num, T>>,
    mut b: impl Iterator<Item = Entry<Num, T>>,
) -> Vec<Entry<Num, T>>
where
    T: Union,
    T: Clone,
    Num: PartialOrd,
{
    let mut res: Vec<Entry<Num, T>> = default();
    let mut next_a = a.next();
    let mut next_b = b.next();
    loop {
        match next_a {
            Some(value_a) => match next_b {
                Some(value_b) => {
                    let value = value_a.value.clone().union(value_b.value.clone());
                    if value_a.key > value_b.key {
                        res.push(Entry {
                            value,
                            key: value_b.key,
                        });
                        next_a = Some(value_a);
                        next_b = b.next();
                    } else if value_a.key < value_b.key {
                        res.push(Entry {
                            value,
                            key: value_a.key,
                        });
                        next_a = a.next();
                        next_b = Some(value_b);
                    } else {
                        res.push(Entry {
                            value,
                            key: value_a.key,
                        });
                        next_a = a.next();
                        next_b = b.next();
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

    use super::{Entry, RangeMap};

    #[test]
    #[wasm_bindgen_test]
    fn test_get() {
        let list = vec![Entry { key: 10, value: 'a'}, Entry { key: 20, value: 'b'}, Entry { key: 30, value: 'c'}];
        let rm = RangeMap { list };
        let result = rm.get(5);
        assert_eq!(result.unwrap(), &'a');
    }
}
