# Development

## Environment

Please setup your development environment with:

- Latest stable C++ toolchain:
  - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
  - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
  - For Windows, please use [Visual Studio with C++/C# Desktop Components](https://visualstudio.microsoft.com/).
- Latest stable [Rust](https://www.rust-lang.org/) compiler, or at least 1.74.0.

## Developing

To develop the code, please setup with:

- [rustfmt](https://github.com/rust-lang/rustfmt): Code formatter, format with `cargo fmt` or other ways you like.
- [rust-clippy](https://github.com/rust-lang/rust-clippy): linter, lint with `cargo clippy` or other ways you like.

  > Recommend using [bacon](https://github.com/Canop/bacon) to setup a background lint service, start with `bacon clippy`.

## Testing

To run the unit tests, please run with:

1. Run with `RUST_LOG=debug cargo test`.

   > It enables all the logs over `debug` level, and prints the logs.

## Debugging

To debug the code, please run with:

1. Build the executable binary `rsvim` with `cargo build`.
2. Run with `RUST_LOG=debug ./target/debug/rsvim`.

   > It enables all the logs over `debug` level, and dumps to the log file in the format `rsvim-YYYYMMDD-HHmmss-SSS.log`.

## Documentation

To write rust docs, please setup with:

- [cargo-watch](https://github.com/watchexec/cargo-watch): Watch project file changes.
- [browser-sync](https://browsersync.io/): Reload generated docs and sync to browser, setup with:

  > 1. Install with `npm install -g browser-sync`.
  > 2. Start service with `cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'`.
  > 3. Open browser with `http://localhost:3000/rsvim`.

To write markdown docs, please setup with:

- [markdownlint](https://github.com/DavidAnson/markdownlint): Markdown linter.
- [prettier](https://prettier.io/): Markdown formatter.

## Release

To release new version, please setup with:

- [git-cliff](https://github.com/orhun/git-cliff): Generate changelog from [conventional commits](https://www.conventionalcommits.org/).

  > 1. Install `git-cliff` with `cargo install git-cliff --all-features`(it will enable github integration feature).

- [cargo-release](https://github.com/crate-ci/cargo-release): Release a new version, run below commands:

  > 1. Dry run with `cargo release patch|minor|major`.
  > 2. Run with `cargo release patch|minor|major --execute`.
