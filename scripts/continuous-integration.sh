#!/bin/bash

set -o errexit
set -o pipefail
set -o nounset

ROOT_DIR="$(realpath "$(pwd)")"

echo "Linting scripts..."
# NOTE: `wiki-link-count` lists pages for error explanations and only works if
# we remove `--format=gcc` which we use because it allows us to directly go to
# the offending line.
shellcheck \
  --check-sourced \
  --enable=all \
  --severity=style \
  --wiki-link-count=1 \
  --format=gcc scripts/*

cd "${ROOT_DIR}/boa"

echo "Cleaning..." >&2
cargo clean

echo "Checking formatting..." >&2
cargo fmt --quiet --all -- --check

# NOTE: The `ANDROID_SDK_ROOT` must be defined and it's typically
# `$HOME/Android/Sdk`. After adding it, you may have to close all VS Code
# instances.

# NOTE: We use `22.1.7171670` because it's the latest version that doesn't produce the `-lgcc` error.
# NOTE: Suppressed error of undefined variable since this is an environment variable.
# shellcheck disable=SC2154
ANDROID_NDK_PATH="${ANDROID_SDK_ROOT}/ndk/22.1.7171670"

echo "Building Android aarch64..." >&2
AARCH64_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${AARCH64_LINKER}" cargo build --quiet --target aarch64-linux-android --release

echo "Building Android x86_64..." >&2
X86_64_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android30-clang"
CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="${X86_64_LINKER}" cargo build --quiet --target x86_64-linux-android --release

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

X86_64_DIR="${ROOT_DIR}/clod/android/app/src/main/jniLibs/x86_64"
mkdir --parents "${X86_64_DIR}"
ln --force --symbolic \
  ../../../../../../../boa/target/x86_64-linux-android/release/libboa.so \
  "${X86_64_DIR}/libboa.so"

cd "${ROOT_DIR}/clod"

echo "Cleaning Flutter build..."
flutter clean

echo "Linting Flutter..."
flutter analyze --fatal-infos --fatal-warnings

mkdir --parents "/tmp/hacky-path-for-strange-path-created-by-flutter"
ln --force --symbolic \
  "/tmp/hacky-path-for-strange-path-created-by-flutter" \
  "$(pwd)/android/?"

echo "Bulding APK..."
flutter build apk --debug
flutter build apk --profile
flutter build apk --release
