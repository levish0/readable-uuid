# Format

The library uses one fixed 65,536-entry codebook.

1. Parse the UUID into its 16 canonical bytes.
2. Split the bytes into eight ordered pairs.
3. Use the first byte of each pair as an index into the 256 modifiers.
4. Use the second byte as an index into the 256 nouns.
5. Render `modifier + CapitalizedNoun` and join the eight codewords.

For example, bytes `55 0e` select modifier 85 and noun 14. Decoding reverses
the eight pairs and returns the original UUID bytes.

Canonical output is lowerCamelCase. Decoding accepts ASCII case variations,
but does not trim input or normalize separators. A phrase must contain exactly
eight codewords separated by the configured separator.

The ordered source files are
[`modifiers.txt`](../wordlists/modifiers.txt) and
[`nouns.txt`](../wordlists/nouns.txt). Their order is part of the
format and must not change. `build.rs` requires 256 unique, sorted, lowercase
ASCII entries in each file and verifies all 65,536 lowercase combinations are
unique.

The format has no hash, encryption or checksum. Every UUID round-trips without
a database, but replacing a valid codeword can decode to a different UUID.
