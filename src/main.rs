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

fn main() {

    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}