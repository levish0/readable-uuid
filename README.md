# readable-uuid

Lossless UUIDs as eight memorable English codewords. Rust core with Node.js bindings.

```text
550e8400-e29b-41d4-a716-446655440000
↕
foggyBirch-lucidAcorn-tinyPearl-easyStool-plushBridge-equalHorse-foggyFence-ableAcorn
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

The built-in codebook has 65,536 entries, formed from 256 modifiers and 256
concrete nouns. Each two-byte UUID chunk selects one modifier and one noun, so
all 128 bits round-trip in exactly eight lowerCamelCase codewords.

The source lists are small and reviewable. `build.rs` validates their size,
order, spelling and all 65,536 combinations. Their contents and order are
frozen as part of the public format.

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
