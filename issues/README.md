# Issues

- simplify JS types for easier use
  - remove `M` as a memory manager type parameter. The default memory manager will be thread-unsafe ref counter with GLOBAL alloc. Later, we may change it.
  - Make `JsAny` as a seprate type
- defining a type in JS in the form that provides
  1. types at compile-time
  2. validations in run-time   
