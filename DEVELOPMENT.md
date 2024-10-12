# Development

- [Rust](#rust)
  - [Toolchain](#toolchain)
  - [Environment Variable](#environment-variable)
  - [Check](#check)
  - [Test](#test)
  - [Debug](#debug)
- [TypeScript/JavaScript](#typescriptjavascript)
  - [Toolchain](#toolchain)
  - [Check](#check)
  - [Transpile (Compile)](#transpile-compile)
- [Markdown Document](#markdown-document)

## Rust

### Toolchain

Please setup your development environment with:

- Latest stable C++ toolchain:
  - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
  - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
  - For Windows, please use [Visual Studio with C++/C# Desktop Components](https://visualstudio.microsoft.com/).
- Latest stable [Rust](https://www.rust-lang.org/) compiler, or at least 1.74.0.

To develop code, please setup with:

- [rustfmt](https://github.com/rust-lang/rustfmt): Code formatter, format with `cargo fmt` or other ways you like.
- [rust-clippy](https://github.com/rust-lang/rust-clippy): linter, lint with `cargo clippy` or other ways you like.

  > Recommend to use [bacon](https://github.com/Canop/bacon) to setup a background lint service, start with `bacon clippy`.

### Coding Style

- Public methods named with `_` prefix are private, the public decorator is only for testing.

### Environment Variable

This project uses environment variables to control some behaviors globally, i.e. you can run the `rsvim` command with prepending some env vars. For example:

```bash
RUST_BACKTRACE=full RUST_LOG=debug cargo test
```

To configure debugging/testing behaviors, please setup with:

- `RUST_BACKTRACE`: Print all backtraces when panics.
- `RUST_LOG`: Set logging level, by default it's `info`. To debug code, please set to `debug`.
- `RUSTFLAGS`: Set extra flags to `rustc` compiler. To enable all warning messages, please set to `-Dwarnings`.

To configure other internal behaviors, please see variables listed in the `./src/glovar.rs` source file.

### Check

To check code, please run with `RUSTFLAGS=-Dwarnings cargo clippy --all-features --all-targets`, it enables all warnings.

### Test

To run the unit tests, please run with:

1. Run all test cases with `RUST_BACKTRACE=full RUST_LOG=debug cargo test`, it enables:

   - All the logs over `debug` level, and prints the logs.
   - The backtrace when panics.

2. Run a specific test case with:

   1. First list all test cases with `cargo test -- --list`.
   2. Run the specific test with `cargo test {TEST_NAME}`, the `TEST_NAME` is the output test names in above step.
   3. In case you want to add/print logs in test cases, please call the `crate::test::log::init` API before running a test case.

> Recommend to use [cargo-nextest](https://github.com/nextest-rs/nextest) instead of `cargo test` for better testing experiences.

### Debug

To debug code, please run with:

1. Build the executable binary `rsvim` with `cargo build`.
2. Run with `RUST_BACKTRACE=full RUST_LOG=debug ./target/debug/rsvim`, it enables:

   - All the logs over `debug` level, and dumps to the log file in the format `rsvim-YYYYMMDD-HHmmss-SSS.log`.
   - The backtrace when panics.

### Docs

To write docs, please setup with:

- [cargo-watch](https://github.com/watchexec/cargo-watch): Watch project file changes.
- [browser-sync](https://browsersync.io/): Reload generated docs and sync to browser, setup with:

  1. Install with `npm install -g browser-sync`.
  2. Start service with `cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'`.
  3. Open browser with `http://localhost:3000/rsvim`.

## TypeScript/JavaScript

### Toolchain

Please setup your development environment with:

- [Node.js](https://nodejs.org/) &ge; v18.x.
- Latest stable [Typescript](https://www.typescriptlang.org/) compiler. Please install with `npm install -g typescript`, run `tsc --version` see if the installation is successful.

To develop code, please setup with:

- [prettier](https://prettier.io/): Code formatter.

### Transpile (Compile)

To transpile ts code to js code (in `./src/js/runtime` folder), please run with `tsc` (it also check the code).

To automatically generate js code, please link (or copy) `./git-hooks/pre-commit` to `./git/hooks/pre-commit` in your local git repository with:

- `ln -s $PWD/git-hooks/pre-commit $PWD/.git/hooks/pre-commit`

## Markdown Document

To write markdown docs, please setup with:

- [markdownlint](https://github.com/DavidAnson/markdownlint): Markdown linter.
- [prettier](https://prettier.io/): Markdown formatter.
