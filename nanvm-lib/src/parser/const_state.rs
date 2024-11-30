use super::{
    any_state::{AnyResult, AnyState},
    json_state::JsonState,
    root_state::{RootState, RootStatus},
};
use crate::{mem::manager::Manager, tokenizer::JsonToken};

pub struct ConstState<M: Manager> {
    pub key: String,
    pub state: AnyState<M>,
}

impl<M: Manager> ConstState<M> {
    pub fn parse(self, manager: M, token: JsonToken) -> JsonState<M> {
        match token {
            JsonToken::Semicolon => todo!(),
            _ => {
                // TODO: use import_path in place of _ below to track possible errors -
                // or provide an explanation on why it's not necessary.
                let (any_result, _) = self.state.parse(manager, token);
                match any_result {
                    AnyResult::Continue(state) => JsonState::ParseConst(ConstState {
                        key: self.key,
                        state,
                    }),
                    AnyResult::Success(mut success) => {
                        success.state.consts.insert(self.key, success.value);
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            state: success.state,
                            new_line: false,
                        })
                    }
                    AnyResult::Error(error) => JsonState::Error(error),
                }
            }
        }
    }
}
