# RSVIM

> The VIM editor reinvented in Rust+Typescript.

<p>
<a href="https://crates.io/crates/rsvim"><img alt="push.yml" src="https://img.shields.io/crates/v/rsvim" /></a>
<a href="https://docs.rs/rsvim/latest/rsvim/"><img alt="push.yml" src="https://img.shields.io/docsrs/rsvim" /></a>
<a href="https://github.com/rsvim/rsvim/actions/workflows/ci.yml"><img alt="ci.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/ci.yml?branch=main&label=ci" /></a>
<a href="https://github.com/rsvim/rsvim/actions/workflows/build.yml"><img alt="build.yml" src="https://img.shields.io/github/actions/workflow/status/rsvim/rsvim/build.yml?branch=main&label=build" /></a>
<a href="https://app.codecov.io/gh/rsvim/rsvim"><img alt="codecov" src="https://img.shields.io/codecov/c/github/rsvim/rsvim/main" /></a>
<a href="https://app.codacy.com/gh/rsvim/rsvim/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade"><img alt="codacy" src="https://img.shields.io/codacy/grade/1c6a3d21352c4f8bb84ff6c7e3ef0399/main" /></a>
</p>

The goal of RSVIM is to following the philosophy of the VIM editor, but improves/reinvents below components:

- A powerfull TUI engine, provides complete functions similar to GUI framework like [Qt](https://www.qt.io/) or [Tk](https://tkdocs.com/), not only support for windows, but also includes popups, dialogs, mouse events, etc.
- A programmable editor that runs like a VM, provides a consistent scripting language runtime environment, with built-in support for async and plugin package manager.
- A background editing service, allows multiple clients to connect remotely and work together.
- Better documentations & development process, leveraging existing community work.
