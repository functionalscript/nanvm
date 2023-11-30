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
    Number(BigFloat),
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
    MissingQuotes
}

#[derive(Debug,PartialEq)]
struct BigFloat {
    m: i128,
    e: i16
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
    ParseExp(ExpState)
}

impl TokenizerState {
    fn push(self, c: char) -> (Vec<JsonToken>, TokenizerState) {
        match self {
            TokenizerState::Initial => tokenize_initial(c),
            TokenizerState::ParseKeyword(s) => tokenize_keyword(c, s),
            TokenizerState::ParseString(s) => tokenize_string(c, s),
            TokenizerState::ParseEscapeChar(s) => tokenize_escape_char(c, s),
            TokenizerState::ParseUnicodeChar(s) => tokenize_unicode_char(c, s),
            TokenizerState::ParseZero(s) => tokenize_zero(c, s),
            TokenizerState::ParseInt(s) => tokenize_integer(c, s),
            TokenizerState::ParseMinus => tokenize_minus(c),
            TokenizerState::ParseFracBegin(s) => tokenize_frac_begin(c, s),
            TokenizerState::ParseFrac(s) => tokenize_frac(c, s),
            TokenizerState::ParseExpBegin(s) => tokenize_exp_begin(c, s),
            TokenizerState::ParseExpSign(s) | TokenizerState::ParseExp(s) => tokenize_exp(c, s),
        }
    }

    fn end(self) -> Vec<JsonToken> {
        match self {
            TokenizerState::Initial => [].vec(),
            TokenizerState::ParseKeyword(s) => [keyword_to_token(&s)].vec(),
            TokenizerState::ParseString(_) | TokenizerState::ParseEscapeChar(_) | TokenizerState::ParseUnicodeChar(_) => [JsonToken::ErrorToken(ErrorType::MissingQuotes)].vec(),
            TokenizerState::ParseZero(_) => [JsonToken::Number(BigFloat { m: 0, e: 0})].vec(),
            TokenizerState::ParseInt(s) => [JsonToken::Number(s.to_big_float())].vec(),
            TokenizerState::ParseFrac(s) => [JsonToken::Number(s.to_big_float())].vec(),
            TokenizerState::ParseMinus | TokenizerState::ParseFracBegin(_) | TokenizerState::ParseExpBegin(_) | TokenizerState::ParseExpSign(_) => [JsonToken::ErrorToken(ErrorType::InvalidNumber)].vec(),
            _ => todo!()
        }
    }
}

struct ParseUnicodeCharState {
    s: String,
    unicode: u32,
    index: u8,
}

enum Sign {
    Plus,
    Minus
}

struct IntegerState {
    s: Sign,
    m: u128,
}

impl IntegerState {
    fn add_digit(mut self, c: char) -> IntegerState {
        self.m = self.m * 10 + u128::from(c) - CP_0 as u128;
        self
    }

    fn to_float_state(self) -> FloatState {
        FloatState { s: self.s, m: self.m, fe: 0 }
    }

    fn to_exp_state(self) -> ExpState {
        ExpState { s: self.s, m: self.m, fe: 0, es: Sign::Plus, e: 0 }
    }

    fn to_big_float(self) -> BigFloat {
        match self.s {
            Sign::Plus => BigFloat { m: self.m as i128, e: 0 },
            Sign::Minus =>  BigFloat { m: -1 * self.m as i128, e: 0 }
        }
    }
}

struct FloatState {
    s: Sign,
    m: u128,
    fe: i16
}

impl FloatState {
    fn add_digit(mut self, c: char) -> FloatState {
        self.m = self.m * 10 + u128::from(c) - CP_0 as u128;
        self.fe = self.fe - 1;
        self
    }

    fn to_exp_state(self) -> ExpState {
        ExpState { s: self.s, m: self.m, fe: self.fe, es: Sign::Plus, e: 0 }
    }

    fn to_big_float(self) -> BigFloat {
        match self.s {
            Sign::Plus => BigFloat { m: self.m as i128, e: self.fe },
            Sign::Minus => BigFloat { m: -1 * self.m as i128, e: self.fe }
        }
    }
}

struct ExpState {
    s: Sign,
    m: u128,
    fe: i16,
    es: Sign,
    e: i16
}

impl ExpState {
    fn add_digit(mut self, c: char) -> ExpState {
        self.e = self.e * 10 + (u32::from(c) - CP_0) as i16;
        self
    }

    fn to_big_float(self) -> BigFloat {
        let e ;
        match self.es {
            Sign::Plus => e = self.fe + self.e,
            Sign::Minus => e = self.fe - self.e
        }
        match self.s {
            Sign::Plus => BigFloat { m: self.m as i128, e: e },
            Sign::Minus => BigFloat { m: -1 * self.m as i128, e: e }
        }
    }
}

const CP_0: u32 = 0x30;
const CP_SMALL_A: u32 = 0x61;
const CP_CAPITAL_A: u32 = 0x41;

fn is_white_space(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false,
    }
}

fn is_operator(c: char) -> bool {
    match c {
        '[' | ']' | '{' | '}' | ',' | ':' => true,
        _ => false,
    }
}

fn is_terminal_for_number(c: char) -> bool {
    match c {
        '"'  => true,
        c if is_white_space(c) => true,
        c if is_operator(c) => true,
        _ => false,
    }
}

fn digit_to_number(cp: u32) -> u32 {
    u32::from(cp - CP_0)
}

fn start_number(s: Sign, c: char) -> IntegerState {
    let cp = u32::from(c);
    IntegerState { s: s, m: digit_to_number(cp) as u128 }
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
        '1'..='9' => ([].vec(), TokenizerState::ParseInt(start_number(Sign::Plus, c))),
        '"' => ([].vec(), TokenizerState::ParseString(String::default())),
        '0' => ([].vec(), TokenizerState::ParseZero(Sign::Plus)),
        '-' => ([].vec(), TokenizerState::ParseMinus),
        'a'..='z' => ([].vec(), TokenizerState::ParseKeyword(c.to_string())),
        c if is_operator(c) => ([operator_to_token(c)].vec(), TokenizerState::Initial),
        c if is_white_space(c) => ([].vec(), TokenizerState::Initial),
        _ => ([JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec(), TokenizerState::Initial)
    }
}

fn tokenize_keyword(c: char, mut s: String) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        'a'..='z' => {
            s.push(c);
            ([].vec(), TokenizerState::ParseKeyword(s))
        }
        _ => {
            let token = keyword_to_token(&s);
            transfer_state([token].vec(), TokenizerState::Initial, c)
        }
    }
}

fn tokenize_string(c: char, mut s: String) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '"' => ([JsonToken::String(s)].vec(), TokenizerState::Initial),
        '\\' => ([].vec(), TokenizerState::ParseEscapeChar(s)),
        _ => {
            s.push(c);
            ([].vec(), TokenizerState::ParseString(s))
        }
    }
}

fn continue_string_state(c: char, mut s: String) -> (Vec<JsonToken>, TokenizerState) {
    s.push(c);
    ([].vec(), TokenizerState::ParseString(s))
}

fn transfer_state(mut vec: Vec<JsonToken>, mut state: TokenizerState, c: char) -> (Vec<JsonToken>, TokenizerState) {
    let next_tokens;
    (next_tokens, state) = state.push(c);
    vec.extend(next_tokens);
    (vec, state)
}

fn tokenize_escape_char(c: char, s: String) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '\"' | '\\' | '/' => continue_string_state(c, s),
        'b' => continue_string_state('\u{8}', s),
        'f' => continue_string_state('\u{c}', s),
        'n' => continue_string_state('\n', s),
        'r' => continue_string_state('\r', s),
        't' => continue_string_state('\t', s),
        'u' => ([].vec(), TokenizerState::ParseUnicodeChar(ParseUnicodeCharState { s: s.to_owned(), unicode: 0, index: 0 })),
        _ => transfer_state([JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec(), TokenizerState::ParseString(s), c)
    }
}

fn continue_unicode_state(i: u32, state: ParseUnicodeCharState) -> (Vec<JsonToken>, TokenizerState) {
    let new_unicode = state.unicode | (i << (3 - state.index) * 4);
    match state.index {
        3 => {
            let c = char::from_u32(new_unicode);
            match c {
                Some(c) => continue_string_state(c, state.s),
                None => panic!("invalid hex")
            }
        },
        0..=2 => {
            ([].vec(), TokenizerState::ParseUnicodeChar(ParseUnicodeCharState { s: state.s, unicode: new_unicode, index: state.index + 1 }))
        },
        _ => panic!("invalid index")
    }
}

fn tokenize_unicode_char(c: char, state: ParseUnicodeCharState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => continue_unicode_state(u32::from(c) - CP_0, state),
        'a'..='f' => continue_unicode_state(u32::from(c) - CP_SMALL_A + 10, state),
        'A'..='F' => continue_unicode_state(u32::from(c) - CP_CAPITAL_A + 10, state),
        _ => transfer_state([JsonToken::ErrorToken(ErrorType::InvalidHex)].vec(), TokenizerState::ParseString(state.s), c),
    }
}

fn tokenize_zero(c: char, s: Sign) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => tokenize_invalid_number(c),
        '.' => ([].vec(), TokenizerState::ParseFracBegin(IntegerState { s: s, m: 0 })),
        'e' | 'E' => ([].vec(), TokenizerState::ParseExpBegin(ExpState { s: s, m: 0, fe: 0, es: Sign::Plus, e: 0 })),
        c if is_terminal_for_number(c) => transfer_state([JsonToken::Number(BigFloat { m: 0, e: 0 })].vec(), TokenizerState::Initial, c),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_integer(c: char, s: IntegerState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => ([].vec(), TokenizerState::ParseInt(s.add_digit(c))),
        '.' => ([].vec(), TokenizerState::ParseFracBegin(s)),
        'e' | 'E' =>  ([].vec(), TokenizerState::ParseExpBegin(s.to_exp_state())),
        c if is_terminal_for_number(c) => transfer_state([JsonToken::Number(s.to_big_float())].vec(), TokenizerState::Initial, c),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_frac_begin(c: char, s: IntegerState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => {
            let float_state = s.to_float_state();
            ([].vec(), TokenizerState::ParseFrac(float_state.add_digit(c)))
        },
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_frac(c: char, s: FloatState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => ([].vec(), TokenizerState::ParseFrac(s.add_digit(c))),
        'e' | 'E' => ([].vec(), TokenizerState::ParseExpBegin(s.to_exp_state())),
        c if is_terminal_for_number(c) =>transfer_state([JsonToken::Number(s.to_big_float())].vec(), TokenizerState::Initial, c),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_minus(c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0' => ([].vec(), TokenizerState::ParseZero(Sign::Minus)),
        '1'..='9' => ([].vec(), TokenizerState::ParseInt(start_number(Sign::Minus, c))),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_exp_begin(c: char, mut s: ExpState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => ([].vec(), TokenizerState::ParseExp(s.add_digit(c))),
        '+' => ([].vec(), TokenizerState::ParseExpSign(s)),
        '-' => ([].vec(), {
            s.es = Sign::Minus;
            TokenizerState::ParseExpSign(s)
        }),
        c if is_terminal_for_number(c) =>transfer_state([JsonToken::Number(s.to_big_float())].vec(), TokenizerState::Initial, c),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_exp(c: char, s: ExpState) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '0'..='9' => ([].vec(), TokenizerState::ParseExp(s.add_digit(c))),
        c if is_terminal_for_number(c) =>transfer_state([JsonToken::Number(s.to_big_float())].vec(), TokenizerState::Initial, c),
        _ => tokenize_invalid_number(c),
    }
}

fn tokenize_invalid_number(c: char) -> (Vec<JsonToken>, TokenizerState) {
    transfer_state([JsonToken::ErrorToken(ErrorType::InvalidNumber)].vec(), TokenizerState::Initial, c)
}

fn tokenize(input: String) -> Vec<JsonToken> {
    let mut state = TokenizerState::Initial;
    let mut res = [].vec();
    for c in input.chars() {
        let tokens;
        (tokens, state) = state.push(c);
        res.extend(tokens);
    }
    res.extend(state.end());
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

    use crate::{tokenize, JsonToken, ErrorType, BigFloat};

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

    #[test]
    #[wasm_bindgen_test]
    fn test_integer() {
        let result = tokenize(String::from("0"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:0, e: 0 })]);

        let result = tokenize(String::from("-0"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:0, e: 0 })]);

        let result = tokenize(String::from("1234567890"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:1234567890, e: 0 })]);

        let result = tokenize(String::from("-1234567890"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:-1234567890, e: 0 })]);

        let result = tokenize(String::from("[0,1]"));
        assert_eq!(&result, &[JsonToken::ArrayBegin, JsonToken::Number(BigFloat { m:0, e: 0 }), JsonToken::Comma, JsonToken::Number(BigFloat { m:1, e: 0 }), JsonToken::ArrayEnd]);

        let result = tokenize(String::from("001"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber), JsonToken::ErrorToken(ErrorType::InvalidNumber), JsonToken::Number(BigFloat { m:1, e: 0 }),]);

        let result = tokenize(String::from("-"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber)]);

        let result = tokenize(String::from("-{}"));
        assert_eq!(&result, &[JsonToken::ErrorToken(ErrorType::InvalidNumber), JsonToken::ObjectBegin, JsonToken::ObjectEnd]);

        let result = tokenize(String::from("9007199254740991"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:9007199254740991, e: 0 })]);

        let result = tokenize(String::from("9007199254740992"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:9007199254740992, e: 0 })]);

        let result = tokenize(String::from("9007199254740993"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:9007199254740993, e: 0 })]);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_float() {
        let result = tokenize(String::from("0.01"));
        assert_eq!(&result, &[JsonToken::Number(BigFloat { m:1, e: -2 })]);
    }

}