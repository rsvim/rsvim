# Development

- [Requirements](#requirements)
- [Rust](#rust)
  - [Lint](#lint)
  - [Test](#test)
  - [Debug](#debug)
- [TypeScript/JavaScript](#typescriptjavascript)
  - [Transpile/Compile](#transpilecompile)

## Requirements

Please setup your development environment with:

- Latest stable C++ toolchain:
  - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
  - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
  - For Windows, please use [Visual Studio with C++/C# Desktop Components](https://visualstudio.microsoft.com/).
- Latest stable version management tools [mise](https://github.com/jdx/mise) and [pipx](https://github.com/pypa/pipx), then install multiple command lines with `mise i`.

## Rust

> We provide the `dev.py` script to help running below commands, use `dev.py -h` for more details.

### Lint

To check code, please use `RUSTFLAGS='-Dwarnings' bacon -j clippy-all`.

### Test

1. To run all test cases, please use `cargo nextest run`.
2. To run all test cases with full backtrace and logging message, please use `RUST_BACKTRACE=full RUST_LOG=trace cargo nextest run --no-capture`.
3. To run a specific test, please use `cargo nextest run [TEST]`.
4. To list all test cases, please use `cargo nextest list`.

### Debug

To debug code, please:

1. Build binary with `cargo build`.
2. Build binary with `RUST_BACKTRACE=full RUST_LOG=trace ./target/debug/rsvim`, it enables all the logs to a logging file named with format `rsvim-YYYYMMDD-HHmmss-SSS.log`.

### Docs

To write docs, please:

1. Start local service with `cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'`.
2. Open browser with `http://localhost:3000/rsvim`.

## TypeScript/JavaScript

### Transpile/Compile

To transpile/compile typescript code to javascript code, please run `tsc`.

> To automatically generate js code, please link (or copy) `./git-hooks/pre-commit` to `./git/hooks/pre-commit` in local git repository with `ln -s $PWD/git-hooks/pre-commit $PWD/.git/hooks/pre-commit`.
