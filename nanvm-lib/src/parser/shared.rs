/*pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: u32,
}

pub enum TokenKind {
    // Add your token kinds here
}

pub struct ParseError {
    pub message: String,
    pub line: u32,
}

pub type ParseResult<T> = Result<T, ParseError>;*/

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
