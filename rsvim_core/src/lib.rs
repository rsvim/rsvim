//! The core library for the [RSVIM](https://github.com/rsvim/rsvim) editor.

pub mod buf;
pub mod cfg;
pub mod cli;
pub mod consts;
pub mod content;
pub mod coord;
pub mod evloop;
pub mod js;
pub mod lock;
pub mod log;
pub mod msg;
pub mod prelude;
pub mod results;
pub mod state;
pub mod ui;
pub mod util;

// Only for unit test or benches.
#[cfg(test)]
mod buf_tests;
#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod coord_tests;
#[cfg(test)]
mod js_tests;
#[cfg(any(test, feature = "benchmarks"))]
pub mod tests;
