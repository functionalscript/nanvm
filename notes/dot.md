# Dot

```ts
const x = {
   f: a => a * 2
}

// 6
const v = x.f(3)

const lazy_number = a => ({
   add: b => a + b
   sub: b => a - b
})

const my5 = lazy_number(5)

// 8
const my5_3 = my_5.add(3)

const my_5_lazy_add = my_5.add

// 12
const my5_7 = my_5_lazy_add(7)

const ar = [3, 5]

// 2
const ar_len = ar.len()

const ar_len_lazy = ar.len

const ar_len_ = ar_lazy_len()

```
