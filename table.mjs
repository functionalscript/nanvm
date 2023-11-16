const size = v => {
    let r = 0n
    while (v > 0) {
        v >>= 1n
        r++
    }
    return r
}

const hex = v => v.toString('16')

const row1 = ([acc, total], [name, v]) => {
    total += v
    acc.push({
        name,
        range: hex(v),
        size: size(v),
        'total range': hex(total),
        'total size': size(total - 1n)
    })
    return [acc, total]
}

const row = (x, [name, a, b]) => row1(x, [name, 1n << (a * b)])

const print = name => x => {
    console.group(name)
    console.table(x.reduce(row, [[], 0n])[0])
    console.groupEnd()
}

const json = [
    ['+Inf', 0n, 0n],
    ['NaN', 0n, 0n],
    ['-Inf', 0n, 0n],
    ['bool', 1n, 1n],
    ['null', 0n, 0n],
    ['&string', 1n, 45n],
    ['&object', 1n, 45n],
]

print('JSON')(json)

const stringIndex = ['stringIndex', 1n, 32n]

print('JSON Extended')([
    ...json,
    stringIndex,
])

print('FunctionalScript')([
    ...json,
    ['undefined', 0n, 0n],
    ['string1', 1n, 16n],
    ['string2', 2n, 16n],
    ['string3', 3n, 16n],
    ['string4', 4n, 12n],
    ['string5', 5n, 10n],
    ['string6', 6n, 8n],
    ['string7', 7n, 7n],
    ['string8', 8n, 6n],
    ['string9', 9n, 5n],
    ['string10', 10n, 5n],
    ['bigInt', 1n, 51n],
    stringIndex,
])

console.group('Integer Index')

console.log({ a: 0, [-1]: 1 })
console.log({ a: 0, [0]: 1 })
console.log({ a: 0, [2 ** 32 - 2]: 1 })
console.log({ a: 0, [2 ** 32 - 1]: 1 })
console.log({ a: 0, [2 ** 32]: 1 })

console.groupEnd()