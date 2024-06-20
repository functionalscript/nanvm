I think we should start with the current DJS file. We can assume that one DJS file is a function that returns an objects. If we skip `imports` we can see that it looks like a body of the function:

```js
const a = "a"
const b = 3
export default { a, b }
```

can be seen as 
```
const f = () => {
    const a = "a"
    const b = 3
    return { a, b }
}
export default f()
```
