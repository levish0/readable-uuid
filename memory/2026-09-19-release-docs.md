# Release flow, benchmarks and documentation — 2026-09-19

## Decisions

- The root Cargo version is the only public version source. `cargo xtask
  release[-dry]` preflights both registries, then publishes crates.io before
  the npm platform packages and loader. Cross-registry publication is
  sequential, not atomic; retries keep the same version.
- The Node benchmark compares only direct UUID-to-word APIs: readable-uuid,
  humanhash and wordhash. Random name generators and npm popularity data are
  not benchmark targets.
- The public documentation is intentionally concise like README. `format.md`,
  custom dictionary rules, performance numbers and release commands retain
  only information needed to use or maintain the package.
- Node tests and benchmarks remain `.cjs` because the generated N-API loader is
  CommonJS. `eslint.config.mjs` is explicit ESM configuration. TypeScript is
  not used for tooling because it would add a runtime and build layer without
  changing the generated JS and declaration package.

## Validation

- Scoped Prettier, ESLint, Rust fmt/clippy/tests and Node tests passed after the
  documentation and benchmark cleanup.
