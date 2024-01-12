use std::collections::HashMap;

use crate::{
    common::cast::Cast,
    js::{any::Any, js_string::new_string, null::Null},
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
            JsonToken::String(s) => Any::move_from(
                new_string(manager, s.encode_utf16().collect::<Vec<_>>().into_iter()).to_ref(),
            ),
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

    fn push_key(&self, s: String) -> JsonState<M> {
        todo!()
    }

    fn start_array(&self) -> JsonState<M> {
        todo!()
    }

    fn end_array(&self) -> JsonState<M> {
        todo!()
    }

    fn start_object(&self) -> JsonState<M> {
        todo!()
    }

    fn end_object(&self) -> JsonState<M> {
        todo!()
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
            JsonToken::ArrayEnd => self.end_array(),
            JsonToken::ObjectBegin => self.start_object(),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_array_value(self, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::ArrayEnd => self.end_array(),
            JsonToken::Comma => JsonState::Parse(ParseState {
                status: ParseStatus::ArrayComma,
                top: self.top,
                stack: self.stack,
            }),
            _ => JsonState::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_start(self, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(),
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

    fn parse_object_next(self, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::ObjectEnd => self.end_object(),
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
                ParseStatus::ArrayValue => parse_state.parse_array_value(token),
                ParseStatus::ObjectStart => parse_state.parse_object_start(token),
                ParseStatus::ObjectKey => parse_state.parse_object_key(token),
                ParseStatus::ObjectValue => parse_state.parse_object_next(token),
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

fn parse<'a, M>(
    manager: &'a M,
    iter: impl Iterator<Item = JsonToken>,
) -> Result<Any<<&'a M as Manager>::Dealloc>, ParseError>
where
    &'a M: Manager,
{
    let mut state: JsonState<&'a M> = JsonState::Parse(ParseState {
        status: ParseStatus::Initial,
        top: None,
        stack: [].cast(),
    });
    for token in iter {
        state = state.push(manager, token);
    }
    state.end()
}
