I think we should start with the current DJS file. We can assume that one DJS file is a function that returns an object. If we skip `import` statements, we can see that it looks like a body of the function:

```js
const a = "a"
const b = 3
export default { a, b }
```

can be seen as

```js
const f = () => {
    const a = "a"
    const b = 3
    return { a, b }
}
export default f()
```

So, a minimal function contains multiple sequential statements and a `return` statement at the end. Each statement creates a named constant.

`import` should be outside the scope of the function.

```js
import m from "module.cjs"
const a = "a"
const b = [3, m]
export default { a, b }
```

will be interpreted as

```js
import m from "module.cjs"
export default () => {
    const a = "a"
    const b = [3, m]
    return { a, b }
}()
```

It should help as to introduce more complicated expressions into DJS. For example, this DJS code:

```js
import m from "module.cjs"
const a = "a" 
const b = [3 + 4, m[12 - 6]]
export default { a, b }
```

will be parsed as

```js
import m from "module.cjs"
export default () => {
    const a = "a" 
    const b = [3 + 4, m[12 - 6]]
    return { a, b }
}()
```
