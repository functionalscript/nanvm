```
subset: {010, 011 }
definition: 01?
bit_intersection: 010
bit_union: 011
superposition = bit_intersection ^ union: 001
mask = !superposition = 110
is(v) = (v & mask) == bit_intersection

restore `bit_union`:
    superposition = !mask: 001
    bit_union = bit_intersection ^ superposition: 011
    bit_union = bit_intersection ^ !mask: 011

union(a, b) = {
    bit_union: | {...a, ...b} =
        a.bit_union | b.bit_union =
        (a.bit_intersection ^ !a.mask) | (b.bit_intersection ^ !b.mask)
    bit_intersection: & {...a, ...b} = a.bit_intersection & b.bit_intersection,
}
```

|union|0|1|?|
|-----|-|-|-|
|0    |0| | |
|1    |?|1| |
|?    |?|?|?|

|intersection|0|1|?|
|------------|-|-|-|
|0           |0| | |
|1           |X|1| |
|?           |0|1|?|

## Union

| |definition|bi        |bu        |sp    |mask  |
|-|----------|----------|----------|------|------|
|a|00011?    |000110    |000111    |000001|111110|
|b|01?1??    |010100    |011111    |001011|110100|
|u|0??1??    |000100 and|011111 or |011011|100100|
|i|0X011?    |0X0110 or |0X0111 and|0X0001|1X1110|

| |definition|bi       |bu       |sp   |mask |
|-|----------|---------|---------|-----|-----|
|a|0011?     |00110    |00111    |00001|11110|
|b|0?1??     |00100    |01111    |01011|10100|
|u|0?1??     |00100 and|01111 or |01011|10100|
|i|0011?     |00110 or |00111 and|00001|11110|
