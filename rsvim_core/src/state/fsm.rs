//! Finite-state machine collections.

pub mod cmdline_ex;
pub mod command_line_search_backward;
pub mod command_line_search_forward;
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

pub use cmdline_ex::CmdlineEx;
pub use command_line_search_backward::CmdlineSearchBackward;
pub use command_line_search_forward::CmdlineSearchForward;
pub use insert::Insert;
pub use normal::Normal;
pub use operator_pending::OperatorPending;
pub use select::Select;
pub use terminal::Terminal;
pub use visual::Visual;
