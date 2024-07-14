use std::ops::Range;

use crate::{common::default::default, static_ref_default::StaticRefDefault};

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

#[derive(Clone, PartialEq, Debug)]
pub struct State<T> {
    pub value: Option<T>,
}

impl<T: 'static> StaticRefDefault for State<T> {
    const STATIC_REF_DEFAULT: &'static Self = &Self { value: None };
}

impl<T> Union for State<T>
where
    T: Eq,
{
    fn union(self, other: Self) -> Self {
        match self.value {
            Some(a) => match other.value {
                Some(b) => {
                    if a.eq(&b) {
                        State { value: Some(b) }
                    } else {
                        panic!("state values should be the same")
                    }
                }
                None => State { value: Some(a) },
            },
            None => other,
        }
    }
}

impl<Num, T> RangeMap<Num, T>
where
    Num: PartialOrd,
    T: StaticRefDefault,
{
    pub fn get(&self, key: Num) -> &T {
        let len = self.list.len() as i32;
        let mut b = 0;
        let mut e = len - 1;
        loop {
            if b >= len {
                return T::STATIC_REF_DEFAULT;
            }
            if e < b {
                return &self.list.get(b as usize).unwrap().value;
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

pub fn merge_list<Num, T>(list: Vec<RangeMap<Num, T>>) -> RangeMap<Num, T>
where
    T: Union,
    T: Clone,
    Num: PartialOrd,
{
    let mut result = RangeMap { list: vec![] };
    for x in list {
        result = merge(x, result);
    }
    result
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

pub fn from_range<T>(range: Range<char>, value: T) -> RangeMap<char, State<T>> {
    RangeMap {
        list: vec![
            Entry {
                key: char::from_u32(range.start as u32 - 1).unwrap_or(range.start),
                value: State { value: None },
            },
            Entry {
                key: range.end,
                value: State { value: Some(value) },
            },
        ],
    }
}

pub fn from_one<T>(c: char, value: T) -> RangeMap<char, State<T>> {
    RangeMap {
        list: match char::from_u32(c as u32 - 1) {
            Some(p) => vec![
                Entry {
                    key: p,
                    value: State { value: None },
                },
                Entry {
                    key: c,
                    value: State { value: Some(value) },
                },
            ],
            None => {
                vec![Entry {
                    key: c,
                    value: State { value: Some(value) },
                }]
            }
        },
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{from_range, merge, merge_list, Entry, RangeMap, State};

    #[test]
    #[wasm_bindgen_test]
    fn test_get() {
        let list = vec![
            Entry {
                key: 10,
                value: 'a',
            },
            Entry {
                key: 20,
                value: 'b',
            },
            Entry {
                key: 30,
                value: 'c',
            },
        ];
        let rm = RangeMap { list };
        let result = rm.get(5);
        assert_eq!(result, &'a');
        let result = rm.get(10);
        assert_eq!(result, &'a');
        let result = rm.get(15);
        assert_eq!(result, &'b');
        let result = rm.get(20);
        assert_eq!(result, &'b');
        let result = rm.get(25);
        assert_eq!(result, &'c');
        let result = rm.get(30);
        assert_eq!(result, &'c');
        let result = rm.get(35);
        assert_eq!(*result, 0 as char);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_get_from_empty() {
        let list = vec![];
        let rm: RangeMap<i32, char> = RangeMap { list };
        let result = rm.get(10);
        assert_eq!(*result, 0 as char);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_merge() {
        let a = RangeMap {
            list: vec![Entry {
                key: 10,
                value: State { value: Some('a') },
            }],
        };
        let b = RangeMap { list: vec![] };
        let result = merge(a, b);
        assert_eq!(result.list.len(), 1);
        assert_eq!(result.list[0].key, 10);
        assert_eq!(result.list[0].value, State { value: Some('a') });

        let a = RangeMap { list: vec![] };
        let b = RangeMap {
            list: vec![Entry {
                key: 10,
                value: State { value: Some('a') },
            }],
        };
        let result = merge(a, b);
        assert_eq!(result.list.len(), 1);
        assert_eq!(result.list[0].key, 10);
        assert_eq!(result.list[0].value, State { value: Some('a') });

        let a = RangeMap {
            list: vec![
                Entry {
                    key: 10,
                    value: State { value: Some('a') },
                },
                Entry {
                    key: 20,
                    value: State { value: Some('b') },
                },
                Entry {
                    key: 30,
                    value: State { value: Some('c') },
                },
                Entry {
                    key: 40,
                    value: State { value: None },
                },
            ],
        };
        let b = RangeMap {
            list: vec![
                Entry {
                    key: 10,
                    value: State { value: Some('a') },
                },
                Entry {
                    key: 20,
                    value: State { value: None },
                },
                Entry {
                    key: 30,
                    value: State { value: Some('c') },
                },
                Entry {
                    key: 40,
                    value: State { value: None },
                },
                Entry {
                    key: 50,
                    value: State { value: Some('d') },
                },
            ],
        };
        let result = merge(a, b);
        assert_eq!(result.list.len(), 5);
        assert_eq!(result.list[0].key, 10);
        assert_eq!(result.list[0].value, State { value: Some('a') });
        assert_eq!(result.list[1].key, 20);
        assert_eq!(result.list[1].value, State { value: Some('b') });
        assert_eq!(result.list[2].key, 30);
        assert_eq!(result.list[2].value, State { value: Some('c') });
        assert_eq!(result.list[3].key, 40);
        assert_eq!(result.list[3].value, State { value: None });
        assert_eq!(result.list[4].key, 50);
        assert_eq!(result.list[4].value, State { value: Some('d') });
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_merge_list() {
        let result: RangeMap<i32, State<char>> = merge_list(vec![]);
        assert_eq!(result.list.len(), 0);

        let result: RangeMap<i32, State<char>> = merge_list(vec![
            RangeMap {
                list: vec![Entry {
                    key: 10,
                    value: State { value: Some('a') },
                }],
            },
            RangeMap {
                list: vec![
                    Entry {
                        key: 10,
                        value: State { value: None },
                    },
                    Entry {
                        key: 20,
                        value: State { value: Some('b') },
                    },
                ],
            },
            RangeMap {
                list: vec![
                    Entry {
                        key: 20,
                        value: State { value: None },
                    },
                    Entry {
                        key: 30,
                        value: State { value: Some('c') },
                    },
                ],
            },
        ]);
        assert_eq!(result.list.len(), 3);
        assert_eq!(result.list[0].key, 10);
        assert_eq!(result.list[0].value, State { value: Some('a') });
        assert_eq!(result.list[1].key, 20);
        assert_eq!(result.list[1].value, State { value: Some('b') });
        assert_eq!(result.list[2].key, 30);
        assert_eq!(result.list[2].value, State { value: Some('c') });
    }

    #[test]
    #[wasm_bindgen_test]
    #[should_panic(expected = "state values should be the same")]
    fn test_merge_panic() {
        let a = RangeMap {
            list: vec![Entry {
                key: 10,
                value: State { value: Some('a') },
            }],
        };
        let b = RangeMap {
            list: vec![Entry {
                key: 20,
                value: State { value: Some('b') },
            }],
        };
        let _result = merge(a, b);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_from_range() {
        let state = 'A';
        let range = 'b'..'d';
        let rm = from_range(range, state);
        assert_eq!(rm.get('a'), &State { value: None });
        assert_eq!(rm.get('b'), &State { value: Some('A') });
        assert_eq!(rm.get('c'), &State { value: Some('A') });
        assert_eq!(rm.get('d'), &State { value: Some('A') });
        assert_eq!(rm.get('e'), &State { value: None });
    }
}
