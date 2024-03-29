use crate::{
    js::{any::Any, js_array::JsArrayRef, js_object::JsObjectRef, type_::Type},
    mem::manager::Dealloc,
};

use core::{
    fmt::{self},
    result,
};

use super::to_json::WriteJson;

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
        // We've visited `any` more than once before, no action is needed here.
        true
    } else if const_tracker.visited_once.contains(&any) {
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

/// Traverse a DAG referred by `any` (that is an object or an array), tracking objects and arrays
/// via devoted ConstTracker-s. Each object / array referred more than once gets tracked in `to_do`
/// set - to be written as a const later on.
fn collect_to_do_consts_for_compound<D: Dealloc>(
    any: Any<D>,
    is_object: bool, // otherwise `any` is an array
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    if !is_visited(
        any.clone(),
        if is_object {
            object_const_tracker
        } else {
            array_const_tracker
        },
    ) {
        if is_object {
            any.clone()
                .try_move::<JsObjectRef<D>>()
                .unwrap()
                .items()
                .iter()
                .for_each(|(_k, v)| {
                    collect_to_do_consts(v.clone(), object_const_tracker, array_const_tracker);
                });
            object_const_tracker.visited_once.insert(any);
        } else {
            any.clone()
                .try_move::<JsArrayRef<D>>()
                .unwrap()
                .items()
                .iter()
                .for_each(|i| {
                    collect_to_do_consts(i.clone(), object_const_tracker, array_const_tracker);
                });
            array_const_tracker.visited_once.insert(any);
        }
    }
}

/// Traverse a DAG referred by `any`, tracking objects and arrays via devoted ConstTracker-s.
/// Each object / array referred more than once gets tracked in `to_do` set - to be written as
/// a const later on.
fn collect_to_do_consts<D: Dealloc>(
    any: Any<D>,
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    match any.get_type() {
        Type::Object => {
            collect_to_do_consts_for_compound(any, true, object_const_tracker, array_const_tracker)
        }
        Type::Array => {
            collect_to_do_consts_for_compound(any, false, object_const_tracker, array_const_tracker)
        }
        _ => {}
    }
}

fn collect_consts<D: Dealloc>(any: Any<D>) {
    let mut object_const_tracker = new_const_tracker();
    let mut array_const_tracker = new_const_tracker();
    collect_to_do_consts(any, &mut object_const_tracker, &mut array_const_tracker);
}

/// Given an `any` object, writes it out as a const - ensuring that all consts it depends on are
/// written out in front of it.
fn write_out_consts<D: Dealloc>(
    any: Any<D>,
    object_const_tracker: &mut ConstTracker<D>,
    _array_const_tracker: &mut ConstTracker<D>,
) {
    match any.get_type() {
        Type::Object => {
            let opt_k_v = object_const_tracker.done.get_key_value(&any);
            if opt_k_v.is_some() {}
        }
        _ => {}
    }
}

pub trait WriteDjs: WriteJson {
    fn write_djs(&mut self, any: Any<impl Dealloc>) -> fmt::Result {
        self.write_json(any)
    }
}

impl<T: WriteJson> WriteDjs for T {}

pub fn to_djs(any: Any<impl Dealloc>) -> result::Result<String, fmt::Error> {
    let mut s = String::default();
    s.write_djs(any)?;
    Ok(s)
}
