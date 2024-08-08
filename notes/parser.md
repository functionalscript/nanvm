# Parser

```rust
trait Parser {
    fn new() -> Self;

    fn push(&mut self, c: char) -> i32 /* a number of errors */;

    fn end(self) -> ParsedModule;

    fn sync_parse(it: impl Iter<Item = char>) -> ParsedModule {
        /// ....
    }
}

MyParser::sync_parse("export default = { a: 5 }")
```

```rust
trait Parser {
    fn new() -> Self;

    fn push(&mut self, token: Token) -> i32 /* a number of errors */;

    fn end(self) -> ParsedModule;

    fn sync_parse(it: impl Iter<Item = char>) -> ParsedModule {
        /// ....
    }
}

MyParser::sync_parse("export default = { a: 5 }")
```

```rust
struct Parser { ... }

impl Parser {
    const fn new() -> Self { ... }

    const fn push(self, token: Token) -> Self { ... }

    const fn end(self) -> ParsedModule { ... }

    const fn sync_parse(it: &[char]) -> ParsedModule {
        /// ....
        while ... {
        }
    }
}

const MODULE: Parser = Parser::sync_parse(inline_file!("my.f.js"));

MyParser::sync_parse("export default = { a: 5 }")
```

