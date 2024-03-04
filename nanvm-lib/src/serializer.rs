use crate::{
    js::{
        any::Any,
        js_string::JsStringRef,
        visitor::{to_visitor, Visitor},
    },
    mem::manager::Dealloc,
};

pub fn to_string(s: JsStringRef<impl Dealloc>) -> String {
    let mut r = String::new();
    r.push('"');
    for &c in s.items().iter() {
        // TODO: escape
        r.push(c as u8 as _);
    }
    r.push('"');
    r
}

pub fn to_json(any: Any<impl Dealloc>) -> String {
    match to_visitor(any) {
        Visitor::Number(n) => n.to_string(),
        Visitor::Null => "null".to_string(),
        Visitor::Bool(b) => b.to_string(),
        Visitor::String(s) => to_string(s),
        Visitor::Object(o) => {
            let mut r = "{".to_string();
            for (k, v) in o.items().iter() {
                r.push_str(&to_string(k.clone()));
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
