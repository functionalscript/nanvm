use crate::{
    js::{
        any::Any,
        visitor::{to_visitor, Visitor},
    },
    mem::manager::Dealloc,
};

use core::{
    fmt::{self},
    result,
};

use std::collections::HashSet;

struct Visited<D: Dealloc> {
    once: HashSet<Any<D>>,
    multiple: HashSet<Any<D>>,
}

/// Returns true if the `any` was visited before; updates the `visited` set listing the `any` as visited.
fn is_visited<D: Dealloc>(any: Any<D>, visited: &mut Visited<D>) -> bool {
    if visited.multiple.contains(&any) {
        true
    } else {
        if visited.once.contains(&any) {
            visited.once.remove(&any);
            visited.multiple.insert(any);
            true
        } else {
            visited.once.insert(any);
            false
        }
    }
}

/// Given an `any` object, collects all the objects and arrays it references -
/// separately tracking visited once / multiple times objects and arrays.
fn collect_visited<D: Dealloc>(
    any: Any<D>,
    visited_objects: &mut Visited<D>,
    visited_arrays: &mut Visited<D>,
) {
    let clone = any.clone();
    match to_visitor(any) {
        Visitor::Object(o) => {
            if !is_visited(clone, visited_objects) {
                o.items().iter().for_each(|(_k, v)| {
                    collect_visited(v.clone(), visited_objects, visited_arrays);
                });
            }
        }
        Visitor::Array(a) => {
            if !is_visited(clone, visited_arrays) {
                a.items().iter().for_each(|i| {
                    collect_visited(i.clone(), visited_objects, visited_arrays);
                });
            }
        }
        _ => {}
    }
}

fn collect_consts<D: Dealloc>(a: Any<D>) {
    let mut visited_objects = Visited::<D> {
        once: HashSet::new(),
        multiple: HashSet::new(),
    };
    let mut visited_arrays = Visited::<D> {
        once: HashSet::new(),
        multiple: HashSet::new(),
    };
    collect_visited(a, &mut visited_objects, &mut visited_arrays);
}

pub fn to_djs(_a: Any<impl Dealloc>) -> result::Result<String, fmt::Error> {
    let s = String::default();
    Ok(s)
}
