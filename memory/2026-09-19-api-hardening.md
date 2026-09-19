# API hardening and Dependabot — 2026-09-19

## Changes

- `WordSet` and `Error` are non-exhaustive. `WordSet::ALL` is a static slice so
  adding built-ins does not change its public type. Consumers use iter().copied().
- `MAX_DICTIONARY_WORDS` (65,536) and `MAX_WORD_BYTES` (64) are shared between
  build-time dictionary checks and runtime custom-list validation.
- Oversized words return `WordTooLong(index)`; capacity arithmetic overflow uses
  `OutputTooLong`. Existing BLAKE3-v1 rules, dictionaries and output remain unchanged.
- Existing frozen-output, list-fingerprint, UUID normalization and prefix tests
  were already present. Added dictionary-size and word-length boundary tests,
  checked built-in max-length metadata, and fixed 16-word golden vectors derived
  from the existing frozen 64-word values without regenerating previous outputs.
- Custom dictionaries remain a small optional API, not a Cargo feature flag.
  docs/custom-dictionaries.md explains borrowing, validation, list ownership,
  ASCII-only scope and the small combination space of toy dictionaries.

## Dependabot

- Retain pnpm 11.5.0. Do not downgrade based on the outdated supported-version
  table. Upstream support issue #14794 is closed; July 29 comments confirm success.
- Correct scopes: npm at /bindings/node, Cargo at / for the workspace, and
  github-actions at /. Weekly schedules retained; only explanatory comments changed.
- Live successful jobs verified for repository revision 568187d:
  npm 35446515113, Cargo 35446514850, GitHub Actions 35446515045.
- User merged Actions dependency updates during this task. Preserve those commits.

## Validation

- just check-all passed: rustfmt, Rust clippy, 9 Rust tests plus one doctest,
  scoped Prettier/ESLint, Windows native build and 3 Node test cases.
- Shared Rust/Node fixtures now contain 72 cases. Existing labels and dictionary
  fingerprints still pass unchanged.
- crates.io publication dry-run with --allow-dirty passed. No upload performed.
- Remote native workflow was queued when inspected; do not claim all-platform
  native validation based on the Dependabot job results.
