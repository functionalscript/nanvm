use super::{
    any_state::{AnyResult, AnyState},
    path::{concat, split},
};
use crate::{
    common::default::default,
    js::{
        any::Any,
        js_string::{new_string, JsStringRef},
        null::Null,
    },
    mem::manager::{Dealloc, Manager},
    tokenizer::JsonToken,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;

#[derive(Debug, Default, PartialEq)]
pub enum DataType {
    #[default]
    Json,
    Djs,
    Cjs,
    Mjs,
}

impl DataType {
    pub fn to_djs(&self) -> DataType {
        match self {
            DataType::Json | DataType::Djs => DataType::Djs,
            DataType::Cjs => DataType::Cjs,
            DataType::Mjs => DataType::Mjs,
        }
    }

    pub fn is_djs(&self) -> bool {
        matches!(self, DataType::Djs | DataType::Cjs | DataType::Mjs)
    }

    pub fn is_cjs_compatible(&self) -> bool {
        matches!(self, DataType::Json | DataType::Djs | DataType::Cjs)
    }

    pub fn is_mjs_compatible(&self) -> bool {
        matches!(self, DataType::Json | DataType::Djs | DataType::Mjs)
    }
}

#[derive(Default, Debug)]
pub enum ParsingStatus {
    #[default]
    Initial,
    ArrayBegin,
    ArrayValue,
    ArrayComma,
    ObjectBegin,
    ObjectKey,
    ObjectColon,
    ObjectValue,
    ObjectComma,
    ImportBegin,
    ImportValue,
    ImportEnd,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken,
    UnexpectedEnd,
    WrongExportStatement,
    WrongConstStatement,
    WrongRequireStatement,
    WrongImportStatement,
    CannotReadFile,
    CircularDependency,
    NewLineExpected,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ParseError::UnexpectedToken => "UnexpectedToken",
            ParseError::UnexpectedEnd => "UnexpectedEnd",
            ParseError::WrongExportStatement => "WrongExportStatement",
            ParseError::WrongConstStatement => "WrongConstStatement",
            ParseError::WrongRequireStatement => "WrongRequireStatement",
            ParseError::WrongImportStatement => "WrongImportStatement",
            ParseError::CannotReadFile => "CannotReadFile",
            ParseError::CircularDependency => "CircularDependency",
            ParseError::NewLineExpected => "NewLineExpected",
        })
    }
}

pub struct ModuleCache<D: Dealloc> {
    pub complete: BTreeMap<String, Any<D>>,
    pub progress: BTreeSet<String>,
}

impl<D: Dealloc> Default for ModuleCache<D> {
    fn default() -> Self {
        Self {
            complete: default(),
            progress: default(),
        }
    }
}

pub enum JsonStackElement<D: Dealloc> {
    Object(JsonStackObject<D>),
    Array(Vec<Any<D>>),
}

pub struct JsonStackObject<D: Dealloc> {
    pub map: BTreeMap<String, Any<D>>,
    pub key: String,
}

pub enum JsonElement<D: Dealloc> {
    None,
    Stack(JsonStackElement<D>),
    Any(Any<D>),
}

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

#[derive(Debug)]
pub struct ParseResult<D: Dealloc> {
    pub data_type: DataType,
    pub any: Any<D>,
}

pub struct RootState<M: Manager> {
    pub status: RootStatus,
    pub state: AnyState<M>,
    pub new_line: bool,
}

impl<M: Manager> RootState<M> {
    fn parse(
        self,
        manager: M,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> JsonStateWithImportPath<M> {
        match self.status {
            RootStatus::Initial => match token {
                JsonToken::NewLine => {
                    JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                        new_line: true,
                        ..self
                    }))
                }
                JsonToken::Id(s) => match self.new_line {
                    true => match s.as_ref() {
                        "const" => JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                            status: RootStatus::Const,
                            state: self.state.set_djs(),
                            new_line: false,
                        })),
                        "export" if self.state.data_type.is_mjs_compatible() => {
                            JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                                status: RootStatus::Export,
                                state: self.state.set_mjs(),
                                new_line: false,
                            }))
                        }
                        "module" if self.state.data_type.is_cjs_compatible() => {
                            JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                                status: RootStatus::Module,
                                state: self.state.set_cjs(),
                                new_line: false,
                            }))
                        }
                        "import" if self.state.data_type.is_mjs_compatible() => {
                            JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                                status: RootStatus::Import,
                                state: self.state.set_mjs(),
                                new_line: false,
                            }))
                        }
                        _ => {
                            // TODO: this case requires a thorough analysis and testing.
                            self.state.parse_for_module(
                                manager,
                                JsonToken::Id(s),
                                module_cache,
                                context_path,
                            )
                        }
                    },
                    false => JsonStateWithImportPath::new_error(ParseError::NewLineExpected),
                },
                _ => match self.new_line {
                    true => self
                        .state
                        .parse_for_module(manager, token, module_cache, context_path),
                    false => JsonStateWithImportPath::new_error(ParseError::NewLineExpected),
                },
            },
            RootStatus::Export => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "default" => JsonStateWithImportPath::new(JsonState::ParseModule(self.state)),
                    _ => JsonStateWithImportPath::new_error(ParseError::WrongExportStatement),
                },
                _ => JsonStateWithImportPath::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::Module => match token {
                JsonToken::Dot => JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                    status: RootStatus::ModuleDot,
                    state: self.state,
                    new_line: false,
                })),
                _ => JsonStateWithImportPath::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::ModuleDot => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "exports" => JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                        status: RootStatus::ModuleDotExports,
                        state: self.state,
                        new_line: false,
                    })),
                    _ => JsonStateWithImportPath::new_error(ParseError::WrongExportStatement),
                },
                _ => JsonStateWithImportPath::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::ModuleDotExports => match token {
                JsonToken::Equals => {
                    JsonStateWithImportPath::new(JsonState::ParseModule(self.state))
                }
                _ => JsonStateWithImportPath::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::Const => match token {
                JsonToken::Id(s) => JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                    status: RootStatus::ConstId(s),
                    state: self.state,
                    new_line: false,
                })),
                _ => JsonStateWithImportPath::new_error(ParseError::WrongConstStatement),
            },
            RootStatus::ConstId(s) => match token {
                JsonToken::Equals => {
                    JsonStateWithImportPath::new(JsonState::ParseConst(ConstState {
                        key: s,
                        state: self.state,
                    }))
                }
                _ => JsonStateWithImportPath::new_error(ParseError::WrongConstStatement),
            },
            RootStatus::Import => match token {
                JsonToken::Id(s) => JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                    status: RootStatus::ImportId(s),
                    state: self.state,
                    new_line: false,
                })),
                _ => JsonStateWithImportPath::new_error(ParseError::WrongImportStatement),
            },
            RootStatus::ImportId(id) => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "from" => JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                        status: RootStatus::ImportIdFrom(id),
                        state: self.state,
                        new_line: false,
                    })),
                    _ => JsonStateWithImportPath::new_error(ParseError::WrongImportStatement),
                },
                _ => JsonStateWithImportPath::new_error(ParseError::WrongImportStatement),
            },
            RootStatus::ImportIdFrom(id) => match token {
                JsonToken::String(s) => {
                    let import_path = concat(split(&context_path).0, s.as_str());
                    if let Some(any) = module_cache.complete.get(&import_path) {
                        let mut state = self.state;
                        state.consts.insert(id, any.clone());
                        return JsonStateWithImportPath::new(JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            state,
                            new_line: false,
                        }));
                    }
                    if module_cache.progress.contains(&import_path) {
                        return JsonStateWithImportPath::new_error(ParseError::CircularDependency);
                    }
                    module_cache.progress.insert(import_path.clone());
                    JsonStateWithImportPath::new_with_import_path(
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            new_line: false,
                            ..self
                        }),
                        import_path,
                    )
                }
                _ => JsonStateWithImportPath::new_error(ParseError::WrongImportStatement),
            },
        }
    }
}

pub struct ConstState<M: Manager> {
    pub key: String,
    pub state: AnyState<M>,
}

impl<M: Manager> ConstState<M> {
    fn parse(
        self,
        manager: M,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> JsonState<M> {
        match token {
            JsonToken::Semicolon => todo!(),
            _ => {
                // TODO: use import_path in place of _ below to track possible errors - or provide an explanation on why it's not necessary.
                let (any_result, _) = self.state.parse(manager, token, module_cache, context_path);
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
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> JsonStateWithImportPath<M> {
        if token == JsonToken::NewLine {
            return match self {
                JsonState::ParseRoot(state) => {
                    state.parse(manager, token, module_cache, context_path)
                }
                _ => JsonStateWithImportPath::new(self),
            };
        }
        match self {
            JsonState::ParseRoot(state) => state.parse(manager, token, module_cache, context_path),
            JsonState::Result(_) => JsonStateWithImportPath::new_error(ParseError::UnexpectedToken),
            JsonState::ParseModule(state) => {
                state.parse_for_module(manager, token, module_cache, context_path)
            }
            JsonState::ParseConst(state) => JsonStateWithImportPath::new(state.parse(
                manager,
                token,
                module_cache,
                context_path,
            )),
            _ => JsonStateWithImportPath::new(self),
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

pub struct JsonStateWithImportPath<M: Manager> {
    pub json_state: JsonState<M>,
    pub import_path: Option<String>, // an extra output in case of "import from" statement
}

impl<M: Manager> JsonStateWithImportPath<M> {
    pub fn new(json_state: JsonState<M>) -> Self {
        JsonStateWithImportPath {
            json_state,
            import_path: None,
        }
    }

    pub fn new_error(error: ParseError) -> Self {
        Self {
            json_state: JsonState::Error(error),
            import_path: None,
        }
    }

    pub fn new_with_import_path(json_state: JsonState<M>, import_path: String) -> Self {
        Self {
            json_state,
            import_path: Some(import_path),
        }
    }
}

pub fn to_js_string<M: Manager>(manager: M, s: String) -> JsStringRef<M::Dealloc> {
    new_string(manager, s.encode_utf16().collect::<Vec<_>>()).to_ref()
}

fn try_id_to_any<M: Manager>(
    s: &str,
    _manager: M,
    consts: &BTreeMap<String, Any<M::Dealloc>>,
) -> Option<Any<M::Dealloc>> {
    match s {
        "null" => Some(Any::move_from(Null())),
        "true" => Some(Any::move_from(true)),
        "false" => Some(Any::move_from(false)),
        s if consts.contains_key(s) => Some(consts.get(s).unwrap().clone()),
        _ => None,
    }
}

impl JsonToken {
    pub fn try_to_any<M: Manager>(
        self,
        manager: M,
        consts: &BTreeMap<String, Any<M::Dealloc>>,
    ) -> Option<Any<M::Dealloc>> {
        match self {
            JsonToken::Number(f) => Some(Any::move_from(f)),
            JsonToken::String(s) => Some(Any::move_from(to_js_string(manager, s))),
            JsonToken::Id(s) => try_id_to_any(&s, manager, consts),
            _ => None,
        }
    }
}
