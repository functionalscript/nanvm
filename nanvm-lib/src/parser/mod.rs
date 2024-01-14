use std::collections::HashMap;

use crate::{
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
    pub map: HashMap<String, Any<D>>,
    pub key: String,
}

pub enum ParseStatus {
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

pub struct ParseState<M: Manager> {
    pub status: ParseStatus,
    pub top: Option<JsonStackElement<M::Dealloc>>,
    pub stack: Vec<JsonStackElement<M::Dealloc>>,
}

pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
}

pub enum JsonState<M: Manager> {
    Parse(ParseState<M>),
    Result(Any<M::Dealloc>),
    Error(ParseError),
}

fn to_js_string<M: Manager>(manager: M, s: String) -> JsStringRef<M::Dealloc> {
    new_string(manager, s.encode_utf16().collect::<Vec<_>>().into_iter()).to_ref()
}

impl JsonToken {
    fn is_value_token(&self) -> bool {
        match self {
            JsonToken::Null
            | JsonToken::False
            | JsonToken::True
            | JsonToken::Number(_)
            | JsonToken::String(_) => true,
            _ => false,
        }
    }

    fn to_any<M: Manager>(self, manager: M) -> Any<M::Dealloc> {
        match self {
            JsonToken::Null => Any::move_from(Null()),
            JsonToken::False => Any::move_from(false),
            JsonToken::True => Any::move_from(true),
            JsonToken::Number(f) => Any::move_from(f),
            JsonToken::String(s) => Any::move_from(to_js_string(manager, s)),
            _ => panic!(),
        }
    }
}

impl<M: Manager> ParseState<M> {
    fn push_value(self, value: Any<M::Dealloc>) -> JsonState<M> {
        match self.top {
            None => JsonState::Result(value),
            Some(top) => match top {
                JsonStackElement::Array(mut arr) => {
                    arr.push(value);
                    JsonState::Parse(ParseState {
                        status: ParseStatus::ArrayValue,
                        top: Option::Some(JsonStackElement::Array(arr)),
                        stack: self.stack,
                    })
                }
                JsonStackElement::Object(mut stack_obj) => {
                    stack_obj.map.insert(stack_obj.key, value);
                    let new_stack_obj = JsonStackObject {
                        map: stack_obj.map,
                        key: String::default(),
                    };
                    JsonState::Parse(ParseState {
                        status: ParseStatus::ObjectValue,
                        top: Option::Some(JsonStackElement::Object(new_stack_obj)),
                        stack: self.stack,
                    })
                }
            },
        }
    }

    fn push_key(self, s: String) -> JsonState<M> {
        match self.top {
            Some(JsonStackElement::Object(mut stack_obj)) => {
                let new_stack_obj = JsonStackObject {
                    map: stack_obj.map,
                    key: s,
                };
                JsonState::Parse(ParseState {
                    status: ParseStatus::ObjectKey,
                    top: Option::Some(JsonStackElement::Object(new_stack_obj)),
                    stack: self.stack,
                })
            }
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn start_array(mut self) -> JsonState<M> {
        let new_top = JsonStackElement::Array(Vec::default());
        match self.top {
            Some(top) => {
                self.stack.push(top);
            }
            None => {}
        }
        JsonState::Parse(ParseState {
            status: ParseStatus::ArrayStart,
            top: Some(new_top),
            stack: self.stack,
        })
    }

    fn end_array(mut self, manager: M) -> JsonState<M> {
        match self.top {
            Some(top) => match top {
                JsonStackElement::Array(array) => {
                    let js_array = new_array(manager, array.into_iter()).to_ref();
                    let new_state = ParseState {
                        status: ParseStatus::ArrayStart,
                        top: self.stack.pop(),
                        stack: self.stack,
                    };
                    return new_state.push_value(Any::move_from(js_array));
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn start_object(mut self) -> JsonState<M> {
        let new_top: JsonStackElement<<M as Manager>::Dealloc> =
            JsonStackElement::Object(JsonStackObject {
                map: HashMap::default(),
                key: String::default(),
            });
        match self.top {
            Some(top) => {
                self.stack.push(top);
            }
            None => {}
        }
        JsonState::Parse(ParseState {
            status: ParseStatus::ObjectStart,
            top: Some(new_top),
            stack: self.stack,
        })
    }

    fn end_object(mut self, manager: M) -> JsonState<M> {
        match self.top {
            Some(top) => match top {
                JsonStackElement::Object(object) => {
                    let vec = object
                        .map
                        .into_iter()
                        .map(|kv| (to_js_string(manager, kv.0), kv.1))
                        .collect::<Vec<_>>();
                    let js_object = new_object(manager, vec.into_iter()).to_ref();
                    let new_state = ParseState {
                        status: ParseStatus::ArrayStart,
                        top: self.stack.pop(),
                        stack: self.stack,
                    };
                    return new_state.push_value(Any::move_from(js_object));
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    fn parse_value(self, manager: M, token: JsonToken) -> JsonState<M> {
        if token.is_value_token() {
            let any = token.to_any(manager);
            return self.push_value(any);
        }
        match token {
            JsonToken::ArrayBegin => self.start_array(),
            JsonToken::ObjectBegin => self.start_object(),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_array_start(self, manager: M, token: JsonToken) -> JsonState<M> {
        if token.is_value_token() {
            let any = token.to_any(manager);
            return self.push_value(any);
        }
        match token {
            JsonToken::ArrayBegin => self.start_array(),
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::ObjectBegin => self.start_object(),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_array_value(self, manager: M, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::Comma => JsonState::Parse(ParseState {
                status: ParseStatus::ArrayComma,
                top: self.top,
                stack: self.stack,
            }),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_start(self, manager: M, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_key(self, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::Colon => JsonState::Parse(ParseState {
                status: ParseStatus::ObjectColon,
                top: self.top,
                stack: self.stack,
            }),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_next(self, manager: M, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::ObjectEnd => self.end_object(manager),
            JsonToken::Comma => JsonState::Parse(ParseState {
                status: ParseStatus::ObjectComma,
                top: self.top,
                stack: self.stack,
            }),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_comma(self, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }
}

impl<M: Manager> JsonState<M> {
    fn push(self, manager: M, token: JsonToken) -> JsonState<M> {
        match self {
            JsonState::Result(_) => JsonState::Error(ParseError::UnexpectedToken),
            JsonState::Parse(parse_state) => match parse_state.status {
                ParseStatus::Initial | ParseStatus::ArrayComma | ParseStatus::ObjectColon => {
                    parse_state.parse_value(manager, token)
                }
                ParseStatus::ArrayStart => parse_state.parse_array_start(manager, token),
                ParseStatus::ArrayValue => parse_state.parse_array_value(manager, token),
                ParseStatus::ObjectStart => parse_state.parse_object_start(manager, token),
                ParseStatus::ObjectKey => parse_state.parse_object_key(token),
                ParseStatus::ObjectValue => parse_state.parse_object_next(manager, token),
                ParseStatus::ObjectComma => parse_state.parse_object_comma(token),
            },
            _ => self,
        }
    }

    fn end(self) -> Result<Any<M::Dealloc>, ParseError> {
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
) -> Result<Any<M::Dealloc>, ParseError> {
    let mut state: JsonState<M> = JsonState::Parse(ParseState {
        status: ParseStatus::Initial,
        top: None,
        stack: Vec::from([]),
    });
    for token in iter {
        state = state.push(manager, token);
    }
    state.end()
}

#[cfg(test)]
mod test {
    use crate::mem::{global::GLOBAL, local::Local};

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
}
