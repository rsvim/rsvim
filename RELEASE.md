# Release

To release new version, please setup with:

- [git-cliff](https://github.com/orhun/git-cliff): Generate changelog from [conventional commits](https://www.conventionalcommits.org/).

  > 1. Install `git-cliff` with `cargo install git-cliff --all-features` (it enables the GitHub integration feature).

- [cargo-release](https://github.com/crate-ci/cargo-release): Release a new version, run below commands:

  > 1. Dry run with `cargo release patch|minor|major`.
  > 2. Run with `cargo release patch|minor|major --execute`.
