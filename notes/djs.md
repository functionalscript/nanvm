# DJS

## Tasks

- [ ] Support for [shorter property definitions](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer#property_definitions)
  - [ ] Parser
  - [ ] Serializer
    ```js
    const a = ["x"]
    export default { a }
    ```
- [ ] string deduplication
  ```js
  const a = "Hello world!"
  export default { a, b: a }
  ```
- [ ] non-value types deduplication as an option
