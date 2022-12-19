#!/bin/bash

# NOTE(ERREXIT-FIRST-THING-FOR-AVOIDING-SILENT-INITIALIZATION-ERRORS)
set -o errexit
source ./scripts/_base.sh

IS_IN_CONTAINER="${IS_IN_CONTAINER:?"IS_IN_CONTAINER env. var. missing!"}"
if [[ "${IS_IN_CONTAINER}" != "0" && "${IS_IN_CONTAINER}" != "1" ]]; then
  echo "Invalid 'IS_IN_CONTAINER' env. var. (${IS_IN_CONTAINER})" >&2
  echo "Expected ('0' or '1')" >&2
  exit 1
fi

# NOTE: To avoid calling exit trap twice since container calls back to script.
if [[ "${IS_IN_CONTAINER}" = "0" ]]; then
  function on-exit-trap {
    local EXIT_CODE="$?"

    # NOTE(SOUND-NOTIFICATION)
    mpv \
        --really-quiet \
        \
        /usr/share/sounds/freedesktop/stereo/phone-incoming-call.oga \
        \
        2>/dev/null \
      || echo "can't play sound notification and that's fine." >&2

    if [[ "${EXIT_CODE}" = "0" ]]; then
      echo "================CONTINUOUS-INTEGRATION SUCCEEDED================" >&2
    else
      echo "================CONTINUOUS-INTEGRATION FAILED================" >&2
    fi
  }

  trap on-exit-trap EXIT
fi

if [[ "$#" != "0" ]]; then
  echo "Invalid no. of arguments: this script only takes env. vars." >&2
  exit 1
fi
if [[ \
  "${0}" != "./scripts/continuous-integration.sh" && \
  "${0}" != "${PWD}/scripts/continuous-integration.sh"
  ]]; then
  echo "Invalid script-file (${0})" >&2
  exit 1
fi

# NOTE: The `ANDROID_SDK_ROOT` must be defined and it's typically
# `$HOME/Android/Sdk`. After adding it, you may have to close all VS Code
# instances.
ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:?"ANDROID_SDK_ROOT env. var. missing!"}"
if [[ ! -d "${ANDROID_SDK_ROOT}" ]]; then
  echo "Invalid 'ANDROID_SDK_ROOT' env. var. (${ANDROID_SDK_ROOT})" >&2
  echo "Expected path to exist" >&2
  exit 1
fi

# NOTE: Use of `getpots` was considered but eliminated due to very little value
# and its major drawback of seeming to encourage mysterious single-letter flags.
# SEE: https://devdocs.io/bash/bourne-shell-builtins#getopts
# SEE: https://github.com/abbaspour/auth0-bash/blob/9606b477dee6b89dbe913a230d4dbba60d4356ab/tenant/debug.sh
CONTAINER_COMMAND="${CONTAINER_COMMAND:?"CONTAINER_COMMAND env. var. missing!"}"
if [[ \
  "${CONTAINER_COMMAND}" != "podman" && \
  "${CONTAINER_COMMAND}" != "docker" \
]]; then
  echo "Invalid 'CONTAINER_COMMAND' env. var. (${CONTAINER_COMMAND})" >&2
  echo "Expected ('podman' or 'docker')" >&2
  exit 1
fi
PRE_COMMIT_CHECK="${PRE_COMMIT_CHECK:?"PRE_COMMIT_CHECK env. var. missing!"}"
if [[ "${PRE_COMMIT_CHECK}" != "0" && "${PRE_COMMIT_CHECK}" != "1" ]]; then
  echo "Invalid 'PRE_COMMIT_CHECK' env. var. (${PRE_COMMIT_CHECK})" >&2
  echo "Expected ('0' or '1')" >&2
  exit 1
fi
RUN_IN_CONTAINER="${RUN_IN_CONTAINER:?"RUN_IN_CONTAINER env. var. missing!"}"
RUN_IN_CONTAINER="${RUN_IN_CONTAINER:?"RUN_IN_CONTAINER env. var. missing!"}"
if [[ "${RUN_IN_CONTAINER}" != "0" && "${RUN_IN_CONTAINER}" != "1" ]]; then
  echo "Invalid 'RUN_IN_CONTAINER' env. var. (${RUN_IN_CONTAINER})" >&2
  echo "Expected ('0' or '1')" >&2
  exit 1
fi
if [[ "${IS_IN_CONTAINER}" = "1" && "${RUN_IN_CONTAINER}" = "1" ]]; then
  echo "Invalid 'IS_IN_CONTAINER' (${IS_IN_CONTAINER}) and" >&2
  echo "'RUN_IN_CONTAINER' (${RUN_IN_CONTAINER}) env. var. combination" >&2
  exit 1
fi
if [[ "${PRE_COMMIT_CHECK}" = "1" && "${RUN_IN_CONTAINER}" != "1" ]]; then
  echo "Invalid 'PRE_COMMIT_CHECK' (${PRE_COMMIT_CHECK}) and" >&2
  echo "'RUN_IN_CONTAINER' (${RUN_IN_CONTAINER}) env. var. combination" >&2
  exit 1
fi

source ./public.env

ANDROID_SDK_CMDLINE_TOOLS_VERSION="${ANDROID_SDK_CMDLINE_TOOLS_VERSION:?"ANDROID_SDK_CMDLINE_TOOLS_VERSION env. var. missing!"}"
ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384="${ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384:?"ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384 env. var. missing!"}"
ANDROID_BUILD_TOOLS_VERSION="${ANDROID_BUILD_TOOLS_VERSION:?"ANDROID_BUILD_TOOLS_VERSION env. var. missing!"}"
ANDROID_COMPILE_SDK_VERSION="${ANDROID_COMPILE_SDK_VERSION:?"ANDROID_COMPILE_SDK_VERSION env. var. missing!"}"
ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION:?"ANDROID_NDK_VERSION env. var. missing!"}"

if [[ "${RUN_IN_CONTAINER}" = "1" ]]; then
  # NOTE: We use exclusive execution instead of building different images for 2
  # main reasons:
  # 1. The main way to build different/independent images is by using randomized
  # tags but this may produce too many images that podman/docker can't cleanup
  # automatically and cause the storage to fill unnecessarily quickly.
  # 2. There would still be an inherent race-condition and unexpected behavior
  # where project files only copied/used late in the image build process.

  # NOTE(EXCLUSIVE-SCRIPT-EXECUTION)
  LOCK_FILE="/tmp/lop-continuous-integration-container.lock"
  LOCK_FD="4243"
  # NOTE(LOCK-FD-HARDCODED-SINCE-IT-ONLY-WORKS-THIS-WAY)
  exec 4243>|"${LOCK_FILE}"
  if ! flock --exclusive --nonblock "${LOCK_FD}"; then
    echo "Continuous-integration (with container) script already running somewhere else!" >&2
    exit 1
  fi

  "${CONTAINER_COMMAND}" build \
    --tag lop \
    \
    --build-arg PRE_COMMIT_CHECK="${PRE_COMMIT_CHECK}" \
    --build-arg ANDROID_SDK_CMDLINE_TOOLS_VERSION="${ANDROID_SDK_CMDLINE_TOOLS_VERSION}" \
    --build-arg ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384="${ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384}" \
    --build-arg ANDROID_BUILD_TOOLS_VERSION="${ANDROID_BUILD_TOOLS_VERSION}" \
    --build-arg ANDROID_COMPILE_SDK_VERSION="${ANDROID_COMPILE_SDK_VERSION}" \
    --build-arg ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION}" \
    \
    --file ./Containerfile \
    .
  "${CONTAINER_COMMAND}" run --rm lop

  exit 0
fi

# NOTE(EXCLUSIVE-SCRIPT-EXECUTION)
LOCK_FILE="/tmp/lop-continuous-integration.lock"
LOCK_FD="4445"
# NOTE(LOCK-FD-HARDCODED-SINCE-IT-ONLY-WORKS-THIS-WAY)
exec 4445>|"${LOCK_FILE}"
if ! flock --exclusive --nonblock "${LOCK_FD}"; then
  echo "Continuous-integration script already running somewhere else!" >&2
  exit 1
fi

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

pushd "./boa" >/dev/null

function boa-cargo() {
  cargo --quiet "$@"
}
function boa-cargo-build() {
  # SEE: https://doc.rust-lang.org/cargo/commands/cargo-build.html#feature-selection
  # NOTE(CARGO-BUILD)
  boa-cargo build --jobs "$(nproc)" --no-default-features "$@"
}
function boa-cargo-test() {
  boa-cargo test --jobs "$(nproc)" --no-default-features "$@" >/dev/null
}

echo >&2
echo "Cleaning for Rust using 'cargo clean' skipped as it slows build" >&2
echo "significantly and incremental Rust builds are so reliable and we use" >&2
echo "tooling/linting that ensures that warnings omitted in incremental" >&2
echo "fail build." >&2
echo >&2

echo "Checking boa formatting..." >&2
boa-cargo fmt --all -- --check

export RUST_BACKTRACE=1

echo "Building & testing boa (debug for host)..." >&2
boa-cargo-build
boa-cargo-test

echo "Building & testing boa (release for host)..." >&2
boa-cargo-build --release
boa-cargo-test --release

unset RUST_BACKTRACE

echo "Linting boa..." >&2
# NOTE: We use all the lints available and make all warnings errors. And to keep
# up with this we should add new lint categories that are added in the future
# that can be found on the project's main GitHub page.
# SEE: https://github.com/rust-lang/rust-clippy
# NOTE: `clippy::nursery` is in development but used because it has some very
# useful lints and only its broken `redundant_pub_crate` is disabled.
# NOTE: We disable the following lint categories:
# 1. `clippy::multiple-crate-versions`: Since its errors are caused by upstream
# dependencies and we have little control over them.
# 2. `double-must-use`: Because it requires global knowledge and ensuring that
# only a single must-use is used. Which is just an invitation for errors.
boa-cargo clippy --quiet -- \
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

echo "Building boa (release for Linux x86_64)..." >&2
boa-cargo-build --target x86_64-unknown-linux-gnu --release

ANDROID_NDK_PATH="${ANDROID_SDK_ROOT}/ndk/${ANDROID_NDK_VERSION}"
ANDROID_AR="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"

echo "Building boa (release for Android aarch64)..." >&2
# NOTE(RUST-ANDROID-ENV-VARS-AARCH64)
AARCH64_COMPILER_AND_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android${ANDROID_COMPILE_SDK_VERSION}-clang"
# SEE: https://github.com/rust-embedded/cross/blob/2f1ef07fdaf92ba31e6d6ce0ab4c5dca63ca0aa7/docker/Dockerfile.aarch64-linux-android#L26
export CC_aarch64_linux_android="${AARCH64_COMPILER_AND_LINKER}"
export CXX_aarch64_linux_android="${AARCH64_COMPILER_AND_LINKER}"
export AR_aarch64_linux_android="${ANDROID_AR}"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${AARCH64_COMPILER_AND_LINKER}"
boa-cargo-build --target aarch64-linux-android --release

echo "Building boa (release for Android x86_64)..." >&2
# NOTE(RUST-ANDROID-ENV-VARS-X8664)
X86_64_COMPILER_AND_LINKER="${ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android${ANDROID_COMPILE_SDK_VERSION}-clang"
export CC_x86_64_linux_android="${X86_64_COMPILER_AND_LINKER}"
export CXX_x86_64_linux_android="${X86_64_COMPILER_AND_LINKER}"
export AR_x86_64_linux_android="${ANDROID_AR}"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="${X86_64_COMPILER_AND_LINKER}"
boa-cargo-build --target x86_64-linux-android --release

X86_64_DIR="${PWD}/../clod/android/app/src/main/jniLibs/x86_64"
mkdir --parents "${X86_64_DIR}"
ln --force --symbolic \
  ../../../../../../../boa/target/x86_64-linux-android/release/libboa.so \
  "${X86_64_DIR}/libboa.so"

popd >/dev/null
pushd "./clod" >/dev/null

echo >&2
echo "Cleaning for Flutter using 'flutter clean' skipped as it slows build" >&2
echo "significantly and incremental Flutter builds are so reliable and we" >&2
echo "use tooling/linting that ensures that warnings omitted in incremental" >&2
echo "fail build." >&2
echo >&2

echo "Check clod formatting..."
flutter format \
  --set-exit-if-changed \
  --output none \
  --line-length 80 \
  "${PWD}" \
  >/dev/null

echo "Linting clod..."
flutter analyze --fatal-infos --fatal-warnings >/dev/null

echo "Bulding clod (debug for Linux)..."
flutter build linux --debug >/dev/null

echo "Bulding clod (debug APK for Android)..."
flutter build apk --debug >/dev/null
