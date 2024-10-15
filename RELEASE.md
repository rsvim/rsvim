# Release

Please setup release tools with:

- [git-cliff](https://github.com/orhun/git-cliff): Generate changelog from [conventional commits](https://www.conventionalcommits.org/).
  > Install `git-cliff` with `cargo install git-cliff` (it enables the [GitHub integration](https://git-cliff.org/docs/integration/github)).
- [cargo-release](https://github.com/crate-ci/cargo-release): Release new version and upload to [crates.io](https://crates.io/).

To release a new version, please run below script:

- Dry run with `bash ./release.sh -p [PACKAGE] [LEVEL]`.
- Release run with `bash ./release.sh -p [PACKAGE] [LEVEL] --execute`.

The `[PACKAGE]` is a cargo package inside this workspace:

- `rsvim` (`rsvim_cli`)
- `rsvim_core`

The `[LEVEL]` is a publish level:

- Pre-release: `alpha`, `beta`, `rc`
- Release: `patch`, `minor`, `major`

To skip the error when publishing the `rsvim` package (since it's been renamd from `rsvim_cli` to `rsvim`), please add `--no-verify` option:

- Run with `./release.sh -p [PACKAGE] --no-verify [LEVEL]`
