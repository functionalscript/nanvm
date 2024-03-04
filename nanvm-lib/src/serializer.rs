use crate::{
    common::default::default,
    js::{
        any::Any,
        js_string::JsStringRef,
        visitor::{to_visitor, Visitor},
    },
    mem::{
        flexible_array::{header::FlexibleArrayHeader, FlexibleArray},
        manager::Dealloc,
        ref_::Ref,
    },
};

fn push_js_string(s: &JsStringRef<impl Dealloc>, r: &mut String) {
    r.push('"');
    for &c in s.items().iter() {
        // TODO: escape
        r.push(c as u8 as _);
    }
    r.push('"');
}

fn push_list<I>(
    mut open: char,
    close: char,
    v: Ref<FlexibleArray<I, impl FlexibleArrayHeader>, impl Dealloc>,
    f: impl Fn(&I, &mut String),
    r: &mut String,
) {
    for i in v.items().iter() {
        r.push(open);
        f(i, r);
        open = ',';
    }
    r.push(close);
}

pub fn to_json(any: Any<impl Dealloc>, r: &mut String) {
    match to_visitor(any) {
        Visitor::Number(n) => r.push_str(n.to_string().as_str()),
        Visitor::Null => r.push_str("null"),
        Visitor::Bool(b) => r.push_str(if b { "true" } else { "false" }),
        Visitor::String(s) => push_js_string(&s, r),
        Visitor::Object(o) => {
            let mut r = default();
            push_list(
                '{',
                '}',
                o,
                |kv, r| {
                    push_js_string(&kv.0, r);
                    r.push(':');
                    to_json(kv.1.clone(), r);
                },
                &mut r,
            );
        }
        Visitor::Array(a) => push_list('[', ']', a, |i, r| to_json(i.clone(), r), r),
    }
}
