import json from './json.mjs'
import table from './table.mjs'

const { print } = table

/** @typedef {readonly[string, bigint, bigint]} Row3 */

/** @type {(_: Row3) => import('./table.mjs').Row} */
const multi = ([name, a, b]) => [`${name} ${a}x${b}`, 1n << (a * b)]

print([
    ...json.json,
    ['undefined', 1n],
    .../** @type {Row3[]} */([
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
])('FunctionalScript')