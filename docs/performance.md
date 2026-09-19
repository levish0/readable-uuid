# Performance

## Node.js

Measured on 2026-09-19 with Node 24.12.0, Windows x64 and an AMD Ryzen 7 8845HS.
The native binding was built in release mode. Run `just npm-bench` after
`just js-install` and `just npm-build` to reproduce the comparison.

### UUID string to four words

| Implementation | Version | Median | p10–p90 |
| --- | --- | ---: | ---: |
| readable-uuid | 0.1.0 | 392 ns | 382–425 ns |
| wordhash | 1.0.1 | 1,193 ns | 1,174–1,217 ns |
| humanhash | 1.0.4 | 1,633 ns | 1,589–1,718 ns |

A 1,000-UUID readable-uuid batch measured 412 ns per UUID (398–475 ns p10–p90).
It was not faster than single calls on this corpus: array conversion and result
allocation are part of the measurement.

### Method and limits

- 4,096 precomputed canonical UUID strings: half v4, half v7 sharing one millisecond.
  SHA256 is used only to generate reproducible input outside the timed region.
- Three warmup rounds, followed by 15 rounds of 16,384 conversions. Execution
  order rotates between rounds. Results are consumed with a checksum.
- Parsing, N-API calls and returned strings are included. Installation, module
  loading and preconstructed dictionary objects are excluded.
- `humanhash` receives the UUID with hyphens removed; this adapter cost is included.
  `wordhash` hashes the UUID string directly. Only direct UUID-to-words APIs are
  measured.
- Dictionary sizes, word lengths, input validation and algorithms differ. This
  compares public APIs doing similar work, not isolated hash speeds or identical
  collision resistance. No claim of universal superiority is made.

The executable benchmark and machine-readable samples are in
[`bindings/node/bench`](../bindings/node/bench/). Timings vary by CPU, runtime,
load and corpus. The randomized rows will not produce repeatable names.

### Ecosystem context

The npm downloads API reported the following for 2026-09-12 through 2026-09-18:

| Package | Weekly downloads | Role |
| --- | ---: | --- |
| human-id | 3,765,131 | Widely used random readable IDs; not a benchmark target |
| unique-names-generator | 577,651 | Widely used configurable names; not a benchmark target |
| humanhash | 934 | Direct deterministic digest-to-words comparator |
| wordhash | 5 | Direct UUID-string-to-words comparator |

Popularity does not imply semantic equivalence. Only `humanhash` and `wordhash`
are included in the UUID conversion benchmark; the other packages are listed
only to show broader npm ecosystem usage.
The [saved snapshot](../bindings/node/bench/popularity.json) includes the exact
npm API URLs and dates. Refresh it with `pnpm --dir bindings/node run bench:popularity`.
Package documentation: [human-id](https://github.com/RienNeVaPlus/human-id),
[unique-names-generator](https://github.com/andreasonny83/unique-names-generator),
[humanhash](https://github.com/SEBv15/humanhash), [wordhash](https://github.com/mistersimon/wordhash).

## Rust

Initial local release-build measurements on the same machine:

| Operation | Estimate |
| --- | ---: |
| Reusable formatter, parsed UUID | 129 ns |
| Including UUID parsing | 171 ns |
| Reused output buffer | 121 ns |
| humanhash 0.2.2, bytes | 91 ns |
| humanhash 0.2.2, string | 513 ns |

Run `cargo bench -p readable-uuid --bench format -- --sample-size 20 --measurement-time 1 --warm-up-time 1`.
The Rust baseline uses one fixed UUID. It excludes construction and cold start.
Both produce four words, but readable-uuid uses BLAKE3 and 553 words while
humanhash uses FNV-1a and 2,048 words. The byte-input competitor was faster.
