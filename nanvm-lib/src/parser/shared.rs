use crate::{
    big_numbers::big_int::{BigInt, Sign},
    common::default::default,
    js::{
        any::Any,
        js_bigint::{new_bigint, JsBigintRef},
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
pub struct ParseResult<D: Dealloc> {
    pub data_type: DataType,
    pub any: Any<D>,
}

pub fn to_js_string<M: Manager>(manager: M, s: String) -> JsStringRef<M::Dealloc> {
    new_string(manager, s.encode_utf16().collect::<Vec<_>>()).to_ref()
}

pub fn to_js_bigint<M: Manager>(manager: M, b: BigInt) -> JsBigintRef<M::Dealloc> {
    let sign = match b.sign {
        Sign::Positive => crate::js::js_bigint::Sign::Positive,
        Sign::Negative => crate::js::js_bigint::Sign::Negative,
    };
    new_bigint(manager, sign, b.value.value).to_ref()
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

impl<D: Dealloc> JsonToken<D> {
    pub fn try_to_any<M: Manager<Dealloc = D>>(
        self,
        manager: M,
        consts: &BTreeMap<String, Any<M::Dealloc>>,
    ) -> Option<Any<M::Dealloc>> {
        match self {
            JsonToken::Number(f) => Some(Any::move_from(f)),
            JsonToken::String(s) => Some(Any::move_from(to_js_string(manager, s))),
            JsonToken::Id(s) => try_id_to_any(&s, manager, consts),
            JsonToken::BigInt(b) => Some(Any::move_from(b.to_ref())),
            _ => None,
        }
    }
}
