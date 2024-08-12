# Documentation

## Rust Docs

To write rust docs, please setup with:

- [cargo-watch](https://github.com/watchexec/cargo-watch): Watch project file changes.
- [browser-sync](https://browsersync.io/): Reload generated docs and sync to browser, setup with:

  > 1. Install with `npm install -g browser-sync`.
  > 2. Start service with `cargo watch -s 'cargo doc && browser-sync start --ss target/doc -s target/doc --directory --no-open'`.
  > 3. Open browser with `http://localhost:3000/rsvim`.

## Markdown Docs

To write markdown docs, please setup with:

- [markdownlint](https://github.com/DavidAnson/markdownlint): Markdown linter.
- [prettier](https://prettier.io/): Markdown formatter.
