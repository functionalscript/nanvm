use super::{
    json_state::JsonState,
    path::{concat, split},
    shared::{
        to_js_string, DataType, JsonElement, JsonStackElement, JsonStackObject, ModuleCache,
        ParseError, ParseResult, ParsingStatus,
    },
};
use crate::{
    common::{cast::Cast, default::default},
    js::{any::Any, js_array::new_array, js_object::new_object},
    mem::manager::Manager,
    tokenizer::JsonToken,
};
use std::collections::BTreeMap;

pub struct AnySuccess<M: Manager> {
    pub state: AnyState<M>,
    pub value: Any<M::Dealloc>,
}

pub enum AnyResult<M: Manager> {
    Continue(AnyState<M>),
    Success(AnySuccess<M>),
    Error(ParseError),
}

pub struct AnyState<M: Manager> {
    pub data_type: DataType,
    pub status: ParsingStatus,
    pub current: JsonElement<M::Dealloc>,
    pub stack: Vec<JsonStackElement<M::Dealloc>>,
    pub consts: BTreeMap<String, Any<M::Dealloc>>,
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

// AnyState methods that use Dealloc only - methods that use Manager are in AnyStateExtension.
impl<M: Manager> AnyState<M> {
    pub fn set_djs(self) -> Self {
        AnyState {
            data_type: DataType::Djs,
            ..self
        }
    }

    pub fn set_mjs(self) -> Self {
        AnyState {
            data_type: DataType::Mjs,
            ..self
        }
    }

    pub fn set_cjs(self) -> Self {
        AnyState {
            data_type: DataType::Cjs,
            ..self
        }
    }

    pub fn parse(
        self,
        manager: M,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> (
        /*any_result:*/ AnyResult<M>,
        /*import_path*/ Option<String>,
    ) {
        match self.status {
            ParsingStatus::Initial | ParsingStatus::ObjectColon => {
                (self.parse_value(manager, token), None)
            }
            ParsingStatus::ArrayBegin => (self.parse_array_begin(manager, token), None),
            ParsingStatus::ArrayValue => (self.parse_array_value(manager, token), None),
            ParsingStatus::ArrayComma => (self.parse_array_comma(manager, token), None),
            ParsingStatus::ObjectBegin => (self.parse_object_begin(manager, token), None),
            ParsingStatus::ObjectKey => (self.parse_object_key(token), None),
            ParsingStatus::ObjectValue => (self.parse_object_next(manager, token), None),
            ParsingStatus::ObjectComma => (self.parse_object_comma(manager, token), None),
            ParsingStatus::ImportBegin => (self.parse_import_begin(token), None),
            ParsingStatus::ImportValue => {
                self.parse_import_value(token, module_cache, context_path)
            }
            ParsingStatus::ImportEnd => (self.parse_import_end(token), None),
        }
    }

    pub fn parse_for_module(
        self,
        manager: M,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> (
        /*json_state:*/ JsonState<M>,
        /*import_path:*/ Option<String>,
    ) {
        let (any_result, import_path) = self.parse(manager, token, module_cache, context_path);
        match import_path {
            Some(import_path) => {
                if let AnyResult::Continue(state) = any_result {
                    (JsonState::ParseModule(state), Some(import_path))
                } else {
                    panic!("Import path should be returned only with Continue result");
                }
            }
            None => match any_result {
                AnyResult::Continue(state) => (JsonState::ParseModule(state), None),
                AnyResult::Success(success) => (
                    JsonState::Result(ParseResult {
                        data_type: success.state.data_type,
                        any: success.value,
                    }),
                    None,
                ),
                AnyResult::Error(error) => (JsonState::Error(error), None),
            },
        }
    }

    pub fn parse_import_begin(self, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::OpeningParenthesis => AnyResult::Continue(AnyState {
                status: ParsingStatus::ImportValue,
                ..self
            }),
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    pub fn parse_import_end(self, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ClosingParenthesis => self.end_import(),
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    fn parse_import_value(
        self,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> (
        AnyResult<M>,   /*any_result*/
        Option<String>, /*import_path*/
    ) {
        match token {
            JsonToken::String(s) => {
                let import_path = concat(split(&context_path).0, s.as_str());
                if let Some(any) = module_cache.complete.get(&import_path) {
                    return (
                        AnyResult::Continue(AnyState {
                            status: ParsingStatus::ImportEnd,
                            current: JsonElement::Any(any.clone()),
                            ..self
                        }),
                        None,
                    );
                }
                if module_cache.progress.contains(&import_path) {
                    return (AnyResult::Error(ParseError::CircularDependency), None);
                }
                module_cache.progress.insert(import_path.clone());
                (
                    AnyResult::Continue(AnyState {
                        status: ParsingStatus::ImportEnd,
                        ..self
                    }),
                    Some(import_path),
                )
            }
            _ => (AnyResult::Error(ParseError::WrongRequireStatement), None),
        }
    }

    pub fn begin_import(mut self) -> AnyResult<M> {
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top);
        }
        AnyResult::Continue(AnyState {
            data_type: DataType::Cjs,
            status: ParsingStatus::ImportBegin,
            current: JsonElement::None,
            ..self
        })
    }

    pub fn end_import(mut self) -> AnyResult<M> {
        match self.current {
            JsonElement::Any(any) => {
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState {
                    status: ParsingStatus::Initial,
                    current,
                    ..self
                };
                new_state.push_value(any)
            }
            _ => unreachable!(),
        }
    }

    pub fn parse_value(self, manager: M, token: JsonToken) -> AnyResult<M> {
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

    pub fn push_value(self, value: Any<M::Dealloc>) -> AnyResult<M> {
        match self.current {
            JsonElement::None => AnyResult::Success(AnySuccess {
                state: AnyState {
                    status: ParsingStatus::Initial,
                    ..self
                },
                value,
            }),
            JsonElement::Stack(top) => match top {
                JsonStackElement::Array(mut arr) => {
                    arr.push(value);
                    AnyResult::Continue(AnyState {
                        status: ParsingStatus::ArrayValue,
                        current: JsonElement::Stack(JsonStackElement::Array(arr)),
                        ..self
                    })
                }
                JsonStackElement::Object(mut stack_obj) => {
                    stack_obj.map.insert(stack_obj.key, value);
                    let new_stack_obj = JsonStackObject {
                        map: stack_obj.map,
                        key: String::default(),
                    };
                    AnyResult::Continue(AnyState {
                        status: ParsingStatus::ObjectValue,
                        current: JsonElement::Stack(JsonStackElement::Object(new_stack_obj)),
                        ..self
                    })
                }
            },
            _ => todo!(),
        }
    }

    pub fn push_key(self, s: String) -> AnyResult<M> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Object(stack_obj)) => {
                let new_stack_obj = JsonStackObject {
                    map: stack_obj.map,
                    key: s,
                };
                AnyResult::Continue(AnyState {
                    status: ParsingStatus::ObjectKey,
                    current: JsonElement::Stack(JsonStackElement::Object(new_stack_obj)),
                    ..self
                })
            }
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn parse_array_comma(self, manager: M, token: JsonToken) -> AnyResult<M> {
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

    pub fn parse_array_begin(self, manager: M, token: JsonToken) -> AnyResult<M> {
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

    pub fn parse_array_value(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::Comma => AnyResult::Continue(AnyState {
                status: ParsingStatus::ArrayComma,
                ..self
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn begin_array(mut self) -> AnyResult<M> {
        let new_top = JsonStackElement::Array(Vec::default());
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top);
        }
        AnyResult::Continue(AnyState {
            status: ParsingStatus::ArrayBegin,
            current: JsonElement::Stack(new_top),
            ..self
        })
    }

    pub fn end_array(mut self, manager: M) -> AnyResult<M> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Array(array)) => {
                let js_array = new_array(manager, array).to_ref();
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState { current, ..self };
                new_state.push_value(Any::move_from(js_array))
            }
            _ => unreachable!(),
        }
    }

    pub fn parse_object_begin(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::Id(s) if self.data_type.is_djs() => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn parse_object_next(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::ObjectEnd => self.end_object(manager),
            JsonToken::Comma => AnyResult::Continue(AnyState {
                status: ParsingStatus::ObjectComma,
                ..self
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn parse_object_comma(self, manager: M, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn parse_object_key(self, token: JsonToken) -> AnyResult<M> {
        match token {
            JsonToken::Colon => AnyResult::Continue(AnyState {
                status: ParsingStatus::ObjectColon,
                ..self
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn begin_object(mut self) -> AnyResult<M> {
        let new_top: JsonStackElement<M::Dealloc> = JsonStackElement::Object(JsonStackObject {
            map: BTreeMap::default(),
            key: String::default(),
        });
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top)
        }
        AnyResult::Continue(AnyState {
            status: ParsingStatus::ObjectBegin,
            current: JsonElement::Stack(new_top),
            ..self
        })
    }

    pub fn end_object(mut self, manager: M) -> AnyResult<M> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Object(object)) => {
                let vec = object
                    .map
                    .into_iter()
                    .map(|kv| (to_js_string(manager, kv.0), kv.1))
                    .collect::<Vec<_>>();
                let js_object = new_object(manager, vec).to_ref();
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState { current, ..self };
                new_state.push_value(Any::move_from(js_object))
            }
            _ => unreachable!(),
        }
    }
}
