# Lop

[![Rust](https://github.com/ahimta/lop/actions/workflows/continuous-integration.yml/badge.svg)](https://github.com/ahimta/lop/actions/workflows/continuous-integration.yml)

Milking the mincut-maxflow cow.

## Recommended Environment

- Ubuntu 22.04 LTS x86-64
- VS Code (easier debugging and full-support)

## Getting Started

```bash
sudo apt install -qq --yes \
  curl \
  git \
  shellcheck \
  \
  >/dev/null

sudo snap install --classic code

# SEE: https://www.rust-lang.org/learn/get-started
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# NOTE: We use it with script-finished sound notification.
sudo apt install -qq --yes mpv >/dev/null

ln --force --symbolic ../../scripts/pre-commit.sh ./.git/hooks/pre-commit

# SEE: https://podman.io/getting-started/installation
sudo apt install -qq --yes podman >/dev/null

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
```

## Getting Started (Flutter)

```bash
sudo apt install -qq --yes \
  clang \
  cmake \
  libgtk-3-dev \
  liblzma-dev \
  ninja-build \
  pkg-config \
  \
  >/dev/null

# SEE: https://docs.flutter.dev/get-started/install/linux#install-flutter-using-snapd
sudo snap install flutter --classic
flutter sdk-path
flutter upgrade
# NOTE: `precache` downloads all platform-specific files eagerly.
flutter precache
mkdir --parents ~/.local/share/bash-completion/completions
# FIXME: Command fails due to a mysterious permission error.
flutter bash-completion > ~/.local/share/bash-completion/completions/flutter

flutter config \
  --no-analytics \
  --enable-android \
  --enable-ios \
  --enable-linux-desktop \
  --enable-macos-desktop \
  --enable-web \
  --enable-windows-desktop \
  --enable-windows-uwp-desktop

dart --disable-analytics
# NOTE: Install Google Chrome.
# SEE: https://www.google.com/chrome
sudo snap install --classic code
sudo snap install --classic android-studio
flutter doctor --android-licenses
# FIXME: Android setup done manually using Android Studio. Make it so that at
# least only command-line tools are installed manually. And extract common
# variables to `public.env`.
flutter doctor
```

## Continuous Integration

```bash
# NOTE: This is the core and includes all checks, builds, tests, etc...
CONTAINER_COMMAND=podman \
  PRE_COMMIT_CHECK=0 \
  RUN_IN_CONTAINER=0 \
  ./scripts/continuous-integration.sh
```

## BASH Completions

```bash
# NOTE: Make sure first to run continuous-integration/build once so that all
# tools, for which BASH completions will be generated, have already been
# installed.
mkdir --parents ~/.local/share/bash-completion/completions
rustup completions bash > ~/.local/share/bash-completion/completions/rustup
rustup completions bash cargo > ~/.local/share/bash-completion/completions/cargo
```

## Using Podman/Docker

```bash
CONTAINER_COMMAND=podman \
  PRE_COMMIT_CHECK=0 \
  RUN_IN_CONTAINER=1 \
  ./scripts/continuous-integration.sh; \
  ./scripts/notify-user.sh
```

## Using Nix (not functional and only as a starting point)

```bash
nix-shell \
  --pure \
  --packages rustc cargo \
  --run ./scripts/continuous-integration.sh

# NOTE: For a shell.
nix-shell --pure --packages rustc cargo
```

## Debugging

Using VS Code by simply toggling breakpoints and running the debugger (F5).

## Editor/IDE Support

Full VS Code support including debugging and checks/checks/builds/tests
(Ctrl+Shift+B this also saves all files before running). This, of course,
requires installing project recommended extensions.

## Snippets

```bash
rustup update
cargo update
cargo doc --open

cargo fmt
cargo check
cargo build
cargo test

cargo fix --edition
```

## Snippets (Flutter)

```bash
flutter pub add english_words
flutter pub remove english_words

flutter pub outdated
flutter pub upgrade --dry-run
flutter pub upgrade
flutter pub upgrade --dry-run --major-versions
flutter pub upgrade --major-versions

LD_LIBRARY_PATH=../boa/target/release flutter run --device-id linux
```

## Record of Setup of Already Generated Configuration

```bash
# NOTE: Generated file causes warnings when `cargo fmt` is called as most
# options are only supported in nightly builds. To fix this, we used
# `rustfmt --help=config` which only includes supported options.
# FIXME: Seems to not support `2021` `edition` and we should use it as soon as
# it's available.
rustfmt --edition 2018 --print-config default rustfmt.toml
```

## Record of Setup of Already Generated Configuration (Flutter)

```bash
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
```

## General Resources

- [DevDocs (many technologies' docs)](https://devdocs.io)
- [grep.app (searching all the code in the internet)](https://grep.app)
- [TLDR pages (summarized manpages focused on examples)](https://tldr.ostera.io)
- [linux.die.net (more manpages than available locally)](https://linux.die.net)
- [man7.org (for superior manpages)](https://www.man7.org)
- [IBM z/OS Library Functions (POSIX functions not in any other manpages)](https://www.ibm.com/docs/en/zos/2.5.0?topic=reference-library-functions)
- [flutter.dev](https://flutter.dev)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon)
- [Premier League - Results Page (can easily be used to convert to our format)](https://www.premierleague.com/results)
- [Programming Rust: Fast, Safe Systems Development](https://read.amazon.com/?asin=B0979PWD4Z&language=en-US)
- [Rust Official Formatter](https://github.com/rust-lang/rustfmt)

## Academic Resources

- [All Course Assignments](https://introcs.cs.princeton.edu/java/assignments/)
- [Baseball Elimination Assignment](https://www.cs.princeton.edu/courses/archive/spring04/cos226/assignments/baseball.html)
- [Mincut/Maxflow Reduction Assignments](https://www.cs.princeton.edu/courses/archive/spring03/cs226/assignments/assign.html)
- [Course on Coursera - Part 1](https://www.coursera.org/learn/algorithms-part1)
- [Course on Coursera - Part 2](https://www.coursera.org/learn/algorithms-part2)
- [Lectures on Official Booksite](https://algs4.cs.princeton.edu/lectures/)
- [Lections on Oreilly](https://www.oreilly.com/library/view/algorithms-24-part-lecture/9780134384528/)
- [Mincut/Maxflow Slides](https://algs4.cs.princeton.edu/lectures/keynote/64MaxFlow-2x2.pdf)
- [Course Code](https://algs4.cs.princeton.edu/code/)
- [Ford Fulkerson Code](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/FordFulkerson.java.html)
- [Flow Network Code](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/FlowNetwork.java.html)
- [Flow Edge Code](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/FlowEdge.java.html)

## Possible Usecases/Ideas

- Pandemic contagion depending on quarantine/curfew decisions
- Tournament elimination
- Job/request matching
- And much much more that can be found from the academic-resources section here

## Credits

All credits for algorithm used (Ford Fulkerson) core-implementation (originally
in Java) goes to [Prof. Robert Sedgewick](<https://en.wikipedia.org/wiki/Robert_Sedgewick_(computer_scientist)>).
