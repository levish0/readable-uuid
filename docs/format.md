# blake3-v1

This is a lossy alias protocol, not UUID encoding, encryption, or a unique key.

1. Parse input with `uuid::Uuid::try_parse`. Hash its 16 `as_bytes()` bytes,
   in canonical UUID/network order. Case and accepted UUID string wrappers do not matter.
2. Unkeyed BLAKE3 hashes the ASCII bytes `readable-uuid/blake3-v1`, a zero byte,
   then the 16 UUID bytes. Use the BLAKE3 XOF stream from byte offset zero.
3. Read consecutive unsigned little-endian 32-bit candidates from that stream.
   For dictionary length N, let L = floor(2^32 / N) * N, calculated in 64 bits.
   Reject candidates x >= L. Otherwise select dictionary[x % N]. Rejected
   candidates consume four bytes, produce no word, and never reset the stream.
4. Repeat until the requested word count is reached, allowing repeated words.
   Join using the separator. Dictionary, count and separator are not hashed.
   Increasing count therefore preserves all previous words.

All targets use the same integer widths, byte order, and frozen dictionaries.
Package upgrades must not change this protocol or replace a versioned dictionary.
A new dictionary requires a new identifier. Custom list order is significant;
callers must retain it. Built-in files have one word per line; fingerprint tests
hash the canonical LF-terminated representation, independent of checkout EOLs.

English lists are project-curated common words under the repository MIT license.
`short-v1` was initially selected from `english-v1` with length <=4 and then
frozen as its own file. `nature-v1` covers animals, plants, weather and landscapes.
These are not BIP39 lists, natural-language sentences, or a profanity guarantee.
No runtime dictionary parsing, random state, I/O or dictionary copying is needed.

The word-sequence space is N^k and `combination_bits()` returns k*log2(N).
Actual information is bounded by the UUID input and its generation scheme.
Under a uniform-output approximation, the birthday collision probability among m
labels is approximately 1-exp(-m*(m-1)/(2*N^k)). For the 553-word default list,
four words and one million identifiers give approximately 99.5%; six words give
approximately 0.00175%. These are estimates, not measured guarantees. Keep UUIDs
for uniqueness, lookups, authentication and authorization.

Separators are nonempty ASCII punctuation/spaces, and words lowercase ASCII,
so distinct word sequences cannot collapse through ambiguous concatenation.
Counts and separators are bounded to avoid accidentally allocating huge labels.

`tests/vectors.json` contains frozen outputs and list fingerprints. Tests must
compare against it, not regenerate it during normal checks. `examples/vectors.rs`
is a maintainer aid for a deliberate new format, never an upgrade repair command.
