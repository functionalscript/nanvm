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
    bits(1n)(16n),
    bits(2n)(16n),
    bits(3n)(16n),
    bits(4n)(12n),
    bits(5n)(9n),
    bits(6n)(8n),
    bits(7n)(7n),
    bits(8n)(6n),
    bits(9n)(5n),
    bits(10n)(5n),
])