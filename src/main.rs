pub trait ArrayEx {
    type Item;
    /// Move the array into a vector.
    /// Compare to `.to_vec()`, the function doesn't require `Clone` trait.
    fn vec(self) -> Vec<Self::Item>;
}

impl<T: Sized, const N: usize> ArrayEx for [T; N] {
    type Item = T;
    fn vec(self) -> Vec<Self::Item> {
        let mut result = Vec::with_capacity(N);
        for i in self {
            result.push(i);
        }
        result
    }
}

#[derive(Debug,PartialEq)]
enum JsonToken {
    True,
    False,
    Null,
    String(String),
    Number(f64),
    ObjectBegin,
    ObjectEnd,
    ArrayBegin,
    ArrayEnd,
    Colon,
    Comma,
    ErrorToken(ErrorType),
}

#[derive(Debug,PartialEq)]
enum ErrorType {
    UnexpectedCharacter,
    InvalidToken,
    InvalidNumber,
    InvalidHex,
    MissingQuotes,
    Eof
}

enum TokenizerState {
    Initial,
    ParseKeyword(String),
    ParseString(String),
    ParseEscapeChar(String),
    ParseUnicodeChar(ParseUnicodeCharState),
    ParseNumber(ParseNumberState),
    ParseMinus,
    InvalidNumber,
    Eof
}

enum ParseNumberState {
    Zero(Sign),
    Int(Integer),
    Dot(Integer),
    Frac(Integer),
    Exp(Integer),
    ExpPlus(Integer),
    ExpMinus(Integer),
    ExpDigits(Integer)
}

struct ParseUnicodeCharState {
    s: String,
    unicode: u32,
    index: u8,
}

struct Integer {
    s: Sign,
    m: u128,
}

enum Sign {
    Plus,
    Minus
}

const CP_0: u32 = 0x30;
const CP_SMALL_A: u32 = 0x61;
const CP_CAPITAL_A: u32 = 0x41;

fn digit_to_number(cp: u32) -> u128 {
    u128::from(cp - CP_0)
}

fn start_number(c: char) -> ParseNumberState {
    let cp = u32::from(c);
    ParseNumberState::Int(Integer { s: Sign::Plus, m: digit_to_number(cp) })
}

fn operator_to_token(c: char) -> JsonToken {
    match c {
        '{' => JsonToken::ObjectBegin,
        '}' => JsonToken::ObjectEnd,
        '[' => JsonToken::ArrayBegin,
        ']' => JsonToken::ArrayEnd,
        ':' => JsonToken::Colon,
        ',' => JsonToken::Comma,
        _ => panic!("unexpected operator")
    }
}

fn keyword_to_token(s: &str) -> JsonToken {
    match s {
        "true" => JsonToken::True,
        "false" => JsonToken::False,
        "null" => JsonToken::Null,
        _ => JsonToken::ErrorToken(ErrorType::InvalidToken)
    }
}

fn tokenize_initial(c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '1'..='9' => ([].vec(), TokenizerState::ParseNumber(start_number(c))),
        '\t' | '\n' | '\r' | ' ' => ([].vec(), TokenizerState::Initial),
        '"' => ([].vec(), TokenizerState::ParseString(String::from(""))),
        '0' => ([].vec(), TokenizerState::ParseNumber(ParseNumberState::Zero(Sign::Plus))),
        '{' | '}' | '[' | ']' | ':' | ',' => ([operator_to_token(c)].vec(), TokenizerState::Initial),
        '-' => ([].vec(), TokenizerState::ParseMinus),
        'a'..='z' => ([].vec(), TokenizerState::ParseKeyword(c.to_string())),
        _ => ([JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec(), TokenizerState::Initial)
    }
}

fn tokenize_keyword(c: char, s: &str) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        'a'..='z' => {
            let mut new_string = s.to_owned();
            new_string.push(c);
            ([].vec(), TokenizerState::ParseKeyword(new_string))
        }
        _ => {
            let token = keyword_to_token(s);
            let (next_tokens, next_state) = tokenize_initial(c);
            let mut vec = [token].vec();
            vec.extend(next_tokens);
            (vec, next_state)
        }
    }
}

fn tokenize_string(c: char, s: &str) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '"' => ([JsonToken::String(s.to_owned())].vec(), TokenizerState::Initial),
        '\\' => ([].vec(), TokenizerState::ParseEscapeChar(s.to_owned())),
        _ => {
            let mut new_string = s.to_owned();
            new_string.push(c);
            ([].vec(), TokenizerState::ParseString(new_string))
        }
    }
}

fn continue_string_state(c: char, s: &str) -> (Vec<JsonToken>, TokenizerState) {
    let mut new_string = s.to_owned();
    new_string.push(c);
    ([].vec(), TokenizerState::ParseString(new_string))
}

fn tokenize_escape_char(c: char, s: &str) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '\"' | '\\' | '/' => continue_string_state(c, s),
        'b' => continue_string_state('\u{8}', s),
        'f' => continue_string_state('\u{c}', s),
        'n' => continue_string_state('\n', s),
        'r' => continue_string_state('\r', s),
        't' => continue_string_state('\t', s),
        'u' => ([].vec(), TokenizerState::ParseUnicodeChar(ParseUnicodeCharState { s: s.to_owned(), unicode: 0, index: 0 })),
        _ => {
            let (next_tokens, next_state) = tokenize_string(c, s);
            let mut vec = [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec();
            vec.extend(next_tokens);
            (vec, next_state)
        }
    }
}

fn continue_unicode_state(i: u32, state: &ParseUnicodeCharState) -> (Vec<JsonToken>, TokenizerState) {
    let new_unicode = state.unicode | (i << (3 - state.index) * 4);
    match state.index {
        3 => {
            let c = char::from_u32(new_unicode);
            match c {
                Some(c) => continue_string_state(c, &state.s),
                None => panic!("invalid hex")
            }
        },
        0..=2 => {
            ([].vec(), TokenizerState::ParseUnicodeChar(ParseUnicodeCharState { s: state.s.clone(), unicode: new_unicode, index: state.index + 1 }))
        },
        _ => panic!("invalid index")
    }
}

fn tokenize_unicode_char(c: char, state: &ParseUnicodeCharState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => continue_unicode_state(u32::from(c) - CP_0, state),
        'a'..='f' => continue_unicode_state(u32::from(c) - CP_SMALL_A + 10, state),
        'A'..='F' => continue_unicode_state(u32::from(c) - CP_CAPITAL_A + 10, state),
        _ => {
            let (next_tokens, next_state) = tokenize_string(c, &state.s);
            let mut vec = [JsonToken::ErrorToken(ErrorType::InvalidHex)].vec();
            vec.extend(next_tokens);
            (vec, next_state)
        }
    }
}

fn tokenize_eof(state: &TokenizerState) -> (Vec<JsonToken>, TokenizerState) {
    match state {
        TokenizerState::Initial => ([].vec(), TokenizerState::Eof),
        TokenizerState::ParseKeyword(s) => ([keyword_to_token(s)].vec(), TokenizerState::Eof),
        TokenizerState::ParseString(_) | TokenizerState::ParseEscapeChar(_) | TokenizerState::ParseUnicodeChar(_) => ([JsonToken::ErrorToken(ErrorType::MissingQuotes)].vec(), TokenizerState::Eof),
        TokenizerState::InvalidNumber | TokenizerState::ParseMinus => ([JsonToken::ErrorToken(ErrorType::InvalidNumber)].vec(), TokenizerState::Eof),
        TokenizerState::ParseNumber(_) => todo!(),
        TokenizerState::Eof => ([JsonToken::ErrorToken(ErrorType::Eof)].vec(), TokenizerState::Eof)
    }
}

fn tokenize_next_char(c: char, state: &TokenizerState) -> (Vec<JsonToken>, TokenizerState) {
    match state {
        TokenizerState::Initial => tokenize_initial(c),
        TokenizerState::ParseKeyword(s) => tokenize_keyword(c, s),
        TokenizerState::ParseString(s) => tokenize_string(c, s),
        TokenizerState::ParseEscapeChar(s) => tokenize_escape_char(c, s),
        TokenizerState::ParseUnicodeChar(s) => tokenize_unicode_char(c, s),
        TokenizerState::InvalidNumber => todo!(),
        TokenizerState::ParseNumber(_) => todo!(),
        TokenizerState::ParseMinus => todo!(),
        TokenizerState::Eof => ([JsonToken::ErrorToken(ErrorType::Eof)].vec(), TokenizerState::Eof)
    }
}

fn tokenize(input: String) -> Vec<JsonToken> {
    let mut state = TokenizerState::Initial;
    let mut res = [].vec();
    for c in input.chars() {
        let (tokens, new_state) = tokenize_next_char(c, &state);
        res.extend(tokens);
        state = new_state;
    }
    let (tokens, _) = tokenize_eof(&state);
    res.extend(tokens);
    res
}

fn main() {
    let result = tokenize(String::from(""));
    println!("{:?}", result);
    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{tokenize, JsonToken, ErrorType};

    #[test]
    #[wasm_bindgen_test]
    fn test_empty() {
        let result = tokenize(String::from(""));
        assert_eq!(result.len(), 0);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_ops() {
        let result = tokenize(String::from("{"));
        assert_eq!(&result, &[JsonToken::ObjectBegin]);

        let result = tokenize(String::from("}"));
        assert_eq!(&result, &[JsonToken::ObjectEnd]);

        let result = tokenize(String::from("["));
        assert_eq!(&result, &[JsonToken::ArrayBegin]);

        let result = tokenize(String::from("]"));
        assert_eq!(&result, &[JsonToken::ArrayEnd]);

        let result = tokenize(String::from(":"));
        assert_eq!(&result, &[JsonToken::Colon]);

        let result = tokenize(String::from(","));
        assert_eq!(&result, &[JsonToken::Comma]);

        let result = tokenize(String::from("[{ :, }]"));
        assert_eq!(&result, &[JsonToken::ArrayBegin, JsonToken::ObjectBegin, JsonToken::Colon, JsonToken::Comma, JsonToken::ObjectEnd, JsonToken::ArrayEnd]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_keyword() {
        let result = tokenize(String::from("true"));
        assert_eq!(&result, &[JsonToken::True]);

        let result = tokenize(String::from("false"));
        assert_eq!(&result, &[JsonToken::False]);

        let result = tokenize(String::from("null"));
        assert_eq!(&result, &[JsonToken::Null]);

        let result = tokenize(String::from("true false null"));
        assert_eq!(&result, &[JsonToken::True, JsonToken::False, JsonToken::Null]);

        let result = tokenize(String::from("tru tru"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidToken), JsonToken::ErrorToken(ErrorType::InvalidToken)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_whitespace() {
        let result = tokenize(String::from(" \t\n\r"));
        assert_eq!(&result, &[]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_string() {
        let result = tokenize(String::from("\"\""));
        assert_eq!(&result, &[JsonToken::String("".to_string())]);

        let result = tokenize(String::from("\"value\""));
        assert_eq!(&result, &[JsonToken::String("value".to_string())]);

        let result = tokenize(String::from("\"value1\" \"value2\""));
        assert_eq!(&result, &[JsonToken::String("value1".to_string()), JsonToken::String("value2".to_string())]);

        let result = tokenize(String::from("\"value"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_escaped_characters() {
        let result = tokenize(String::from("\"\\b\\f\\n\\r\\t\""));
        assert_eq!(&result, &[JsonToken::String("\u{8}\u{c}\n\r\t".to_string())]);

        let result = tokenize(String::from("\"\\x\""));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::UnexpectedCharacter), JsonToken::String("x".to_string())]);

        let result = tokenize(String::from("\"\\"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_unicode() {
        let result = tokenize(String::from("\"\\u1234\""));
        assert_eq!(&result, &[JsonToken::String("ሴ".to_string())]);

        let result = tokenize(String::from("\"\\uaBcDEeFf\""));
        assert_eq!(&result, &[JsonToken::String("ꯍEeFf".to_string())]);

        let result = tokenize(String::from("\"\\uEeFg\""));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidHex), JsonToken::String("g".to_string())]);

        let result = tokenize(String::from("\"\\uEeF"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }
}