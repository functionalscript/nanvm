use super::path::{concat, split};
use crate::{
    common::{cast::Cast, default::default},
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

pub struct AnySuccess<D: Dealloc> {
    pub state: AnyState<D>,
    pub value: Any<D>,
}

pub enum AnyResult<D: Dealloc> {
    Continue(AnyState<D>),
    Success(AnySuccess<D>),
    Error(ParseError),
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

pub struct RootStateParseResult<D: Dealloc> {
    pub json_state: JsonState<D>,
    pub import_from_path: Option<String>, // an extra output in case of "import from" statement
}

impl<D: Dealloc> RootStateParseResult<D> {
    pub fn new(json_state: JsonState<D>) -> RootStateParseResult<D> {
        RootStateParseResult {
            json_state,
            import_from_path: None,
        }
    }

    pub fn new_error(error: ParseError) -> RootStateParseResult<D> {
        RootStateParseResult {
            json_state: JsonState::Error(error),
            import_from_path: None,
        }
    }

    pub fn new_with_import_from(
        json_state: JsonState<D>,
        import_from_path: String,
    ) -> RootStateParseResult<D> {
        RootStateParseResult {
            json_state,
            import_from_path: Some(import_from_path),
        }
    }
}

pub struct RootState<D: Dealloc> {
    pub status: RootStatus,
    pub state: AnyState<D>,
    pub new_line: bool,
}

/// AnyStateExtension contains RootState methods that use Manager (not only Dealloc).
pub trait RootStateExtension<M: Manager> {
    fn parse(
        self,
        manager: M,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> RootStateParseResult<M::Dealloc>;
}

impl<M: Manager> RootStateExtension<M> for RootState<M::Dealloc> {
    fn parse(
        self,
        manager: M,
        token: JsonToken,
        module_cache: &mut ModuleCache<M::Dealloc>,
        context_path: String,
    ) -> RootStateParseResult<M::Dealloc> {
        match self.status {
            RootStatus::Initial => match token {
                JsonToken::NewLine => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                    new_line: true,
                    ..self
                })),
                JsonToken::Id(s) => match self.new_line {
                    true => match s.as_ref() {
                        "const" => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                            status: RootStatus::Const,
                            state: self.state.set_djs(),
                            new_line: false,
                        })),
                        "export" if self.state.data_type.is_mjs_compatible() => {
                            RootStateParseResult::new(JsonState::ParseRoot(RootState {
                                status: RootStatus::Export,
                                state: self.state.set_mjs(),
                                new_line: false,
                            }))
                        }
                        "module" if self.state.data_type.is_cjs_compatible() => {
                            RootStateParseResult::new(JsonState::ParseRoot(RootState {
                                status: RootStatus::Module,
                                state: self.state.set_cjs(),
                                new_line: false,
                            }))
                        }
                        "import" if self.state.data_type.is_mjs_compatible() => {
                            RootStateParseResult::new(JsonState::ParseRoot(RootState {
                                status: RootStatus::Import,
                                state: self.state.set_mjs(),
                                new_line: false,
                            }))
                        }
                        _ => RootStateParseResult::new(self.state.parse_for_module(
                            manager,
                            JsonToken::Id(s),
                            &mut module_cache.progress,
                        )),
                    },
                    false => RootStateParseResult::new_error(ParseError::NewLineExpected),
                },
                _ => match self.new_line {
                    true => RootStateParseResult::new(self.state.parse_for_module(
                        manager,
                        token,
                        &mut module_cache.progress,
                    )),
                    false => RootStateParseResult::new_error(ParseError::NewLineExpected),
                },
            },
            RootStatus::Export => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "default" => RootStateParseResult::new(JsonState::ParseModule(self.state)),
                    _ => RootStateParseResult::new_error(ParseError::WrongExportStatement),
                },
                _ => RootStateParseResult::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::Module => match token {
                JsonToken::Dot => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                    status: RootStatus::ModuleDot,
                    state: self.state,
                    new_line: false,
                })),
                _ => RootStateParseResult::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::ModuleDot => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "exports" => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                        status: RootStatus::ModuleDotExports,
                        state: self.state,
                        new_line: false,
                    })),
                    _ => RootStateParseResult::new_error(ParseError::WrongExportStatement),
                },
                _ => RootStateParseResult::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::ModuleDotExports => match token {
                JsonToken::Equals => RootStateParseResult::new(JsonState::ParseModule(self.state)),
                _ => RootStateParseResult::new_error(ParseError::WrongExportStatement),
            },
            RootStatus::Const => match token {
                JsonToken::Id(s) => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                    status: RootStatus::ConstId(s),
                    state: self.state,
                    new_line: false,
                })),
                _ => RootStateParseResult::new_error(ParseError::WrongConstStatement),
            },
            RootStatus::ConstId(s) => match token {
                JsonToken::Equals => RootStateParseResult::new(JsonState::ParseConst(ConstState {
                    key: s,
                    state: self.state,
                })),
                _ => RootStateParseResult::new_error(ParseError::WrongConstStatement),
            },
            RootStatus::Import => match token {
                JsonToken::Id(s) => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                    status: RootStatus::ImportId(s),
                    state: self.state,
                    new_line: false,
                })),
                _ => RootStateParseResult::new_error(ParseError::WrongImportStatement),
            },
            RootStatus::ImportId(id) => match token {
                JsonToken::Id(s) => match s.as_ref() {
                    "from" => RootStateParseResult::new(JsonState::ParseRoot(RootState {
                        status: RootStatus::ImportIdFrom(id),
                        state: self.state,
                        new_line: false,
                    })),
                    _ => RootStateParseResult::new_error(ParseError::WrongImportStatement),
                },
                _ => RootStateParseResult::new_error(ParseError::WrongImportStatement),
            },
            RootStatus::ImportIdFrom(id) => match token {
                JsonToken::String(s) => {
                    let import_from_path = concat(split(&context_path).0, s.as_str());
                    if let Some(any) = module_cache.complete.get(&import_from_path) {
                        let mut state = self.state;
                        state.consts.insert(id, any.clone());
                        return RootStateParseResult::new(JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            state,
                            new_line: false,
                        }));
                    }
                    if module_cache.progress.contains(&import_from_path) {
                        return RootStateParseResult::new_error(ParseError::CircularDependency);
                    }
                    module_cache.progress.insert(import_from_path.clone());
                    RootStateParseResult::new_with_import_from(
                        JsonState::ParseRoot(RootState {
                            status: RootStatus::Initial,
                            new_line: false,
                            ..self
                        }),
                        import_from_path,
                    )
                }
                _ => RootStateParseResult::new_error(ParseError::WrongImportStatement),
            },
        }
    }
}

pub struct ConstState<D: Dealloc> {
    pub key: String,
    pub state: AnyState<D>,
}

pub enum JsonState<D: Dealloc> {
    ParseRoot(RootState<D>),
    ParseConst(ConstState<D>),
    ParseModule(AnyState<D>),
    Result(ParseResult<D>),
    Error(ParseError),
}

impl<D: Dealloc> JsonState<D> {
    pub fn end(self) -> Result<ParseResult<D>, ParseError> {
        match self {
            JsonState::Result(result) => Ok(result),
            JsonState::Error(error) => Err(error),
            _ => Err(ParseError::UnexpectedEnd),
        }
    }
}

pub struct AnyState<D: Dealloc> {
    pub data_type: DataType,
    pub status: ParsingStatus,
    pub current: JsonElement<D>,
    pub stack: Vec<JsonStackElement<D>>,
    pub consts: BTreeMap<String, Any<D>>,
}

impl<D: Dealloc> Default for AnyState<D> {
    fn default() -> Self {
        AnyState {
            data_type: default(),
            status: ParsingStatus::Initial,
            current: JsonElement::None,
            stack: [].cast(),
            consts: default(),
        }
    }
}

// AnyState methods that use Dealloc only - methods that use Manager are in AnyStateExtension.
impl<D: Dealloc> AnyState<D> {
    pub fn set_djs(self) -> Self {
        AnyState {
            data_type: DataType::Djs,
            ..self
        }
    }

    pub fn set_mjs(self) -> Self {
        AnyState {
            data_type: DataType::Mjs,
            ..self
        }
    }

    pub fn set_cjs(self) -> Self {
        AnyState {
            data_type: DataType::Cjs,
            ..self
        }
    }

    pub fn parse_import_begin(self, token: JsonToken) -> AnyResult<D> {
        match token {
            JsonToken::OpeningParenthesis => AnyResult::Continue(AnyState {
                status: ParsingStatus::ImportValue,
                ..self
            }),
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    pub fn parse_import_end(self, token: JsonToken) -> AnyResult<D> {
        match token {
            JsonToken::ClosingParenthesis => self.end_import(),
            _ => AnyResult::Error(ParseError::WrongRequireStatement),
        }
    }

    pub fn end_import(mut self) -> AnyResult<D> {
        match self.current {
            JsonElement::Any(any) => {
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState {
                    status: ParsingStatus::Initial,
                    current,
                    ..self
                };
                new_state.push_value(any)
            }
            _ => unreachable!(),
        }
    }

    pub fn push_value(self, value: Any<D>) -> AnyResult<D> {
        match self.current {
            JsonElement::None => AnyResult::Success(AnySuccess {
                state: AnyState {
                    status: ParsingStatus::Initial,
                    ..self
                },
                value,
            }),
            JsonElement::Stack(top) => match top {
                JsonStackElement::Array(mut arr) => {
                    arr.push(value);
                    AnyResult::Continue(AnyState {
                        status: ParsingStatus::ArrayValue,
                        current: JsonElement::Stack(JsonStackElement::Array(arr)),
                        ..self
                    })
                }
                JsonStackElement::Object(mut stack_obj) => {
                    stack_obj.map.insert(stack_obj.key, value);
                    let new_stack_obj = JsonStackObject {
                        map: stack_obj.map,
                        key: String::default(),
                    };
                    AnyResult::Continue(AnyState {
                        status: ParsingStatus::ObjectValue,
                        current: JsonElement::Stack(JsonStackElement::Object(new_stack_obj)),
                        ..self
                    })
                }
            },
            _ => todo!(),
        }
    }

    pub fn begin_import(mut self) -> AnyResult<D> {
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top);
        }
        AnyResult::Continue(AnyState {
            data_type: DataType::Cjs,
            status: ParsingStatus::ImportBegin,
            current: JsonElement::None,
            ..self
        })
    }

    pub fn push_key(self, s: String) -> AnyResult<D> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Object(stack_obj)) => {
                let new_stack_obj = JsonStackObject {
                    map: stack_obj.map,
                    key: s,
                };
                AnyResult::Continue(AnyState {
                    status: ParsingStatus::ObjectKey,
                    current: JsonElement::Stack(JsonStackElement::Object(new_stack_obj)),
                    ..self
                })
            }
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    pub fn begin_array(mut self) -> AnyResult<D> {
        let new_top = JsonStackElement::Array(Vec::default());
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top);
        }
        AnyResult::Continue(AnyState {
            status: ParsingStatus::ArrayBegin,
            current: JsonElement::Stack(new_top),
            ..self
        })
    }

    pub fn begin_object(mut self) -> AnyResult<D> {
        let new_top: JsonStackElement<D> = JsonStackElement::Object(JsonStackObject {
            map: BTreeMap::default(),
            key: String::default(),
        });
        if let JsonElement::Stack(top) = self.current {
            self.stack.push(top)
        }
        AnyResult::Continue(AnyState {
            status: ParsingStatus::ObjectBegin,
            current: JsonElement::Stack(new_top),
            ..self
        })
    }

    pub fn parse_object_key(self, token: JsonToken) -> AnyResult<D> {
        match token {
            JsonToken::Colon => AnyResult::Continue(AnyState {
                status: ParsingStatus::ObjectColon,
                ..self
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }
}

/// AnyStateExtension contains AnyState methods that use Manager (not only Dealloc).
pub trait AnyStateExtension<M: Manager> {
    fn end_array(self, manager: M) -> AnyResult<M::Dealloc>;
    fn end_object(self, manager: M) -> AnyResult<M::Dealloc>;
    fn parse(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_for_module(
        self,
        manager: M,
        token: JsonToken,
        _module_cache_progress: &mut BTreeSet<String>,
    ) -> JsonState<M::Dealloc>;
    fn parse_value(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_array_comma(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_array_begin(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_array_value(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_object_begin(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_object_next(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
    fn parse_object_comma(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc>;
}

impl<M: Manager> AnyStateExtension<M> for AnyState<M::Dealloc> {
    fn end_array(mut self, manager: M) -> AnyResult<M::Dealloc> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Array(array)) => {
                let js_array = new_array(manager, array).to_ref();
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState { current, ..self };
                new_state.push_value(Any::move_from(js_array))
            }
            _ => unreachable!(),
        }
    }

    fn end_object(mut self, manager: M) -> AnyResult<M::Dealloc> {
        match self.current {
            JsonElement::Stack(JsonStackElement::Object(object)) => {
                let vec = object
                    .map
                    .into_iter()
                    .map(|kv| (to_js_string(manager, kv.0), kv.1))
                    .collect::<Vec<_>>();
                let js_object = new_object(manager, vec).to_ref();
                let current = match self.stack.pop() {
                    Some(element) => JsonElement::Stack(element),
                    None => JsonElement::None,
                };
                let new_state = AnyState { current, ..self };
                new_state.push_value(Any::move_from(js_object))
            }
            _ => unreachable!(),
        }
    }

    fn parse(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match self.status {
            ParsingStatus::Initial | ParsingStatus::ObjectColon => self.parse_value(manager, token),
            ParsingStatus::ArrayBegin => self.parse_array_begin(manager, token),
            ParsingStatus::ArrayValue => self.parse_array_value(manager, token),
            ParsingStatus::ArrayComma => self.parse_array_comma(manager, token),
            ParsingStatus::ObjectBegin => self.parse_object_begin(manager, token),
            ParsingStatus::ObjectKey => self.parse_object_key(token),
            ParsingStatus::ObjectValue => self.parse_object_next(manager, token),
            ParsingStatus::ObjectComma => self.parse_object_comma(manager, token),
            ParsingStatus::ImportBegin => self.parse_import_begin(token),
            ParsingStatus::ImportValue => todo!(), // any_state, context, token),
            ParsingStatus::ImportEnd => self.parse_import_end(token),
        }
    }

    fn parse_for_module(
        self,
        manager: M,
        token: JsonToken,
        _module_cache_progress: &mut BTreeSet<String>,
    ) -> JsonState<M::Dealloc> {
        let result = self.parse(manager, token);
        match result {
            AnyResult::Continue(state) => JsonState::ParseModule(state),
            AnyResult::Success(success) => JsonState::Result(ParseResult {
                data_type: success.state.data_type,
                any: success.value,
            }),
            AnyResult::Error(error) => JsonState::Error(error),
        }
    }

    fn parse_value(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::ArrayBegin => self.begin_array(),
            JsonToken::ObjectBegin => self.begin_object(),
            JsonToken::Id(s) if self.data_type.is_cjs_compatible() && s == "require" => {
                self.begin_import()
            }
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => AnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_comma(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::ArrayBegin => self.begin_array(),
            JsonToken::ObjectBegin => self.begin_object(),
            JsonToken::Id(s) if self.data_type == DataType::Cjs && s == "require" => {
                self.begin_import()
            }
            JsonToken::ArrayEnd => self.end_array(manager),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => AnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_begin(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::ArrayBegin => self.begin_array(),
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::ObjectBegin => self.begin_object(),
            _ => {
                let option_any = token.try_to_any(manager, &self.consts);
                match option_any {
                    Some(any) => self.push_value(any),
                    None => AnyResult::Error(ParseError::UnexpectedToken),
                }
            }
        }
    }

    fn parse_array_value(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::ArrayEnd => self.end_array(manager),
            JsonToken::Comma => AnyResult::Continue(AnyState {
                status: ParsingStatus::ArrayComma,
                ..self
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_begin(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::Id(s) if self.data_type.is_djs() => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_next(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::ObjectEnd => self.end_object(manager),
            JsonToken::Comma => AnyResult::Continue(AnyState {
                status: ParsingStatus::ObjectComma,
                ..self
            }),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }

    fn parse_object_comma(self, manager: M, token: JsonToken) -> AnyResult<M::Dealloc> {
        match token {
            JsonToken::String(s) => self.push_key(s),
            JsonToken::ObjectEnd => self.end_object(manager),
            _ => AnyResult::Error(ParseError::UnexpectedToken),
        }
    }
}

fn to_js_string<M: Manager>(manager: M, s: String) -> JsStringRef<M::Dealloc> {
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
    fn try_to_any<M: Manager>(
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
