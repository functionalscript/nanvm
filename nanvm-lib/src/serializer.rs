use crate::{
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

use core::fmt::{self, Write};

fn write_js_string(s: &JsStringRef<impl Dealloc>, r: &mut impl Write) -> fmt::Result {
    r.write_char('"')?;
    for &c in s.items().iter() {
        // TODO: escape
        r.write_char(c as u8 as _)?;
    }
    r.write_char('"')
}

fn write_list<I, W: Write>(
    mut open: char,
    close: char,
    v: Ref<FlexibleArray<I, impl FlexibleArrayHeader>, impl Dealloc>,
    f: impl Fn(&I, &mut W) -> fmt::Result,
    r: &mut W,
) -> fmt::Result {
    for i in v.items().iter() {
        r.write_char(open)?;
        f(i, r)?;
        open = ',';
    }
    r.write_char(close)
}

pub fn write_json(any: Any<impl Dealloc>, r: &mut String) -> fmt::Result {
    match to_visitor(any) {
        Visitor::Number(n) => r.write_str(n.to_string().as_str()),
        Visitor::Null => r.write_str("null"),
        Visitor::Bool(b) => r.write_str(if b { "true" } else { "false" }),
        Visitor::String(s) => write_js_string(&s, r),
        Visitor::Object(o) => write_list(
            '{',
            '}',
            o,
            |(k, v), r| {
                write_js_string(k, r)?;
                r.write_char(':')?;
                write_json(v.clone(), r)
            },
            r,
        ),
        Visitor::Array(a) => write_list('[', ']', a, |i, r| write_json(i.clone(), r), r),
    }
}
