# Typed JS objects in Rust

## Wrappers

Thin wrappers. Requirement:
- size and alignment MUST be the same as JSAny.

https://doc.rust-lang.org/nomicon/other-reprs.html

```rust
#[repr(transparent)]
struct Array(JSAny);
```

## Modifying JSAny

```rust
trait TypeScriptType { ... }
struct JSAny<D: Dealloc, T: TypeScriptType> {
   ...,
   _p: PhantomData<T>,
}

impl<D: Dealloc, T: TypeScriptType> JSAny<D, TSArray<T>> {
   fn at(self, i: i32) -> JSAny<D, T>;
}
```
