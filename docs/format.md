# `blake3-v1` format

This is a lossy display alias, not UUID encoding, encryption or a unique key.

1. Parse the UUID and use its 16 bytes in canonical order.
2. Hash `readable-uuid/blake3-v1`, a zero byte and the UUID bytes with unkeyed
   BLAKE3.
3. Read little-endian `u32` values from the XOF stream. For dictionary length
   `N`, reject values outside `floor(2^32 / N) * N`; otherwise select `value % N`.
4. Repeat until the requested word count and join the words with the separator.

Rejected values consume four bytes. The dictionary, word count and separator
are not hashed. Increasing the word count preserves previous words.

All targets use this protocol and the frozen built-in lists. A new list needs a
new identifier. Custom list order is part of the input and must be preserved.

Words are lowercase ASCII and separators are nonempty ASCII punctuation or
spaces. Dictionaries contain 2–65,536 entries with words of 1–64 bytes.

Keep the original UUID for identity, lookup and authorization. See
[custom dictionaries](custom-dictionaries.md) for the custom-list contract.
