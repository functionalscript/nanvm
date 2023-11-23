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

/** @type {(f: () => void) => (name: string) => void} */
const group = f => name => {
    console.group(name)
    f()
    console.groupEnd()
}

/** @type {(x: RowArray) => (name: string) => void} */
const print = x => group(() => console.table(x.reduce(row, [[], 0n])[0]))

/** @type {(a: readonly unknown[]) => void} */
const printTypeTable = a => console.table(a.map(v => [v, typeof v]))

group(() => {
    printTypeTable([
        void 0,
        true,
        9,
        NaN,
        Infinity,
        -Infinity,
        -0,
        null,
        'string',
        {},
        [5, 7]
    ])
    console.log(`Object.is(0, -0) = ${Object.is(0, -0)}`)
    console.log(`[] instanceof Array = ${[] instanceof Array}`)
})('Types')

group(() => {
    console.log({ a: 0, [-1]: 1 })
    console.log({ a: 0, [0]: 1 })
    console.log({ a: 0, [2 ** 32 - 2]: 1 })
    console.log({ a: 0, [2 ** 32 - 1]: 1 })
    console.log({ a: 0, [2 ** 32]: 1 })
})('Array Index')

/** @typedef {readonly[string, (a: bigint, b: bigint) => bigint]} Op */

group(() => {
    const infinity = 0x7FF0_0000_0000_0000n
    const nan = 0x7FF8_0000_0000_0000n
    const negativeInfinity = 0xFFF0_0000_0000_0000n
    /** @type {(a: Op) => readonly[string, string]} */
    const f = ([name, op]) => [name, hex(op(op(infinity, nan), negativeInfinity))]
    console.table(/** @type {readonly Op[]} */([
        ['&', (a, b) => a & b],
        ['|', (a, b) => a | b],
    ]).map(f))
})('Number')

group(() => {
    /** @type {(t: unknown) => readonly[unknown, string]} */
    const f = t => [t, typeof t]
    console.table([
        f(15),
        f("Hello world!"),
        f(true),
        f([]),
        f({}),
        f(null),
    ])
})('typeof')

export default {
    print,
}