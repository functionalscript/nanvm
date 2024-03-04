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

trait WriteJson: Write {
    fn write_hex(&mut self, v: u16) -> fmt::Result {
        self.write_char(b"0123456789ABCDEF"[v as usize & 0xF] as char)
    }
    fn write_js_string(&mut self, s: &JsStringRef<impl Dealloc>) -> fmt::Result {
        self.write_char('"')?;
        for &c in s.items().iter() {
            if 0x20 <= c && c < 0x80 {
                match c as u8 as char {
                    '\\' => self.write_str(r#"\\"#)?,
                    '"' => self.write_str(r#"\""#)?,
                    c => self.write_char(c)?,
                }
            } else {
                self.write_str("\\u")?;
                self.write_hex(c >> 12)?;
                self.write_hex(c >> 8)?;
                self.write_hex(c >> 4)?;
                self.write_hex(c)?;
            }
        }
        self.write_char('"')
    }

    fn write_list<I>(
        &mut self,
        open: char,
        close: char,
        v: Ref<FlexibleArray<I, impl FlexibleArrayHeader>, impl Dealloc>,
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

    fn write_json(&mut self, any: Any<impl Dealloc>) -> fmt::Result {
        match to_visitor(any) {
            // TODO: replace with proper JSON number serializer.
            Visitor::Number(n) => self.write_str(n.to_string().as_str()),
            Visitor::Null => self.write_str("null"),
            Visitor::Bool(b) => self.write_str(if b { "true" } else { "false" }),
            Visitor::String(s) => self.write_js_string(&s),
            Visitor::Object(o) => self.write_list('{', '}', o, |w, (k, v)| {
                w.write_js_string(k)?;
                w.write_char(':')?;
                w.write_json(v.clone())
            }),
            Visitor::Array(a) => self.write_list('[', ']', a, |w, i| w.write_json(i.clone())),
        }
    }
}

impl<T: Write> WriteJson for T {}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        js::{
            any::Any, js_array::new_array, js_object::new_object, js_string::new_string, null::Null,
        },
        mem::global::{Global, GLOBAL},
        serializer::WriteJson,
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
        let a = new_array(
            GLOBAL,
            [
                A::move_from(1.0),
                A::move_from(true),
                A::move_from(Null()),
                A::move_from(new_array(GLOBAL, []).to_ref()),
                A::move_from(new_string(GLOBAL, []).to_ref()),
                A::move_from(new_object(GLOBAL, []).to_ref()),
                A::move_from(new_object(GLOBAL, [(s, A::move_from(2.0))]).to_ref()),
            ],
        );
        let mut s = String::new();
        s.write_json(A::move_from(a.to_ref())).unwrap();
        assert_eq!(s, r#"[1,true,null,[],"",{},{"a\\b\"\u001F":2}]"#);
    }
}
