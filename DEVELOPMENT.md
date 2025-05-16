# Development

## Requirements

Please setup your development environment with:

- Latest C++ toolchain:
  - For Linux, install [GNU GCC](https://gcc.gnu.org/).
  - For macOS, install [Xcode Clang](https://developer.apple.com/xcode/).
  - For Windows, install [Visual Studio with C++/C# Desktop Components](https://visualstudio.microsoft.com/).
- LLVM [lld](https://lld.llvm.org/) linker.
- Version management tool [mise](https://github.com/jdx/mise), then run `mise i`.
- Link (or copy) `.pre-commit-hooks` to `./git/hooks/pre-commit` in your local git repository with `ln -s $PWD/.pre-commit-hooks $PWD/.git/hooks/pre-commit`.

## Rust

The `dev.py` script is provided to help running cargo commands, use `dev.py -h` for more details. For window, please use `dev.cmd`.

- To lint code, please use `./dev.py clippy` (`cargo clippy`).
- To run unit test, please use `./dev.py test` (`cargo test`).
- To debug code, please run binary with `RUST_BACKTRACE=full RSVIM_LOG=trace ./target/debug/rsvim`, it enables all the logs to a logging file named with format `rsvim_YYYY-MM-DD_HH-mm-ss-SSS.log`.
- To write docs, please use `./dev.py doc` (`cargo doc`).
- To release a new version, please use `./dev.py release [LEVEL]` (`cargo release`):

## TypeScript/JavaScript

- To transpile/compile typescript code to javascript code, please run `tsc`.
