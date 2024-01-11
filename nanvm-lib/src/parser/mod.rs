use std::collections::HashMap;

use crate::{
    common::cast::Cast,
    js::any::Any,
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

impl<M: Manager> JsonState<M> {
    fn push(&mut self, token: JsonToken) {
        match self {
            JsonState::Result(result) => *self = JsonState::Error(ParseError::UnexpectedToken),
            JsonState::Parse(state_parse) => match state_parse {
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
