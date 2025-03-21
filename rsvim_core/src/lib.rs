//! The core library for the [RSVIM](https://github.com/rsvim/rsvim) editor.

pub mod buf;
pub mod cli;
pub mod coord;
pub mod defaults;
pub mod envar;
pub mod evloop;
pub mod js;
pub mod lock;
pub mod log;
pub mod res;
pub mod state;
pub mod ui;

// Only for unit test.
#[cfg(test)]
pub mod test;
