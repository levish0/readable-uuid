# Performance

Measured on Node.js 24.12.0 and Rust 1.98.1 on Windows x64 with an AMD Ryzen 7
8845HS. Lower is better.

## Node.js

The benchmark converts 4,096 canonical UUID strings to space-separated phrases
and back. Parsing, joining and WebAssembly calls are included.

| Implementation      |    Encode |     Decode | Words | Mean length |
|---------------------|----------:|-----------:|------:|------------:|
| readable-uuid 0.1.0 |     881 ns |   4,186 ns |     8 |       73.31 |
| niceware 4.0.0      |   1,495 ns |   3,738 ns |     8 |       71.89 |
| @scure/bip39 2.4.0  | 105,548 ns | 118,196 ns |    12 |       76.77 |

For batches of 1,000 UUIDs, readable-uuid measured 778 ns per encode and 4,068
ns per decode.

## Rust

Criterion measures one parsed UUID and includes phrase allocation unless marked
as reused.

| Implementation               | Encode |   Decode |
|------------------------------|-------:|---------:|
| readable-uuid 0.1.0          |  99 ns | 1,068 ns |
| niceware 1.0.0               |  79 ns | 1,315 ns |
| bip39 3.0.0                  | 526 ns | 1,140 ns |
| readable-uuid, reused buffer |  78 ns |        — |

BIP39 includes a four-bit checksum. readable-uuid and Niceware do not.

```sh
just bench
just npm-bench
```

Raw Node.js results are in
[`bindings/wasm/bench/results.json`](../bindings/wasm/bench/results.json).
