//! Finite-state machine collections.

pub mod command_line_ex;
pub mod command_line_search_backward;
pub mod command_line_search_forward;
pub mod insert;
pub mod normal;
pub mod operator_pending;
pub mod select;
pub mod terminal;
pub mod visual;

pub use command_line_ex::CommandLineExStateful;
pub use command_line_search_backward::CommandLineSearchBackwardStateful;
pub use command_line_search_forward::CommandLineSearchForwardStateful;
pub use insert::InsertStateful;
pub use normal::NormalStateful;
pub use operator_pending::OperatorPendingStateful;
pub use select::SelectStateful;
pub use terminal::TerminalStateful;
pub use visual::VisualStateful;

#[cfg(test)]
mod command_line_ex_tests;
#[cfg(test)]
mod insert_tests;
#[cfg(test)]
mod normal_tests;
