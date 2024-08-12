# Development

## Environment

Please setup your development environment with:

- Latest stable C++ toolchain:
  - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
  - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
  - For Windows, please use [Visual Studio with C++/C# Desktop Components](https://visualstudio.microsoft.com/).
- Latest stable [Rust](https://www.rust-lang.org/) compiler, or at least 1.74.0.

## Coding

To develop the code, please setup with:

- [rustfmt](https://github.com/rust-lang/rustfmt): Code formatter, format with `cargo fmt` or other ways you like.
- [rust-clippy](https://github.com/rust-lang/rust-clippy): linter, lint with `cargo clippy` or other ways you like.

  > Recommend using [bacon](https://github.com/Canop/bacon) to setup a background lint service, start with `bacon clippy`.

### Global Control

This project uses environment variables to control some behaviors globally, i.e. you can run the `rsvim` command with prepending some env vars. For example:

```bash
RUST_BACKTRACE=full RUST_LOG=debug cargo test
```

To configure debugging/testing behaviors, please setup with:

- `RUST_BACKTRACE`: Print all backtraces when panics.
- `RUST_LOG`: Set logging level, by default it's `info`. To debug the code, please set to `debug`.
- `RUSTFLAGS`: Set extra flags to `rustc` compiler. To enable all warning messages, please set to `-Dwarnings`.

To configure other internal behaviors, please setup with:

- `RSVIM_MUTEX_TIMEOUT`: Set the threading locks timeout by seconds, by default it's [`u64::MAX`](https://doc.rust-lang.org/1.80.0/std/primitive.u64.html#associatedconstant.MAX) (forever).

### Linting

To lint the code, please run with `RUSTFLAGS=-Dwarnings cargo clippy --all-features --all-targets`, it enables all warnings.

### Testing

To run the unit tests, please run with:

1. Run all test cases with `RUST_BACKTRACE=full RUST_LOG=debug cargo test`, it enables:

   > - All the logs over `debug` level, and prints the logs.
   > - The backtrace when panics.

2. Run a specific test case with:

   > 1. First list all test cases with `cargo test -- --list`.
   > 2. Run the specific test with `cargo test {TEST_NAME} -j 1 -- --test-threads 1`, the `TEST_NAME` is the output test names in above step. It also uses single thread to run the test case, instead of multiple threadings.

### Debugging

To debug the code, please run with:

1. Build the executable binary `rsvim` with `cargo build`.
2. Run with `RUST_BACKTRACE=full RUST_LOG=debug ./target/debug/rsvim`, it enables:

   > - All the logs over `debug` level, and dumps to the log file in the format `rsvim-YYYYMMDD-HHmmss-SSS.log`.
   > - The backtrace when panics.

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
