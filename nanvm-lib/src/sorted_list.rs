use crate::{common::default::default, range_map::Union};

pub struct SortedList<T>
where
    T: PartialOrd,
{
    pub list: Vec<T>,
}

// impl<T> Union for SortedList<T> where T: PartialOrd {
//     fn union(&self, other: &Self) -> Self {
//         merge(self, other)
//     }
// }

pub fn merge<T>(a: SortedList<T>, b: SortedList<T>) -> SortedList<T>
where
    T: PartialOrd,
{
    let list = merge_iter(a.list.into_iter(), b.list.into_iter());
    SortedList { list }
}

pub fn merge_iter<T>(mut a: impl Iterator<Item = T>, mut b: impl Iterator<Item = T>) -> Vec<T>
where
    T: PartialOrd,
{
    let mut res: Vec<T> = default();
    let mut next_a = a.next();
    let mut next_b = b.next();
    loop {
        match next_a {
            Some(value_a) => match next_b {
                Some(value_b) => {
                    if value_a > value_b {
                        res.push(value_b);
                        next_a = Some(value_a);
                        next_b = b.next();
                    } else if value_a < value_b {
                        res.push(value_a);
                        next_a = a.next();
                        next_b = Some(value_b);
                    } else {
                        res.push(value_a);
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

    use crate::sorted_list::SortedList;

    use super::merge;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let a = SortedList {
            list: [2, 3, 4].to_vec(),
        };
        let b = SortedList {
            list: [1, 3, 5].to_vec(),
        };
        let result = merge(a, b);
        assert_eq!(result.list, [1, 2, 3, 4, 5].to_vec());

        let a = SortedList {
            list: [1, 2, 3].to_vec(),
        };
        let b = SortedList { list: [].to_vec() };
        let result = merge(a, b);
        assert_eq!(result.list, [1, 2, 3].to_vec());

        let a = SortedList { list: [].to_vec() };
        let b = SortedList {
            list: [1, 2, 3].to_vec(),
        };
        let result = merge(a, b);
        assert_eq!(result.list, [1, 2, 3].to_vec());
    }
}
