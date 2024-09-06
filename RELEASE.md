# Release

To release new version, please setup with:

- [git-cliff](https://github.com/orhun/git-cliff): Generate changelog from [conventional commits](https://www.conventionalcommits.org/).

  1. Install `git-cliff` with `cargo install git-cliff` (it enables the [GitHub integration](https://git-cliff.org/docs/integration/github)).

- [cargo-release](https://github.com/crate-ci/cargo-release): Release a new version, run below commands:

  1. Dry run with `cargo release [LEVEL]`.
  2. Run with `cargo release [LEVEL] --execute`.
