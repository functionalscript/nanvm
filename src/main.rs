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

enum ParseNumberKind {
    Zero,
    Int,
    Dot,
    Frac,
    Exp,
    ExpPlus,
    ExpMinus,
    ExpDigits
}

struct ParseNumberState {
    kind: ParseNumberKind,
    value: String,
    s: Sign,
    m: i128,
    f: f64,
    es: Sign,
    e: f64
}

enum Sign {
    Plus,
    Minus
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
    todo!()
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
    print!("{}", result.len());
    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}