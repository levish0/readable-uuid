# readable-uuid

Deterministic English word aliases for UUIDs, using BLAKE3.

## Rust

```toml
[dependencies]
readable-uuid = "0.1"
```

```rust
use readable_uuid::{readable_uuid, ReadableUuid, WordSet, Uuid};

let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let label = readable_uuid(&id); // english-v1, four words, hyphens

let formatter = ReadableUuid::builder()
    .word_set(WordSet::ShortV1)
    .words(6)
    .separator("-")
    .build()?;
let label = formatter.format(&id);
let same = formatter.format_str(&id.to_string())?;

let mut buffer = String::new();
formatter.write_into(&id, &mut buffer); // appends; clear to reuse
```

## Word sets

| Set          | Words | Characters per word | Bits per word |
|--------------|------:|---------------------|--------------:|
| `english-v1` |   553 | 3–9                 |          9.11 |
| `short-v1`   |   191 | 3–4                 |          7.58 |
| `nature-v1`  |   280 | 3–9                 |          8.13 |

Built-in sets work out of the box; no custom dictionary is required.
Options are validated once when building the formatter. Word count: 1–64;
separator: 1–32 ASCII punctuation or space characters.
For service-specific vocabularies, see [custom dictionaries](docs/custom-dictionaries.md).

Labels are **lossy and may collide**: retain the original UUID as the identifier.
Four default words represent about 36.45 bits of combinations, not 128 bits.
Use more words for larger populations; `combination_bits()` exposes the space.

The algorithm and each dictionary's contents/order are frozen independently of
package versions. See the [conversion specification](docs/format.md).

## Node.js

```js
const { readableUuid } = require("readable-uuid");
const label = readableUuid("550e8400-e29b-41d4-a716-446655440000", {
  wordSet: "short-v1",
  words: 6,
});
```

Also exports `readableUuidBatch` and `wordSets`. See [Node.js details](bindings/node/README.npm.md).

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
just publish-crates-dry
just publish-crates
just publish-npm-dry
just publish-npm
```

Native npm releases require all platform artifacts from CI; see
[release instructions](docs/releasing.md). Benchmarks compare UUID/string inputs
and buffer reuse against `humanhash`. Node benchmarks include `human-id`,
`unique-names-generator`, `humanhash` and `wordhash`; see [results](docs/performance.md).

MIT licensed, including the project-curated English word lists.
