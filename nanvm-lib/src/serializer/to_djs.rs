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

/// ConstTracker holds references to js compounds (objects or arrays) in two sets:
/// `visited_once` refers to compounds that we've seen just once so far;
/// `visited_repeatedly` refers to compounds that we've seen more than once.
/// When djs tracking pass is done, `visited_repeatedly` refers to compounds that will be written
/// out via const definitions.
/// Note that we use one ConstTracker for js objects and another for js arrays, keeping them
/// separate - to reduce set sizes.
struct ConstTracker<D: Dealloc> {
    visited_once: HashSet<Any<D>>,
    visited_repeatedly: HashSet<Any<D>>,
}

fn new_const_tracker<D: Dealloc>() -> ConstTracker<D> {
    ConstTracker {
        visited_once: HashSet::new(),
        visited_repeatedly: HashSet::new(),
    }
}

/// ConstBuilder holds info on compounds that we write out as consts - initially having all
/// referneces in `to_do`, moving them to `done` as we write them out one by one, with each
/// compound written out after compounds it refers to (if any). Thus a const definition written
/// out earlier can be used below by its name, and never above.
pub struct ConstBuilder<D: Dealloc> {
    to_do: HashSet<Any<D>>,
    done: HashMap<Any<D>, usize>,
}

fn new_const_builder<D: Dealloc>(visited_repeatedly: HashSet<Any<D>>) -> ConstBuilder<D> {
    ConstBuilder {
        to_do: visited_repeatedly,
        done: HashMap::new(),
    }
}

/// Returns true if the `any` was visited before; updates the `const_tracker` set, tracking whether
/// `any` was visited just once (it's in `const_tracker.visited_once`) or more than once (it's in
/// `const_tracker.visited_repeatedly` in this case since we are up to writing it out as a const).
fn is_visited<D: Dealloc>(any: Any<D>, const_tracker: &mut ConstTracker<D>) -> bool {
    if const_tracker.visited_repeatedly.contains(&any) {
        // We've visited `any` more than once before, no action is needed here.
        true
    } else if const_tracker.visited_once.contains(&any) {
        // It's the second time we visit `any`, move it from `visited_once` to `to_do`.
        const_tracker.visited_once.remove(&any);
        const_tracker.visited_repeatedly.insert(any);
        true
    } else {
        // It's the first time we visit `any`, add it to `visited_once` (that is the only
        // branch where we return `false`).
        const_tracker.visited_once.insert(any);
        false
    }
}

/// Traverse a DAG referred by `compound` (that is an object or an array), tracking objects and
/// arrays, including `compound` itself.
fn track_consts_for_compound<D: Dealloc>(
    compound: Any<D>,
    is_object: bool, // otherwise `any` is an array
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    if !is_visited(
        compound.clone(),
        if is_object {
            object_const_tracker
        } else {
            array_const_tracker
        },
    ) {
        if is_object {
            compound
                .clone()
                .try_move::<JsObjectRef<D>>()
                .unwrap()
                .items()
                .iter()
                .for_each(|(_k, v)| {
                    track_consts_for_any(v.clone(), object_const_tracker, array_const_tracker);
                });
            object_const_tracker.visited_once.insert(compound);
        } else {
            compound
                .clone()
                .try_move::<JsArrayRef<D>>()
                .unwrap()
                .items()
                .iter()
                .for_each(|i| {
                    track_consts_for_any(i.clone(), object_const_tracker, array_const_tracker);
                });
            array_const_tracker.visited_once.insert(compound);
        }
    }
}

/// Traverse a DAG referred by `any` (of any js type), tracking objects and arrays, including `any`
/// itself.
fn track_consts_for_any<D: Dealloc>(
    any: Any<D>,
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    match any.get_type() {
        Type::Object => {
            track_consts_for_compound(any, true, object_const_tracker, array_const_tracker)
        }
        Type::Array => {
            track_consts_for_compound(any, false, object_const_tracker, array_const_tracker)
        }
        _ => {}
    }
}

/// Traverse a DAG referred by `any` - returning two sets of to-be consts (objects, arrays).
fn track_consts<D: Dealloc>(any: Any<D>) -> (HashSet<Any<D>>, HashSet<Any<D>>) {
    let mut object_const_tracker = new_const_tracker();
    let mut array_const_tracker = new_const_tracker();
    track_consts_for_any(any, &mut object_const_tracker, &mut array_const_tracker);
    (
        object_const_tracker.visited_repeatedly,
        array_const_tracker.visited_repeatedly,
    )
}

fn take_from_set<D: Dealloc>(set: &mut HashSet<Any<D>>) -> Option<Any<D>> {
    if set.is_empty() {
        None
    } else {
        let any = set.iter().next()?.clone();
        set.remove(&any);
        Some(any)
    }
}

pub trait WriteDjs: WriteJson {
    /// Writes out consts objects or arrays (according to `iterate_objects` flag) in the right
    /// order (with no forward references).
    /// TODO make this function private, this will help to make ConstBuilder private as well.
    fn write_compounds<D: Dealloc>(
        &mut self,
        iterate_objects: bool,
        objects_const_builder: &mut ConstBuilder<D>,
        arrays_const_builder: &mut ConstBuilder<D>,
    ) -> fmt::Result {
        // do the following match in a loop may be factoing out wirte_compund (single)
        match take_from_set(
            &mut (if iterate_objects {
                objects_const_builder
            } else {
                arrays_const_builder
            })
            .to_do,
        ) {
            None => fmt::Result::Ok(()),
            Some(_any) => fmt::Result::Ok(()),
        }
    }

    /// Writes out const objects, arrays in the right order (with no forward references).
    fn write_out_consts<D: Dealloc>(
        &mut self,
        objects_visited_repeatedly: HashSet<Any<D>>,
        arrays_visited_repeatedly: HashSet<Any<D>>,
    ) -> fmt::Result {
        let mut _object_const_builder = new_const_builder(objects_visited_repeatedly);
        let mut _array_const_builder = new_const_builder(arrays_visited_repeatedly);
        fmt::Result::Ok(())
    }

    ///
    fn write_djs(&mut self, any: Any<impl Dealloc>) -> fmt::Result {
        let (objects_visited_repeatedly, arrays_visited_repeatedly) = track_consts(any.clone());
        self.write_out_consts(objects_visited_repeatedly, arrays_visited_repeatedly)?;
        self.write_json(any)
    }
}

impl<T: WriteJson> WriteDjs for T {}

pub fn to_djs(any: Any<impl Dealloc>) -> result::Result<String, fmt::Error> {
    let mut s = String::default();
    s.write_djs(any)?;
    Ok(s)
}
