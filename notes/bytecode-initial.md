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

Function body commands:

```ts
type Body = { local: Expression[], result: Expression }
type ExpressionType = `localRef` | `argRef` | `object` | `array` | `const`
type Expression =
    [`localRef`, number] |
    [`argRef`, number] |
    [`const`, number|string|bool|null]
    [`object`, Property[]]
    [`array`, Expression[]]
type Property = [string, Expression] 
```
