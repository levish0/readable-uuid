# readable-uuid

Deterministic English word aliases for UUIDs. Rust core with Node.js bindings.

```js
const { readableUuid, readableUuidBatch, wordSets } = require('readable-uuid');
const label = readableUuid('550e8400-e29b-41d4-a716-446655440000', {
  wordSet: 'short-v1',
  words: 6,
  separator: '-',
});
```

Word sets: `english-v1` (553), `short-v1` (191), `nature-v1` (280).
Defaults: `english-v1`, four words, `-`. Built-ins need no custom dictionary.
Batch conversion validates options once per batch. Errors throw; a batch with
any invalid UUID throws.

For advanced usage, `customWords` accepts 2–65,536 unique lowercase ASCII words
of 1–64 bytes each, instead of `wordSet`. Preserve their exact order and contents
for stable output. See [custom dictionaries](https://github.com/levish0/readable-uuid/blob/main/docs/custom-dictionaries.md).

Labels are lossy and may collide. Keep the original UUID as the identifier.
Supports Node.js 20+ on the published native targets; not browsers.

See the [repository](https://github.com/levish0/readable-uuid) for the Rust API
and the frozen `blake3-v1` conversion specification.
