# Traits

```rust
struct Null();
trait Any {
    
}
trait Vm {
    type Object: Into<Any>;
    type Array: Into<Any>;
    type String: Into<Any>;
    type Bigint: Into<Any>;
    type Any: Any where double: Into<Any>, bool: Into<Any>, Null: Into<Any>;
    fn string(self, value: &[u16]) -> Self::String;
    fn array(self, value: &[Self::Any]) -> Self::Array;
    fn object(self, value: &[(Self::String, Self::Any)] -> Self::Object;
    fn bigint(self, value: (bool, &[u64])) -> Self::Bigint; 
}
```