/** @type {(v: bigint) => bigint} */
const size = v => {
    let r = 0n
    while (v > 0) {
        v >>= 1n
        r++
    }
    return r
}

/** @type {(v: bigint) => string} */
const hex = v => v.toString(16)

/**
 * @typedef {{
 *  readonly name: string
 *  readonly range: string
 *  readonly size: bigint
 *  readonly['total range']: string
 *  readonly['total size']: bigint
 * }} TableRow
 */

/** @type {(a: readonly[TableRow[], bigint], b: readonly[string, bigint]) => readonly[TableRow[], bigint]} */
const row = ([acc, total], [name, v]) => {
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

/** @typedef {readonly[string, bigint]} Row */

/** @typedef {readonly[string, bigint, bigint]} Row3 */

/** @type {(_: Row3) => Row} */
const multi = ([name, a, b]) => [`${name} ${a}x${b}`, 1n << (a * b)]

/** @typedef {readonly Row[]} RowArray*/

/** @type {(name: string) => (x: RowArray) => void} */
const print = name => x => {
    console.group(name)
    console.table(x.reduce(row, [[], 0n])[0])
    console.groupEnd()
}

/** @type {RowArray} */
const json = [
    ['+Inf', 1n],
    ['NaN', 1n],
    ['-Inf', 1n],
    ['bool', 2n],
    ['null', 1n],
    ['stringArrayIndex', (1n << 32n) - 1n],
    ['&string', (1n << 45n) - 1n],
    ['&object', (1n << 45n) - 1n],
]

print('JSON')(json)

print('FunctionalScript')([
    ...json,
    ['undefined', 1n],
    .../** @type {readonly Row3[]} */([
        ['string1', 1n, 16n],
        ['string2', 2n, 16n],
        ['string3', 3n, 16n],
        ['string4', 4n, 12n],
        ['string5', 5n, 10n],
        ['string6', 6n, 8n],
        ['string7', 7n, 7n],
        ['string8', 8n, 6n],
        ['string9', 9n, 5n],
        ['string10', 10n, 5n]
    ]).map(multi),
    ['bigInt', 1n << 51n],
])

console.group('Array Index')

console.log({ a: 0, [-1]: 1 })
console.log({ a: 0, [0]: 1 })
console.log({ a: 0, [2 ** 32 - 2]: 1 })
console.log({ a: 0, [2 ** 32 - 1]: 1 })
console.log({ a: 0, [2 ** 32]: 1 })

console.groupEnd()

export default {}