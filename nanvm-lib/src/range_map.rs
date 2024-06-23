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

    #[test]
    #[wasm_bindgen_test]
    fn test() {}
}
