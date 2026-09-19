# Releases

Rust is the primary package. `just publish-crates-dry` runs cargo publish's
packaging and compilation checks; `just publish-crates` uploads it.
For validation of uncommitted work use `just publish-crates-dry --allow-dirty`.

`Cargo.toml` at the workspace root is the only public version source. The
generated npm loader, all five platform packages and their artifact provenance
use that same version. `npm-build` reads it through `cargo metadata`, so there
is no second npm version to edit.

The root Cargo package is the source of the public package version. The Rust `xtask`
builder reads it and generates `bindings/node/pkg-npm/package.json` from
`package.template.json`. This generated directory contains only distributable
files. The binding crate itself is private, as is its pnpm development package.

## Node.js

All JS tooling, dependencies, lockfile and configuration live in `bindings/node`.
Use the version in its `.nvmrc` and the pnpm version in its `packageManager` field.
Consumers can install the published package with any npm-compatible client.
`pnpm fmt`, `pnpm fmt:check` and ESLint operate only inside the binding directory.
All build, pack and publish orchestration lives in Rust `xtask`; pnpm scripts
for these operations are only command aliases.

The supported native targets are Windows x64, Linux glibc x64/arm64, and macOS
x64/arm64. CI builds and tests each native runner. Browser and musl targets are
not included in this initial release.

1. Run `just js-install`, `just js-check`, `just npm-build`, `just npm-test`.
2. `just npm-pack` creates a local package tarball under `bindings/node/pkg-npm`.
   It includes the locally built binary and is useful for installation smoke tests.
3. Download all five native CI artifacts from the same source revision into
   `bindings/node/artifacts/`, with `.node` files and their matching `.json`
   provenance files directly inside that directory.
4. Run `just publish-npm-dry`. It checks that all binaries exist and match the
   configured targets, package version and Git revision, then packs the platform packages and loader package into
   `bindings/node/release/<version>/`. It does not publish or run package scripts.
5. Commit release sources, then run `just publish-npm`. It requires a clean Git
   worktree, repeats checks, and publishes platform packages before the loader.

Once the five native artifacts are available, `just release-dry` runs both the
crates.io and npm preflights. After it succeeds on a clean commit, `just release`
runs those preflights again, publishes `readable-uuid` to crates.io, and then
publishes the npm platform packages followed by the root loader package.

Keep the Rust package version, binding crate version, changelog and Cargo.lock
aligned. Refresh the pnpm lockfile when JS development dependencies change.
No npm version needs a manual bump: distribution metadata is generated from Cargo.

Publishing requires your registry credentials. Multi-package npm publication is
not atomic, and crates.io and npm cannot share a transaction. If the combined
release fails after crates.io succeeds, rerun the npm part for the same version;
never bump the version just to retry. Inspect already-published npm versions
before retrying because the Rust xtask stops at the first error. CI never
publishes automatically.
