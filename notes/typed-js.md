# Typed JS objects in Rust

Thin wrappers. Requirement:
- size and alignment MUST be the same as JSAny.

https://doc.rust-lang.org/nomicon/other-reprs.html

```rust
#[repr(transparent)]
struct Array(JSAny);
```
