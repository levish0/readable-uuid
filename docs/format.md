# Format

The library uses one fixed 65,536-entry codebook.

1. Parse the UUID into its 16 canonical bytes.
2. Split the bytes into eight ordered pairs.
3. Read each pair as a big-endian `u16`.
4. Use that value as an index into the codebook.
5. Join the eight codewords with the configured separator.

For example, bytes `55 0e` select codeword 21,774. Decoding reverses the eight
indices and returns the original UUID bytes.

Canonical output is lowercase. Decoding accepts ASCII case variations, but
does not trim input or normalize separators. A phrase must contain exactly
eight codewords separated by the configured separator.

The ordered source file is [`codewords.txt`](../wordlists/codewords.txt). Its
order is part of the format and must not change. `build.rs` requires exactly
65,536 unique, sorted, lowercase ASCII entries of 3 to 12 bytes.

The format has no hash, encryption or checksum. Every UUID round-trips without
a database, but replacing a valid codeword can decode to a different UUID.
