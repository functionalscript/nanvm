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
const f = (...args) => {
    const _0 = { m2: args[1], f: true }
    const _1 = [3, args[0]]
    return { a: _0, b: _1 }
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
