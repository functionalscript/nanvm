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

## 64-bit Byte Code

```
0_111_1111__1111_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 | -Inf
1_111_1111__1111_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 | +Inf
0_111_1111__1111_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0000 . 0000_0000__0000_0001 | NaN

Rest = 2^(64 - 11) - 3 = 2^53 - 3, 3 values: -Inf, +Inf, NaN.

Additional constants (4): undefined, null, true, false

    52 letters: 26: `a..z` + 26: `A..Z`
    2 symbols: `$`, `_`
    10 digits: `0..9`
    total: 64 = 6 bits

String:
    String8 x6 : 8 x 6 bit = 48 bit

    String7 x7 : 7 x 7 bit = 49 bit
    String6 x7 : 6 x 7 bit = 42 bit
    String5 x7 : 5 x 7 bit = 35 bit
    String4 x7 : 4 x 7 bit = 28 bit

    String3: 3 x 16 bit = 48 bit
    String2: 2 x 16 bit = 32 bit
    String1: 1 x 16 bit = 16 bit

    String0.

    StringU32: 32 bit, See 'array index'.

BigInt33:
```
