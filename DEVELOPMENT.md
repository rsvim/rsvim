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
- [rust-clippy](https://github.com/rust-lang/rust-clippy) and [bacon](https://github.com/Canop/bacon): linter, lint with `RUSTFLAGS="-Dwarnings" bacon -j clippy-all`.
- [cargo-nextest](https://github.com/nextest-rs/nextest): Test runner, run with `RUST_LOG=trace cargo nextest run --no-capture`.
- [taplo](https://github.com/tamasfe/taplo): Toml code formatter, format with `taplo format [FILE]` or other ways you like.
- [sccache](https://github.com/mozilla/sccache): Compiler cache to improve building speed.
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat): Executable size analyzer.

### Lint

To check code, please use `RUSTFLAGS="-Dwarnings" bacon -j clippy-all`.

### Test

1. To run all test cases, please use `cargo nextest run`.
2. To run all test cases with full backtrace and logging message, please use `RUST_BACKTRACE=full RUST_LOG=trace cargo nextest run --no-capture`.
3. To run a specific test, please use `cargo nextest run [TEST]`.
4. To list all test cases, please use `cargo nextest list`.

### Debug

To debug code, please:

1. Build `rsvim` with `cargo build`.
2. Run with `RUST_BACKTRACE=full RUST_LOG=trace ./target/debug/rsvim`, it enables all the logs to a logging file named with format `rsvim-YYYYMMDD-HHmmss-SSS.log`.

### Analyze

To analyze executable sizes, please use `cargo bloat --release --bin rsvim`.

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
- Latest stable [Typescript](https://www.typescriptlang.org/), please install with `npm install -g typescript`, and run `tsc --version` to verify the installation is successful.

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
