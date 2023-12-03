mod big_int;

use std::{collections::VecDeque, mem::take};

use crate::common::{array::ArrayEx, default::default};

#[derive(Debug, PartialEq)]
pub enum JsonToken {
    True,
    False,
    Null,
    String(String),
    Number(BigFloat),
    ObjectBegin,
    ObjectEnd,
    ArrayBegin,
    ArrayEnd,
    Colon,
    Comma,
    ErrorToken(ErrorType),
}

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    UnexpectedCharacter,
    InvalidToken,
    InvalidNumber,
    InvalidHex,
    MissingQuotes,
}

#[derive(Debug, PartialEq, Default)]
pub struct BigFloat {
    m: i128,
    e: i16,
}

enum TokenizerState {
    Initial,
    ParseKeyword(String),
    ParseString(String),
    ParseEscapeChar(String),
    ParseUnicodeChar(ParseUnicodeCharState),
    ParseMinus,
    ParseZero(Sign),
    ParseInt(IntegerState),
    ParseFracBegin(IntegerState),
    ParseFrac(FloatState),
    ParseExpBegin(ExpState),
    ParseExpSign(ExpState),
    ParseExp(ExpState),
}

impl Default for TokenizerState {
    fn default() -> Self {
        TokenizerState::Initial
    }
}

impl TokenizerState {
    fn push(self, c: char) -> (Vec<JsonToken>, TokenizerState) {
        match self {
            TokenizerState::Initial => tokenize_initial(c),
            TokenizerState::ParseKeyword(s) => tokenize_keyword(s, c),
            TokenizerState::ParseString(s) => tokenize_string(s, c),
            TokenizerState::ParseEscapeChar(s) => tokenize_escape_char(s, c),
            TokenizerState::ParseUnicodeChar(s) => tokenize_unicode_char(s, c),
            TokenizerState::ParseZero(s) => tokenize_zero(s, c),
            TokenizerState::ParseInt(s) => tokenize_integer(s, c),
            TokenizerState::ParseMinus => tokenize_minus(c),
            TokenizerState::ParseFracBegin(s) => tokenize_frac_begin(s, c),
            TokenizerState::ParseFrac(s) => tokenize_frac(s, c),
            TokenizerState::ParseExpBegin(s) => tokenize_exp_begin(s, c),
            TokenizerState::ParseExpSign(s) | TokenizerState::ParseExp(s) => tokenize_exp(s, c),
        }
    }

    fn push_mut(&mut self, c: char) -> Vec<JsonToken> {
        let tokens;
        (tokens, *self) = take(self).push(c);
        tokens
    }

    fn end(self) -> Vec<JsonToken> {
        match self {
            TokenizerState::Initial => default(),
            TokenizerState::ParseKeyword(s) => [keyword_to_token(&s)].vec(),
            TokenizerState::ParseString(_)
            | TokenizerState::ParseEscapeChar(_)
            | TokenizerState::ParseUnicodeChar(_) => {
                [JsonToken::ErrorToken(ErrorType::MissingQuotes)].vec()
            }
            TokenizerState::ParseZero(_) => [JsonToken::Number(default())].vec(),
            TokenizerState::ParseInt(s) => [s.to_token()].vec(),
            TokenizerState::ParseFrac(s) => [s.to_token()].vec(),
            TokenizerState::ParseExp(s) => [s.to_token()].vec(),
            TokenizerState::ParseMinus
            | TokenizerState::ParseFracBegin(_)
            | TokenizerState::ParseExpBegin(_)
            | TokenizerState::ParseExpSign(_) => {
                [JsonToken::ErrorToken(ErrorType::InvalidNumber)].vec()
            }
        }
    }
}

struct ParseUnicodeCharState {
    s: String,
    unicode: u32,
    index: u8,
}

impl ParseUnicodeCharState {
    fn push(mut self, i: u32) -> (Vec<JsonToken>, TokenizerState) {
        let new_unicode = self.unicode | (i << (3 - self.index) * 4);
        match self.index {
            3 => {
                let c = char::from_u32(new_unicode);
                match c {
                    Some(c) => continue_string_state(self.s, c),
                    None => (
                        [JsonToken::ErrorToken(ErrorType::InvalidHex)].vec(),
                        TokenizerState::Initial,
                    ),
                }
            }
            0..=2 => {
                self.unicode = new_unicode;
                self.index = self.index + 1;
                (default(), TokenizerState::ParseUnicodeChar(self))
            }
            _ => unreachable!(),
        }
    }
}

enum Sign {
    Plus,
    Minus,
}

struct IntegerState {
    s: Sign,
    m: i128,
}

impl IntegerState {
    const fn add_digit(mut self, c: char) -> IntegerState {
        self.m = self.m * 10 + c as i128 - CP_0 as i128;
        self
    }

    const fn to_float_state(self) -> FloatState {
        FloatState {
            s: self.s,
            m: self.m,
            fe: 0,
        }
    }

    const fn to_exp_state(self) -> ExpState {
        ExpState {
            s: self.s,
            m: self.m,
            fe: 0,
            es: Sign::Plus,
            e: 0,
        }
    }

    const fn to_token(self) -> JsonToken {
        JsonToken::Number(match self.s {
            Sign::Plus => BigFloat {
                m: self.m as i128,
                e: 0,
            },
            Sign::Minus => BigFloat {
                m: -1 * self.m as i128,
                e: 0,
            },
        })
    }
}

struct FloatState {
    s: Sign,
    m: i128,
    fe: i16,
}

impl FloatState {
    const fn add_digit(mut self, c: char) -> FloatState {
        self.m = self.m * 10 + c as i128 - CP_0 as i128;
        self.fe = self.fe - 1;
        self
    }

    const fn to_exp_state(self) -> ExpState {
        ExpState {
            s: self.s,
            m: self.m,
            fe: self.fe,
            es: Sign::Plus,
            e: 0,
        }
    }

    const fn to_token(self) -> JsonToken {
        JsonToken::Number(match self.s {
            Sign::Plus => BigFloat {
                m: self.m as i128,
                e: self.fe,
            },
            Sign::Minus => BigFloat {
                m: -1 * self.m as i128,
                e: self.fe,
            },
        })
    }
}

struct ExpState {
    s: Sign,
    m: i128,
    fe: i16,
    es: Sign,
    e: i16,
}

impl ExpState {
    const fn add_digit(mut self, c: char) -> ExpState {
        self.e = self.e * 10 + (c as u32 - CP_0) as i16;
        self
    }

    const fn to_token(self) -> JsonToken {
        let e = self.fe
            + match self.es {
                Sign::Plus => self.e,
                Sign::Minus => -self.e,
            };
        JsonToken::Number(match self.s {
            Sign::Plus => BigFloat {
                m: self.m as i128,
                e,
            },
            Sign::Minus => BigFloat {
                m: -1 * self.m as i128,
                e,
            },
        })
    }
}

const CP_0: u32 = 0x30;
const CP_SMALL_A: u32 = 0x61;
const CP_CAPITAL_A: u32 = 0x41;

const fn is_white_space(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false,
    }
}

fn is_operator(c: char) -> bool {
    to_operator(c).is_some()
}

const fn to_operator(c: char) -> Option<JsonToken> {
    match c {
        '{' => Some(JsonToken::ObjectBegin),
        '}' => Some(JsonToken::ObjectEnd),
        '[' => Some(JsonToken::ArrayBegin),
        ']' => Some(JsonToken::ArrayEnd),
        ':' => Some(JsonToken::Colon),
        ',' => Some(JsonToken::Comma),
        _ => None,
    }
}

fn is_terminal_for_number(c: char) -> bool {
    match c {
        '"' => true,
        c if is_white_space(c) => true,
        c if is_operator(c) => true,
        _ => false,
    }
}

const fn digit_to_number(cp: u32) -> u32 {
    cp - CP_0
}

const fn start_number(s: Sign, c: char) -> IntegerState {
    let cp = c as u32;
    IntegerState {
        s: s,
        m: digit_to_number(cp) as i128,
    }
}

const fn operator_to_token(c: char) -> JsonToken {
    match c {
        '{' => JsonToken::ObjectBegin,
        '}' => JsonToken::ObjectEnd,
        '[' => JsonToken::ArrayBegin,
        ']' => JsonToken::ArrayEnd,
        ':' => JsonToken::Colon,
        ',' => JsonToken::Comma,
        _ => unreachable!(),
    }
}

fn keyword_to_token(s: &str) -> JsonToken {
    match s {
        "true" => JsonToken::True,
        "false" => JsonToken::False,
        "null" => JsonToken::Null,
        _ => JsonToken::ErrorToken(ErrorType::InvalidToken),
    }
}

fn tokenize_initial(c: char) -> (Vec<JsonToken>, TokenizerState) {
    if let Some(t) = to_operator(c) {
        return ([t].vec(), TokenizerState::Initial);
    }
    match c {
        '1'..='9' => (
            default(),
            TokenizerState::ParseInt(start_number(Sign::Plus, c)),
        ),
        '"' => (default(), TokenizerState::ParseString(String::default())),
        '0' => (default(), TokenizerState::ParseZero(Sign::Plus)),
        '-' => (default(), TokenizerState::ParseMinus),
        'a'..='z' => (default(), TokenizerState::ParseKeyword(c.to_string())),
        c if is_white_space(c) => (default(), TokenizerState::Initial),
        _ => (
            [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec(),
            TokenizerState::Initial,
        ),
    }
}

fn tokenize_keyword(mut s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        'a'..='z' => {
            s.push(c);
            (default(), TokenizerState::ParseKeyword(s))
        }
        _ => {
            let token = keyword_to_token(&s);
            transfer_state([token].vec(), TokenizerState::Initial, c)
        }
    }
}

fn tokenize_string(mut s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '"' => ([JsonToken::String(s)].vec(), TokenizerState::Initial),
        '\\' => (default(), TokenizerState::ParseEscapeChar(s)),
        _ => {
            s.push(c);
            (default(), TokenizerState::ParseString(s))
        }
    }
}

fn continue_string_state(mut s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    s.push(c);
    (default(), TokenizerState::ParseString(s))
}

fn transfer_state(
    mut vec: Vec<JsonToken>,
    mut state: TokenizerState,
    c: char,
) -> (Vec<JsonToken>, TokenizerState) {
    let next_tokens;
    (next_tokens, state) = state.push(c);
    vec.extend(next_tokens);
    (vec, state)
}

fn tokenize_escape_char(s: String, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '\"' | '\\' | '/' => continue_string_state(s, c),
        'b' => continue_string_state(s, '\u{8}'),
        'f' => continue_string_state(s, '\u{c}'),
        'n' => continue_string_state(s, '\n'),
        'r' => continue_string_state(s, '\r'),
        't' => continue_string_state(s, '\t'),
        'u' => (
            default(),
            TokenizerState::ParseUnicodeChar(ParseUnicodeCharState {
                s,
                unicode: 0,
                index: 0,
            }),
        ),
        _ => transfer_state(
            [JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec(),
            TokenizerState::ParseString(s),
            c,
        ),
    }
}

fn tokenize_unicode_char(
    state: ParseUnicodeCharState,
    c: char,
) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => state.push(c as u32 - CP_0),
        'a'..='f' => state.push(c as u32 - (CP_SMALL_A - 10)),
        'A'..='F' => state.push(c as u32 - (CP_CAPITAL_A - 10)),
        _ => transfer_state(
            [JsonToken::ErrorToken(ErrorType::InvalidHex)].vec(),
            TokenizerState::ParseString(state.s),
            c,
        ),
    }
}

fn tokenize_zero(s: Sign, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => tokenize_invalid_number(c),
        '.' => (
            default(),
            TokenizerState::ParseFracBegin(IntegerState { s, m: 0 }),
        ),
        'e' | 'E' => (
            default(),
            TokenizerState::ParseExpBegin(ExpState {
                s,
                m: 0,
                fe: 0,
                es: Sign::Plus,
                e: 0,
            }),
        ),
        c if is_terminal_for_number(c) => transfer_state(
            [JsonToken::Number(default())].vec(),
            TokenizerState::Initial,
            c,
        ),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_integer(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => (default(), TokenizerState::ParseInt(s.add_digit(c))),
        '.' => (default(), TokenizerState::ParseFracBegin(s)),
        'e' | 'E' => (default(), TokenizerState::ParseExpBegin(s.to_exp_state())),
        c if is_terminal_for_number(c) => {
            transfer_state([s.to_token()].vec(), TokenizerState::Initial, c)
        }
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_frac_begin(s: IntegerState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => (
            default(),
            TokenizerState::ParseFrac(s.to_float_state().add_digit(c)),
        ),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_frac(s: FloatState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => (default(), TokenizerState::ParseFrac(s.add_digit(c))),
        'e' | 'E' => (default(), TokenizerState::ParseExpBegin(s.to_exp_state())),
        c if is_terminal_for_number(c) => {
            transfer_state([s.to_token()].vec(), TokenizerState::Initial, c)
        }
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_minus(c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0' => (default(), TokenizerState::ParseZero(Sign::Minus)),
        '1'..='9' => (
            default(),
            TokenizerState::ParseInt(start_number(Sign::Minus, c)),
        ),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_exp_begin(mut s: ExpState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => (default(), TokenizerState::ParseExp(s.add_digit(c))),
        '+' => (default(), TokenizerState::ParseExpSign(s)),
        '-' => (default(), {
            s.es = Sign::Minus;
            TokenizerState::ParseExpSign(s)
        }),
        c if is_terminal_for_number(c) => {
            transfer_state([s.to_token()].vec(), TokenizerState::Initial, c)
        }
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_exp(s: ExpState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => (default(), TokenizerState::ParseExp(s.add_digit(c))),
        c if is_terminal_for_number(c) => {
            transfer_state([s.to_token()].vec(), TokenizerState::Initial, c)
        }
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_invalid_number(c: char) -> (Vec<JsonToken>, TokenizerState) {
    transfer_state(
        [JsonToken::ErrorToken(ErrorType::InvalidNumber)].vec(),
        TokenizerState::Initial,
        c,
    )
}

pub fn tokenize(input: String) -> Vec<JsonToken> {
    TokenizerStateIterator::new(input.chars()).collect()
}

pub struct TokenizerStateIterator<T: Iterator<Item = char>> {
    chars: T,
    cache: VecDeque<JsonToken>,
    state: TokenizerState,
    end: bool,
}

impl<T: Iterator<Item = char>> TokenizerStateIterator<T> {
    pub fn new(chars: T) -> Self {
        Self {
            chars,
            cache: default(),
            state: default(),
            end: false,
        }
    }
}

impl<T: Iterator<Item = char>> Iterator for TokenizerStateIterator<T> {
    type Item = JsonToken;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(result) = self.cache.pop_front() {
                return Some(result);
            }
            if self.end {
                return None;
            }
            match self.chars.next() {
                Some(c) => self.cache.extend(self.state.push_mut(c)),
                None => {
                    self.end = true;
                    self.cache.extend(take(&mut self.state).end())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::{tokenize, BigFloat, ErrorType, JsonToken};

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
        assert_eq!(
            &result,
            &[
                JsonToken::ArrayBegin,
                JsonToken::ObjectBegin,
                JsonToken::Colon,
                JsonToken::Comma,
                JsonToken::ObjectEnd,
                JsonToken::ArrayEnd
            ]
        );
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
        assert_eq!(
            &result,
            &[JsonToken::True, JsonToken::False, JsonToken::Null]
        );

        let result = tokenize(String::from("tru tru"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidToken),
                JsonToken::ErrorToken(ErrorType::InvalidToken)
            ]
        );
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
        assert_eq!(
            &result,
            &[
                JsonToken::String("value1".to_string()),
                JsonToken::String("value2".to_string())
            ]
        );

        let result = tokenize(String::from("\"value"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_escaped_characters() {
        let result = tokenize(String::from("\"\\b\\f\\n\\r\\t\""));
        assert_eq!(
            &result,
            &[JsonToken::String("\u{8}\u{c}\n\r\t".to_string())]
        );

        let result = tokenize(String::from("\"\\x\""));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::UnexpectedCharacter),
                JsonToken::String("x".to_string())
            ]
        );

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
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidHex),
                JsonToken::String("g".to_string())
            ]
        );

        let result = tokenize(String::from("\"\\uEeF"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::MissingQuotes)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {
        let result = tokenize(String::from("0"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 0, e: 0 })]);

        let result = tokenize(String::from("-0"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 0, e: 0 })]);

        let result = tokenize(String::from("0abc"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ErrorToken(ErrorType::InvalidToken)
            ]
        );

        let result = tokenize(String::from("0. 2"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Number(BigFloat { m: 2, e: 0 })
            ]
        );

        let result = tokenize(String::from("1234567890"));
        assert_eq!(
            &result,
            &[JsonToken::Number(BigFloat {
                m: 1234567890,
                e: 0
            })]
        );

        let result = tokenize(String::from("-1234567890"));
        assert_eq!(
            &result,
            &[JsonToken::Number(BigFloat {
                m: -1234567890,
                e: 0
            })]
        );

        let result = tokenize(String::from("[0,1]"));
        assert_eq!(
            &result,
            &[
                JsonToken::ArrayBegin,
                JsonToken::Number(BigFloat { m: 0, e: 0 }),
                JsonToken::Comma,
                JsonToken::Number(BigFloat { m: 1, e: 0 }),
                JsonToken::ArrayEnd
            ]
        );

        let result = tokenize(String::from("001"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::Number(BigFloat { m: 1, e: 0 }),
            ]
        );

        let result = tokenize(String::from("-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("-{}"));
        assert_eq!(
            &result,
            &[
                JsonToken::ErrorToken(ErrorType::InvalidNumber),
                JsonToken::ObjectBegin,
                JsonToken::ObjectEnd
            ]
        );

        let result = tokenize(String::from("9007199254740991"));
        assert_eq!(
            &result,
            &[JsonToken::Number(BigFloat {
                m: 9007199254740991,
                e: 0
            })]
        );

        let result = tokenize(String::from("9007199254740992"));
        assert_eq!(
            &result,
            &[JsonToken::Number(BigFloat {
                m: 9007199254740992,
                e: 0
            })]
        );

        let result = tokenize(String::from("9007199254740993"));
        assert_eq!(
            &result,
            &[JsonToken::Number(BigFloat {
                m: 9007199254740993,
                e: 0
            })]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_float() {
        let result = tokenize(String::from("0.01"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 1, e: -2 })]);

        let result = tokenize(String::from("[-12.34]"));
        assert_eq!(
            &result,
            &[
                JsonToken::ArrayBegin,
                JsonToken::Number(BigFloat { m: -1234, e: -2 }),
                JsonToken::ArrayEnd
            ]
        );
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_exp() {
        let result = tokenize(String::from("1e2"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 1, e: 2 })]);

        let result = tokenize(String::from("1E+2"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 1, e: 2 })]);

        let result = tokenize(String::from("0e-2"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 0, e: -2 })]);

        let result = tokenize(String::from("1e-2"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 1, e: -2 })]);

        let result = tokenize(String::from("1.2e+2"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 12, e: 1 })]);

        let result = tokenize(String::from("12e0000"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m: 12, e: 0 })]);

        let result = tokenize(String::from("1e"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("1e+"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("1e-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_errors() {
        let result = tokenize(String::from("ᄑ"));
        assert_eq!(
            &result,
            &[JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)]
        );
    }
}
