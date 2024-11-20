use super::{
    any_state::AnyState,
    const_state::ConstState,
    root_state::RootState,
    shared::{ParseError, ParseResult},
};
use crate::{mem::manager::Manager, tokenizer::JsonToken};

pub enum JsonState<M: Manager> {
    ParseRoot(RootState<M>),
    ParseConst(ConstState<M>),
    ParseModule(AnyState<M>),
    Result(ParseResult<M::Dealloc>),
    Error(ParseError),
}

impl<M: Manager> JsonState<M> {
    pub fn push(
        self,
        manager: M,
        token: JsonToken,
    ) -> (
        /*json_state:*/ JsonState<M>,
        /*import:*/ Option<(/*id:*/ String, /*module:*/ String)>,
    ) {
        if token == JsonToken::NewLine {
            return match self {
                JsonState::ParseRoot(state) => state.parse(manager, token),
                _ => (self, None),
            };
        }
        match self {
            JsonState::ParseRoot(state) => state.parse(manager, token),
            JsonState::ParseConst(state) => (state.parse(manager, token), None),
            JsonState::Result(_) => (JsonState::Error(ParseError::UnexpectedToken), None),
            JsonState::ParseModule(state) => {
                let (json_state, _module_name) = state.parse_for_module(manager, token);
                // TODO: figure out id and use _module_name to return Some in place of None below.
                (json_state, None)
            }
            _ => (self, None),
        }
    }

    pub fn end(self) -> Result<ParseResult<M::Dealloc>, ParseError> {
        match self {
            JsonState::Result(result) => Ok(result),
            JsonState::Error(error) => Err(error),
            _ => Err(ParseError::UnexpectedEnd),
        }
    }
}
