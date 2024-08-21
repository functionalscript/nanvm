use crate::{
    common::{cast::Cast, default::default},
    js::any::Any,
    mem::manager::Dealloc,
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

pub struct AnyStateStruct<D: Dealloc> {
    pub data_type: DataType,
    pub status: ParsingStatus,
    pub current: JsonElement<D>,
    pub stack: Vec<JsonStackElement<D>>,
    pub consts: BTreeMap<String, Any<D>>,
}

impl<D: Dealloc> Default for AnyStateStruct<D> {
    fn default() -> Self {
        AnyStateStruct {
            data_type: default(),
            status: ParsingStatus::Initial,
            current: JsonElement::None,
            stack: [].cast(),
            consts: default(),
        }
    }
}

pub struct AnySuccess<D: Dealloc> {
    pub state: AnyStateStruct<D>,
    pub value: Any<D>,
}

pub enum AnyResult<D: Dealloc> {
    Continue(AnyStateStruct<D>),
    Success(AnySuccess<D>),
    Error(ParseError),
}

impl<D: Dealloc> AnyStateStruct<D> {
    pub fn set_djs(self) -> Self {
        AnyStateStruct {
            data_type: DataType::Djs,
            ..self
        }
    }

    pub fn set_mjs(self) -> Self {
        AnyStateStruct {
            data_type: DataType::Mjs,
            ..self
        }
    }

    pub fn set_cjs(self) -> Self {
        AnyStateStruct {
            data_type: DataType::Cjs,
            ..self
        }
    }
}
