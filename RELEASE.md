# Release

Please setup release tools with:

- [git-cliff](https://github.com/orhun/git-cliff): Generate changelog from [conventional commits](https://www.conventionalcommits.org/).
  > Install `git-cliff` with `cargo install git-cliff` (it enables the [GitHub integration](https://git-cliff.org/docs/integration/github)).
- [cargo-release](https://github.com/crate-ci/cargo-release): Release new version and upload to [crates.io](https://crates.io/).

To release a new version, please run below script:

- Dry run with `bash ./release.sh [LEVEL]`.
- Release run with `bash ./release.sh [LEVEL] --execute --no-verify`.

  > Note: You will have to use `--no-verify` to skip the check about the difference between `rsvim_cli` and the package name `rsvim`.

The `[LEVEL]` is the release version:

- Pre-release: `alpha`, `beta`, `rc`
- Release: `patch`, `minor`, `major`
