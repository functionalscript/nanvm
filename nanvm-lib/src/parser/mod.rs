use std::collections::HashMap;

pub enum JsonUnknown {
    Object(HashMap<String, JsonUnknown>),
    Boolean(bool),
    String(String),
    Number(f64),
    Null,
    Array(Vec<JsonUnknown>),
}

pub enum JsonStackElement {
    Object(JsonStackObject),
    Array(Vec<JsonUnknown>),
}

pub struct JsonStackObject {
    pub map: HashMap<String, JsonUnknown>,
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

pub struct StateParse {
    pub status: ParseStatus,
    pub top: Option<JsonStackElement>,
    pub stack: JsonStackElement,
}

pub enum JsonState {
    Parse(StateParse),
    Result(JsonUnknown),
    Error(String),
}
