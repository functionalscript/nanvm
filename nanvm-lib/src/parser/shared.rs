use crate::{
    common::{cast::Cast, default::default},
    js::any::Any,
    mem::manager::Dealloc,
    tokenizer::JsonToken,
};
use std::collections::BTreeMap;
use std::fmt::Display;

#[derive(Debug, Default, PartialEq)]
pub enum DataType {
    #[default]
    Json,
    Djs,
    Cjs,
    Mjs,
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

pub struct RootState<D: Dealloc> {
    pub status: RootStatus,
    pub state: AnyState<D>,
    pub new_line: bool,
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

pub struct AnyState<D: Dealloc> {
    pub data_type: DataType,
    pub status: ParsingStatus,
    pub current: JsonElement<D>,
    pub stack: Vec<JsonStackElement<D>>,
    pub consts: BTreeMap<String, Any<D>>,
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
