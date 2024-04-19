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

use std::collections::{HashMap, HashSet};

use super::to_json::WriteJson;

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
/// references in `to_do`, moving them to `done` as we write them out one by one, with each
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
fn is_visited<D: Dealloc>(any: &Any<D>, const_tracker: &mut ConstTracker<D>) -> bool {
    if const_tracker.visited_repeatedly.contains(any) {
        // We've visited `any` more than once before, no action is needed here.
        true
    } else if const_tracker.visited_once.contains(any) {
        // It's the second time we visit `any`, move it from `visited_once` to `to_do`.
        const_tracker.visited_once.remove(any);
        const_tracker.visited_repeatedly.insert(any.clone());
        true
    } else {
        // It's the first time we visit `any`, add it to `visited_once` (that is the only
        // branch where we return `false`).
        const_tracker.visited_once.insert(any.clone());
        false
    }
}

/// Traverse a DAG referred by `object` (a js object), tracking objects and arrays, including `object`
/// itself.
fn track_consts_for_object<D: Dealloc>(
    object: &Any<D>,
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    if !is_visited(object, object_const_tracker) {
        object
            .clone()
            .try_move::<JsObjectRef<D>>()
            .unwrap()
            .items()
            .iter()
            .for_each(|(_k, v)| {
                track_consts_for_any(v, object_const_tracker, array_const_tracker);
            });
        object_const_tracker.visited_once.insert(object.clone());
    }
}

/// Traverse a DAG referred by `array` (a js object), tracking objects and arrays, including `array`
/// itself.
fn track_consts_for_array<D: Dealloc>(
    array: &Any<D>,
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    if !is_visited(array, array_const_tracker) {
        array
            .clone()
            .try_move::<JsArrayRef<D>>()
            .unwrap()
            .items()
            .iter()
            .for_each(|i| {
                track_consts_for_any(i, object_const_tracker, array_const_tracker);
            });
        array_const_tracker.visited_once.insert(array.clone());
    }
}

/// Traverse a DAG referred by `any` (of any js type), tracking objects and arrays, including `any`
/// itself.
fn track_consts_for_any<D: Dealloc>(
    any: &Any<D>,
    object_const_tracker: &mut ConstTracker<D>,
    array_const_tracker: &mut ConstTracker<D>,
) {
    match any.get_type() {
        Type::Object => track_consts_for_object(any, object_const_tracker, array_const_tracker),
        Type::Array => track_consts_for_array(any, object_const_tracker, array_const_tracker),
        _ => {}
    }
}

/// Traverses a DAG referred by `any` - returning two sets of to-be consts (objects, arrays).
fn track_consts<D: Dealloc>(any: &Any<D>) -> (HashSet<Any<D>>, HashSet<Any<D>>) {
    let mut object_const_tracker = new_const_tracker();
    let mut array_const_tracker = new_const_tracker();
    track_consts_for_any(any, &mut object_const_tracker, &mut array_const_tracker);
    (
        object_const_tracker.visited_repeatedly,
        array_const_tracker.visited_repeatedly,
    )
}

/// Peeks one value from a set.
fn peek<D: Dealloc>(set: &HashSet<Any<D>>) -> Option<Any<D>> {
    Some(set.iter().next()?.clone())
}

pub trait WriteDjs: WriteJson {
    /// Writes a const object, ensuring that its const dependencies are written out as well
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
        if object_const_builder.to_do.contains(any) {
            object_const_builder.to_do.remove(any);
            let id = object_const_builder.done.len() + array_const_builder.done.len();
            self.write_str("const _")?;
            self.write_str(id.to_string().as_str())?;
            self.write_char('=')?;
            self.write_with_const_refs(
                any.clone(),
                &object_const_builder.done,
                &array_const_builder.done,
            )?;
            object_const_builder.done.insert(any.clone(), id);
            self.write_char(';')
        } else {
            fmt::Result::Ok(())
        }
    }

    /// Writes a const array, ensuring that its const dependencies are written out as well
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
        if array_const_builder.to_do.contains(any) {
            array_const_builder.to_do.remove(any);
            let id = object_const_builder.done.len() + array_const_builder.done.len();
            self.write_str("const _")?;
            self.write_str(id.to_string().as_str())?;
            self.write_char('=')?;
            self.write_with_const_refs(
                any.clone(),
                &object_const_builder.done,
                &array_const_builder.done,
            )?;
            array_const_builder.done.insert(any.clone(), id);
            self.write_char(';')
        } else {
            fmt::Result::Ok(())
        }
    }

    /// Writes a const js entity of any type (skipping over types other than object, array),
    /// ensuring that its const dependencies are written out as well in the right order (with no
    /// forward references).
    fn write_consts_for_any<D: Dealloc>(
        &mut self,
        any: &Any<D>,
        object_const_builder: &mut ConstBuilder<D>,
        array_const_builder: &mut ConstBuilder<D>,
    ) -> fmt::Result {
        match any.get_type() {
            Type::Object => {
                self.write_consts_for_object(any, object_const_builder, array_const_builder)?;
            }
            Type::Array => {
                self.write_consts_for_array(any, object_const_builder, array_const_builder)?;
            }
            _ => {}
        }
        fmt::Result::Ok(())
    }

    /// Writes const objects, arrays in the right order (with no forward references).
    fn write_consts<D: Dealloc>(
        &mut self,
        objects_to_be_cosnt: HashSet<Any<D>>,
        arrays_to_be_const: HashSet<Any<D>>,
        object_const_refs: &mut HashMap<Any<D>, usize>,
        array_const_refs: &mut HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        let mut object_const_builder = new_const_builder(objects_to_be_cosnt);
        let mut array_const_builder = new_const_builder(arrays_to_be_const);
        while let Some(any) = peek(&object_const_builder.to_do) {
            self.write_consts_for_object(
                &any,
                &mut object_const_builder,
                &mut array_const_builder,
            )?;
        }
        while let Some(any) = peek(&array_const_builder.to_do) {
            self.write_consts_for_array(&any, &mut object_const_builder, &mut array_const_builder)?;
        }
        swap(&mut object_const_builder.done, object_const_refs);
        swap(&mut array_const_builder.done, array_const_refs);
        fmt::Result::Ok(())
    }

    /// Writes `v` (an object or an array) using const references.
    fn write_list_with_const_refs<I, D: Dealloc>(
        &mut self,
        open: char,
        close: char,
        v: &Ref<FlexibleArray<I, impl FlexibleArrayHeader>, D>,
        f: impl Fn(&mut Self, &I) -> fmt::Result,
    ) -> fmt::Result {
        let mut comma = "";
        self.write_char(open)?;
        for i in v.items().iter() {
            self.write_str(comma)?;
            f(self, i)?;
            comma = ",";
        }
        self.write_char(close)
    }

    /// Writes `any` using const references.
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
                        |w, (k, v)| {
                            w.write_js_string(k)?;
                            w.write_char(':')?;
                            w.write_with_const_refs(v.clone(), object_const_refs, array_const_refs)
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
                        |w, i| {
                            w.write_with_const_refs(i.clone(), object_const_refs, array_const_refs)
                        },
                    )
                }
            }
            _ => self.write_json(any),
        }
    }

    /// Writes a DAG referred by `any` with const definitions for objects, arrays that are referred
    /// multiple times.
    fn write_djs<D: Dealloc>(&mut self, any: Any<D>, common_js: bool) -> fmt::Result {
        let (objects_to_be_cosnt, arrays_to_be_const) = track_consts(&any);
        let mut object_const_refs = HashMap::<Any<D>, usize>::new();
        let mut array_const_refs = HashMap::<Any<D>, usize>::new();
        self.write_consts(
            objects_to_be_cosnt,
            arrays_to_be_const,
            &mut object_const_refs,
            &mut array_const_refs,
        )?;
        if common_js {
            self.write_str("module.exports=")?;
        } else {
            self.write_str("export default ")?;
        }
        self.write_with_const_refs(any, &object_const_refs, &array_const_refs)
    }
}

impl<T: WriteJson> WriteDjs for T {}

pub fn to_djs(any: Any<impl Dealloc>, common_js: bool) -> result::Result<String, fmt::Error> {
    let mut s = String::default();
    s.write_djs(any, common_js)?;
    Ok(s)
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        js::{any::Any, any_cast::AnyCast, js_string::new_string, new::New, null::Null},
        mem::global::{Global, GLOBAL},
        serializer::to_djs::WriteDjs,
    };

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        type A = Any<Global>;
        let s = new_string(
            GLOBAL,
            ['a' as u16, '\\' as u16, 'b' as u16, '"' as u16, 31],
        )
        .to_ref();
        let o = GLOBAL.new_js_object([(s, 2.0.move_to_any())]);
        let a0 = GLOBAL.new_js_array([
            1.0.move_to_any(),
            true.move_to_any(),
            Null().move_to_any(),
            GLOBAL.new_js_array([]),
            GLOBAL.new_js_string([]),
            o.clone(),
        ]);
        let a0_as_any: Any<Global> = a0;
        let a1: A = GLOBAL.new_js_array([a0_as_any.clone(), a0_as_any, o]);
        let mut s = String::new();
        s.write_djs(a1, false).unwrap();
        assert_eq!(
            s,
            r#"const _0={"a\\b\"\u001F":2};const _1=[1,true,null,[],"",_0];export default [_1,_1,_0]"#
        );
    }
}
