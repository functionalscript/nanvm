use nanvm::tokenizer::{tokenize, TokenizerStateIterator};

fn main() {
    let s = "[0,1";
    let result = tokenize(s.to_string());
    println!("{:?}", result);

    let result = TokenizerStateIterator::new(s.chars());
    let result: Vec<_> = result.collect();
    println!("{:?}", result);

    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}
