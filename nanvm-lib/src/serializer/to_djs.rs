use crate::{
    js::{any::Any, js_array::JsArrayRef, js_object::JsObjectRef, type_::Type},
    mem::manager::Dealloc,
};

use core::{
    fmt::{self},
    result,
};

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

impl<D: Dealloc> ConstTracker<D> {
    /// Returns true if `any` was visited before; updates the `const_tracker` set, tracking whether
    /// `any` was visited just once (it's in `const_tracker.visited_once`) or more than once (it's
    /// in `visited_repeatedly` in this case since we are up to writing it out as a const).
    fn is_visited(&mut self, any: &Any<D>) -> bool {
        if self.visited_repeatedly.contains(any) {
            // We've visited `any` more than once before, no action is needed here.
            true
        } else if self.visited_once.contains(any) {
            // It's the second time we visit `any`, move it from `visited_once` to `to_do`.
            self.visited_once.remove(any);
            self.visited_repeatedly.insert(any.clone());
            true
        } else {
            // It's the first time we visit `any`, add it to `visited_once` (that is the only
            // branch where we return `false`).
            self.visited_once.insert(any.clone());
            false
        }
    }

    /// Traverse a DAG referred by `object` (a js object), tracking objects and arrays, including
    /// `object` itself.
    fn track_consts_for_object(&mut self, object: &Any<D>) {
        if !self.is_visited(object) {
            object
                .clone()
                .try_move::<JsObjectRef<D>>()
                .unwrap()
                .items()
                .iter()
                .for_each(|(_k, v)| {
                    self.track_consts_for_any(v);
                });
            self.visited_once.insert(object.clone());
        }
    }

    /// Traverse a DAG referred by `array` (a js object), tracking objects and arrays, including
    /// `array` itself.
    fn track_consts_for_array(&mut self, array: &Any<D>) {
        if !self.is_visited(array) {
            array
                .clone()
                .try_move::<JsArrayRef<D>>()
                .unwrap()
                .items()
                .iter()
                .for_each(|i| {
                    self.track_consts_for_any(i);
                });
            self.visited_once.insert(array.clone());
        }
    }

    /// Traverse a DAG referred by `any` (of any js type), tracking objects and arrays, including
    /// `any` itself.
    fn track_consts_for_any(&mut self, any: &Any<D>) {
        match any.get_type() {
            Type::Array => self.track_consts_for_array(any),
            Type::Object => self.track_consts_for_object(any),
            _ => {}
        }
    }
}

/// Peeks one value from a set.
fn peek<D: Dealloc>(set: &HashSet<Any<D>>) -> Option<Any<D>> {
    Some(set.iter().next()?.clone())
}

pub trait WriteDjs: WriteJson {
    /// Writes a const compound (an array or an object), ensuring that its const dependencies are
    /// written out as well in the right order (with no forward references).
    fn write_compound_const<D: Dealloc>(
        &mut self,
        any: &Any<D>,
        to_be_consts: &mut HashSet<Any<D>>,
        const_refs: &mut HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        if to_be_consts.contains(any) {
            to_be_consts.remove(any);
            let id = const_refs.len();
            self.write_str("const _")?;
            self.write_str(id.to_string().as_str())?;
            self.write_char('=')?;
            self.write_with_const_refs(any.clone(), const_refs)?;
            const_refs.insert(any.clone(), id);
            self.write_char(';')
        } else {
            fmt::Result::Ok(())
        }
    }

    /// Writes a const js entity of any type (skipping over types other than object, array),
    /// ensuring that its const dependencies are written out as well in the right order (with no
    /// forward references).
    fn write_consts_and_any<D: Dealloc>(
        &mut self,
        any: &Any<D>,
        to_be_consts: &mut HashSet<Any<D>>,
        const_refs: &mut HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        match any.get_type() {
            Type::Array => {
                let array = any.clone().try_move::<JsArrayRef<D>>().unwrap();
                for i in array.items().iter() {
                    self.write_consts_and_any(i, to_be_consts, const_refs)?;
                }
                self.write_compound_const(any, to_be_consts, const_refs)?;
            }
            Type::Object => {
                let object = any.clone().try_move::<JsObjectRef<D>>().unwrap();
                for i in object.items().iter() {
                    self.write_consts_and_any(&i.1, to_be_consts, const_refs)?;
                }
                self.write_compound_const(any, to_be_consts, const_refs)?;
            }
            _ => {}
        }
        fmt::Result::Ok(())
    }

    /// Writes const objects, arrays in the right order (with no forward references).
    fn write_consts<D: Dealloc>(
        &mut self,
        to_be_consts: &mut HashSet<Any<D>>,
        const_refs: &mut HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        while let Some(any) = peek(to_be_consts) {
            self.write_consts_and_any(&any, to_be_consts, const_refs)?;
        }
        fmt::Result::Ok(())
    }

    /// Writes `any` using const references.
    fn write_with_const_refs<D: Dealloc>(
        &mut self,
        any: Any<D>,
        const_refs: &HashMap<Any<D>, usize>,
    ) -> fmt::Result {
        match any.get_type() {
            Type::Object => {
                if let Some(n) = const_refs.get(&any) {
                    self.write_str("_")?;
                    self.write_str(n.to_string().as_str())
                } else {
                    self.write_list(
                        '{',
                        '}',
                        any.try_move::<JsObjectRef<D>>().unwrap(),
                        |w, (k, v)| {
                            w.write_js_string(k)?;
                            w.write_char(':')?;
                            w.write_with_const_refs(v.clone(), const_refs)
                        },
                    )
                }
            }
            Type::Array => {
                if let Some(n) = const_refs.get(&any) {
                    self.write_str("_")?;
                    self.write_str(n.to_string().as_str())
                } else {
                    self.write_list(
                        '[',
                        ']',
                        any.try_move::<JsArrayRef<D>>().unwrap(),
                        |w, i| w.write_with_const_refs(i.clone(), const_refs),
                    )
                }
            }
            _ => self.write_json(any),
        }
    }

    /// Writes a DAG referred by `any` with const definitions for objects, arrays that are referred
    /// multiple times.
    fn write_djs<D: Dealloc>(&mut self, any: Any<D>, common_js: bool) -> fmt::Result {
        let mut const_refs = HashMap::<Any<D>, usize>::new();
        let mut const_tracker = ConstTracker {
            visited_once: HashSet::new(),
            visited_repeatedly: HashSet::new(),
        };
        const_tracker.track_consts_for_any(&any);
        self.write_consts(&mut const_tracker.visited_repeatedly, &mut const_refs)?;
        if common_js {
            self.write_str("module.exports=")?;
        } else {
            self.write_str("export default ")?;
        }
        self.write_with_const_refs(any, &const_refs)
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
