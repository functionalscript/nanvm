use std::collections::HashMap;

use crate::{
    common::cast::Cast,
    js::any::Any,
    mem::manager::{Dealloc, Manager},
    tokenizer::JsonToken,
    tokenizer::JsonToken::Null,
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

pub struct StateParse<D: Dealloc> {
    pub status: ParseStatus,
    pub top: Option<JsonStackElement<D>>,
    pub stack: Vec<JsonStackElement<D>>,
}

pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
}

pub enum JsonState<M: Manager> {
    Parse(StateParse<M::Dealloc>),
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
fn token_to_value<M: Manager>(token: JsonToken) -> Any<M::Dealloc> {
    match token {
        //JsonToken::Null => Null {},
        //JsonToken::False => false,
        //JsonToken::True => true,
        //JsonToken::Number(f) => f,
        //JsonToken::String(s) => Js_String {},
        _ => todo!(),
    }
}

// impl<M: Manager> StateParse<M::Dealloc> {
// }

fn push_value<M: Manager>(
    mut state_parse: StateParse<M::Dealloc>,
    value: Any<M::Dealloc>,
) -> JsonState<M> {
    match state_parse.top {
        None => JsonState::Result(value),
        Some(top) => match top {
            JsonStackElement::Array(mut arr) => {
                arr.push(value);
                JsonState::Parse(StateParse {
                    status: ParseStatus::ArrayValue,
                    top: Option::Some(JsonStackElement::Array(arr)),
                    stack: state_parse.stack,
                })
            },
            JsonStackElement::Object(mut stack_obj) => {
                stack_obj.map.insert(stack_obj.key, value);
                JsonState::Parse(StateParse {
                    status: ParseStatus::ObjectValue,
                    top: Option::Some(JsonStackElement::Object(stack_obj)),
                    stack: state_parse.stack,
                })
            }
        },
    }
}

fn parse_value<M: Manager>(state_parse: StateParse<M::Dealloc>, token: JsonToken) -> JsonState<M> {
    if is_value_token(token) {}
    todo!()
}

impl<M: Manager> JsonState<M> {
    fn push(&mut self, token: JsonToken) {
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

fn parse<M: Manager>(iter: impl Iterator<Item = JsonToken>) -> Result<Any<M::Dealloc>, ParseError> {
    let mut state: JsonState<M> = JsonState::Parse(StateParse {
        status: ParseStatus::Initial,
        top: None,
        stack: [].cast(),
    });
    for token in iter {
        state.push(token);
    }
    state.end()
}
