FROM rust:1.55

# FIXME: Answer Stackoverflow questions after making sure everything works.
# SEE: https://stackoverflow.com/questions/65262340/cmdline-tools-could-not-determine-sdk-root
# SEE: https://stackoverflow.com/questions/17963508/how-to-install-android-sdk-build-tools-on-the-command-line

RUN echo Installing Rust dependencies...
RUN rustup target add aarch64-linux-android
RUN rustup target add x86_64-linux-android
RUN rustup component add rustfmt
RUN rustup component add clippy

RUN echo Installing Android dependencies...
RUN apt-get -qq update --yes
RUN apt-get -qq install --yes \
  lib32stdc++6 \
  lib32z1 \
  openjdk-11-jdk \
  tar \
  unzip \
  wget \
  > /dev/null

ARG ANDROID_BUILD_TOOLS=31.0.0
# NOTE: API level `30` is for Android `11`/`R`.
ARG ANDROID_COMPILE_SDK=30
ARG ANDROID_SDK_ROOT=/tmp/Android/Sdk
# SEE: https://developer.android.com/studio/index.html#downloads
ARG ANDROID_SDK_TOOLS=7583922
ARG ANDROID_SDK_TOOLS_CHECKSUM_SHA256=124f2d5115eee365df6cf3228ffbca6fc3911d16f8025bebd5b1c6e2fcfa7faf
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

# SEE: https://developer.android.com/studio/command-line/variables
ENV ANDROID_SDK_ROOT ${ANDROID_SDK_ROOT}

# NOTE: `/project` should be mounted when running the container.
CMD /project/scripts/continuous-integration.sh
