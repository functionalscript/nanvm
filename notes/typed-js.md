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
trait TsType {}

struct Unknown();
impl TsType for Unknown {}

struct JSAny<D: Dealloc, T: TsType = Unknown> {
   ...,
   _p: PhantomData<T>,
}

struct Array<T: TsType>();

impl<D: Dealloc, T: TsType> JSAny<D, Array<T>> {
   fn at(self, i: i32) -> JSAny<D, T>;
}
```

## Combining

```rust
// `TSAny` has the same binary layout as `JSAny`, but `TSAny` also contains additional compile-time type information (restrictions). 
#[repr(transparent)]
struct TSAny<D, T: TsType = Unknown>(JSAny<D>, PhantomData<T>);
```

## Notes about JS

```js
const X = void 0
// typedef X === `undefined`
```
