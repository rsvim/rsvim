<p align="center">
  <img alt="logo.svg" src="https://raw.githubusercontent.com/rsvim/assets/main/logo/RSVIM-logo.svg" />
</p>

<p align="center">
The VIM editor reinvented in Rust+Typescript.
</p>

<p>
  <a href="https://crates.io/crates/rsvim"><img alt="push.yml" src="https://img.shields.io/crates/v/rsvim" /></a>
  <a href="https://docs.rs/rsvim/latest/rsvim/"><img alt="push.yml" src="https://img.shields.io/docsrs/rsvim" /></a>
  <a href="https://github.com/rsvim/rsvim/actions/workflows/ci.yml"><img alt="ci.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/ci.yml?branch=main&label=ci" /></a>
  <a href="https://github.com/rsvim/rsvim/actions/workflows/build.yml"><img alt="build.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/build.yml?branch=main&label=build" /></a>
  <a href="https://app.codecov.io/gh/rsvim/rsvim"><img alt="codecov" src="https://img.shields.io/codecov/c/github/rsvim/rsvim/main" /></a>
  <a href="https://app.codacy.com/gh/rsvim/rsvim/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade"><img alt="codacy" src="https://img.shields.io/codacy/grade/1c6a3d21352c4f8bb84ff6c7e3ef0399/main" /></a>
  <a href="https://discord.gg/5KtRUCAByB"><img alt="push.yml" src="https://img.shields.io/discord/1220171472329379870?label=discord" /></a>
</p>

> [!CAUTION]
>
> _**This project is still in the very early stages of development and not ready for use. Please choose alternatives [Neovim](https://neovim.io/) and [Vim](https://www.vim.org/).**_

## About

The goal of RSVIM is to be a highly extensible text editor by following the main features and philosophy of ([Neo](https://neovim.io/))[VIM](https://www.vim.org/) editor, while also to be:

- A powerful TUI engine similar to GUI frameworks that provides widgets, event handlers, layouts, etc.
- A programmable editor that provides a consistent script runtime, with builtin support for type system, async/await, plugin management, etc.
- A background editing service that allows multiple remote clients to access and work together.
- A text processing tool that batch processes text contents and integrates with shell environment.
- A modern project that leverages community works for theme, documentation, development, etc.

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
- [DOCUMENTATION.md](https://github.com/rsvim/rsvim/blob/main/DOCUMENTATION.md)
- [RELEASE.md](https://github.com/rsvim/rsvim/blob/main/RELEASE.md)

Roadmap and high-level design can be found in [RFC](https://github.com/rsvim/rfc), please submit your ideas and feature requests there if they need fairly large effort.

## Supporting the Project

If you like RSVIM, please consider sponsoring it. Your support encourages contributors and maintainers of this project, and other fees or efforts spent on it.

- [GitHub Sponsor](https://github.com/rsvim)
- [Open Collective](https://opencollective.com/dashboard/rsvim)

## License

Licensed under [Vim License](https://github.com/rsvim/rsvim/blob/main/LICENSE-VIM.txt) and [Apache License Version 2.0](https://github.com/rsvim/rsvim/blob/main/LICENSE-APACHE.txt).
