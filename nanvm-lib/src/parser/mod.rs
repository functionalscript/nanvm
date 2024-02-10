use std::collections::BTreeMap;

use io_trait::Io;

//

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
    tokenizer::{tokenize, JsonToken},
};

pub enum JsonElement<D: Dealloc> {
    None,
    Stack(JsonStackElement<D>),
    Any(Any<D>),
}

pub enum JsonStackElement<D: Dealloc> {
    Object(JsonStackObject<D>),
    Array(Vec<Any<D>>),
}

pub struct JsonStackObject<D: Dealloc> {
    pub map: BTreeMap<String, Any<D>>,
    pub key: String,
}

pub struct Context<M: Manager, I: Io> {
    manager: M,
    io: I,
    path: String,
}

#[derive(Default, Debug)]
pub enum ParsingStatus {
    #[default]
    Initial,
    ArrayBegin,
    ArrayValue,
    ArrayComma,
    ObjectBegin,
    ObjectKey,
    ObjectColon,
    ObjectValue,
    ObjectComma,
    ImportBegin,
    ImportValue,
    ImportEnd,
}

pub struct AnyState<M: Manager> {
    pub data_type: DataType,
    pub status: ParsingStatus,
    pub current: JsonElement<M::Dealloc>,
    pub stack: Vec<JsonStackElement<M::Dealloc>>,
    pub consts: BTreeMap<String, Any<M::Dealloc>>,
}

pub struct AnySuccess<M: Manager> {
    pub state: AnyState<M>,
    pub value: Any<M::Dealloc>,
}

pub enum AnyResult<M: Manager> {
    Continue(AnyState<M>),
    Success(AnySuccess<M>),
    Error(ParseError),
}

impl<M: Manager> Default for AnyState<M> {
    fn default() -> Self {
        AnyState {
            data_type: default(),
            status: ParsingStatus::Initial,
            current: JsonElement::None,
            stack: [].cast(),
            consts: default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
    WrongExportStatement,
    WrongConstStatement,
    WrongRequireStatement,
    WrongImportStatement,
    CannotReadFile,
}

#[derive(Debug, Default, PartialEq)]
pub enum DataType {
    #[default]
    Json,
    Djs,
    Cjs,
    Mjs,
}

#[derive(Debug)]
pub struct ParseResult<M: Manager> {
    pub data_type: DataType,
    pub any: Any<M::Dealloc>,
}

#[derive(Debug)]
pub enum RootStatus {
    Initial,
    Export,
    Module,
    ModuleDot,
    ModuleDotExports,
    Const,
    ConstId(String),
    Import,
    ImportId(String),
    ImportIdFrom(String),
}

pub struct RootState<M: Manager> {
    pub status: RootStatus,
    pub state: AnyState<M>,
}

pub struct ConstState<M: Manager> {
    pub key: String,
    pub state: AnyState<M>,
}

pub enum JsonState<M: Manager> {
    ParseRoot(RootState<M>),
    ParseConst(ConstState<M>),
    ParseModule(AnyState<M>),
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

impl DataType {
    fn to_djs(&self) -> DataType {
        match self {
            DataType::Json | DataType::Djs => DataType::Djs,
            DataType::Cjs => DataType::Cjs,
            DataType::Mjs => DataType::Mjs,
        }
    }

    fn is_djs(&self) -> bool {
        matches!(self, DataType::Djs | DataType::Cjs | DataType::Mjs)
    }

    fn is_cjs_compatible(&self) -> bool {
        matches!(self, DataType::Json | DataType::Djs | DataType::Cjs)
    }

    fn is_mjs_compatible(&self) -> bool {
        matches!(self, DataType::Json | DataType::Djs | DataType::Mjs)
    }
}

impl<M: Manager> AnyState<M> {
    fn default(data_type: DataType) -> Self {
        AnyState {
            data_type,
            status: ParsingStatus::Initial,
            current: JsonElement::None,
            stack: [].cast(),
            consts: default(),
        }
    }
}

impl<M: Manager> RootState<M> {
    fn parse<I: Io>(mut self, context: &Context<M, I>, token: JsonToken) -> JsonState<M> {
        match self.status {
            RootStatus::Initial => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "const" => JsonState::ParseRoot(RootState {
                        status: RootStatus::Const,
                        state: self.state.set_djs(),
                    }),
                    "export" if self.state.data_type.is_mjs_compatible() => {
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Export,
                            state: self.state.set_data_type(DataType::Mjs),
                        })
                    }
                    "module" if self.state.data_type.is_cjs_compatible() => {
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Module,
                            state: self.state.set_data_type(DataType::Cjs),
                        })
                    }
                    "import" if self.state.data_type.is_mjs_compatible() => {
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Import,
                            state: self.state.set_data_type(DataType::Mjs),
                        })
                    }
                    _ => self.state.parse_for_module(context, JsonToken::Id(s)),
                },
                _ => self.state.parse_for_module(context, token),
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
                JsonToken::Equals => JsonState::ParseConst(ConstState {
                    key: s,
                    state: self.state,
                }),
                _ => JsonState::Error(ParseError::WrongConstStatement),
            },
            RootStatus::Import => match token {
                JsonToken::Id(s) => JsonState::ParseRoot(RootState {
                    status: RootStatus::ImportId(s),
                    state: self.state,
                }),
                _ => JsonState::Error(ParseError::WrongImportStatement),
            },
            RootStatus::ImportId(id) => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "from" => JsonState::ParseRoot(RootState {
                        status: RootStatus::ImportIdFrom(id),
                        state: self.state,
                    }),
                    _ => JsonState::Error(ParseError::WrongImportStatement),
                },
                _ => JsonState::Error(ParseError::WrongImportStatement),
            },
            RootStatus::ImportIdFrom(id) => match token {
                JsonToken::String(s) => {
                    let read_result = context.io.read_to_string(s.as_str()); //todo: concatnate paths
                    match read_result {
                        Ok(s) => {
                            let tokens = tokenize(s);
                            let res = parse_with_tokens(context, tokens.into_iter());
                            match res {
                                Ok(r) => {
                                    self.state.consts.insert(id, r.any);
                                    JsonState::ParseRoot(RootState {
                                        status: RootStatus::Initial,
                                        state: self.state,
                                    })
                                }
                                Err(e) => JsonState::Error(e),
                            }
                        }
                        Err(_) => JsonState::Error(ParseError::CannotReadFile),
                    }
                }
                _ => JsonState::Error(ParseError::WrongImportStatement),
            },
        }
    }
}

impl<M: Manager> ConstState<M> {
    fn parse<I: Io>(self, context: &Context<M, I>, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::Semicolon => todo!(),
            _ => {
                let result = self.state.parse(context, token);
                match result {
                    AnyResult::Continue(state) => JsonState::ParseConst(ConstState {
                        key: self.key,
                        state,
                    }),
                    AnyResult::Success(mut success) => {
                        success.state.consts.insert(self.key, success.value);
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            state: success.state,
                        })
                    }
                    AnyResult::Error(error) => JsonState::Error(error),
                }
            }
        }
    }
}

impl<M: Manager> AnyState<M> {
    fn set_djs(self) -> Self {
        AnyState {
            data_type: self.data_type.to_djs(),
            status: self.status,
            current: self.current,
            stack: self.stack,
            consts: self.consts,
        }
    }

    fn set_data_type(self, data_type: DataType) -> Self {
        AnyState {
            data_type,
            status: self.status,
            current: self.current,
            stack: self.stack,
            consts: self.consts,
        }
    }

    fn parse_for_module<I: Io>(self, context: &Context<M, I>, token: JsonToken) -> JsonState<M> {
        let result = self.parse(context, token);
        match result {
            AnyResult::Continue(state) => JsonState::ParseModule(state),
            AnyResult::Success(success) => JsonState::Result(ParseResult {
                data_type: success.state.data_type,
                any: success.value,
            }),
            AnyResult::Error(error) => JsonState::Error(error),
        }
    }

    fn parse_import_begin(self, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::OpeningParenthesis => AnyResult::Continue(AnyState {
                data_type: self.data_type,
                status: ParsingStatus::ImportValue,
                current: self.current,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    fn parse_import_value<I: Io>(self, context: &Context<M, I>, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::String(s) => {
                let read_result = context.io.read_to_string(s.as_str()); //todo: concatnate paths
                match read_result {
                    Ok(s) => {
                        let tokens = tokenize(s);
                        let res = parse_with_tokens(context, tokens.into_iter());
                        match res {
                            Ok(r) => AnyResult::Continue(AnyState {
                                data_type: self.data_type,
                                status: ParsingStatus::ImportEnd,
                                current: JsonElement::Any(r.any),
                                stack: self.stack,
                                consts: self.consts,
                            }),
                            Err(e) => AnyResult::Error(e),
                        }
                    }
                    Err(_) => AnyResult::Error(ParseError::CannotReadFile),
                }
            }
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    fn parse_import_end(self, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ClosingParenthesis => self.end_import(),
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    fn parse<I: Io>(self, context: &Context<M, I>, token: JsonToken) -> AnyResult<M> {
        match self.status {
            ParsingStatus::Initial | ParsingStatus::ObjectColon => {
                self.parse_value(context.manager, token)
            }
            ParsingStatus::ArrayBegin => self.parse_array_begin(context.manager, token),
            ParsingStatus::ArrayValue => self.parse_array_value(context.manager, token),
            ParsingStatus::ArrayComma => self.parse_array_comma(context.manager, token),
            ParsingStatus::ObjectBegin => self.parse_object_begin(context.manager, token),
            ParsingStatus::ObjectKey => self.parse_object_key(token),
            ParsingStatus::ObjectValue => self.parse_object_next(context.manager, token),
            ParsingStatus::ObjectComma => self.parse_object_comma(context.manager, token),
            ParsingStatus::ImportBegin => self.parse_import_begin(token),
            ParsingStatus::ImportValue => self.parse_import_value(context, token),
            ParsingStatus::ImportEnd => self.parse_import_end(token),
        }
    }

    fn begin_import(mut self) -> AnyResult<M> {
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top);
        }
        AnyResult::Continue(AnyState {
            data_type: DataType::Cjs,
            status: ParsingStatus::ImportBegin,
            current: JsonElement::None,
            stack: self.stack,
            consts: self.consts,
        })
    }

    fn end_import(mut self) -> AnyResult<M> {
        match self.current {
            JsonElement::Any(any) => {
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState {
                    data_type: self.data_type,
                    status: ParsingStatus::Initial,
                    current,
                    stack: self.stack,
                    consts: self.consts,
                };
                new_state.push_value(any)
            }
            _ => unreachable!(),
        }
    }

    fn push_value(self, value: Any<M::Dealloc>) -> AnyResult<M> {
        match self.current {
            JsonElement::None => AnyResult::Success(AnySuccess {
                state: AnyState {
                    data_type: self.data_type,
                    status: ParsingStatus::Initial,
                    current: self.current,
                    stack: self.stack,
                    consts: self.consts,
                },
                value,
            }),
            JsonElement::Stack(top) => match top {
                JsonStackElement::Array(mut arr) => {
                    arr.push(value);
                    AnyResult::Continue(AnyState {
                        data_type: self.data_type,
                        status: ParsingStatus::ArrayValue,
                        current: JsonElement::Stack(JsonStackElement::Array(arr)),
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
                    AnyResult::Continue(AnyState {
                        data_type: self.data_type,
                        status: ParsingStatus::ObjectValue,
                        current: JsonElement::Stack(JsonStackElement::Object(new_stack_obj)),
                        stack: self.stack,
                        consts: self.consts,
                    })
                }
            },
            _ => todo!(),
        }
    }

    fn push_key(self, s: String) -> AnyResult<M> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Object(stack_obj)) => {
                let new_stack_obj = JsonStackObject {
                    map: stack_obj.map,
                    key: s,
                };
                AnyResult::Continue(AnyState {
                    data_type: self.data_type,
                    status: ParsingStatus::ObjectKey,
                    current: JsonElement::Stack(JsonStackElement::Object(new_stack_obj)),
                    stack: self.stack,
                    consts: self.consts,
                })
            }
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn begin_array(mut self) -> AnyResult<M> {
        let new_top = JsonStackElement::Array(Vec::default());
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top);
        }
        AnyResult::Continue(AnyState {
            data_type: self.data_type,
            status: ParsingStatus::ArrayBegin,
            current: JsonElement::Stack(new_top),
            stack: self.stack,
            consts: self.consts,
        })
    }

    fn end_array(mut self, manager: M) -> AnyResult<M> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Array(array)) => {
                let js_array = new_array(manager, array.into_iter()).to_ref();
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState {
                    data_type: self.data_type,
                    status: self.status,
                    current,
                    stack: self.stack,
                    consts: self.consts,
                };
                new_state.push_value(Any::move_from(js_array))
            }
            _ => unreachable!(),
        }
    }

    fn begin_object(mut self) -> AnyResult<M> {
        let new_top: JsonStackElement<<M as Manager>::Dealloc> =
            JsonStackElement::Object(JsonStackObject {
                map: BTreeMap::default(),
                key: String::default(),
            });
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top)
        }
        AnyResult::Continue(AnyState {
            data_type: self.data_type,
            status: ParsingStatus::ObjectBegin,
            current: JsonElement::Stack(new_top),
            stack: self.stack,
            consts: self.consts,
        })
    }

    fn end_object(mut self, manager: M) -> AnyResult<M> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Object(object)) => {
                let vec = object
                    .map
                    .into_iter()
                    .map(|kv| (to_js_string(manager, kv.0), kv.1))
                    .collect::<Vec<_>>();
                let js_object = new_object(manager, vec.into_iter()).to_ref();
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState {
                    data_type: self.data_type,
                    status: self.status,
                    current,
                    stack: self.stack,
                    consts: self.consts,
                };
                new_state.push_value(Any::move_from(js_object))
            }
            _ => unreachable!(),
        }
    }

    fn parse_value(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ArrayBegin => self.begin_array(),
            JsonToken::ObjectBegin => self.begin_object(),
            JsonToken::Id(s) if self.data_type.is_cjs_compatible() && s == "require" => {
                self.begin_import()
            }
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => AnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_comma(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ArrayBegin => self.begin_array(),
            JsonToken::ObjectBegin => self.begin_object(),
            JsonToken::Id(s) if self.data_type == DataType::Cjs && s == "require" => {
                self.begin_import()
            }
            JsonToken::ArrayEnd => self.end_array(manager),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => AnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_begin(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ArrayBegin => self.begin_array(),
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::ObjectBegin => self.begin_object(),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => AnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_value(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::Comma => AnyResult::Continue(AnyState {
                data_type: self.data_type,
                status: ParsingStatus::ArrayComma,
                current: self.current,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_begin(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::Id(s) if self.data_type.is_djs() => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_key(self, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::Colon => AnyResult::Continue(AnyState {
                data_type: self.data_type,
                status: ParsingStatus::ObjectColon,
                current: self.current,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_next(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ObjectEnd => self.end_object(manager),
            JsonToken::Comma => AnyResult::Continue(AnyState {
                data_type: self.data_type,
                status: ParsingStatus::ObjectComma,
                current: self.current,
                stack: self.stack,
                consts: self.consts,
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_comma(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }
}

impl<M: Manager> JsonState<M> {
    fn push<I: Io>(self, context: &Context<M, I>, token: JsonToken) -> JsonState<M> {
        if token == JsonToken::NewLine {
            return self;
        }
        match self {
            JsonState::ParseRoot(state) => state.parse(context, token),
            JsonState::Result(_) => JsonState::Error(ParseError::UnexpectedToken),
            JsonState::ParseModule(state) => state.parse_for_module(context, token),
            JsonState::ParseConst(state) => state.parse(context, token),
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

fn parse<M: Manager, I: Io>(context: &Context<M, I>) -> Result<ParseResult<M>, ParseError> {
    let read_result = context.io.read_to_string(context.path.as_str());
    match read_result {
        Ok(s) => {
            let tokens = tokenize(s);
            parse_with_tokens(context, tokens.into_iter())
        }
        Err(_) => Err(ParseError::CannotReadFile),
    }
}

fn parse_with_tokens<M: Manager, I: Io>(
    context: &Context<M, I>,
    iter: impl Iterator<Item = JsonToken>,
) -> Result<ParseResult<M>, ParseError> {
    let mut state: JsonState<M> = JsonState::ParseRoot(RootState {
        status: RootStatus::Initial,
        state: default(),
    });
    for token in iter {
        state = state.push(context, token);
    }
    state.end()
}

#[cfg(test)]
mod test {
    use io_test::VirtualIo;
    use io_trait::Io;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::default::default,
        js::{js_array::JsArrayRef, js_object::JsObjectRef, js_string::JsStringRef, type_::Type},
        mem::{global::GLOBAL, local::Local, manager::Manager},
        parser::{parse, DataType},
        tokenizer::{tokenize, ErrorType, JsonToken},
    };

    use super::{parse_with_tokens, Context, ParseError, ParseResult};

    fn create_test_context<M: Manager>(manager: M) -> Context<M, VirtualIo> {
        Context {
            manager,
            io: VirtualIo::new(&[]),
            path: default(),
        }
    }

    fn parse_with_virutal_io<M: Manager>(
        manager: M,
        iter: impl Iterator<Item = JsonToken>,
    ) -> Result<ParseResult<M>, ParseError> {
        parse_with_tokens(&create_test_context(manager), iter)
    }

    fn test_local() {
        let local = Local::default();
        let _ = parse_with_tokens(&create_test_context(&local), [].into_iter());
    }

    fn test_global() {
        let _ = {
            let global = GLOBAL;
            parse_with_tokens(&create_test_context(global), [].into_iter())
        };
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_json() {
        let json_str = include_str!("../../test/test-json.json");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse_with_virutal_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Json);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_djs() {
        let json_str = include_str!("../../test/test-djs.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse_with_virutal_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Cjs);

        let json_str = include_str!("../../test/test-djs.d.mjs");
        let tokens = tokenize(json_str.to_owned());
        let local = Local::default();
        let result = parse_with_virutal_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Mjs);
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(2.0));
        let item1 = items[1].clone();
        assert_eq!(item1.try_move(), Ok(2.0));

        let json_str = include_str!("../../test/test-const-error.d.cjs.txt");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_stack() {
        let local = Local::default();
        test_stack_with_manager(&local);
    }

    fn test_stack_with_manager<M: Manager>(manager: M) {
        let json_str = include_str!("../../test/test-stack.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        let result_unwrap = item0.try_move::<JsObjectRef<M::Dealloc>>().unwrap();
        let items = result_unwrap.items();
        let (key0, value0) = items[0].clone();
        let key0_items = key0.items();
        assert_eq!(key0_items, [0x61]);
        let result_unwrap = value0.try_move::<JsArrayRef<M::Dealloc>>().unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.get_type(), Type::Null);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_import() {
        let local = Local::default();
        test_import_with_manager(&local);
    }

    fn test_import_with_manager<M: Manager>(manager: M) {
        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_main.d.cjs");
        //let path = "../../test/test-import-main.d.cjs";
        let main_path = "test_import_main.d.cjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.cjs");
        let module_path = "test_import_module.d.cjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let context = Context {
            manager,
            io,
            path: String::from(main_path),
        };

        let result = parse(&context);
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(3.0));

        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_main.d.mjs");
        //let path = "../../test/test-import-main.d.mjs";
        let main_path = "test_import_main.d.mjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.mjs");
        let module_path = "test_import_module.d.mjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let context = Context {
            manager,
            io,
            path: String::from(main_path),
        };

        let result = parse(&context);
        assert!(result.is_ok());
        let result_unwrap = result
            .unwrap()
            .any
            .try_move::<JsArrayRef<M::Dealloc>>()
            .unwrap();
        let items = result_unwrap.items();
        let item0 = items[0].clone();
        assert_eq!(item0.try_move(), Ok(4.0));
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_import_error() {
        let local = Local::default();
        test_import_error_with_manager(&local);
    }

    fn test_import_error_with_manager<M: Manager>(manager: M) {
        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_error.d.cjs.txt");
        //let path = "../../test/test-import-main.d.cjs";
        let main_path = "test_import_error.d.cjs.txt";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.mjs");
        let module_path = "test_import_module.d.mjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let context = Context {
            manager,
            io,
            path: String::from(main_path),
        };

        let result = parse(&context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::UnexpectedToken);

        let io: VirtualIo = VirtualIo::new(&[]);

        let main = include_str!("../../test/test_import_error.d.mjs.txt");
        //let path = "../../test/test-import-main.d.cjs";
        let main_path = "test_import_error.d.mjs.txt";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.cjs");
        let module_path = "test_import_module.d.cjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let context = Context {
            manager,
            io,
            path: String::from(main_path),
        };

        let result = parse(&context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::UnexpectedToken);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_trailing_comma() {
        let local = Local::default();
        let json_str = include_str!("../../test/test-trailing-comma.d.cjs");
        let tokens = tokenize(json_str.to_owned());
        let result = parse_with_virutal_io(&local, tokens.into_iter());
        assert!(result.is_ok());
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
                let result = parse_with_virutal_io(&local, tokens.into_iter());
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
                let result = parse_with_virutal_io(&local, tokens.into_iter());
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
        let result = parse_with_virutal_io(&local, tokens.into_iter());
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
        let result = parse_with_virutal_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Mjs);

        let local = Local::default();
        let tokens = [
            JsonToken::Id(String::from("module")),
            JsonToken::Dot,
            JsonToken::Id(String::from("exports")),
            JsonToken::Equals,
            JsonToken::Id(String::from("null")),
        ];
        let result = parse_with_virutal_io(&local, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data_type, DataType::Cjs);
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
        let result = parse_with_virutal_io(&local, tokens.into_iter());
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
        let result = parse_with_virutal_io(&local, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.get_type(), Type::Null);

        let tokens = [JsonToken::Id(String::from("true"))];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(true));

        let tokens = [JsonToken::Id(String::from("false"))];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(false));

        let tokens = [JsonToken::Number(0.1)];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().any.try_move(), Ok(0.1));

        let tokens = [JsonToken::String(String::from("abc"))];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_ok());
        let result = result.unwrap().any.try_move::<JsStringRef<M::Dealloc>>();
        assert!(result.is_ok());
        let result = result.unwrap();
        let items = result.items();
        assert_eq!(items, [0x61, 0x62, 0x63]);

        let tokens = [JsonToken::ArrayBegin, JsonToken::ArrayEnd];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
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
            let result = parse_with_virutal_io(manager, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ErrorToken(ErrorType::InvalidNumber)];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::Comma, JsonToken::ArrayEnd];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(0.0),
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::ArrayEnd,
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::String(String::default())];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayBegin, JsonToken::Colon, JsonToken::ArrayEnd];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ArrayEnd];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Comma,
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Number(0.0),
            JsonToken::Comma,
            JsonToken::Number(1.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
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
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::ObjectEnd,
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::Comma,
            JsonToken::String(String::from("key")),
            JsonToken::Colon,
            JsonToken::Number(0.0),
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [JsonToken::ObjectEnd];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ArrayBegin,
            JsonToken::ObjectBegin,
            JsonToken::ArrayEnd,
            JsonToken::ObjectEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());

        let tokens = [
            JsonToken::ObjectBegin,
            JsonToken::ArrayBegin,
            JsonToken::ObjectEnd,
            JsonToken::ArrayEnd,
        ];
        let result = parse_with_virutal_io(manager, tokens.into_iter());
        assert!(result.is_err());
    }
}
