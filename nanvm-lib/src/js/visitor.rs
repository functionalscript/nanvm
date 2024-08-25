use crate::mem::manager::Dealloc;

use super::{
    any::Any, js_array::JsArrayRef, js_bigint::JsBigintRef, js_object::JsObjectRef,
    js_string::JsStringRef, type_::Type,
};

pub enum Visitor<T: Dealloc> {
    Number(f64),
    Null,
    Bool(bool),
    String(JsStringRef<T>),
    Object(JsObjectRef<T>),
    Array(JsArrayRef<T>),
    Bigint(JsBigintRef<T>),
}

pub fn to_visitor<T: Dealloc>(any: Any<T>) -> Visitor<T> {
    match any.get_type() {
        Type::Number => Visitor::Number(any.try_move().unwrap()),
        Type::Null => Visitor::Null,
        Type::Bool => Visitor::Bool(any.try_move().unwrap()),
        Type::String => Visitor::String(any.try_move().unwrap()),
        Type::Object => Visitor::Object(any.try_move().unwrap()),
        Type::Array => Visitor::Array(any.try_move().unwrap()),
        Type::Bigint => Visitor::Bigint(any.try_move().unwrap()),
    }
}
