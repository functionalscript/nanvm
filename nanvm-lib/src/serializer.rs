use crate::{
    common::default::default, js::{
        any::Any,
        js_string::JsStringRef,
        visitor::{to_visitor, Visitor},
    }, mem::manager::Dealloc
};

pub fn push_js_string(s: &JsStringRef<impl Dealloc>, r: &mut String) {
    r.push('"');
    for &c in s.items().iter() {
        // TODO: escape
        r.push(c as u8 as _);
    }
    r.push('"');
}

pub fn to_json(any: Any<impl Dealloc>) -> String {
    match to_visitor(any) {
        Visitor::Number(n) => n.to_string(),
        Visitor::Null => "null".to_string(),
        Visitor::Bool(b) => b.to_string(),
        Visitor::String(s) => {
            let mut r = default();
            push_js_string(&s, &mut r);
            r
        }
        Visitor::Object(o) => {
            let mut r = "{".to_string();
            for (k, v) in o.items().iter() {
                push_js_string(k, &mut r);
                r.push(':');
                r.push_str(&to_json(v.clone()));
                r.push(',');
            }
            r.pop();
            r.push('}');
            r
        }
        Visitor::Array(a) => {
            let mut r = "[".to_string();
            for v in a.items().iter() {
                r.push_str(&to_json(v.clone()));
                r.push(',');
            }
            r.pop();
            r.push(']');
            r
        }
    }
}
