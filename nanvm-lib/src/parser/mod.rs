use std::collections::BTreeMap;

use crate::{
    common::{cast::Cast, default::default},
    js::{
        any::Any,
        js_array::new_array,
        js_object::new_object,
        js_string::{new_string, JsStringRef},
        null::Null,
    },
    mem::manager::{Dealloc, Manager},
    tokenizer::JsonToken,
};

pub enum JsonStackElement<D: Dealloc> {
    Object(JsonStackObject<D>),
    Array(Vec<Any<D>>),
}

pub struct JsonStackObject<D: Dealloc> {
    pub map: BTreeMap<String, Any<D>>,
    pub key: String,
}

#[derive(Default)]
pub enum ParseStatus {
    #[default]
    Initial,
    ArrayStart,
    ArrayValue,
    ArrayComma,
    ObjectStart,
    ObjectKey,
    ObjectColon,
    ObjectValue,
    ObjectComma,
}

pub struct ParseAnyState<M: Manager> {
    pub data_type: DataType,
    pub status: ParseStatus,
    pub top: Option<JsonStackElement<M::Dealloc>>,
    pub stack: Vec<JsonStackElement<M::Dealloc>>,
    pub consts: BTreeMap<String, Any<M::Dealloc>>,
}

pub struct ParseAnySuccess<M: Manager> {
    pub state: ParseAnyState<M>,
    pub value: Any<M::Dealloc>,
}

pub enum ParseAnyResult<M: Manager> {
    Continue(ParseAnyState<M>),
    Success(ParseAnySuccess<M>),
    Error(ParseError),
}

impl<M: Manager> Default for ParseAnyState<M> {
    fn default() -> Self {
        ParseAnyState {
            data_type: default(),
            status: ParseStatus::Initial,
            top: None,
            stack: [].cast(),
            consts: default(),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
    WrongExportStatement,
    WrongConstStatement,
}

#[derive(Debug, Default, PartialEq)]
pub enum DataType {
    #[default]
    Json,
    Djs,
}

pub struct ParseResult<M: Manager> {
    pub data_type: DataType,
    pub any: Any<M::Dealloc>,
}

pub enum RootStatus {
    Initial,
    Export,
    Module,
    ModuleDot,
    ModuleDotExports,
    Const,
    ConstId(String),
}

pub struct RootState<M: Manager> {
    pub status: RootStatus,
    pub state: ParseAnyState<M>,
}

pub struct ParseConstState<M: Manager> {
    pub key: String,
    pub state: ParseAnyState<M>,
}

pub enum JsonState<M: Manager> {
    ParseRoot(RootState<M>),
    ParseConst(ParseConstState<M>),
    ParseModule(ParseAnyState<M>),
    Result(ParseResult<M>),
    Error(ParseError),
}

fn to_js_string<M: Manager>(manager: M, s: String) -> JsStringRef<M::Dealloc> {
    new_string(manager, s.encode_utf16().collect::<Vec<_>>().into_iter()).to_ref()
}

fn try_id_to_any<M: Manager>(
    s: &str,
    _manager: M,
    consts: &BTreeMap<String, Any<M::Dealloc>>,
) -> Option<Any<M::Dealloc>> {
    match s {
        "null" => Some(Any::move_from(Null())),
        "true" => Some(Any::move_from(true)),
        "false" => Some(Any::move_from(false)),
        s if consts.contains_key(s) => Some(consts.get(s).unwrap().clone()),
        _ => None,
    }
}

impl JsonToken {
    fn try_to_any<M: Manager>(
        self,
        manager: M,
        consts: &BTreeMap<String, Any<M::Dealloc>>,
    ) -> Option<Any<M::Dealloc>> {
        match self {
            JsonToken::Number(f) => Some(Any::move_from(f)),
            JsonToken::String(s) => Some(Any::move_from(to_js_string(manager, s))),
            JsonToken::Id(s) => try_id_to_any(&s, manager, consts),
            _ => None,
        }
    }
}

impl<M: Manager> ParseAnyState<M> {
    fn default(data_type: DataType) -> Self {
        ParseAnyState {
            data_type,
            status: ParseStatus::Initial,
            top: None,
            stack: [].cast(),
            consts: default(),
        }
    }
}

impl<M: Manager> RootState<M> {
    fn parse(self, manager: M, token: JsonToken) -> JsonState<M> {
        match self.status {
            RootStatus::Initial => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "const" => JsonState::ParseRoot(RootState {
                        status: RootStatus::Const,
                        state: self.state.set_djs(),
                    }),
                    "export" => JsonState::ParseRoot(RootState {
                        status: RootStatus::Export,
                        state: self.state.set_djs(),
                    }),
                    "module" => JsonState::ParseRoot(RootState {
                        status: RootStatus::Module,
                        state: self.state.set_djs(),
                    }),
                    _ => self.state.parse_for_module(manager, JsonToken::Id(s)),
                },
                _ => self.state.parse_for_module(manager, token),
            },
            RootStatus::Export => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "default" => JsonState::ParseModule(self.state),
                    _ => JsonState::Error(ParseError::WrongExportStatement),
                },
                _ => JsonState::Error(ParseError::WrongExportStatement),
            },
            RootStatus::Module => match token {
                JsonToken::Dot => JsonState::ParseRoot(RootState {
                    status: RootStatus::ModuleDot,
                    state: self.state,
                }),
                _ => JsonState::Error(ParseError::WrongExportStatement),
            },
            RootStatus::ModuleDot => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "exports" => JsonState::ParseRoot(RootState {
                        status: RootStatus::ModuleDotExports,
                        state: self.state,
                    }),
                    _ => JsonState::Error(ParseError::WrongExportStatement),
                },
                _ => JsonState::Error(ParseError::WrongExportStatement),
            },
            RootStatus::ModuleDotExports => match token {
                JsonToken::Equals => JsonState::ParseModule(self.state),
                _ => JsonState::Error(ParseError::WrongExportStatement),
            },
            RootStatus::Const => match token {
                JsonToken::Id(s) => JsonState::ParseRoot(RootState {
                    status: RootStatus::ConstId(s),
                    state: self.state,
                }),
                _ => JsonState::Error(ParseError::WrongConstStatement),
            },
            RootStatus::ConstId(s) => match token {
                JsonToken::Equals => JsonState::ParseConst(ParseConstState {
                    key: s,
                    state: self.state,
                }),
                _ => JsonState::Error(ParseError::WrongConstStatement),
            },
        }
    }
}

impl<M: Manager> ParseConstState<M> {
    fn parse(self, manager: M, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::Semicolon => todo!(),
            _ => {
                let result = self.state.parse(manager, token);
                match result {
                    ParseAnyResult::Continue(state) => JsonState::ParseConst(ParseConstState {
                        key: self.key,
                        state,
                    }),
                    ParseAnyResult::Success(mut success) => {
                        success.state.consts.insert(self.key, success.value);
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            state: success.state,
                        })
                    }
                    ParseAnyResult::Error(error) => JsonState::Error(error),
                }
            }
        }
    }
}

impl<M: Manager> ParseAnyState<M> {
    fn set_djs(self) -> Self {
        ParseAnyState {
            data_type: DataType::Djs,
            status: self.status,
            top: self.top,
            stack: self.stack,
            consts: self.consts,
        }
    }

    fn parse_for_module(self, manager: M, token: JsonToken) -> JsonState<M> {
        let result = self.parse(manager, token);
        match result {
            ParseAnyResult::Continue(state) => JsonState::ParseModule(state),
            ParseAnyResult::Success(success) => JsonState::Result(ParseResult {
                data_type: success.state.data_type,
                any: success.value,
            }),
            ParseAnyResult::Error(error) => JsonState::Error(error),
        }
    }

    fn parse(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match self.status {
            ParseStatus::Initial | ParseStatus::ObjectColon => self.parse_value(manager, token),
            ParseStatus::ArrayStart => self.parse_array_start(manager, token),
            ParseStatus::ArrayValue => self.parse_array_value(manager, token),
            ParseStatus::ArrayComma => self.parse_array_comma(manager, token),
            ParseStatus::ObjectStart => self.parse_object_start(manager, token),
            ParseStatus::ObjectKey => self.parse_object_key(token),
            ParseStatus::ObjectValue => self.parse_object_next(manager, token),
            ParseStatus::ObjectComma => self.parse_object_comma(token),
        }
    }

    fn push_value(self, value: Any<M::Dealloc>) -> ParseAnyResult<M> {
        match self.top {
            None => ParseAnyResult::Success(ParseAnySuccess { state: self, value }),
            Some(top) => match top {
                JsonStackElement::Array(mut arr) => {
                    arr.push(value);
                    ParseAnyResult::Continue(ParseAnyState {
                        data_type: self.data_type,
                        status: ParseStatus::ArrayValue,
                        top: Option::Some(JsonStackElement::Array(arr)),
                        stack: self.stack,
                        consts: self.consts,
                    })
                }
                JsonStackElement::Object(mut stack_obj) => {
                    stack_obj.map.insert(stack_obj.key, value);
                    let new_stack_obj = JsonStackObject {
                        map: stack_obj.map,
                        key: String::default(),
                    };
                    ParseAnyResult::Continue(ParseAnyState {
                        data_type: self.data_type,
                        status: ParseStatus::ObjectValue,
                        top: Option::Some(JsonStackElement::Object(new_stack_obj)),
                        stack: self.stack,
                        consts: self.consts,
                    })
                }
            },
        }
    }

    fn push_key(self, s: String) -> ParseAnyResult<M> {
        match self.top {
            Some(JsonStackElement::Object(stack_obj)) => {
                let new_stack_obj = JsonStackObject {
                    map: stack_obj.map,
                    key: s,
                };
                ParseAnyResult::Continue(ParseAnyState {
                    data_type: self.data_type,
                    status: ParseStatus::ObjectKey,
                    top: Option::Some(JsonStackElement::Object(new_stack_obj)),
                    stack: self.stack,
                    consts: self.consts,
                })
            }
            _ => ParseAnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn start_array(mut self) -> ParseAnyResult<M> {
        let new_top = JsonStackElement::Array(Vec::default());
        if let Some(top) = self.top {
            self.stack.push(top);
        }
        ParseAnyResult::Continue(ParseAnyState {
            data_type: self.data_type,
            status: ParseStatus::ArrayStart,
            top: Some(new_top),
            stack: self.stack,
            consts: self.consts,
        })
    }

    fn end_array(mut self, manager: M) -> ParseAnyResult<M> {
        match self.top {
            Some(JsonStackElement::Array(array)) => {
                let js_array = new_array(manager, array.into_iter()).to_ref();
                let new_state = ParseAnyState {
                    data_type: self.data_type,
                    status: ParseStatus::ArrayStart,
                    top: self.stack.pop(),
                    stack: self.stack,
                    consts: self.consts,
                };
                new_state.push_value(Any::move_from(js_array))
            }
            _ => unreachable!(),
        }
    }

    fn start_object(mut self) -> ParseAnyResult<M> {
        let new_top: JsonStackElement<<M as Manager>::Dealloc> =
            JsonStackElement::Object(JsonStackObject {
                map: BTreeMap::default(),
                key: String::default(),
            });
        if let Some(top) = self.top {
            self.stack.push(top)
        }
        ParseAnyResult::Continue(ParseAnyState {
            data_type: self.data_type,
            status: ParseStatus::ObjectStart,
            top: Some(new_top),
            stack: self.stack,
            consts: self.consts,
        })
    }

    fn end_object(mut self, manager: M) -> ParseAnyResult<M> {
        match self.top {
            Some(JsonStackElement::Object(object)) => {
                let vec = object
                    .map
                    .into_iter()
                    .map(|kv| (to_js_string(manager, kv.0), kv.1))
                    .collect::<Vec<_>>();
                let js_object = new_object(manager, vec.into_iter()).to_ref();
                let new_state = ParseAnyState {
                    data_type: self.data_type,
                    status: ParseStatus::ArrayStart,
                    top: self.stack.pop(),
                    stack: self.stack,
                    consts: self.consts,
                };
                new_state.push_value(Any::move_from(js_object))
            }
            _ => unreachable!(),
        }
    }

    fn parse_value(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::ArrayBegin => self.start_array(),
            JsonToken::ObjectBegin => self.start_object(),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => ParseAnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_comma(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::ArrayBegin => self.start_array(),
            JsonToken::ObjectBegin => self.start_object(),
            JsonToken::ArrayEnd => self.end_array(manager),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => ParseAnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_start(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::ArrayBegin => self.start_array(),
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::ObjectBegin => self.start_object(),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => ParseAnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_value(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::Comma => ParseAnyResult::Continue(ParseAnyState {
                data_type: self.data_type,
                status: ParseStatus::ArrayComma,
                top: self.top,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => ParseAnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_start(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::Id(s) if self.data_type == DataType::Djs => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => ParseAnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_key(self, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::Colon => ParseAnyResult::Continue(ParseAnyState {
                data_type: self.data_type,
                status: ParseStatus::ObjectColon,
                top: self.top,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => ParseAnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_next(self, manager: M, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::ObjectEnd => self.end_object(manager),
            JsonToken::Comma => ParseAnyResult::Continue(ParseAnyState {
                data_type: self.data_type,
                status: ParseStatus::ObjectComma,
                top: self.top,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => ParseAnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_comma(self, token: JsonToken) -> ParseAnyResult<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            _ => ParseAnyResult::Error(ParseError::UnexpectedToken),
        }
    }
}

impl<M: Manager> JsonState<M> {
    fn push(self, manager: M, token: JsonToken) -> JsonState<M> {
        if token == JsonToken::NewLine {
            return self;
        }
        match self {
            JsonState::ParseRoot(state) => state.parse(manager, token),
            JsonState::Result(_) => JsonState::Error(ParseError::UnexpectedToken),
            JsonState::ParseModule(state) => state.parse_for_module(manager, token),
            JsonState::ParseConst(state) => state.parse(manager, token),
            _ => self,
        }
    }

    fn end(self) -> Result<ParseResult<M>, ParseError> {
        match self {
            JsonState::Result(result) => Ok(result),
            JsonState::Error(error) => Err(error),
            _ => Err(ParseError::UnexpectedEnd),
        }
    }
}

fn parse<M: Manager>(
    manager: M,
    iter: impl Iterator<Item = JsonToken>,
) -> Result<ParseResult<M>, ParseError> {
    let mut state: JsonState<M> = JsonState::ParseRoot(RootState {
        status: RootStatus::Initial,
        state: default(),
    });
    for token in iter {
        state = state.push(manager, token);
    }
    state.end()
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        js::{js_array::JsArrayRef, js_object::JsObjectRef, js_string::JsStringRef, type_::Type},
        mem::{global::GLOBAL, local::Local, manager::Manager},
        parser::DataType,
        tokenizer::{tokenize, ErrorType, JsonToken},
    };

    use super::parse;

    fn test_local() {
        let local = Local::default();
        let _ = parse(&local, [].into_iter());
    }

    fn test_global() {
        let _ = {
            let global = GLOBAL;
            parse(global, [].into_iter())
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_json() {
        let json_str = include_str!("../../test/test-json.json");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_djs() {
        let json_str = include_str!("../../test/test-djs.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Djs);

        let json_str = include_str!("../../test/test-djs.d.mjs");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Djs);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_const() {
        let local = Local::default();
        test_const_with_manager(&local);
    }

    fn test_const_with_manager<M: Manager>(manager: M) {
        let json_str = include_str!("../../test/test-const.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(2.0));

        let json_str = include_str!("../../test/test-const-error.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_check_sizes() {
        let local = Local::default();
        {
            let tokens = [
                JsonToken::ObjectBegin,
                JsonToken::String(String::from("k")),
                JsonToken::Colon,
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd,
                JsonToken::ObjectEnd,
            ];
            {
                let result = parse(&local, tokens.into_iter());
                assert!(result.is_ok());
                let _result_unwrap = result.unwrap();
            }
            assert_eq!(local.size(), 0);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_check_sizes2() {
        let local = Local::default();
        {
            let tokens = [
                JsonToken::ObjectBegin,
                JsonToken::String(String::from("k")),
                JsonToken::Colon,
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd,
                JsonToken::ObjectEnd,
            ];
            {
                let result = parse(&local, tokens.into_iter());
                assert!(result.is_ok());
                let result_unwrap = result.unwrap().any;
                let _result_unwrap = result_unwrap.try_move::<JsObjectRef<_>>();
            }
            assert_eq!(local.size(), 0);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_data_type() {
        let local = Local::default();
        let tokens = [JsonToken::Id(String::from("null"))];
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_export_block() {
        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("export")),
            JsonToken::Id(String::from("default")),
            JsonToken::Id(String::from("null")),
        ];
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Djs);

        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("module")),
            JsonToken::Dot,
            JsonToken::Id(String::from("exports")),
            JsonToken::Equals,
            JsonToken::Id(String::from("null")),
        ];
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Djs);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_id_in_objects() {
        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("export")),
            JsonToken::Id(String::from("default")),
            JsonToken::ObjectBegin,
            JsonToken::Id(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result.unwrap().any.try_move::<JsObjectRef<_>>().unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x6b, 0x65, 0x79]);
        assert_eq!(value0.try_move(), Ok(0.0));

        let local = Local::default();
        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Id(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(&local, tokens.into_iter());
        assert!(result.is_err());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_valid_local() {
        test_valid_with_manager(&Local::default());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_valid_global() {
        test_valid_with_manager(GLOBAL);
    }

    fn test_valid_with_manager<M: Manager>(manager: M) {
        let tokens = [JsonToken::Id(String::from("null"))];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.get_type(), Type::Null);

        let tokens = [JsonToken::Id(String::from("true"))];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(true));

        let tokens = [JsonToken::Id(String::from("false"))];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(false));

        let tokens = [JsonToken::Number(0.1)];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(0.1));

        let tokens = [JsonToken::String(String::from("abc"))];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result = result.unwrap().any.try_move::<JsStringRef<M::Dealloc>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        let items = result.items();
        assert_eq!(items, [0x61, 0x62, 0x63]);

        let tokens = [JsonToken::ArrayBegin, JsonToken::ArrayEnd];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        assert!(items.is_empty());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(1.0),
            JsonToken::Comma,
            JsonToken::Id(String::from("true")),
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(1.0));
        let item1 = items[1].clone();
        assert_eq!(item1.try_move(), Ok(true));

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(1.0),
            JsonToken::Comma,
            JsonToken::Id(String::from("true")),
            JsonToken::Comma,
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("k1")),
            JsonToken::Colon,
            JsonToken::Number(1.0),
            JsonToken::Comma,
            JsonToken::String(String::from("k0")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::String(String::from("k2")),
            JsonToken::Colon,
            JsonToken::Number(2.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsObjectRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x6b, 0x30]);
        assert_eq!(value0.try_move(), Ok(0.0));
        let (key1, value1) = items[1].clone();
        let key1_items = key1.items();
        assert_eq!(key1_items, [0x6b, 0x31]);
        assert_eq!(value1.try_move(), Ok(1.0));
        let (key2, value2) = items[2].clone();
        let key2_items = key2.items();
        assert_eq!(key2_items, [0x6b, 0x32]);
        assert_eq!(value2.try_move(), Ok(2.0));

        let tokens = [JsonToken::ObjectBegin, JsonToken::ObjectEnd];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsObjectRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        assert!(items.is_empty());
        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("k")),
            JsonToken::Colon,
            JsonToken::ObjectBegin,
            JsonToken::ObjectEnd,
            JsonToken::ObjectEnd,
        ];
        {
            let result = parse(manager, tokens.into_iter());
            assert!(result.is_ok());
            let result_unwrap = result.unwrap();
            let result_unwrap = result_unwrap
                .any
                .try_move::<JsObjectRef<M::Dealloc>>()
                .unwrap();
            let items = result_unwrap.items();
            let (_, value0) = items[0].clone();
            let value0_unwrap = value0.try_move::<JsObjectRef<M::Dealloc>>().unwrap();
            let value0_items = value0_unwrap.items();
            assert!(value0_items.is_empty());
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_invalid_local() {
        test_invalid_with_manager(&Local::default());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_invalid_global() {
        test_invalid_with_manager(GLOBAL);
    }

    fn test_invalid_with_manager<M: Manager>(manager: M) {
        let tokens = [];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ErrorToken(ErrorType::InvalidNumber)];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::Comma, JsonToken::ArrayEnd];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(0.0),
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::ArrayEnd,
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::String(String::default())];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::Colon, JsonToken::ArrayEnd];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayEnd];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Comma,
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key0")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Comma,
            JsonToken::String(String::from("key1")),
            JsonToken::Colon,
            JsonToken::Number(1.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::ObjectEnd,
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Comma,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ObjectEnd];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::ObjectBegin,
            JsonToken::ArrayEnd,
            JsonToken::ObjectEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::ArrayBegin,
            JsonToken::ObjectEnd,
            JsonToken::ArrayEnd,
        ];
        let result = parse(manager, tokens.into_iter());
        assert!(result.is_err());
    }
}
