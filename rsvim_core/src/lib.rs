//! The core library for the [RSVIM](https://github.com/rsvim/rsvim) editor.

pub mod buf;
pub mod cli;
pub mod constant;
pub mod content;
pub mod coord;
pub mod defaults;
pub mod evloop;
pub mod ex_command;
pub mod js;
pub mod lock;
pub mod log;
pub mod prelude;
pub mod results;
pub mod state;
pub mod ui;

// Only for unit test.
#[cfg(test)]
mod buf_tests;
#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod constant_tests;
#[cfg(test)]
mod js_tests;
#[cfg(test)]
pub mod tests;
