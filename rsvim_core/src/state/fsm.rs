//! Finite-state machine collections.

pub mod cmdline_backward;
pub mod cmdline_ex;
pub mod cmdline_forward;
pub mod insert;
pub mod normal;
pub mod operator_pending;
pub mod select;
pub mod terminal;
pub mod visual;

#[cfg(test)]
mod cmdline_ex_tests;
#[cfg(test)]
mod insert_tests;
#[cfg(test)]
mod normal_tests;

pub use cmdline_backward::CmdlineBackwardStateful;
pub use cmdline_ex::CmdlineExStateful;
pub use cmdline_forward::CmdlineForwardStateful;
pub use insert::InsertStateful;
pub use normal::NormalStateful;
pub use operator_pending::OperatorPendingStateful;
pub use select::SelectStateful;
pub use terminal::TerminalStateful;
pub use visual::VisualStateful;
