# Rust run-time types

```rust
enum Ref {
    ArgRef(...)
    ConstRef(...)
}

// plus
impl Add for Ref {
   fn add(self, b) -> Self { ... } 
}

fn my_js_code() {
   let a = Ref ...;
   let b = Ref ...;
   let r = a + b;
}
```

```rust
struct Type { ... }
struct TypedRef {
   untyped: Ref
   type: Type
}

// plus
impl TypeRef {
   fn add(self, b: Self) -> Result<Self> { ... }
   fn add_(self, b: Self) -> Self { ... } 
}
```
