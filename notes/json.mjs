import table from './table.mjs'

const { print } = table

/** @type {import('./table.mjs').RowArray} */
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

print(json)('JSON')

export default {
    json
}
