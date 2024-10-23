//! The core library for the [RSVIM](https://github.com/rsvim/rsvim) editor.

extern crate num;
#[macro_use]
extern crate num_derive;

pub mod buf;
pub mod cart;
pub mod cli;
pub mod defaults;
pub mod envar;
pub mod evloop;
pub mod js;
pub mod locks;
pub mod log;
pub mod res;
pub mod state;
pub mod test;
pub mod ui;
