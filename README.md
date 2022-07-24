# Lop

[![Continuous Integration](https://github.com/ahimta/lop/actions/workflows/continuous-integration.yml/badge.svg)](https://github.com/ahimta/lop/actions/workflows/continuous-integration.yml)

Milking the mincut-maxflow cow.

## Recommended Environment

- Ubuntu 22.04 LTS x86_64
- VS Code (easier debugging and full-support)

## Getting Started

```bash
# FIXME: Make everything in readme idempotent.
# FIXME: Ensure readme includes all procedures (e.g., updating deps.).
echo "Installing general dependencies..." >&2
# NOTE: We use `mpv` with script-finished sound notification.
sudo apt install -qq --yes \
  curl \
  git \
  mpv \
  podman \
  shellcheck \
  \
  >/dev/null

echo "Installing VS Code..." >&2
sudo snap install --classic code

echo "Installing Nix..." >&2
# NOTE: Verify installation page is more of a verified installation: it includes
# both verification and installation. And in the right order too (verify then
# install).
# SEE: https://nixos.org/download.html#nix-verify-installation
LOP_NIX_WORKING_PATH="/tmp"
LOP_NIX_VERSION="2.8.0"
curl \
  --silent \
  --tlsv1.2 \
  --proto '=https' \
  --output "${LOP_NIX_WORKING_PATH}/install-nix-${LOP_NIX_VERSION}" \
  "https://releases.nixos.org/nix/nix-${LOP_NIX_VERSION}/install"
curl \
  --silent \
  --tlsv1.2 \
  --proto '=https' \
  --output "${LOP_NIX_WORKING_PATH}/install-nix-${LOP_NIX_VERSION}.asc" \
  "https://releases.nixos.org/nix/nix-${LOP_NIX_VERSION}/install.asc"
(
  gpg \
    --keyserver hkps://keyserver.ubuntu.com \
    --recv-keys B541D55301270E0BCF15CA5D8170B4726D7198DE &&
  cd "${LOP_NIX_WORKING_PATH}" &&
  gpg --verify "./install-nix-${LOP_NIX_VERSION}.asc" &&
  sh "./install-nix-${LOP_NIX_VERSION}" --daemon
)
# NOTE: `nix` will only be available in new terminal sessions.
# NOTE: To verify `nix` installation is successfull.
nix-env --version

echo "Installing Rust..." >&2
# SEE: https://www.rust-lang.org/learn/get-started
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

echo "Initializing pre-commit hook..." >&2
ln --force --symbolic ../../scripts/pre-commit.sh ./.git/hooks/pre-commit

echo "Installing Flutter dependencies..." >&2
sudo apt install -qq --yes \
  clang \
  cmake \
  libgtk-3-dev \
  liblzma-dev \
  ninja-build \
  pkg-config \
  \
  >/dev/null

echo "Installing Flutter SDK..." >&2
# SEE: https://docs.flutter.dev/get-started/install/linux#install-flutter-using-snapd
sudo snap install flutter --classic
flutter upgrade
flutter config \
  --no-analytics \
  --enable-android \
  --enable-ios \
  --enable-linux-desktop \
  --enable-macos-desktop \
  --enable-web \
  --enable-windows-desktop \
  --enable-windows-uwp-desktop
flutter precache --all-platforms
dart --disable-analytics

echo "Installing Chromium for Flutter SDK..." >&2
sudo snap install chromium
# NOTE: We have to export `CHROME_EXECUTABLE` because some tools (e.g., Flutter)
# require this to detect Chromium and use it.
echo >> ~/.bashrc
echo 'export CHROME_EXECUTABLE="$(which chromium)"' >> ~/.bashrc
export CHROME_EXECUTABLE="$(which chromium)"

echo "Installing Android SDK for Flutter SDK..." >&2

source ./public.env

wget -qq --output-document=android-cmdline-tools.zip \
  http://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_CMDLINE_TOOLS_VERSION}_latest.zip
echo "${ANDROID_SDK_CMDLINE_TOOLS_VERSION_CHECKSUM_SHA384} android-cmdline-tools.zip" | sha384sum --check --quiet --strict -
unzip -qq android-cmdline-tools.zip -d android-cmdline-tools
rm android-cmdline-tools.zip

LOCAL_ANDROID_SDK_ROOT="$HOME/Android/Sdk"
mkdir --parents "${LOCAL_ANDROID_SDK_ROOT}/cmdline-tools"
mv \
  android-cmdline-tools/cmdline-tools \
  "${LOCAL_ANDROID_SDK_ROOT}/cmdline-tools/latest"
rmdir android-cmdline-tools
export PATH="${PATH}:${LOCAL_ANDROID_SDK_ROOT}/cmdline-tools/latest/bin"

# NOTE(SOME-ANDROID-TOOLS-ONLY-SUPPORT-INSTALLING-LATEST)
echo y | sdkmanager "emulator" >/dev/null
echo y | sdkmanager "system-images;android-31;google_apis;x86_64" >/dev/null
echo y | sdkmanager "build-tools;${ANDROID_BUILD_TOOLS_VERSION}" >/dev/null
echo y | sdkmanager "ndk;${ANDROID_NDK_VERSION}" >/dev/null
# NOTE(SOME-ANDROID-TOOLS-ONLY-SUPPORT-INSTALLING-LATEST)
echo y | sdkmanager "platform-tools" >/dev/null
# FIXME: Replace `echo y` with `yes` everywhere.
echo y | sdkmanager "platforms;android-${ANDROID_COMPILE_SDK_VERSION}" >/dev/null
echo y | sdkmanager --licenses >/dev/null

echo >> ~/.bashrc
echo 'export ANDROID_SDK_ROOT="$HOME/Android/Sdk"' >> ~/.bashrc
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"

echo "Creating Android VM for Flutter SDK..." >&2
# NOTE: The generated Android VM works but may have some minor issues.
avdmanager --silent create avd \
  --force \
  --abi x86_64 \
  --name android-12.0-api-31 \
  --package "system-images;android-31;google_apis;x86_64"

echo "Doctoring Flutter (won't detect Android SDK and that's fine)..." >&2
yes | flutter doctor --android-licenses
flutter doctor

echo "Everything in the project should build/work correctly now and you" >&2
echo "should follow 'Continuous Integration' to make sure this is the case." >&2

echo "Installing Android Studio (just to make 'flutter doctor' happy)..." >&2
sudo snap install --classic android-studio
echo "You have to run Android Studio for `flutter doctor` to detect it." >&2
flutter doctor
```

## Continuous Integration

```bash
# NOTE: Using host.
CONTAINER_COMMAND=podman \
  IS_IN_CONTAINER=0 \
  PRE_COMMIT_CHECK=0 \
  RUN_IN_CONTAINER=0 \
  ./scripts/continuous-integration.sh

# NOTE: Using podman/docker.
CONTAINER_COMMAND=podman \
  IS_IN_CONTAINER=0 \
  PRE_COMMIT_CHECK=0 \
  RUN_IN_CONTAINER=1 \
  ./scripts/continuous-integration.sh

# NOTE: Using Nix (not functional and only as a starting point).
nix-shell \
  --pure \
  --packages rustc cargo \
  --run ./scripts/continuous-integration.sh
# NOTE: For a shell.
nix-shell --pure --packages rustc cargo
```

## BASH Completions

```bash
# NOTE: Make sure first to run continuous-integration once so that all tools,
# for which BASH completions will be generated, have already been installed.

echo "Setting up Rust BASH-completions..." >&2
mkdir --parents ~/.local/share/bash-completion/completions
rustup completions bash > ~/.local/share/bash-completion/completions/rustup
rustup completions bash cargo > ~/.local/share/bash-completion/completions/cargo

echo "Setting up Flutter BASH-completions..." >&2
mkdir --parents ~/.local/share/bash-completion/completions
# NOTE: Command fails due to a mysterious permission error. And only seems to
# work when run from home directory.
flutter bash-completion --overwrite > ~/.local/share/bash-completion/completions/flutter
```

## Debugging

Using VS Code by simply toggling breakpoints and running the debugger (F5).

## Editor/IDE Support

Full VS Code support including debugging and checks/builds/tests (Ctrl+Shift+B
also saves all files before running). This, of course, requires installing
project recommended extensions.

## Snippets

```bash
flutter upgrade --verify-only
flutter upgrade
yes | flutter doctor --android-licenses
flutter doctor

(
  cd boa;

  rustup check;
  rustup update;

  cargo clean;

  cargo tree;

  cargo add --dry-run --no-default-features itertools@^0.10.3;
  cargo add --no-default-features itertools@^0.10.3;

  cargo add --dry-run --no-default-features --dev pretty_assertions@^1.2.1;
  cargo add --no-default-features --dev pretty_assertions@^1.2.1;

  cargo update --dry-run;
  cargo update;
)

(
  cd clod;

  flutter clean;

  flutter pub deps;

  flutter pub add --dry-run ffi:^2.0.1;
  flutter pub add ffi:^2.0.1;

  flutter pub add --dev --dry-run flutter_lints:^2.0.0;
  flutter pub add --dev flutter_lints:^2.0.0;

  flutter pub remove --dry-run ffi;
  flutter pub remove ffi;

  flutter pub outdated;
  flutter pub upgrade --dry-run;
  flutter pub upgrade;
  flutter pub upgrade --dry-run --major-versions;
  flutter pub upgrade --major-versions;

  LD_LIBRARY_PATH=../boa/target/release flutter --device-id linux run \
    --build \
    --debug \
    --hot;
  flutter --device-id emulator run --build --debug --hot;
)
```

## Record of Setup of Already Generated Configuration

```bash
# NOTE: Generated file causes warnings when `cargo fmt` is called as most
# options are only supported in nightly builds. To fix this, we used
# `rustfmt --help=config` which only includes supported options.
echo "Setting up Rust formatting..." >&2
(
  cd boa &&
  rustfmt --edition 2021 --print-config default rustfmt.toml
)

echo "Creating Flutter project..." >&2
(
  cd clod &&
  flutter create \
    --template app \
    --project-name clod \
    --description "lop frontend." \
    --org com.lop \
    --platforms android,ios,linux,macos,windows,web \
    --android-language kotlin \
    --ios-language swift \
    \
    clod
)
```

## General Resources

- [DevDocs (many technologies' docs)](https://devdocs.io)
- [grep.app (source-code search-engine)](https://grep.app)
- [TLDR pages (summarized manpages focused on examples)](https://tldr.ostera.io)
- [linux.die.net (more manpages than available locally)](https://linux.die.net)
- [man7.org (far superior manpages)](https://man7.org/linux/man-pages)
- [The Linux Programming Interface](https://man7.org/tlpi)
- [IBM z/OS Library Functions (POSIX functions not typically other manpages)](https://www.ibm.com/docs/en/zos/2.5.0?topic=reference-library-functions)
- [Flutter Developers](https://flutter.dev)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon)
- [Premier League - Results Page (can easily be used to convert to our format)](https://www.premierleague.com/results)
- [Programming Rust: Fast, Safe Systems Development, 2nd Edition](https://www.amazon.com/dp/B0979PWD4Z)
- [Rust Official Formatter](https://github.com/rust-lang/rustfmt)
- [DelftStack (excellent tutorials & howtos)](https://www.delftstack.com)
- [Android Developers](https://developer.android.com/)
- [The Power of Ten Rules](https://en.wikipedia.org/wiki/The_Power_of_10:_Rules_for_Developing_Safety-Critical_Code)

## Academic Resources

- [Course Homepage](https://algs4.cs.princeton.edu)
- [Baseball Elimination Assignment](https://www.cs.princeton.edu/courses/archive/spring04/cos226/assignments/baseball.html)
- [Mincut/Maxflow Reduction Assignments](https://www.cs.princeton.edu/courses/archive/spring03/cs226/assignments/assign.html)
- [Course on Coursera - Part 1](https://www.coursera.org/learn/algorithms-part1)
- [Course on Coursera - Part 2](https://www.coursera.org/learn/algorithms-part2)
- [Lectures on Official Booksite](https://algs4.cs.princeton.edu/lectures/)
- [Lectures on OReilly](https://www.oreilly.com/library/view/algorithms-24-part-lecture/9780134384528/)
- [Mincut/Maxflow Slides](https://algs4.cs.princeton.edu/lectures/keynote/64MaxFlow-2x2.pdf)
- [Course Code](https://algs4.cs.princeton.edu/code/)
- [Ford Fulkerson Code](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/FordFulkerson.java.html)
- [Flow Network Code](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/FlowNetwork.java.html)
- [Flow Edge Code](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/FlowEdge.java.html)

## Possible Usecases/Ideas

- Pandemic contagion depending on quarantine/curfew decisions.
- Tournament elimination.
- Job/request matching.
- And much much more that can be found from the academic-resources section.

## Credits

All credits for algorithm used (Ford Fulkerson) core-implementation (originally
in Java) goes to
[Prof. Robert Sedgewick](<https://en.wikipedia.org/wiki/Robert_Sedgewick_(computer_scientist)>).
