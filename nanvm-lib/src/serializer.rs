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

const ESCAPE_B: u8 = 0x08;
const ESCAPE_F: u8 = 0x0C;

pub trait WriteJson: Write {
    fn write_u4_hex(&mut self, v: u16) -> fmt::Result {
        self.write_char(b"0123456789ABCDEF"[v as usize & 0xF] as char)
    }
    fn write_js_escape(&mut self, c: u16) -> fmt::Result {
        self.write_str("\\u")?;
        self.write_u4_hex(c >> 12)?;
        self.write_u4_hex(c >> 8)?;
        self.write_u4_hex(c >> 4)?;
        self.write_u4_hex(c)
    }
    /// See https://www.json.org/json-en.html
    fn write_js_string(&mut self, s: &JsStringRef<impl Dealloc>) -> fmt::Result {
        self.write_char('"')?;
        for &c in s.items().iter() {
            if c < 0x80 {
                match c as u8 {
                    b'\\' => self.write_str(r#"\\"#)?,
                    b'"' => self.write_str(r#"\""#)?,
                    ESCAPE_B => self.write_str(r#"\b"#)?,
                    ESCAPE_F => self.write_str(r#"\f"#)?,
                    b'\n' => self.write_str(r#"\n"#)?,
                    b'\r' => self.write_str(r#"\r"#)?,
                    b'\t' => self.write_str(r#"\t"#)?,
                    c if c < 0x20 => self.write_js_escape(c as u16)?,
                    c => self.write_char(c as char)?,
                }
            } else {
                self.write_js_escape(c)?;
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
        js::{any::Any, any_cast::AnyCast, js_string::new_string, new::New, null::Null},
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
        let a = GLOBAL.new_js_array([
            1.0.move_to_any(),
            true.move_to_any(),
            Null().move_to_any(),
            GLOBAL.new_js_array([]),
            GLOBAL.new_js_string([]),
            GLOBAL.new_js_object([]),
            GLOBAL.new_js_object([(s, 2.0.move_to_any())]),
        ]);
        let mut s = String::new();
        s.write_json(a).unwrap();
        assert_eq!(s, r#"[1,true,null,[],"",{},{"a\\b\"\u001F":2}]"#);
    }
}
