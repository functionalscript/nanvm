const size = v => {
    let r = 0n
    while (v > 0) {
        v >>= 1n
        r++
    }
    return r
}

const bits = ([acc, total], [a, b]) => {
    const ab = a * b
    const v = 1n << ab
    total += v
    acc.push({
        type: `${a}x${b}`,
        size: ab, num: v.toString('16'),
        total: total.toString('16'),
        'total size': size(total - 1n)
    })
    return [acc, total]
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
].reduce(bits, [[], 0n])[0])
