<!-- markdownlint-disable MD013 -->

# Development

## Environment

Please setup your development environment with:

1. Latest stable C++ toolchain:
   - For Linux, please use builtin [GNU GCC](https://gcc.gnu.org/).
   - For macOS, please use [Xcode Clang](https://developer.apple.com/xcode/).
   - For Windows, please use [Visual Studio with C++/C# Components](https://visualstudio.microsoft.com/).
2. Latest stable [Rust](https://www.rust-lang.org/) compiler, or at least 1.74.0.
3. Cargo commands (for rust core):

   - [rustfmt](https://github.com/rust-lang/rustfmt): Code formatter.
   - [cargo clippy](https://github.com/rust-lang/rust-clippy): Linter.
   - [cargo-watch](https://github.com/watchexec/cargo-watch): Cargo docs.
   - [browser-sync](https://browsersync.io/): Cargo docs reload, setup with below steps:

     > 1. Install with `npm install -g browser-sync`.
     > 2. Start local service with `cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'`.
     > 3. Open cargo docs in browser with `https://localhost:3000/rsvim`.

4. Misc:
   - [markdownlint](https://github.com/DavidAnson/markdownlint): Markdown docs.
