# readable-uuid

Lossless UUIDs as eight memorable English codewords. Rust core with Node.js bindings.

```text
01e3071e-14c2-42ce-8835-a0b4be8afc03
↕
acorn-anchor-birch-dolphin-meadow-pebble-river-willow
```

## Rust

```toml
[dependencies]
readable-uuid = "0.1"
```

```rust
use readable_uuid::{decode, encode, Uuid};

let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let phrase = encode(&id);
assert_eq!(decode(&phrase)?, id);
```

Use a reusable codec to change the separator or append to an existing buffer:

```rust
use readable_uuid::ReadableUuid;

let codec = ReadableUuid::builder().separator(" ").build()?;
let phrase = codec.encode(&id);

let mut output = String::new();
codec.encode_into(&id, &mut output);
```

## Node.js

```ts
import { decode, encode } from 'readable-uuid';

const phrase = encode('550e8400-e29b-41d4-a716-446655440000');
const uuid = decode(phrase);
```

Also exports `encodeBatch` and `decodeBatch`.

## Format

The built-in codebook has 65,536 fixed English words. Each two-byte UUID chunk
is a big-endian index into the codebook, so all 128 bits round-trip in exactly
eight codewords.

`build.rs` validates the codebook's size, order and spelling, then generates a
compact offset table for direct access. The codebook contents and order are
frozen as part of the public format. Its provenance is documented with the
[word list](wordlists/README.md).

Decoding accepts ASCII case variations. There is no checksum: replacing one
valid codeword can produce another valid UUID. See the
[format specification](docs/format.md).

## Performance

Benchmarks compare both directions with Niceware and BIP39. See the
[methodology and current results](docs/performance.md).

## Development

```sh
just check
just bench
just js-install
just js-check
just npm-build
just npm-test
just npm-bench
just npm-pack
just release-dry
```

The root Cargo version is shared by crates.io and npm releases. See
[release instructions](docs/releasing.md).

MIT licensed.
