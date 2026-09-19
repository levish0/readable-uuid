# readable-uuid

Lossless UUIDs as eight memorable English codewords, backed by Rust.

```ts
import { decode, decodeBatch, encode, encodeBatch } from 'readable-uuid';

const phrase = encode('550e8400-e29b-41d4-a716-446655440000');
const uuid = decode(phrase);

const phrases = encodeBatch(uuids);
const restored = decodeBatch(phrases);
```

The built-in codebook contains 65,536 fixed codewords and
preserves every UUID bit in eight lowerCamelCase entries. Decoding accepts ASCII
case variations.

Use `separator` to replace the default hyphen:

```ts
const phrase = encode(uuid, { separator: ' ' });
```

There is no checksum. A different valid codeword can decode to a different UUID.

Supports Node.js 20+ on the published native targets. See the
[repository](https://github.com/levish0/readable-uuid) for the Rust API, format
specification and release targets.
