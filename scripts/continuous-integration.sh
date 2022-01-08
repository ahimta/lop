#!/bin/bash

# SEE: https://devdocs.io/bash/the-set-builtin#set
# NOTE: `errexit` set first to catch errors with other `set`s.
set -o errexit

set -o noclobber
set -o noglob
set -o nounset
set -o pipefail

ROOT_DIR="$(realpath "$(pwd)")"

function on-exit-trap {
  local EXIT_CODE="$?"

  "${ROOT_DIR}/scripts/notify-user.sh"

  if test "${EXIT_CODE}" -eq "0"; then
    echo "============================SUCCEEDED============================" >&2
  else
    echo "=============================FAILED=============================" >&2
  fi
}

trap on-exit-trap EXIT

echo "Linting scripts..."
# NOTE: `wiki-link-count` lists pages for error explanations and only works if
# we remove `--format=gcc` which we use because it allows us to directly go to
# the offending line.
# NOTE: We use `xargs` because `shellcheck` doesn't deal with directories.
find ./scripts -type f -print0 | xargs --null shellcheck \
  --check-sourced \
  --enable=all \
  --severity=style \
  --wiki-link-count=1 \
  --format=gcc

cd "${ROOT_DIR}/boa"

echo >&2
echo "Cleaning for Rust using 'cargo clean' skipped as it slows build" >&2
echo "significantly and incremental Rust builds are so reliable and we use" >&2
echo "tooling/linting that ensures that warnings omitted in incremental" >&2
echo "fail build." >&2
echo >&2

echo "Checking formatting..." >&2
cargo fmt --quiet --all -- --check

export RUST_BACKTRACE=1

echo "Building & testing debug..." >&2
# SEE: https://doc.rust-lang.org/cargo/commands/cargo-build.html#feature-selection
cargo run --quiet --jobs "$(nproc)" --no-default-features

echo "Building & testing release..." >&2
cargo run --quiet --jobs "$(nproc)" --no-default-features --release

unset RUST_BACKTRACE

echo "Linting..." >&2
# NOTE: We use all the lints available and make all warnings errors. And to keep
# up with this we should add new lint categories that are added in the future
# that can be found on the project's main GitHub page.
# SEE: https://github.com/rust-lang/rust-clippy
# NOTE: `clippy::nursery` is in development but used because it has some very
# useful lints and only its broken `redundant_pub_crate` is disabled.
# NOTE: We also disable `clippy::multiple-crate-versions` since this is caused
# by upstream dependencies and we have little control over it.
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
  -A clippy::redundant_pub_crate \
  -A clippy::multiple-crate-versions

# NOTE: The `ANDROID_SDK_ROOT` must be defined and it's typically
# `$HOME/Android/Sdk`. After adding it, you may have to close all VS Code
# instances.

# NOTE: We use `22.1.7171670` because it's the latest version that doesn't produce the `-lgcc` error.
# NOTE: Suppressed error of undefined variable since this is an environment variable.
# shellcheck disable=SC2154
ANDROID_NDK_PATH="${ANDROID_SDK_ROOT}/ndk/22.1.7171670"
ANDROID_AR="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"

echo "Building Android aarch64..." >&2
AARCH64_COMPILER_AND_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"
# SEE: https://github.com/rust-embedded/cross/blob/2f1ef07fdaf92ba31e6d6ce0ab4c5dca63ca0aa7/docker/Dockerfile.aarch64-linux-android#L26
export CC_aarch64_linux_android="${AARCH64_COMPILER_AND_LINKER}"
export CXX_aarch64_linux_android="${AARCH64_COMPILER_AND_LINKER}"
export AR_aarch64_linux_android="${ANDROID_AR}"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${AARCH64_COMPILER_AND_LINKER}"
cargo build --quiet --target aarch64-linux-android --release

echo "Building Android x86_64..." >&2
X86_64_COMPILER_AND_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android30-clang"
export CC_x86_64_linux_android="${X86_64_COMPILER_AND_LINKER}"
export CXX_x86_64_linux_android="${X86_64_COMPILER_AND_LINKER}"
export AR_x86_64_linux_android="${ANDROID_AR}"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="${X86_64_COMPILER_AND_LINKER}"
cargo build --quiet --target x86_64-linux-android --release

X86_64_DIR="${ROOT_DIR}/clod/android/app/src/main/jniLibs/x86_64"
mkdir --parents "${X86_64_DIR}"
ln --force --symbolic \
  ../../../../../../../boa/target/x86_64-linux-android/release/libboa.so \
  "${X86_64_DIR}/libboa.so"

cd "${ROOT_DIR}/clod"

echo >&2
echo "Cleaning for Flutter using 'flutter clean' skipped as it slows build" >&2
echo "significantly and incremental Flutter builds are so reliable and we" >&2
echo "use tooling/linting that ensures that warnings omitted in incremental" >&2
echo "fail build." >&2
echo >&2

echo "Linting Flutter..."
flutter analyze --fatal-infos --fatal-warnings >/dev/null

echo "Bulding APK..."
flutter build apk --debug >/dev/null
flutter build apk --profile >/dev/null
flutter build apk --release >/dev/null
