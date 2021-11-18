# SEE: https://docs.docker.com/develop/develop-images/dockerfile_best-practices

FROM rust:1.56.1-slim-bullseye

# FIXME: Answer Stackoverflow questions after making sure everything works.
# SEE: https://stackoverflow.com/questions/65262340/cmdline-tools-could-not-determine-sdk-root
# SEE: https://stackoverflow.com/questions/17963508/how-to-install-android-sdk-build-tools-on-the-command-line

RUN echo Installing Android dependencies...
RUN apt-get -qq update --yes
RUN apt-get -qq install --yes \
  git \
  lib32stdc++6 \
  lib32z1 \
  openjdk-11-jdk \
  shellcheck \
  tar \
  unzip \
  wget \
  xz-utils \
  > /dev/null

# NOTE: We drop all privileges as we no longer need them.
# NOTE: `--no-log-init` is to avoid a possibly rare case of disk exhaustion.
# SEE: https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
RUN groupadd --system lop
RUN useradd --create-home --no-log-init --system --gid lop lop
USER lop
WORKDIR /home/lop

RUN echo Installing Rust dependencies...
RUN rustup target add aarch64-linux-android
RUN rustup target add x86_64-linux-android
RUN rustup component add rustfmt
RUN rustup component add clippy

# NOTE: This is the latest version that seems to work with Rust.
ARG ANDROID_BUILD_TOOLS=29.0.2
# NOTE: API level `30` is for Android `11`/`R`.
ARG ANDROID_COMPILE_SDK=30
ARG ANDROID_SDK_ROOT=/tmp/Android/Sdk
# SEE: https://developer.android.com/studio/index.html#downloads
ARG ANDROID_SDK_TOOLS=7583922
ARG ANDROID_SDK_TOOLS_CHECKSUM_SHA256=124f2d5115eee365df6cf3228ffbca6fc3911d16f8025bebd5b1c6e2fcfa7faf
# NOTE: This is the latest version that seems to work with Rust.
ARG NDK_VERSION=22.1.7171670
# SEE: https://developer.android.com/studio/command-line/sdkmanager
ARG SDKMANAGER=${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager

RUN echo Installing Android SDK/NDK...
RUN wget -qq --output-document=android-sdk.zip \
  https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_TOOLS}_latest.zip
RUN echo ${ANDROID_SDK_TOOLS_CHECKSUM_SHA256} android-sdk.zip > android-sdk.zip.sha256sum-check-file
RUN sha256sum --check --quiet --strict android-sdk.zip.sha256sum-check-file
RUN unzip -qq android-sdk.zip -d android-sdk
# NOTE: This is the expected path as implied by this error message:
# Error: Could not determine SDK root.
# Error: Either specify it explicitly with --sdk_root= or move this package into its expected location: <sdk>/cmdline-tools/latest/
RUN mkdir --parents ${ANDROID_SDK_ROOT}/cmdline-tools
RUN mv android-sdk/cmdline-tools ${ANDROID_SDK_ROOT}/cmdline-tools/latest
RUN echo y | ${SDKMANAGER} "platforms;android-${ANDROID_COMPILE_SDK}"
RUN echo y | ${SDKMANAGER} "platform-tools" >/dev/null
RUN echo y | ${SDKMANAGER} "build-tools;${ANDROID_BUILD_TOOLS}" >/dev/null
RUN echo y | ${SDKMANAGER} "ndk;${NDK_VERSION}" >/dev/null
RUN echo y | ${SDKMANAGER} --licenses >/dev/null

ARG FLUTTER=2.5.3
ARG FLUTTER_CHECKSUM_SHA256=b32d04a9fa5709326b4e724e0de64ff1b2b70268f89dd3c748e6360ac937fe01
ARG FLUTTER_SDK_ROOT=/tmp/flutter

# SEE: https://flutter.dev/docs/get-started/install/linux
# SEE: https://github.com/flutter/flutter/blob/master/dev/ci/docker_linux/Dockerfile
RUN echo Installing Flutter SDK...
RUN wget -qq --output-document=flutter-sdk.tar.xz \
  https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${FLUTTER}-stable.tar.xz
RUN echo ${FLUTTER_CHECKSUM_SHA256} flutter-sdk.tar.xz > flutter-sdk.tar.xz.sha256sum-check-file
RUN sha256sum --check --quiet --strict flutter-sdk.tar.xz.sha256sum-check-file
RUN tar xf flutter-sdk.tar.xz
RUN mv flutter ${FLUTTER_SDK_ROOT}
ENV PATH "${PATH}:${FLUTTER_SDK_ROOT}/bin"
RUN flutter config --no-analytics
RUN dart --disable-analytics
RUN flutter precache
RUN yes | flutter doctor --android-licenses
# NOTE: Only `Flutter` and `Android toolchain` need to be available. We can
# enforce this, easily (maybe using grep), programmatically so this must be
# checked manually whenever this file changes.
RUN flutter doctor

# SEE: https://developer.android.com/studio/command-line/variables
ENV ANDROID_SDK_ROOT ${ANDROID_SDK_ROOT}

COPY --chown=lop:lop . /lop
WORKDIR /lop
RUN git restore --staged .
RUN git restore .
RUN git clean -fdx

# NOTE: We use `ENTRYPOINT` instead of `CMD` deliberately as it doesn't use a
# shell and doesn't allow using arbitrary commands that we probably don't
# support.
# SEE: https://docs.docker.com/engine/reference/builder/#entrypoint
ENTRYPOINT /lop/scripts/continuous-integration.sh
