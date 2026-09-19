# Releases

The root `Cargo.toml` version is the only public version source. `xtask` uses
it for the crates.io package, npm loader, platform packages and artifact
provenance. `package.template.json` has no separate version.

1. Update the root Cargo version, `Cargo.lock` and `CHANGELOG.md`.
2. Run `just check-all` and `just npm-build`.
3. Commit and push. Wait for all five native targets: Windows x64, Linux glibc
   x64/arm64 and macOS x64/arm64.
4. Put the five `.node` files and matching provenance JSON files from that same
   revision in `bindings/node/artifacts/`.
5. Run `just release-dry`. Use `just release-dry --allow-dirty` only for local
   validation.
6. On the clean release commit, run `just release`.

`release-dry` checks both registries. `release` repeats those checks, publishes
the Rust crate, then the five npm platform packages and the npm loader. The
lower-level `publish-crates` and `publish-npm` commands remain available.

Both registries need credentials. They do not share a transaction. If crates.io
succeeds and npm fails, retry npm with the same version; do not bump it. npm can
also stop after some platform packages, so inspect existing versions before
retrying.
