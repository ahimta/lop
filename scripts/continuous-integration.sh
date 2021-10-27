#!/bin/bash

set -o errexit
set -o pipefail
set -o nounset

echo "Checking formatting..." >&2
cargo fmt --quiet --all -- --check

echo "Cleaning..." >&2
cargo clean

echo "Testing debug..." >&2
RUST_BACKTRACE=1 cargo run --quiet --jobs "$(nproc)"

echo "Testing release..." >&2
RUST_BACKTRACE=1 cargo run --quiet --jobs "$(nproc)" --release

echo "Linting..." >&2
# NOTE: We use all the lints available and make all warnings errors. And to keep
# up with this we should add new lint categories that are added in the future
# that can be found on the project's main GitHub page.
# SEE: https://github.com/rust-lang/rust-clippy
# NOTE: `clippy::nursery` is in development but used because it has some very
# useful lints and only its broken `redundant_pub_crate` is disabled.
cargo clippy --quiet -- \
  -D warnings \
  \
  -W clippy::all \
  -W clippy::correctness \
  -W clippy::suspicious \
  -W clippy::style \
  -W clippy::complexity \
  -W clippy::perf \
  -W clippy::pedantic \
  -W clippy::cargo \
  -W clippy::nursery \
  \
  -A clippy::redundant_pub_crate
