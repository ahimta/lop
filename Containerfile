# syntax=docker/dockerfile:1
# SEE: https://github.com/rust-lang/rust/blob/master/RELEASES.md
# SEE: https://github.com/rust-lang/rust/releases
# NOTE(DUPLICATE-RUST-VERSION)
ARG LOCAL_RUST_VERSION="1.66.0"
# SEE: https://hub.docker.com/_/rust
FROM "docker.io/library/rust:${LOCAL_RUST_VERSION}-slim-bullseye"
LABEL author="Abdullah Alansari <ahimta@gmail.com>"

# SEE: https://docs.docker.com/develop/develop-images/dockerfile_best-practices
# SEE: https://github.com/docker-library/golang/blob/d5ee0588aaa4a7be9bba3d1cb4b1abe0323b6442/1.17/bullseye/Dockerfile
# SEE: https://github.com/rust-lang/docker-rust/blob/4627bd25407065f8f8feafa11a33c46c51f759d8/1.56.1/buster/slim/Dockerfile
# SEE: https://github.com/docker-library/ruby/blob/49168590766ac3eb0ad286154b2e01760b79f4b2/3.0/bullseye/Dockerfile
# SEE: https://github.com/flutter/flutter/blob/570e39d38b799e91abe4f73f120ce494049c4ff0/dev/ci/docker_linux/Dockerfile

# NOTE: Multi-stage builds were considered but when trying to implement them
# they turned to have very little to no benefit and only increase complexity.

# NOTE: Alpine images were considered but dismissed as they provide no value for
# our use-cases. Especially as the image/container isn't used in production
# devices/environments.

# NOTE: Using BASH was considered but we sticked with the default (DASH/SH) as
# otherwise Podman complains about something related to the OCI image format.
# SEE: https://docs.docker.com/engine/reference/builder/#shell

# NOTE(SAFER-BASH-AGAINST-LAX-BEHAVIOR)
# NOTE: Most lightweight shells don't support `pipefail` so we must keep in mind
# that commands that use a pipe only fail if the last command fails.
# NOTE: We use short options since long options don't work with the base-image.
# 1. `-e` instead of `-o errexit` set first to catch errors with other `set`s.
# 2. `-C` instead of `-o noclobber`.
# 3. `-u` instead of `-o noglob`.
# 4. `-u` instead of `-o nounset`.
# SEE: https://devdocs.io/bash/the-set-builtin#set
# NOTE: We add a space after the first `set` as otherwise it'd fail.
ARG LOCAL_SET_SHELL_SAFE_OPTIONS="set -e ; set -Cfu"

# NOTE(SIMPLE-LOCALE-FOR-CONSISTENT-BEHAVIOR)
# SEE: https://unix.stackexchange.com/a/87763
ENV LANG="C"
ENV LANGUAGE="C"
ENV LC_ALL="C"

# NOTE: This is important to avoid build hanging waiting for user-input.
ENV DEBIAN_FRONTEND="noninteractive"

RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  echo Installing dependencies...; \
  apt-get update -qq --yes; \
  apt-get upgrade -qq --yes >/dev/null; \
  apt-get install -qq --yes --no-install-recommends \
  clang \
  cmake \
  git \
  lib32stdc++6 \
  lib32z1 \
  libgtk-3-dev \
  liblzma-dev \
  ninja-build \
  openjdk-11-jdk \
  pkg-config \
  shellcheck \
  tar \
  unzip \
  wget \
  xz-utils \
  >/dev/null; \
  apt-get autoremove -qq --yes; \
  apt-get autoclean -qq --yes; \
  apt-get clean -qq --yes; \
  rm --recursive --force /var/lib/apt/lists/*;

# NOTE: We drop all privileges as we no longer need them.
# NOTE: `--no-log-init` is to avoid a possibly rare case of disk exhaustion.
# SEE: https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  useradd --create-home --no-log-init --system lop;
USER lop:lop
WORKDIR /home/lop
ARG LOCAL_HOME="/home/lop"

ARG LOCAL_ANDROID_HOME="${LOCAL_HOME}/Android/Sdk"
# NOTE(ANDROID-OFFICIAL-ENV-VARIABLES)
# SEE: https://developer.android.com/studio/command-line/variables
ENV ANDROID_HOME="${LOCAL_ANDROID_HOME}"
ENV ANDROID_USER_HOME="${LOCAL_HOME}/.android"
ENV ANDROID_EMULATOR_HOME="${ANDROID_USER_HOME}"
ENV ANDROID_AVD_HOME="${ANDROID_EMULATOR_HOME}/avd"
ARG ANDROID_SDK_CMDLINE_TOOLS_VERSION
ARG ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384
ARG ANDROID_BUILD_TOOLS_VERSION
ARG ANDROID_COMPILE_SDK_VERSION
ARG ANDROID_NDK_VERSION
ARG LOCAL_ANDROID_CLI_TOOLS_ZIP_FILE_NAME="android-cmdline-tools.zip"
ARG LOCAL_ANDROID_CLI_TOOLS_ZIP_DIR_NAME="android-cmdline-tools"
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  echo Installing Android SDK/NDK...; \
  wget -qq --output-document="${LOCAL_ANDROID_CLI_TOOLS_ZIP_FILE_NAME}" \
  "http://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_CMDLINE_TOOLS_VERSION}_latest.zip"; \
  echo "${ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384}  ${LOCAL_ANDROID_CLI_TOOLS_ZIP_FILE_NAME}" | shasum --algorithm 384 --check --quiet --strict -; \
  unzip -qq "${LOCAL_ANDROID_CLI_TOOLS_ZIP_FILE_NAME}" -d "${LOCAL_ANDROID_CLI_TOOLS_ZIP_DIR_NAME}"; \
  rm "${LOCAL_ANDROID_CLI_TOOLS_ZIP_FILE_NAME}"; \
  # NOTE: This is the expected path as implied by this error message:
  # Error: Could not determine SDK root.
  # Error: Either specify it explicitly with --sdk_root= or move this package into its expected location: <sdk>/cmdline-tools/latest/
  mkdir --parents "${LOCAL_ANDROID_HOME}/cmdline-tools"; \
  mv "${LOCAL_ANDROID_CLI_TOOLS_ZIP_DIR_NAME}/cmdline-tools" "${LOCAL_ANDROID_HOME}/cmdline-tools/latest"; \
  rmdir "${LOCAL_ANDROID_CLI_TOOLS_ZIP_DIR_NAME}";
ENV PATH="${PATH}:${LOCAL_ANDROID_HOME}/cmdline-tools/latest/bin"
# SEE: https://developer.android.com/studio/command-line/sdkmanager
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  yes | sdkmanager --install "build-tools;${ANDROID_BUILD_TOOLS_VERSION}" >/dev/null; \
  yes | sdkmanager --install "ndk;${ANDROID_NDK_VERSION}" >/dev/null; \
  # NOTE(SOME-ANDROID-TOOLS-ONLY-SUPPORT-INSTALLING-LATEST)
  yes | sdkmanager --install "platform-tools" >/dev/null; \
  yes | sdkmanager --install "platforms;android-${ANDROID_COMPILE_SDK_VERSION}" >/dev/null; \
  yes | sdkmanager --licenses >/dev/null;

# SEE: https://docs.flutter.dev/release/breaking-changes
# SEE: https://docs.flutter.dev/development/tools/sdk/release-notes
# SEE: https://flutter.dev/docs/get-started/install/linux
ARG LOCAL_FLUTTER_SDK_VERSION="3.3.10"
# NOTE(SHA384-CMD): Can use `shasum --algorithm 384 <flutter-sdk-file-path>` to get checksum.
ARG LOCAL_FLUTTER_SDK_CHECKSUM_SHA384="c8e1974fa054837fcdb21b6612412e66a5799a958b1584c79b59b7d41f19b38cd22a01d6703f11e8a37cb95c026fa17f"
ARG LOCAL_FLUTTER_SDK_ROOT="${LOCAL_HOME}/flutter"
ARG LOCAL_FLUTTER_SDK_TAR_FILE_NAME="flutter-sdk.tar.xz"
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  echo Installing Flutter SDK...; \
  wget -qq --output-document="${LOCAL_FLUTTER_SDK_TAR_FILE_NAME}" \
  "https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${LOCAL_FLUTTER_SDK_VERSION}-stable.tar.xz"; \
  echo "${LOCAL_FLUTTER_SDK_CHECKSUM_SHA384}  ${LOCAL_FLUTTER_SDK_TAR_FILE_NAME}" | shasum --algorithm 384 --check --quiet --strict -; \
  tar xf "${LOCAL_FLUTTER_SDK_TAR_FILE_NAME}"; \
  rm "${LOCAL_FLUTTER_SDK_TAR_FILE_NAME}";
ENV PATH="${PATH}:${LOCAL_FLUTTER_SDK_ROOT}/bin"
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  flutter config --no-analytics >/dev/null; \
  dart --disable-analytics >/dev/null; \
  flutter precache >/dev/null; \
  yes | flutter doctor --android-licenses >/dev/null; \
  # NOTE: Only `Flutter` and `Android toolchain` need to be available. We can
  # enforce this easily (maybe using grep) programmatically. For now, this must
  # be checked manually whenever this file changes.
  flutter doctor;

# NOTE: This section is for optimization and doing as much as possible once.
# This optimization is very effective and reduces container runs significantly.
# FIXME: Build everything else (e.g., flutter) for faster builds.
COPY \
  --chown=lop:lop \
  \
  ./boa/Cargo.lock \
  ./boa/Cargo.toml \
  ./boa/rust-toolchain.toml \
  \
  /lop/boa/
WORKDIR /lop/boa
ARG LOCAL_ANDROID_NDK_PATH="${LOCAL_ANDROID_HOME}/ndk/${ANDROID_NDK_VERSION}"
ARG LOCAL_ANDROID_AR="${LOCAL_ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"
# NOTE(RUST-ANDROID-ENV-VARS-AARCH64)
ARG LOCAL_AARCH64_COMPILER_AND_LINKER="${LOCAL_ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android${ANDROID_COMPILE_SDK_VERSION}-clang"
ENV CC_aarch64_linux_android="${LOCAL_AARCH64_COMPILER_AND_LINKER}"
ENV CXX_aarch64_linux_android="${LOCAL_AARCH64_COMPILER_AND_LINKER}"
ENV AR_aarch64_linux_android="${LOCAL_ANDROID_AR}"
ENV CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${LOCAL_AARCH64_COMPILER_AND_LINKER}"
# NOTE(RUST-ANDROID-ENV-VARS-X8664)
ARG LOCAL_X86_64_COMPILER_AND_LINKER="${LOCAL_ANDROID_NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android${ANDROID_COMPILE_SDK_VERSION}-clang"
ENV CC_x86_64_linux_android="${LOCAL_X86_64_COMPILER_AND_LINKER}"
ENV CXX_x86_64_linux_android="${LOCAL_X86_64_COMPILER_AND_LINKER}"
ENV AR_x86_64_linux_android="${LOCAL_ANDROID_AR}"
ENV CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="${LOCAL_X86_64_COMPILER_AND_LINKER}"
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  mkdir src; \
  # NOTE: We have to include all public APIs here as otherwise Rust build fails
  # in `continuous-integration.sh` with a weird error indicating that these APIs
  # don't exist in `boa`.
  echo 'pub fn test() {' >> src/lib.rs; \
  echo 'println!("Hello, world!");' >> src/lib.rs; \
  echo '}' >> src/lib.rs; \
  echo 'pub fn get_tournaments() {' >> src/lib.rs; \
  echo 'println!("Hello, world!");' >> src/lib.rs; \
  echo '}' >> src/lib.rs; \
  # NOTE: We completely silence noisy next line stderr (`2>/dev/null`) but we
  # need to undo this in case of issues to know what's wrong. The noisiness is
  # due to the first cargo build doing all kinds of stuff and forcibly priting
  # info/progress into stderr.
  cargo --quiet build --no-default-features --jobs "$(nproc)" 2>/dev/null; \
  cargo --quiet test --jobs "$(nproc)" --no-default-features >/dev/null; \
  cargo --quiet build --no-default-features --jobs "$(nproc)" --release; \
  cargo --quiet test --jobs "$(nproc)" --no-default-features --release >/dev/null; \
  cargo --quiet build --no-default-features --jobs "$(nproc)" --target x86_64-unknown-linux-gnu --release; \
  cargo --quiet build --no-default-features --jobs "$(nproc)" --target aarch64-linux-android --release; \
  cargo --quiet build --no-default-features --jobs "$(nproc)" --target x86_64-linux-android --release; \
  rm --force --recursive src; \
  mkdir /tmp/boa-cached-build-files; \
  mv ./target /tmp/boa-cached-build-files/target;

COPY --chown=lop:lop . /lop
WORKDIR /lop
ARG PRE_COMMIT_CHECK
RUN \
  ${LOCAL_SET_SHELL_SAFE_OPTIONS}; \
  \
  if [ "${PRE_COMMIT_CHECK}" = "1" ]; then \
  # NOTE(GIT-RESET-FOR-PRE-COMMIT-CHECK): We don't do `git restore --staged .`
  # here because it discards changes about to be committed. This is important
  # for pre-commit checks and maybe even useful for other usecases.
  git restore .; \
  git submodule --quiet foreach --recursive 'git restore .'; \
  git clean -dx --force --quiet; \
  git submodule --quiet foreach --recursive 'git clean -dx --force --quiet'; \
  fi; \
  mv /tmp/boa-cached-build-files/target boa/target;

ENV ANDROID_NDK_VERSION="${ANDROID_NDK_VERSION}"
ENV ANDROID_COMPILE_SDK_VERSION="${ANDROID_COMPILE_SDK_VERSION}"
# NOTE: Fixed-values for environment variables here are significant.
ENV CONTAINER_COMMAND="podman"
ENV IS_IN_CONTAINER="1"
ENV PRE_COMMIT_CHECK="0"
ENV RUN_IN_CONTAINER="0"

# NOTE: We use `ENTRYPOINT` instead of `CMD` deliberately as it doesn't use a
# shell and doesn't allow using arbitrary commands that we probably don't
# support.
# SEE: https://docs.docker.com/engine/reference/builder/#entrypoint
ENTRYPOINT /lop/scripts/continuous-integration.sh
