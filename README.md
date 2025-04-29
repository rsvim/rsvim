<p align="center">
  <img alt="logo.svg" src="https://raw.githubusercontent.com/rsvim/assets/main/logo/RSVIM-logo.svg" />
</p>

<p align="center">
The VIM editor reinvented in Rust+TypeScript.
</p>

<p align="center">
  <a href="https://crates.io/crates/rsvim"><img alt="rsvim" src="https://img.shields.io/crates/v/rsvim" /></a>
  <a href="https://docs.rs/rsvim_core/latest/"><img alt="rsvim_core" src="https://img.shields.io/docsrs/rsvim_core?label=docs.rs" /></a>
  <a href="https://github.com/rsvim/rsvim/actions/workflows/release.yml"><img alt="release.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/release.yml" /></a>
  <a href="https://github.com/rsvim/rsvim/actions/workflows/ci.yml"><img alt="ci.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/ci.yml?branch=main&label=ci" /></a>
  <a href="https://github.com/rsvim/rsvim/actions/workflows/nightly_miri.yml"><img alt="nightly_miri.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/nightly_miri.yml?branch=main&label=nightly%20miri" /></a>
  <a href="https://app.codecov.io/gh/rsvim/rsvim"><img alt="codecov" src="https://img.shields.io/codecov/c/github/rsvim/rsvim/main" /></a>
  <!-- <a href="https://app.codacy.com/gh/rsvim/rsvim/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade"><img alt="codacy" src="https://img.shields.io/codacy/grade/1c6a3d21352c4f8bb84ff6c7e3ef0399/main" /></a> -->
  <a href="https://discord.gg/5KtRUCAByB"><img alt="discord" src="https://img.shields.io/discord/1220171472329379870?label=discord" /></a>
</p>

> [!CAUTION]
>
> _**This project is still in the very early stages of development and not ready for use. Please choose alternatives [Neovim](https://neovim.io/) and [Vim](https://www.vim.org/).**_

## About

RSVIM is an open source terminal based text editor, strives to be highly extensible by following the main features and philosophy of ([Neo](https://neovim.io/))[VIM](https://www.vim.org/), while also to be:

- A fast editor that fully utilizes all CPU cores and never freezes.
- A powerful TUI engine that provides typeset renderers, widgets, event handlers, layouts, etc.
- A consistent scripting runtime with built-in support for type system, async/await, plugin management, etc.
- An editing service that allows multiple users to access remotely and work together.
- A text processing tool that integrates with the shell environment.

## Installation

Please download pre-built executables in [releases](https://github.com/rsvim/rsvim/releases) page, or build with cargo:

```bash
cargo install --locked rsvim
```

## Get Started

Please checkout [Documentation](https://rsvim.github.io/) for more details!

## Contribution

Some guidelines about contributing to RSVIM can be found in below files:

- [DEVELOPMENT.md](https://github.com/rsvim/rsvim/blob/main/DEVELOPMENT.md)

Roadmap and high-level design can be found in [RFC](https://github.com/rsvim/rfc), please submit your ideas and feature requests there if they need fairly large effort.

## Credits

Some source code is copied and referenced from following projects to implement the initial prototype of javascript runtime, and the [Minimum Common Web Platform API](https://min-common-api.proposal.wintercg.org/).

- [dune](https://github.com/aalykiot/dune): A hobby runtime for JavaScript and TypeScript.
- [deno](https://deno.com/): A modern runtime for JavaScript and TypeScript.

## Supporting the Project

If you like RSVIM, please consider sponsoring it. Your support encourages contributors and maintainers of this project, and other fees or efforts spent on it.

- [GitHub Sponsor](https://github.com/sponsors/rsvim)
- [Open Collective](https://opencollective.com/rsvim)

## License

Licensed under [Vim License](https://github.com/rsvim/rsvim/blob/main/LICENSE.txt).
