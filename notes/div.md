# Division

```
[u8]

//  0,    1,    2
[0x01, 0x02, 0x03] // 0x030201 :BE
                   // 0b0000_0011_0000_0010_0000_0001
                   //          1  1
                   //          2  0         8
len: 3
log2: 0x12 = (len - 1) * 8 + (8 - leading_zeros(array[len - 1])

a/b

x = 0
loop {
  let log2_d = a.log2() - b.log2();

  if log2_d < 0 { return [x, a] }

  delta = a - (b << log2_d);

  if delta < 0 {
     log2_d -= 1
     if log2_d < 0 { return [x, a] } 
    delta = a - (b << log2_d-1)
  }

  x += 1 << log2_d;

  a = delta
}

```
