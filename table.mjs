const size = v => {
    let r = 0n
    while (v > 1) {
        v >>= 1n
        r++
    }
    return r + v
}

const bits = a => b => {
    const ab = a * b
    const v = 1n << ab
    return [`${a}x${b}`, ab, v.toString('16'), size(v - 1n)]
}

console.table([
    [1n, 16n],
    [2n, 16n],
    [3n, 16n],
    [4n, 12n],
    [5n, 9n],
    [6n, 8n],
    [7n, 7n],
    [8n, 6n],
    [9n, 5n],
    [10n, 5n],
].map(([a, b]) => bits(a)(b)))
