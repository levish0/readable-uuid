# Fixed reversible codebook — 2026-09-20

## Architecture

- This entry supersedes the lossy BLAKE3, preset and custom-dictionary design
  recorded in the earlier 2026-09-19 entries.
- The only format is a fixed, sorted 65,536-word English codebook. Each UUID is
  split into eight big-endian `u16` values and encoded as exactly eight words.
- Decoding performs an allocation-free, ASCII-case-insensitive binary search.
  There is no checksum; a different valid word can decode to another UUID.
- `build.rs` validates `wordlists/codewords.txt` and generates a packed `u32`
  offset table. Runtime code includes the text and offsets instead of a large
  generated Rust string array.
- The word list is based on public-domain 12dicts 6.0.2 data, filtered with the
  MIT-licensed cuss list and reviewed additions. Provenance is documented in
  `wordlists/README.md`. Its content and order are public format data.
- The Rust Criterion target is `codec`; `format` is no longer used.

## Public API

- Rust exposes `encode`, `decode`, `ReadableUuid`, `CODEBOOK_SIZE` and
  `CODEWORDS_PER_UUID`. The builder configures only the separator.
- Node exposes `encode`, `decode`, `encodeBatch` and `decodeBatch`; its only
  option is `separator`.
- crates.io and npm continue to read the same version from the root Cargo
  package through `cargo xtask`.

## Validation

- Rust fmt, Clippy, six integration tests, the doctest and exhaustive
  case-variation round trips for all 65,536 codewords passed.
- TypeScript, Prettier, ESLint, N-API release build and four Node tests passed.
- `cargo package --allow-dirty` and `just npm-pack` passed. Nothing was
  published.
- On the recorded Windows machine, Node median encode/decode were 897 ns and
  3,556 ns. Rust estimates were 99 ns and 1,068 ns. See
  `docs/performance.md` for the comparison scope and peer results.
