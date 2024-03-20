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

use std::collections::{HashMap, HashSet};

/// ConstTracker holds const-related data through following passes of djs serialization:
/// 1. At the first pass we place a visited entity (an object or an array) first into
///    `visited_once`, then (on the second visit) we move it to `to_do`. When the first pass
///    is done, we drop `visited_once`, this set is needed only for populating `to_be_const`.
/// 2. At the second pass we take entities from `to_do` and write them out as consts, also
///    placing each written-out entity into done with its id. This is a depth-first pass, so
///    at the moment when we write out an entity, all its const subordinates are already written out
///    with their ids placed into `done_const`.
struct ConstTracker<D: Dealloc> {
    visited_once: HashSet<Any<D>>,
    to_do: HashSet<Any<D>>,
    done: HashMap<Any<D>, usize>,
}

fn new_const_tracker<D: Dealloc>() -> ConstTracker<D> {
    ConstTracker {
        visited_once: HashSet::new(),
        to_do: HashSet::new(),
        done: HashMap::new(),
    }
}

/// Returns true if the `any` was visited before; updates the `const_tracker` set, tracking whether
/// `any` was visited just once (it's in `const_tracker.visited_once`) or more than once (it's in
/// `const_tracker.to_do_consts` in this case since we are up to writing it out as a const).
fn is_visited<D: Dealloc>(any: Any<D>, const_tracker: &mut ConstTracker<D>) -> bool {
    if const_tracker.to_do.contains(&any) {
        // We've visited `any`more than once before, no action needed here.
        true
    } else {
        if const_tracker.visited_once.contains(&any) {
            // It's the second time we visit `any`, move it from `visited_once` to `to_do`.
            const_tracker.visited_once.remove(&any);
            const_tracker.to_do.insert(any);
            true
        } else {
            // It's the first time we visit `any`, add it to `visited_once` (that is the only
            // branch where we return `false`).
            const_tracker.visited_once.insert(any);
            false
        }
    }
}

/// Given an `any` object, collects all the objects and arrays it references -
/// separately tracking visited once / multiple times objects and arrays.
fn collect_to_do_consts<D: Dealloc>(
    any: Any<D>,
    objects_const_tracker: &mut ConstTracker<D>,
    arrays_const_tracker: &mut ConstTracker<D>,
) {
    let clone = any.clone();
    match to_visitor(any) {
        Visitor::Object(o) => {
            if !is_visited(clone, objects_const_tracker) {
                o.items().iter().for_each(|(_k, v)| {
                    collect_to_do_consts(v.clone(), objects_const_tracker, arrays_const_tracker);
                });
            }
        }
        Visitor::Array(a) => {
            if !is_visited(clone, arrays_const_tracker) {
                a.items().iter().for_each(|i| {
                    collect_to_do_consts(i.clone(), objects_const_tracker, arrays_const_tracker);
                });
            }
        }
        _ => {}
    }
}

fn collect_consts<D: Dealloc>(a: Any<D>) {
    let mut objects_const_tracker = new_const_tracker();
    let mut arrays_const_tracker = new_const_tracker();
    collect_to_do_consts(a, &mut objects_const_tracker, &mut arrays_const_tracker);
}

pub fn to_djs(_a: Any<impl Dealloc>) -> result::Result<String, fmt::Error> {
    let s = String::default();
    Ok(s)
}
