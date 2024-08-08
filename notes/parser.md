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
