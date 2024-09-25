# Development

- [Rust](#rust)
  - [Toolchain](#toolchain)
  - [Global Control](#global-control)
  - [Check](#check)
  - [Test](#test)
  - [Debug](#debug)
- [TypeScript/JavaScript](#typescriptjavascript)
  - [Toolchain](#toolchain)
  - [Check](#check)
  - [Transpile (Compile)](#transpile-compile)

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

### Global Control

This project uses environment variables to control some behaviors globally, i.e. you can run the `rsvim` command with prepending some env vars. For example:

```bash
RUST_BACKTRACE=full RUST_LOG=debug cargo test
```

To configure debugging/testing behaviors, please setup with:

- `RUST_BACKTRACE`: Print all backtraces when panics.
- `RUST_LOG`: Set logging level, by default it's `info`. To debug code, please set to `debug`.
- `RUSTFLAGS`: Set extra flags to `rustc` compiler. To enable all warning messages, please set to `-Dwarnings`.

To configure other internal behaviors, please setup with:

- `RSVIM_MUTEX_TIMEOUT`: Set the threading locks timeout by seconds, by default it's [`u64::MAX`](https://doc.rust-lang.org/1.80.0/std/primitive.u64.html#associatedconstant.MAX) (forever).

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

> Recommend to use [cargo-nextest](https://github.com/nextest-rs/nextest) instead of `cargo test` for better testing experiences.

### Debug

To debug code, please run with:

1. Build the executable binary `rsvim` with `cargo build`.
2. Run with `RUST_BACKTRACE=full RUST_LOG=debug ./target/debug/rsvim`, it enables:

   - All the logs over `debug` level, and dumps to the log file in the format `rsvim-YYYYMMDD-HHmmss-SSS.log`.
   - The backtrace when panics.

## TypeScript/JavaScript

### Toolchain

Please setup your development environment with:

- Latest stable [deno](https://deno.com/).
- [Typescript](https://www.typescriptlang.org/) compiler that is compatible with the tsc/swc included with `deno`. Please install with following steps:
  1. Use `deno --version` to find out the typescript compiler version, say `typescript 5.5.2`.
  2. Install typescript compiler with the compatible version:
     - For deno, please install with `deno install -g npm:typescript@5.5`.
     - For npm, please install with `npm install -g typescript@5.5`.
  3. Run `tsc --version` and see if it's successful.

To develop code, please run with:

- Format code with [prettier](https://prettier.io/).

### Check

To check code, please run with `deno check ./src/js/module/runtime.ts`.

### Transpile (Compile)

To transpile ts code to js code (in `src` folder), please run with `tsc`.
