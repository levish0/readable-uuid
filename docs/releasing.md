# Releases

The root `Cargo.toml` version is the only public version source. `xtask` uses
it for both the crates.io package and the WebAssembly npm package.
`package.template.json` has no separate version.

1. Update the root Cargo version, `Cargo.lock` and `CHANGELOG.md`.
2. Run `just check-all` and `just npm-build`.
3. Run `just release-dry`. Use `just release-dry --allow-dirty` only for local
   validation.
4. Commit and push. Wait for the Rust and WebAssembly workflows.
5. On the clean release commit, run `just release`.

`release-dry` checks both registries. `release` repeats those checks, publishes
the Rust crate, then the npm package. The
lower-level `publish-crates` and `publish-npm` commands remain available.

Both registries need credentials. They do not share a transaction. If crates.io
succeeds and npm fails, retry npm with the same version; do not bump it. npm can
be retried independently because it is a single WebAssembly package.
