#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

# NOTE: The `ANDROID_SDK_ROOT` must be defined and it's typically
# `$HOME/Android/Sdk`. After adding it, you may have to close all VS Code
# instances.
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:?"ANDROID_SDK_ROOT env. var. missing!"}"
RUN_IN_CONTAINER="${RUN_IN_CONTAINER:?"RUN_IN_CONTAINER env. var. missing!"}"

source ./public.env

ANDROID_COMPILE_SDK_VERSION="${ANDROID_COMPILE_SDK_VERSION:?"ANDROID_COMPILE_SDK_VERSION env. var. missing!"}"
ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION:?"ANDROID_NDK_VERSION env. var. missing!"}"

if [[ "${RUN_IN_CONTAINER}" = "1" ]]; then
  CONTAINER_COMMAND="${1}"

  # NOTE: This avoids the common occurrence of changing `Containerfile` and
  # forgetting to call build and Docker/Podman caching should only do
  # anything for build if `Containerfile` changes.
  "${CONTAINER_COMMAND}" build \
    --tag lop \
    --build-arg ANDROID_COMPILE_SDK_VERSION="${ANDROID_COMPILE_SDK_VERSION}" \
    --build-arg ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION}" \
    --file ./Containerfile .

  # NOTE: The `exec` trick avoids the need for an additional wrapper script when
  # using container.
  exec "${CONTAINER_COMMAND}" run --rm lop
fi

# NOTE: Exit-trap and its related logic after container-check because it doesn't
# work with an `exec`.
ROOT_DIR="$(realpath "$(pwd)")"

function on-exit-trap {
  local EXIT_CODE="$?"

  cd "${ROOT_DIR}"
  ./scripts/notify-user.sh

  if [[ "${EXIT_CODE}" = "0" ]]; then
    echo "============================SUCCEEDED============================" >&2
  else
    echo "=============================FAILED=============================" >&2
  fi
}

trap on-exit-trap EXIT

echo "Linting scripts..."
# NOTE: We ignore `SC2312` because it protects against discarding the exit
# status but we don't need this since we use `errexit` and fail script with any
# exit status other than success/zero.
# NOTE: `wiki-link-count` lists pages for error explanations and only works if
# we remove `--format=gcc` which we use because it allows us to directly go to
# the offending line.
# NOTE: We use `xargs` because `shellcheck` doesn't deal with directories.
find ./scripts -type f -print0 | xargs --null shellcheck \
  --external-sources \
  --check-sourced \
  --enable=all \
  --exclude=SC2312 \
  --severity=style \
  --wiki-link-count=1

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
cargo build --quiet --jobs "$(nproc)" --no-default-features
cargo test --quiet --jobs "$(nproc)" --no-default-features

echo "Building & testing release..." >&2
cargo build --quiet --jobs "$(nproc)" --no-default-features --release
cargo test --quiet --jobs "$(nproc)" --no-default-features --release

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
# NOTE: We disable `` because it requires global knowledge and ensuring that
# only a single must-use is used. Which is just an invitation for errors.
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
  -A clippy::multiple-crate-versions \
  -A clippy::double-must-use \

echo "Building Linux x86_64..." >&2
cargo build --quiet --target x86_64-unknown-linux-gnu --release

ANDROID_NDK_PATH="${ANDROID_SDK_ROOT}/ndk/${ANDROID_NDK_VERSION}"
ANDROID_AR="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"

echo "Building Android aarch64..." >&2
AARCH64_COMPILER_AND_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android${ANDROID_COMPILE_SDK_VERSION}-clang"
# SEE: https://github.com/rust-embedded/cross/blob/2f1ef07fdaf92ba31e6d6ce0ab4c5dca63ca0aa7/docker/Dockerfile.aarch64-linux-android#L26
export CC_aarch64_linux_android="${AARCH64_COMPILER_AND_LINKER}"
export CXX_aarch64_linux_android="${AARCH64_COMPILER_AND_LINKER}"
export AR_aarch64_linux_android="${ANDROID_AR}"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${AARCH64_COMPILER_AND_LINKER}"
cargo build --quiet --target aarch64-linux-android --release

echo "Building Android x86_64..." >&2
X86_64_COMPILER_AND_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android${ANDROID_COMPILE_SDK_VERSION}-clang"
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

echo "Bulding Linux..."
flutter build linux >/dev/null

echo "Bulding APK..."
flutter build apk --debug >/dev/null
flutter build apk --profile >/dev/null
flutter build apk --release >/dev/null
