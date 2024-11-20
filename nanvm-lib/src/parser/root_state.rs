use super::{
    any_state::AnyState, const_state::ConstState, json_state::JsonState, shared::ParseError,
};
use crate::{mem::manager::Manager, tokenizer::JsonToken};

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
    pub new_line: bool,
}

impl<M: Manager> RootState<M> {
    pub fn parse(
        self,
        manager: M,
        token: JsonToken,
    ) -> (
        /*json_state:*/ JsonState<M>,
        /*import:*/ Option<(/*id:*/ String, /*module:*/ String)>,
    ) {
        match self.status {
            RootStatus::Initial => match token {
                JsonToken::NewLine => (
                    JsonState::ParseRoot(RootState {
                        new_line: true,
                        ..self
                    }),
                    None,
                ),
                JsonToken::Id(s) => match self.new_line {
                    true => match s.as_ref() {
                        "const" => (
                            JsonState::ParseRoot(RootState {
                                status: RootStatus::Const,
                                state: self.state.set_djs(),
                                new_line: false,
                            }),
                            None,
                        ),
                        "export" if self.state.data_type.is_mjs_compatible() => (
                            JsonState::ParseRoot(RootState {
                                status: RootStatus::Export,
                                state: self.state.set_mjs(),
                                new_line: false,
                            }),
                            None,
                        ),
                        "module" if self.state.data_type.is_cjs_compatible() => (
                            JsonState::ParseRoot(RootState {
                                status: RootStatus::Module,
                                state: self.state.set_cjs(),
                                new_line: false,
                            }),
                            None,
                        ),
                        "import" if self.state.data_type.is_mjs_compatible() => (
                            JsonState::ParseRoot(RootState {
                                status: RootStatus::Import,
                                state: self.state.set_mjs(),
                                new_line: false,
                            }),
                            None,
                        ),
                        _ => {
                            let (json_state, _module_name) =
                                self.state.parse_for_module(manager, JsonToken::Id(s));
                            // TODO: figure out id and use _module_name, passing Some in place of None below.
                            (json_state, None)
                        }
                    },
                    false => (JsonState::Error(ParseError::NewLineExpected), None),
                },
                _ => match self.new_line {
                    true => {
                        let (json_state, _module_name) =
                            self.state.parse_for_module(manager, token);
                        // TODO: figure out id and use _module_name, passing Some in place of None below.
                        (json_state, None)
                    }
                    false => (JsonState::Error(ParseError::NewLineExpected), None),
                },
            },
            RootStatus::Export => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "default" => (JsonState::ParseModule(self.state), None),
                    _ => (JsonState::Error(ParseError::WrongExportStatement), None),
                },
                _ => (JsonState::Error(ParseError::WrongExportStatement), None),
            },
            RootStatus::Module => match token {
                JsonToken::Dot => (
                    JsonState::ParseRoot(RootState {
                        status: RootStatus::ModuleDot,
                        state: self.state,
                        new_line: false,
                    }),
                    None,
                ),
                _ => (JsonState::Error(ParseError::WrongExportStatement), None),
            },
            RootStatus::ModuleDot => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "exports" => (
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::ModuleDotExports,
                            state: self.state,
                            new_line: false,
                        }),
                        None,
                    ),
                    _ => (JsonState::Error(ParseError::WrongExportStatement), None),
                },
                _ => (JsonState::Error(ParseError::WrongExportStatement), None),
            },
            RootStatus::ModuleDotExports => match token {
                JsonToken::Equals => (JsonState::ParseModule(self.state), None),
                _ => (JsonState::Error(ParseError::WrongExportStatement), None),
            },
            RootStatus::Const => match token {
                JsonToken::Id(s) => (
                    JsonState::ParseRoot(RootState {
                        status: RootStatus::ConstId(s),
                        state: self.state,
                        new_line: false,
                    }),
                    None,
                ),
                _ => (JsonState::Error(ParseError::WrongConstStatement), None),
            },
            RootStatus::ConstId(s) => match token {
                JsonToken::Equals => (
                    JsonState::ParseConst(ConstState {
                        key: s,
                        state: self.state,
                    }),
                    None,
                ),
                _ => (JsonState::Error(ParseError::WrongConstStatement), None),
            },
            RootStatus::Import => match token {
                JsonToken::Id(s) => (
                    JsonState::ParseRoot(RootState {
                        status: RootStatus::ImportId(s),
                        state: self.state,
                        new_line: false,
                    }),
                    None,
                ),
                _ => (JsonState::Error(ParseError::WrongImportStatement), None),
            },
            RootStatus::ImportId(id) => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "from" => (
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::ImportIdFrom(id),
                            state: self.state,
                            new_line: false,
                        }),
                        None,
                    ),
                    _ => (JsonState::Error(ParseError::WrongImportStatement), None),
                },
                _ => (JsonState::Error(ParseError::WrongImportStatement), None),
            },
            RootStatus::ImportIdFrom(id) => match token {
                JsonToken::String(module) => (
                    JsonState::ParseRoot(RootState {
                        status: RootStatus::Initial,
                        new_line: false,
                        ..self
                    }),
                    Some((id, module)),
                ),
                _ => (JsonState::Error(ParseError::WrongImportStatement), None),
            },
        }
    }
}
