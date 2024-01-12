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

pub struct StateParser<M: Manager> {
    pub status: ParseStatus,
    pub top: Option<JsonStackElement<M::Dealloc>>,
    pub stack: Vec<JsonStackElement<M::Dealloc>>,
}

pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
}

pub enum JsonState<M: Manager> {
    Parse(StateParser<M>),
    Result(Any<M::Dealloc>),
    Error(ParseError),
}

fn is_value_token(token: JsonToken) -> bool {
    match token {
        JsonToken::Null
        | JsonToken::False
        | JsonToken::True
        | JsonToken::Number(_)
        | JsonToken::String(_) => true,
        _ => false,
    }
}
fn token_to_value<M: Manager>(manager: M, token: JsonToken) -> Any<M::Dealloc> {
    match token {
        JsonToken::Null => Any::move_from(Null()),
        JsonToken::False => Any::move_from(false),
        JsonToken::True => Any::move_from(true),
        JsonToken::Number(f) => Any::move_from(f),
        JsonToken::String(s) => Any::move_from(new_string(manager, [].into_iter()).to_ref()),
        _ => panic!(),
    }
}

impl<M: Manager> StateParser<M> {
    fn push_value(self, value: Any<M::Dealloc>) -> JsonState<M> {
        match self.top {
            None => JsonState::Result(value),
            Some(top) => match top {
                JsonStackElement::Array(mut arr) => {
                    arr.push(value);
                    JsonState::Parse(StateParser {
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
                    JsonState::Parse(StateParser {
                        status: ParseStatus::ObjectValue,
                        top: Option::Some(JsonStackElement::Object(new_stack_obj)),
                        stack: self.stack,
                    })
                }
            },
        }
    }

    fn parse_value(self, token: JsonToken) -> JsonState<M> {
        if is_value_token(token) {}
        todo!()
    }
}

impl<M: Manager> JsonState<M> {
    fn push(&mut self, manager: M, token: JsonToken) {
        match self {
            JsonState::Result(result) => *self = JsonState::Error(ParseError::UnexpectedToken),
            JsonState::Parse(state_parse) => match state_parse.status {
                ParseStatus::Initial | ParseStatus::ObjectComma => {}
                _ => todo!(),
            },
            _ => {}
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
    let mut state: JsonState<M> = JsonState::Parse(StateParser {
        status: ParseStatus::Initial,
        top: None,
        stack: [].cast(),
    });
    for token in iter {
        state.push(manager, token);
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
        let _ = parse(GLOBAL, [].into_iter());
    }
}
