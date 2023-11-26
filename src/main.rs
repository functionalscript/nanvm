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
    ErrorToken(ErrorType),
}

#[derive(Debug)]
enum ErrorType {
    InvalidNumber,
    MissingQuotes,
    Eof
}

enum TokenizerState {
    Initial,
    ParseString(String),
    ParseEscapeChar(String),
    ParseUnicodeChar(String),
    ParseNumber(ParseNumberState),
    ParseOperator(String),
    ParseMinus,
    InvalidNumber,
    Eof
}

enum ParseNumberState {
    Zero,
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

fn tokenize_initial(c: char) -> (Vec<JsonToken>, TokenizerState) {
    match c {
        '1'..='9' => (vec![], TokenizerState::ParseNumber(start_number(c))),
        '\t' | '\n' | '\r' | ' ' => (vec![], TokenizerState::Initial),
        '"' => (vec![], TokenizerState::ParseString(String::from(""))),

        _ => todo!()
    }
}

fn tokenize_eof(state: &TokenizerState) -> (Vec<JsonToken>, TokenizerState) {
    match state {
        TokenizerState::Initial => (vec![], TokenizerState::Eof),
        TokenizerState::ParseString(_) | TokenizerState::ParseEscapeChar(_) | TokenizerState::ParseUnicodeChar(_) => (vec![JsonToken::ErrorToken(ErrorType::MissingQuotes)], TokenizerState::Eof),
        TokenizerState::InvalidNumber | TokenizerState::ParseMinus => (vec![JsonToken::ErrorToken(ErrorType::InvalidNumber)], TokenizerState::Eof),
        TokenizerState::ParseNumber(_) => todo!(),
        TokenizerState::ParseOperator(_) => todo!(),
        TokenizerState::Eof => (vec![JsonToken::ErrorToken(ErrorType::Eof)], TokenizerState::Eof),
    }
}

fn tokenize_next_char(c: char, state: &TokenizerState) -> (Vec<JsonToken>, TokenizerState) {
    match state {
        TokenizerState::Initial => tokenize_initial(c),
        _ => todo!()
    }
}

fn tokenize(input: String) -> Vec<JsonToken> {
    let mut state = TokenizerState::Initial;
    let mut res = vec![];
    for c in input.chars() {
        let (tokens, new_state) = tokenize_next_char(c, &state);
        res.extend(tokens);
        state = new_state;
    }
    tokenize_eof(&state);
    res
}

fn main() {
    let result = tokenize(String::from(""));
    println!("{:?}", result);
    //print!("{}", result.len());
    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}