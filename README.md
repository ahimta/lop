# Lop

[![Rust](https://github.com/ahimta/lop/actions/workflows/continuous-integration.yml/badge.svg)](https://github.com/ahimta/lop/actions/workflows/continuous-integration.yml)

Milking the mincut-maxflow cow.

## Recommended Environment

- Ubuntu 20.04 LTS x86-64
- VS Code (easier debugging and full-support)

## Getting Started

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
rustup target add aarch64-linux-android
rustup target add x86_64-linux-android
rustup component add rustfmt
rustup component add clippy

mkdir -p ~/.local/share/bash-completion/completions
rustup completions bash >> ~/.local/share/bash-completion/completions/rustup
rustup completions bash cargo >> ~/.local/share/bash-completion/completions/cargo

snap install --classic lefthook
lefthook install
lefthook run pre-commit

echo 'deb http://download.opensuse.org/repositories/devel:/kubic:/libcontainers:/stable/xUbuntu_20.04/ /' | sudo tee /etc/apt/sources.list.d/devel:kubic:libcontainers:stable.list
curl -fsSL https://download.opensuse.org/repositories/devel:kubic:libcontainers:stable/xUbuntu_20.04/Release.key | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/devel_kubic_libcontainers_stable.gpg > /dev/null
sudo apt update
sudo apt install podman

# SEE: https://nixos.org/download.html#nix-verify-installation
curl -o install-nix-2.3.16 https://releases.nixos.org/nix/nix-2.3.16/install
curl -o install-nix-2.3.16.asc https://releases.nixos.org/nix/nix-2.3.16/install.asc
# NOTE: Receiving keys fails on Ubuntu 20.04 LTS and thus verification. Probably
# due to NixOS assuming every developer in the world uses macOS.
gpg2 --recv-keys B541D55301270E0BCF15CA5D8170B4726D7198DE
gpg2 --verify ./install-nix-2.3.16.asc
sh ./install-nix-2.3.16
rm install-nix-2.3.16
```

### Resources

- [Installing Podman for Ubuntu 20.04 LTS](https://software.opensuse.org//download.html?project=devel%3Akubic%3Alibcontainers%3Astable&package=podman)
- [Fully Rootless Podman Setup](https://github.com/containers/podman/blob/main/docs/tutorials/rootless_tutorial.md)

## Getting Started (Flutter)

```bash
sudo snap install flutter --classic
flutter bash-completion >> ~/.local/share/bash-completion/completions/flutter
flutter config --no-analytics
dart --disable-analytics
flutter sdk-path
flutter upgrade
flutter doctor

# FIXME: Uncomment once Flutter desktop is final (it's beta at the moment).
# flutter config --enable-linux-desktop
# sudo apt-get install clang cmake ninja-build pkg-config libgtk-3-dev
```

### Resources

- [Official Flutter Install Guide](https://flutter.dev/docs/get-started/install/linux)

## Continuous Integration

```bash
# NOTE: This is the core and includes all checks, builds, tests, etc...
./scripts/continuous-integration.sh
```

## Using Docker (and Podman in the future)

```bash
# NOTE: This avoids the common occurrence of changing `Containerfile` and
# forgetting to call build and Docker/Podman caching should only do
# anything for build if `Containerfile` changes.
# FIXME: Pass context (`.`) explicitly, just like in GitHub action.
podman build --tag lop --file ./Containerfile && podman run --rm lop
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
```

## Snippets (Flutter)

```bash
flutter pub add english_words
flutter pub remove english_words
```

## Record of Setup of Already Generated Configuration

```bash
# NOTE: Generated file causes warnings when `cargo fmt` is called as most
# options are only supported in nightly builds. To fix this, we used
# `rustfmt --help=config` which only includes supported options.
rustfmt --edition 2018 --print-config default rustfmt.toml
```

## Record of Setup of Already Generated Configuration (Flutter)

```bash
flutter create \
  --template app \
  --project-name clod \
  --description "lop frontend." \
  --org com.lop \
  --platforms android,ios,web \
  --android-language kotlin \
  --ios-language swift \
  clod
```

## General Resources

- [DevDocs (many technologies' docs)](https://devdocs.io)
- [grep.app (searching all the code in the internet)](https://grep.app)
- [linux.die.net (more manpages than available locally)](https://linux.die.net)
- [man7.org (for superior manpages)](https://www.man7.org)
- [IBM z/OS Library Functions (POSIX functions not in any other manpages)](https://www.ibm.com/docs/en/zos/2.5.0?topic=reference-library-functions)
- [flutter.dev](https://flutter.dev)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon)
- [Premier League](https://www.premierleague.com/clubs)
- [Programming Rust: Fast, Safe Systems Development](https://read.amazon.com/?asin=B0979PWD4Z&language=en-US)
- [Rust Official Formatter](https://github.com/rust-lang/rustfmt)
- [Lefthook](https://github.com/evilmartians/lefthook)

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

## Credits

All credits for algorithm used (Ford Fulkerson) core-implementation (originally
in Java) goes to [Prof. Robert Sedgewick](<https://en.wikipedia.org/wiki/Robert_Sedgewick_(computer_scientist)>).
