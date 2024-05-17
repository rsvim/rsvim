<!-- markdownlint-disable MD013 -->

# Development

## Environment

Please setup your development environment with:

1. Latest stable C++ toolchain:
   - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
   - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
   - For Windows, please use [Visual Studio with C++/C# Components](https://visualstudio.microsoft.com/).
2. Latest stable [Rust](https://www.rust-lang.org/) compiler, or at least 1.74.0.
3. Rust project:

   - [rustfmt](https://github.com/rust-lang/rustfmt): Code formatter.
   - [rust-clippy](https://github.com/rust-lang/rust-clippy): Linter.
   - [bacon](https://github.com/Canop/bacon): Background linter, setup with:

     > 1. Start service with `bacon clippy-all`.

   - [cargo-watch](https://github.com/watchexec/cargo-watch): Rust docs.
   - [browser-sync](https://browsersync.io/): Rust docs reload, setup with:

     > 1. Install with `npm install -g browser-sync`.
     > 2. Start service with `cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'`.
     > 3. Open browser with `https://localhost:3000/rsvim`.

4. Markdown docs:
   - [markdownlint](https://github.com/DavidAnson/markdownlint): Linter.
   - [prettier](https://prettier.io/): Code formatter.
