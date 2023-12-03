## Memory Management

### Glossary

- `manager` a structure which manages a block of memory.
- `object` an object which can be stored in dynamic memory. Each object should implement the [Object](./object.rs#L10) trait.
- `block` an area of memory which can store an object. Each block contains a `header` and an `object`.