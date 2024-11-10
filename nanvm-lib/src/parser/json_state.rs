use super::{
    any_state::AnyState,
    const_state::ConstState,
    root_state::RootState,
    shared::{ModuleCache, ParseError, ParseResult},
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
        token: JsonToken<M::Dealloc>,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> (
        /*json_state:*/ JsonState<M>,
        /*import_path:*/ Option<String>,
    ) {
        if let JsonToken::NewLine = token {
            return match self {
                JsonState::ParseRoot(state) => {
                    state.parse(manager, token, module_cache, context_path)
                }
                _ => (self, None),
            };
        }
        match self {
            JsonState::ParseRoot(state) => state.parse(manager, token, module_cache, context_path),
            JsonState::ParseConst(state) => (
                state.parse(manager, token, module_cache, context_path),
                None,
            ),
            JsonState::Result(_) => (JsonState::Error(ParseError::UnexpectedToken), None),
            JsonState::ParseModule(state) => {
                state.parse_for_module(manager, token, module_cache, context_path)
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
