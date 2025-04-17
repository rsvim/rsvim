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

  > Note: For windows development, please manually install all dependencies in the `mise.toml` file.

- Link (or copy) `./git-hooks/pre-commit` to `./git/hooks/pre-commit` in your local git repository with `ln -s $PWD/git-hooks/pre-commit $PWD/.git/hooks/pre-commit`.

## Rust

> We provide the `dev.py` script to help running below commands, use `dev.py -h` for more details. For window development, please use `dev.cmd`. Checkout the script for commands running underhood.

### Lint

To check code, please use `./dev.py clippy` (`cargo clippy`).

### Test

To run unit test, please use `./dev.py test` (`cargo test`).

### Debug

To debug code, please:

1. Build binary with `RUST_BACKTRACE=full RSVIM_LOG=trace ./target/debug/rsvim`, it enables all the logs to a logging file named with format `rsvim-YYYYMMDD-HHmmss-SSS.log`.

### Docs

To write docs, please use `./dev.py doc` (`cargo doc`).

### Release

To release a new version, please use `./dev.py release [LEVEL]` (`cargo release`):

## TypeScript/JavaScript

### Transpile/Compile

To transpile/compile typescript code to javascript code, please run `tsc`.
