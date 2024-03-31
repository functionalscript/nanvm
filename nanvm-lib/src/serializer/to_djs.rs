use crate::{
    js::{any::Any, js_array::JsArrayRef, js_object::JsObjectRef, type_::Type},
    mem::{
        flexible_array::{header::FlexibleArrayHeader, FlexibleArray},
        manager::Dealloc,
        ref_::Ref,
    },
};

use core::{
    fmt::{self},
    result,
};

use std::mem::swap;

use super::to_json::WriteJson;

use std::collections::{HashMap, HashSet};

/// ConstTracker holds references to js compounds (objects or arrays) in two sets:
/// `visited_once` refers to compounds that we've seen just once so far;
/// `visited_repeatedly` refers to compounds that we've seen more than once.
/// When djs tracking pass is done, `visited_repeatedly` refers to compounds that will be written
/// out via const definitions.
/// Note that we use one ConstTracker for js objects and another for js arrays, keeping them
/// separate - to reduce set sizes and save on operations.
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
    /// Writes out a const object, ensuring that its const dependecies are written out as well
    /// in the right order (with no forward references).
    fn write_consts_for_object<D: Dealloc>(
        &mut self,
        any: &Any<D>,
        object_const_builder: &mut ConstBuilder<D>,
        array_const_builder: &mut ConstBuilder<D>,
    ) -> fmt::Result {
        let object = any.clone().try_move::<JsObjectRef<D>>().unwrap();
        for i in object.items().iter() {
            self.write_consts_for_any(&i.1, object_const_builder, array_const_builder)?;
        }
        // TODO:
        // If `object` is present in `object_const_builder.to_do`:
        // 1. Remove it from `object_const_builder.to_do`;
        // 2. Add it to `object_const_builder.done` - using sum of sizes two `done` maps as its ID;
        // 3. Write out "const _<ID>="
        // 4. Call write_list_with_const_refs with '{', '}', ... - needs to be factored out.
        fmt::Result::Ok(())
    }

    /// Writes out a const array, ensuring that its const dependecies are written out as well
    /// in the right order (with no forward references).
    fn write_consts_for_array<D: Dealloc>(
        &mut self,
        any: &Any<D>,
        object_const_builder: &mut ConstBuilder<D>,
        array_const_builder: &mut ConstBuilder<D>,
    ) -> fmt::Result {
        let array = any.clone().try_move::<JsArrayRef<D>>().unwrap();
        for i in array.items().iter() {
            self.write_consts_for_any(i, object_const_builder, array_const_builder)?;
        }
        // TODO:
        // If `array` is present in `array_const_builder.to_do`:
        // 1. Remove it from `array_const_builder.to_do`;
        // 2. Add it to `array_const_builder.done` - using sum of sizes two `done` maps as its ID;
        // 3. Write out "const _<ID>="
        // 4. Call write_list_with_const_refs with '[', ']', ... - needs to be factored out.
        fmt::Result::Ok(())
    }

    /// Writes out a const js entity of any type (skipping over types other than object, array),
    /// ensuring that its const dependecies are written out as well in the right order (with no
    /// forward references).
    fn write_consts_for_any<D: Dealloc>(
        &mut self,
        any: &Any<D>,
        object_const_tracker: &mut ConstBuilder<D>,
        array_const_tracker: &mut ConstBuilder<D>,
    ) -> fmt::Result {
        match any.get_type() {
            Type::Object => {
                self.write_consts_for_object(&any, object_const_tracker, array_const_tracker)?;
            }
            Type::Array => {
                self.write_consts_for_array(&any, object_const_tracker, array_const_tracker)?;
            }
            _ => {}
        }
        fmt::Result::Ok(())
    }

    /// Writes out const objects, arrays in the right order (with no forward references).
    fn write_consts<D: Dealloc>(
        &mut self,
        objects_to_be_cosnt: HashSet<Any<D>>,
        arrays_to_be_const: HashSet<Any<D>>,
        object_const_refs: &mut HashMap<Any<D>, usize>,
        array_const_refs: &mut HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        let mut object_const_builder = new_const_builder(objects_to_be_cosnt);
        let mut array_const_builder = new_const_builder(arrays_to_be_const);
        while let Some(any) = take_from_set(&mut object_const_builder.to_do) {
            self.write_consts_for_object(
                &any,
                &mut object_const_builder,
                &mut array_const_builder,
            )?;
        }
        while let Some(any) = take_from_set(&mut array_const_builder.to_do) {
            self.write_consts_for_array(&any, &mut object_const_builder, &mut array_const_builder)?;
        }
        swap(&mut object_const_builder.done, object_const_refs);
        swap(&mut array_const_builder.done, array_const_refs);
        fmt::Result::Ok(())
    }

    fn write_list_with_const_refs<I, D: Dealloc>(
        &mut self,
        open: char,
        close: char,
        v: &Ref<FlexibleArray<I, impl FlexibleArrayHeader>, D>,
        object_const_refs: &HashMap<Any<D>, usize>,
        array_const_refs: &HashMap<Any<D>, usize>,
        f: impl Fn(&mut Self, &I, &HashMap<Any<D>, usize>, &HashMap<Any<D>, usize>) -> fmt::Result,
    ) -> fmt::Result {
        let mut comma = "";
        self.write_char(open)?;
        for i in v.items().iter() {
            self.write_str(comma)?;
            f(self, i, &object_const_refs, &array_const_refs)?;
            comma = ",";
        }
        self.write_char(close)
    }

    fn write_with_const_refs<D: Dealloc>(
        &mut self,
        any: Any<D>,
        object_const_refs: &HashMap<Any<D>, usize>,
        array_const_refs: &HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        match any.get_type() {
            Type::Object => {
                if let Some(n) = object_const_refs.get(&any) {
                    self.write_str("_")?;
                    self.write_str(n.to_string().as_str())
                } else {
                    self.write_list_with_const_refs(
                        '{',
                        '}',
                        &any.try_move::<JsObjectRef<D>>().unwrap(),
                        object_const_refs,
                        array_const_refs,
                        |w, (k, v), object_const_refs, array_const_refs| {
                            w.write_js_string(k)?;
                            w.write_char(':')?;
                            w.write_with_const_refs(
                                v.clone(),
                                &object_const_refs,
                                &array_const_refs,
                            )
                        },
                    )
                }
            }
            Type::Array => {
                if let Some(n) = array_const_refs.get(&any) {
                    self.write_str("_")?;
                    self.write_str(n.to_string().as_str())
                } else {
                    self.write_list_with_const_refs(
                        '[',
                        ']',
                        &any.try_move::<JsArrayRef<D>>().unwrap(),
                        object_const_refs,
                        array_const_refs,
                        |w, i, object_const_refs, array_const_refs| {
                            w.write_with_const_refs(i.clone(), object_const_refs, array_const_refs)
                        },
                    )
                }
            }
            _ => self.write_json(any),
        }
    }

    ///
    fn write_djs<D: Dealloc>(&mut self, any: Any<D>) -> fmt::Result {
        let (objects_to_be_cosnt, arrays_to_be_const) = track_consts(any.clone());
        let mut object_const_refs = HashMap::<Any<D>, usize>::new();
        let mut array_const_refs = HashMap::<Any<D>, usize>::new();
        self.write_consts(
            objects_to_be_cosnt,
            arrays_to_be_const,
            &mut object_const_refs,
            &mut array_const_refs,
        )?;
        self.write_with_const_refs(any, &object_const_refs, &array_const_refs)
    }
}

impl<T: WriteJson> WriteDjs for T {}

pub fn to_djs(any: Any<impl Dealloc>) -> result::Result<String, fmt::Error> {
    let mut s = String::default();
    s.write_djs(any)?;
    Ok(s)
}
