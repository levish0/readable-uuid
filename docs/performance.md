# Performance

Measured on 2026-09-19 with Node.js 24.12.0, Windows x64 and an AMD Ryzen 7
8845HS. Run `just npm-bench` after `just js-install` and `just npm-build` to
repeat the Node.js comparison.

## Node.js

| Implementation | Version | Median | p10–p90 |
| --- | --- | ---: | ---: |
| readable-uuid | 0.1.0 | 392 ns | 382–425 ns |
| wordhash | 1.0.1 | 1,193 ns | 1,174–1,217 ns |
| humanhash | 1.0.4 | 1,633 ns | 1,589–1,718 ns |

The 1,000-UUID batch measured 412 ns per UUID (398–475 ns p10–p90).

The benchmark uses 4,096 canonical UUID strings, half v4 and half v7. It runs
15 rounds of 16,384 conversions and includes parsing, N-API calls and returned
strings. Setup and module loading are excluded. Only direct UUID-to-word APIs
are compared; dictionary sizes and algorithms differ.

Results are in [`bindings/node/bench`](../bindings/node/bench/). Timings vary by
CPU, runtime and load.

## Rust

Initial local release-build measurements:

| Operation | Estimate |
| --- | ---: |
| Reusable formatter, parsed UUID | 129 ns |
| Including UUID parsing | 171 ns |
| Reused output buffer | 121 ns |
| humanhash 0.2.2, bytes | 91 ns |
| humanhash 0.2.2, string | 513 ns |

```sh
cargo bench -p readable-uuid --bench format -- --sample-size 20 --measurement-time 1 --warm-up-time 1
```

The Rust benchmark uses one fixed UUID and excludes construction and cold start.
