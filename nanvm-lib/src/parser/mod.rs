use std::collections::HashMap;

pub enum JsonUnknown {
    Object(HashMap<String, JsonUnknown>),
    Boolean(bool),
    String(String),
    Number(f64),
    Null,
    Array(Vec<JsonUnknown>)
}

pub enum JsonStackElement {
    Object(JsonStackObject),
    Array(Vec<JsonUnknown>)
}

pub struct  JsonStackObject {
    pub map: HashMap<String, JsonUnknown>,
    pub key: String,
}
