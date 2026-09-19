# Initial library — 2026-09-19

## Architecture

- Rust core remains the root crate. `bindings/node` is a thin N-API cdylib and
  contains every JavaScript development dependency/configuration, including pnpm,
  `.nvmrc`, ESLint, Prettier, tests and comparative benchmarks.
- Root Cargo default members exclude the binding. Rust-only checks need no Node.
- `just -> cargo xtask` owns crate publication and all npm build/pack/publication.
  No JavaScript build/release orchestration scripts. npm output is generated in
  `bindings/node/pkg-npm`; source metadata lives in `package.template.json`.
- `pnpm fmt` / `fmt:check` operate only inside `bindings/node`, never parent paths.

## Public contract

- BLAKE3-v1: domain prefix + canonical UUID bytes, XOF little-endian u32 rejection
  sampling. Dictionaries are frozen in content and order, separately from package
  versions. More words preserve the previous word prefix.
- English-only built-ins: english-v1 (553), short-v1 (191), nature-v1 (280).
- Default four words with hyphens. Labels are lossy, may collide, and are not keys.
- Rust: readable_uuid, ReadableUuid builder, format / format_str / write_into.
  Custom lists are borrowed and validated once. write_into appends.
- Node: readableUuid, readableUuidBatch, wordSets. Batch options validated once.
- Shared golden vectors in tests/vectors.json; never regenerate during checks.

## Validation and limits

- Rust tests (7 + doctest), clippy, rustfmt; Node shared-vector/error tests;
  ESLint/Prettier; native Windows x64 build; local npm tarball installation passed.
- crates.io publish dry-run passed with --allow-dirty. Nothing published/pushed.
- Five-platform CI configuration exists, but remote CI and Linux/macOS native
  builds have not run. Full npm release dry-run cannot pass without those artifacts.
  Publication checks artifact version, target and Git revision before staging.
- docs/performance.md records Rust and Node baselines, exact dependencies, corpus,
  machine and measurement limits. human-id random generation is a separate group
  from UUID conversion. Seeded unique-names-generator has weaker input mapping.

## User constraints

- Rust is the priority; English only; std is acceptable. No browser/CLI/reversal.
- Keep repository root free of JS package-manager/lint/format configuration.
- Follow good responsibility boundaries rather than copying sevenmark's layout.
- Write readable code with descriptive names, blank lines and expanded JSON data.
