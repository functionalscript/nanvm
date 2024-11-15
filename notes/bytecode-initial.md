I think we should start with the current DJS file. We can assume that one DJS file is a function that returns an object. If we skip `import` statements, we can see that it looks like a body of the function:

```js
import m1 from "module1.d.mjs"
import m2 from "module2.d.mjs"

const a = { m2 }
const b = [3, m1]

export default { a, b }
```

can be seen as

```js
const f = (...args) => {
    const _0 = { m2: args[1] }
    const _1 = [3, args[0]]
    return { a: _0, b: _1 }
}

import m1 from "module1.d.mjs"
import m2 from "module2.d.mjs"
export default f(m1, m2)
```

In this case, we can parse a module without parsing `module1.d.mjs` and `module1.m.js`.

Every function has two arrays, which can be referenced from a function body:
- args, an immutable array.
- locals, works as a stack.

## Module Type

```ts
type Property = [string, Expression]

type Expression =
    [`localRef`, number] |
    [`argRef`, number] |
    [`value`, number|string|bool|null]
    [`object`, Property[]]
    [`array`, Expression[]]

type Body = {
    local: Expression[],
    return: Expression
}

type Module = {
    import: string[],
    body: Body,
}
```

Then a body of this function 

```js
const f = (a0, a1) => {
    const c0 = { m2: a1, f: true }
    const c1 = [3, a0]
    return { a: c0, b: c1 }
}
```

should be represented as:

```json
{
    "local": [
        ["object", [
            ["m2", ["argRef", 1]],
            ["f", ["value", true]]
        ]],
        ["array", [
            ["value", 3],
            ["argRef", 0]
        ]]
    ],
    "return": ["object", [
        ["a", ["localRef", 0]],
        ["b", ["localRef", 1]]
    ]]
}
```

## Rust Module Type

```rust
type Property = (string, Expression);

enum Expression {
    local_ref(uint32),
    arg_ref(uint32),
    value(JSAny),
    object(Vec<Property>),
    array(Vec<Expression>),
}

struct Body {
    local: Vec<Expression>,
    return: Expression,
};

struct Module {
    import: Vec<string>,
    body: Body,
};
```

## Example of a JSON representation of the DJS module

```json
{
    "import": ["module1.d.mjs", "module2.d.mjs"],
    "body": {
        "local": [
            ["object", [
                ["m2", ["argRef", 1]],
                ["f", ["value", true]]
            ]],
            ["array", [
                ["value", 3],
                ["argRef", 0]
            ]]
        ],
        "return": ["object", [
            ["a", ["localRef", 0]],
            ["b", ["localRef", 1]]
        ]]
    }
}
```

## Byte Code

This format is designed for fast and straightforward serialization and doesn't depend on a particular VM implementation.

**Requirements:** 
- VM serializer/deserializer should be very simple.
    - string: UTF16
    - number: in a binary format
    - bigint: in a binary format
    - len: u32
- the byte code doesn't know anything about importing modules or I/O functions.
- the byte code shouldn't contain syntax sugar.
- serialized in a byte array so we can save it into a file. One byte is one unit.
- least-significant byte first.

```rust
struct Array<T> {
    len: u32,
    array: [T; self.len],
}
 
type String = Array<u16>;

// LSB first.
type BigUInt = Array<u64>;

type Object = Array<(String, Any)>;

// This is the main structure for serialization.
type Code = Array<u8>;

struct Function {
    length: u32
    code: Code
}

// This structure is not for serialization because
// a serialized module should resolve all imports.
struct Module {
    import: Array<String>
    code: Code
}
```

|type|any           |tag|                       |                             |
|----|--------------|---|-----------------------|-----------------------------|
|JSON|null          | 00|                       |                             |
|    |number        | 01|u64                    |                             |
|    |false         | 02|                       |                             |
|    |true          | 03|                       |                             |
|    |string        | 04|String                 |                             |
|    |object        | 05|Object                 |                             |
|    |array         | 06|Array<Any>             |                             |
|DJS |bigint+       | 07|BigUInt                |                             |
|    |bigint-       | 08|BigUInt                |                             |
|    |local_ref     | 09|u32                    |consts[i]                    |
|FJS |arg_ref       | 0A|u32                    |args[i]                      |
|    |undefined     | 0B|                       |                             |
|    |function      | 0C|Function               |the last constant is a return|
|    |...           |   |                       |                             |

## Architecture

Because FunctionalScript is a subset of JavaScript, we can use third-party JavaScript engines to bootstrap our parser, which is written in FunctionalScript, without circular dependencies. In Rust, we only need to implement a generic byte code deserializer that reads byte code and invokes VM API functions.

`Deno` is a good candidate because it's written on Rust and can be added as `DevDependency`: https://crates.io/crates/deno

### Build Stage:

Development dependencies:
- Deno
    
Source files:
- `parser.f.cjs` is a generic FunctionalScript parser that generates byte code.
- `selfparse.f.cjs` parses `parser.f.cjs` and generates byte code.
- `build.rs`
- `parser.rs`
- `bc_serializer.rs`
- `vm_api.rs`
- `vm.rs`

Build steps:
1. The build script `build.rs` starts `Deno` with `selfparse.f.cjs` and generates `parser.f.cjs.bc` temporary binary file.
2. During compiling, `parser.rs` includes `parser.f.cjs.bc` as an array of bytes. See https://doc.rust-lang.org/std/macro.include_bytes.html
3. Test `parser.rs` by parsing `parser.f.cjs` using VM and ensure the byte code is the same as in the array that contains `parser.f.cjs.bs`.

### Run-Time

No run-time dependencies.

1. Initialization: send a parser byte code to a deserializer that invokes `VM API`.
2. Parse: call loaded parser in VM.
