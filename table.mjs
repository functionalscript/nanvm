const size = v => {
    let r = 0n
    while (v > 0) {
        v >>= 1n
        r++
    }
    return r
}

const hex = v => v.toString('16')

const row = ([acc, total], [name, a, b]) => {
    const ab = a * b
    const v = 1n << ab
    total += v
    acc.push({
        name: name,
        x: `${a}x${b}`,
        size: ab, num: hex(v),
        total: hex(total),
        'total size': size(total - 1n)
    })
    return [acc, total]
}

const json = [
    ['+inf', 0n, 0n],
    ['NaN', 0n, 0n],
    ['-inf', 0n, 0n],
    ['bool', 1n, 1n],
    ['null', 0n, 0n],
    ['&string', 1n, 45n],
    ['&object', 1n, 45n],
]

const print = name => x => {
    console.log(name)
    console.table(x.reduce(row, [[], 0n])[0])
}

print('JSON')(json)

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
    ['stringIndex', 1n, 51n],
])
