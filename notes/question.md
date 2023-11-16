I'm trying to understand how a JavaSript object sorts its properties. As far as I understood, from [ECMA262](https://262.ecma-international.org/6.0/#sec-ordinary-object-internal-methods-and-internal-slots-ownpropertykeys), the first properties are always an [integer index](https://262.ecma-international.org/6.0/#sec-object-type) properties. For example, if print these objects using Node.js, Deno, or Bun:

```js
console.log({ a: 0, [-1]: 1 })
console.log({ a: 0, [0]: 1 })
console.log({ a: 0, [2 ** 32 - 2]: 1 })
console.log({ a: 0, [2 ** 32 - 1]: 1 })
console.log({ a: 0, [2 ** 32]: 1 })
```

we will have

```
{ a: 0, '-1': 1 }
{ '0': 1, a: 0 }
{ '4294967294': 1, a: 0 }
{ a: 0, '4294967295': 1 }
{ a: 0, '4294967296': 1 }
```

It looks like an integer index is defined in the range `[0, 2^32-2]`. It matches the definition of an `array index`:

> An _array index_ is an integer index whose numeric value _i_ is in the range +0 ≤ _i_ < 2^32 - 1.

However, it's different from the definition of an `integer index`:

> An _integer index_ is a String-valued property key that is a canonical numeric String (see 7.1.16) and whose numeric value is either +0 or a positive integer ≤ 2^53−1.

So, my question is, should JavaScript engines use `[0, 2^53-1]` or ECMAScript 2015 should use `[0, 2^32-2]` for the definition of an _integer index_? Did I miss something?

https://stackoverflow.com/questions/77492134/definition-of-integer-index-in-javascript-ecmascript-2015
