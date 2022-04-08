# syntax=docker/dockerfile:1
# SEE: https://hub.docker.com/_/rust
FROM rust:1.57.0-slim-bullseye
LABEL author "Abdullah Alansari <ahimta@gmail.com>"

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

# NOTE: Using BASH was considered instead of default DASH/SH because Podman
# complains that they are not part of the OCI image format.
# SEE: https://docs.docker.com/engine/reference/builder/#shell

# NOTE(SAFER-BASH-AGAINST-LAX-BEHAVIOR)
# NOTE: Most lightweight shells don't support `pipefail` so we must keep in mind
# that commands that use a pipe only fail if the last command fail.
# NOTE: We use short options because long options don't with base-image.
# 1. `-e` instad of `-o errexit` set first to catch errors with other `set`s.
# 2. `-C` instad of `-o noclobber`.
# 3. `-u` instad of `-o noglob`.
# 4. `-u` instad of `-o nounset`.
# SEE: https://devdocs.io/bash/the-set-builtin#set
# NOTE: We add a space after the first `set` as otherwise it'd fail.
ARG SET_SHELL_SAFE_OPTIONS="set -e ; set -Cfu"

# NOTE(SIMPLE-LOCALE-FOR-CONSISTENT-BEHAVIOR)
# SEE: https://unix.stackexchange.com/a/87763.
ENV LANG C
ENV LANGUAGE C
ENV LC_ALL C

# NOTE: This is important to avoid build hanging waiting for user-input.
ENV DEBIAN_FRONTEND noninteractive

RUN ${SET_SHELL_SAFE_OPTIONS}; \
  echo Installing dependencies...; \
  apt-get update -qq --yes; \
  apt-get upgrade -qq --yes >/dev/null; \
  apt-get install -qq --yes --no-install-recommends \
  git \
  lib32stdc++6 \
  lib32z1 \
  openjdk-11-jdk \
  shellcheck \
  tar \
  unzip \
  wget \
  xz-utils \
  > /dev/null; \
  apt-get autoremove -qq --yes; \
  apt-get autoclean -qq --yes; \
  apt-get clean -qq --yes; \
  rm --recursive --force /var/lib/apt/lists/*;

# NOTE: We drop all privileges as we no longer need them.
# NOTE: `--no-log-init` is to avoid a possibly rare case of disk exhaustion.
# SEE: https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
RUN ${SET_SHELL_SAFE_OPTIONS}; \
  groupadd --system lop; \
  useradd --create-home --no-log-init --system --gid lop lop;
USER lop:lop
WORKDIR /home/lop
ARG HOME=/home/lop

# FIXME: Answer Stackoverflow questions after making sure everything works.
# SEE: https://stackoverflow.com/questions/65262340/cmdline-tools-could-not-determine-sdk-root
# SEE: https://stackoverflow.com/questions/17963508/how-to-install-android-sdk-build-tools-on-the-command-line

# NOTE: This is the latest version that seems to work with Rust.
ARG ANDROID_BUILD_TOOLS=29.0.2
ARG ANDROID_SDK_ROOT=$HOME/Android/Sdk
# SEE: https://developer.android.com/studio/index.html#downloads
ARG ANDROID_SDK_TOOLS=7583922
ARG ANDROID_SDK_TOOLS_CHECKSUM_SHA256=124f2d5115eee365df6cf3228ffbca6fc3911d16f8025bebd5b1c6e2fcfa7faf
ARG ANDROID_COMPILE_SDK_VERSION
ARG ANDROID_NDK_VERSION

RUN ${SET_SHELL_SAFE_OPTIONS}; \
  echo Installing Android SDK/NDK...; \
  wget -qq --output-document=android-sdk.zip \
  http://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_TOOLS}_latest.zip; \
  echo "${ANDROID_SDK_TOOLS_CHECKSUM_SHA256} android-sdk.zip" | sha256sum --check --quiet --strict -; \
  unzip -qq android-sdk.zip -d android-sdk; \
  rm android-sdk.zip; \
  # NOTE: This is the expected path as implied by this error message:
  # Error: Could not determine SDK root.
  # Error: Either specify it explicitly with --sdk_root= or move this package into its expected location: <sdk>/cmdline-tools/latest/
  mkdir --parents ${ANDROID_SDK_ROOT}/cmdline-tools; \
  mv android-sdk/cmdline-tools ${ANDROID_SDK_ROOT}/cmdline-tools/latest; \
  rmdir android-sdk;
ENV PATH "${PATH}:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin"
# SEE: https://developer.android.com/studio/command-line/sdkmanager
RUN ${SET_SHELL_SAFE_OPTIONS}; \
  echo y | sdkmanager "platforms;android-${ANDROID_COMPILE_SDK_VERSION}" >/dev/null; \
  echo y | sdkmanager "platform-tools" >/dev/null; \
  echo y | sdkmanager "build-tools;${ANDROID_BUILD_TOOLS}" >/dev/null; \
  echo y | sdkmanager "ndk;${ANDROID_NDK_VERSION}" >/dev/null; \
  echo y | sdkmanager --licenses >/dev/null;

ARG FLUTTER=2.5.3
ARG FLUTTER_CHECKSUM_SHA256=b32d04a9fa5709326b4e724e0de64ff1b2b70268f89dd3c748e6360ac937fe01
ARG FLUTTER_SDK_ROOT=$HOME/flutter

# SEE: https://flutter.dev/docs/get-started/install/linux
RUN ${SET_SHELL_SAFE_OPTIONS}; \
  echo Installing Flutter SDK...; \
  wget -qq --output-document=flutter-sdk.tar.xz \
  https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${FLUTTER}-stable.tar.xz; \
  echo "${FLUTTER_CHECKSUM_SHA256} flutter-sdk.tar.xz" | sha256sum --check --quiet --strict -; \
  tar xf flutter-sdk.tar.xz; \
  rm flutter-sdk.tar.xz; \
  # NOTE: Just check that the file was extracted in the right location.
  ls ${FLUTTER_SDK_ROOT} >/dev/null;
ENV PATH "${PATH}:${FLUTTER_SDK_ROOT}/bin"
RUN ${SET_SHELL_SAFE_OPTIONS}; \
  flutter config --no-analytics >/dev/null; \
  dart --disable-analytics >/dev/null; \
  flutter precache >/dev/null; \
  yes | flutter doctor --android-licenses >/dev/null; \
  # NOTE: Only `Flutter` and `Android toolchain` need to be available. We can
  # enforce this, easily (maybe using grep), programmatically so this must be
  # checked manually whenever this file changes.
  flutter doctor;

COPY --chown=lop:lop . /lop
WORKDIR /lop
RUN ${SET_SHELL_SAFE_OPTIONS}; \
  # NOTE: We don't do `git restore --staged .` here because it discards changes
  # about to be committed. This is important for pre-commit checks and maybe
  # even useful for other usecases.
  git restore .; \
  git clean -dx --force --quiet; \
  # NOTE: This is mostly just to kick of installing all Rust tooling and project
  # dependencies once and avoiding repeating this for each run/container.
  (cd boa && cargo --quiet check --no-default-features --jobs "$(nproc)");

# NOTE: Only `ANDROID_SDK_ROOT` is an official Android environment-variables.
# SEE: https://developer.android.com/studio/command-line/variables
ENV ANDROID_SDK_ROOT ${ANDROID_SDK_ROOT}
ENV ANDROID_NDK_VERSION ${ANDROID_NDK_VERSION}
ENV ANDROID_COMPILE_SDK_VERSION ${ANDROID_COMPILE_SDK_VERSION}
ENV RUN_IN_CONTAINER 0

# NOTE: We use `ENTRYPOINT` instead of `CMD` deliberately as it doesn't use a
# shell and doesn't allow using arbitrary commands that we probably don't
# support.
# SEE: https://docs.docker.com/engine/reference/builder/#entrypoint
ENTRYPOINT /lop/scripts/continuous-integration.sh
