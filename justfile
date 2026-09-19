set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

fmt:
    cargo fmt --all

# Rust-only checks; Node.js is not required.
check:
    cargo fmt --all --check
    cargo clippy -p readable-uuid -p xtask --all-targets -- -D warnings
    cargo test -p readable-uuid

test *args:
    cargo test -p readable-uuid {{args}}

bench *args:
    cargo bench -p readable-uuid --bench codec {{args}}

publish-crates-dry *args:
    cargo xtask publish-dry {{args}}

publish-crates *args:
    cargo xtask publish {{args}}

release-dry *args:
    cargo xtask release-dry {{args}}

release:
    cargo xtask release

js-install:
    pnpm --dir bindings/node install --frozen-lockfile

js-format:
    pnpm --dir bindings/node run fmt

js-check:
    pnpm --dir bindings/node run check
    cargo clippy -p readable-uuid-node --all-targets -- -D warnings

npm-build:
    cargo xtask npm-build

npm-test:
    pnpm --dir bindings/node run test

npm-bench:
    pnpm --dir bindings/node run bench

npm-pack:
    cargo xtask npm-pack

publish-npm-dry:
    cargo xtask npm-publish --dry-run

publish-npm:
    cargo xtask npm-publish

check-all: check js-check npm-build npm-test
