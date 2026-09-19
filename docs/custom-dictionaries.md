# Custom dictionaries

Start with a built-in `WordSet`. Custom dictionaries are an optional API for
service-specific vocabularies, available without Cargo feature flags or loaders.

## Rust

```rust
use readable_uuid::{ReadableUuid, Uuid};

let dictionary = ["cedar", "finch", "moss", "river"];
let formatter = ReadableUuid::builder()
    .custom_words(&dictionary)
    .words(4)
    .build()?;

let label = formatter.format(&Uuid::nil());
```

The formatter borrows the slice and its words. Keep them alive while using it.
Validation happens once at `build()`, and subsequent conversions reuse the list.
The four-word dictionary above is only an API example: four output words provide
just `4^4 = 256` combinations. Use a larger dictionary or a longer label for real
populations, and retain the UUID as the unique identifier.

## Node.js

```js
const { readableUuidBatch } = require('readable-uuid');

const labels = readableUuidBatch(uuids, {
  customWords: ['cedar', 'finch', 'moss', 'river'],
  words: 4,
});
```

Choose either `wordSet` or `customWords`. Single calls validate the custom list
each time; a batch validates it once for the whole batch. Invalid lists throw.

## Contract

- A list contains 2–65,536 unique words, in the supplied order.
- Each word contains 1–64 lowercase ASCII letters. Custom does not imply
  multilingual support. Empty, duplicate, non-ASCII and oversized words fail
  validation; no trimming, sorting or case conversion is applied.
- The same UUID, conversion protocol, ordered list and output options produce
  the same label in Rust and Node.js.
- The library preserves built-in versioned lists. For custom lists, the caller
  must retain the exact contents and order. Changes can change every label.

`MAX_DICTIONARY_WORDS` and `MAX_WORD_BYTES` expose the limits in Rust.
`WordTooLong(index)` identifies oversized entries. The length bound also limits
the formatter's output reservation; an arithmetic overflow is reported as
`OutputTooLong`, not an invalid dictionary size.
