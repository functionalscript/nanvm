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

/** @typedef {readonly Row[]} RowArray*/

/** @type {(name: string) => (x: RowArray) => void} */
const print = name => x => {
    console.group(name)
    console.table(x.reduce(row, [[], 0n])[0])
    console.groupEnd()
}

/** @type {(v: unknown) => void} */
const printTypeof = v => console.log(`typeof ${v} = ${typeof v}`)

console.group('Types')

printTypeof(void 0)
printTypeof(true)
printTypeof(9)
printTypeof(NaN)
printTypeof(Infinity)
printTypeof(-Infinity)
printTypeof(-0)
printTypeof(null)
printTypeof('string')
printTypeof({})
printTypeof([5,7])

console.log(`Object.is(0, -0) = ${Object.is(0, -0)}`)
console.log(`[] instanceof Array = ${[] instanceof Array}`)

console.groupEnd()

console.group('Array Index')

console.log({ a: 0, [-1]: 1 })
console.log({ a: 0, [0]: 1 })
console.log({ a: 0, [2 ** 32 - 2]: 1 })
console.log({ a: 0, [2 ** 32 - 1]: 1 })
console.log({ a: 0, [2 ** 32]: 1 })

console.groupEnd()

export default {
    print,
}