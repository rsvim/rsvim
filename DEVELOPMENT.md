# Development

## Requirements

Please setup your development environment with:

- Latest stable C++ toolchain:
  - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
  - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
  - For Windows, please use [Visual Studio with C++/C# Desktop Components](https://visualstudio.microsoft.com/).
- Latest stable version management tools [mise](https://github.com/jdx/mise) and [pipx](https://github.com/pypa/pipx), then install multiple command lines with `mise i`.

  > Note: For windows development, please manually install all dependencies in the `mise.toml` file.

- Link (or copy) `.pre-commit-hooks` to `./git/hooks/pre-commit` in your local git repository with `ln -s $PWD/.pre-commit-hooks $PWD/.git/hooks/pre-commit`.

## Rust

> We provide the `dev.py` script to help running below commands, use `dev.py -h` for more details. For window development, please use `dev.cmd`. Checkout the script for commands running underhood.

- To lint code, please use `./dev.py clippy` (`cargo clippy`).
- To run unit test, please use `./dev.py test` (`cargo test`).
- To debug code, please build binary with `RUST_BACKTRACE=full RSVIM_LOG=trace ./target/debug/rsvim`, it enables all the logs to a logging file named with format `rsvim-YYYYMMDD-HHmmss-SSS.log`.
- To write docs, please use `./dev.py doc` (`cargo doc`).
- To release a new version, please use `./dev.py release [LEVEL]` (`cargo release`):

## TypeScript/JavaScript

- To transpile/compile typescript code to javascript code, please run `tsc`.
