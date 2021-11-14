FROM rust:1.55

RUN rustup target add aarch64-linux-android
RUN rustup target add x86_64-linux-android
RUN rustup component add rustfmt
RUN rustup component add clippy

RUN apt-get -qq update --yes
RUN apt-get -qq install --yes \
  lib32stdc++6 \
  lib32z1 \
  tar \
  unzip \
  wget \
  > /dev/null
ENV INTERNAL_ARCHIVE_FOLDER_NAME android-ndk-r22b
RUN wget -qq --output-document=${INTERNAL_ARCHIVE_FOLDER_NAME} \
  https://dl.google.com/android/repository/${INTERNAL_ARCHIVE_FOLDER_NAME}-linux-x86_64.zip
RUN mkdir --parents /tmp/Android/Sdk/ndk
RUN unzip -qq ${INTERNAL_ARCHIVE_FOLDER_NAME} -d /tmp
ENV NDK_VERSION 22.1.7171670
RUN mv /tmp/${INTERNAL_ARCHIVE_FOLDER_NAME} /tmp/Android/Sdk/ndk/${NDK_VERSION}
ENV ANDROID_SDK_ROOT /tmp/Android/Sdk

CMD /project/scripts/continuous-integration.sh
