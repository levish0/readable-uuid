# Custom dictionaries

Built-in word sets are the normal path. Use a custom dictionary when a service
needs its own vocabulary.

## Rust

```rust
use readable_uuid::{ReadableUuid, Uuid};

let words = ["cedar", "finch", "moss", "river"];
let formatter = ReadableUuid::builder()
    .custom_words(&words)
    .words(4)
    .build()?;

let label = formatter.format(&Uuid::nil());
```

The formatter borrows the slice, so keep the words alive while using it.
Validation runs once at `build()`.

## Node.js

```js
const { readableUuidBatch } = require('readable-uuid');

const labels = readableUuidBatch(uuids, {
  customWords: ['cedar', 'finch', 'moss', 'river'],
  words: 4,
});
```

Use either `wordSet` or `customWords`. Single calls validate custom words per
call; batch calls validate them once per batch. Invalid input throws.

Rules:

- 2–65,536 unique words, in the supplied order.
- 1–64 lowercase ASCII bytes per word.
- No trimming, sorting, case conversion or multilingual words.
- The same UUID, ordered list and options always produce the same label.

Keep the exact custom list and the original UUID. Labels are lossy and may
collide.
