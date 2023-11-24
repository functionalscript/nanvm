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
    ErrorToken(String),
}

enum TokenizerState {
    Initial,
    ParseString(String),
    ParseEscapeChar(String),
    //ParseUnicodeChar(String), //todo: implement
    ParseNumber(ParseNumberState),
    ParseOperator(String),
    ParseMinus,
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

fn tokenize(input: String) -> Vec<JsonToken> {

    todo!()
}

fn main() {
    let res = tokenize(String::from(""));
    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}