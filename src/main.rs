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

#[derive(Debug)]
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
    ErrorToken(ErrorType),
}

#[derive(Debug)]
enum ErrorType {
    UnexpectedCharacter,
    InvalidToken,
    InvalidNumber,
    MissingQuotes,
    Eof
}

enum TokenizerState {
    Initial,
    ParseKeyword(String),
    ParseString(String),
    ParseEscapeChar(String),
    ParseUnicodeChar(String),
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

struct Integer {
    s: Sign,
    m: u128,
}

enum Sign {
    Plus,
    Minus
}

fn digit_to_number(cp: u32) -> u128 {
    u128::from(cp - u32::from('0'))
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
        _ => panic!("unexpected operator")
    }
}

fn keyword_to_token(s: &String) -> JsonToken {
    match s.as_str() {
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
        '{' | '}' | '[' | ']' | ':' => ([operator_to_token(c)].vec(), TokenizerState::Initial),
        '-' => ([].vec(), TokenizerState::ParseMinus),
        _ => ([JsonToken::ErrorToken(ErrorType::UnexpectedCharacter)].vec(), TokenizerState::Initial)
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
        TokenizerState::ParseKeyword(_) => todo!(),
        TokenizerState::ParseString(_) => todo!(),
        TokenizerState::ParseEscapeChar(_) => todo!(),
        TokenizerState::ParseUnicodeChar(_) => todo!(),
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