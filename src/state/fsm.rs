//! The finite-state machine for VIM's editing mode.
//! The editing mode of the editor is a global state, and moves from one to another.

use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

// Re-export
pub use crate::state::fsm::command_line::CommandLineStateful;
pub use crate::state::fsm::insert::InsertStateful;
pub use crate::state::fsm::normal::NormalStateful;
pub use crate::state::fsm::operator_pending::OperatorPendingStateful;
pub use crate::state::fsm::select::SelectStateful;
pub use crate::state::fsm::terminal::TerminalStateful;
pub use crate::state::fsm::visual::VisualStateful;

pub mod command_line;
pub mod insert;
pub mod normal;
pub mod operator_pending;
pub mod select;
pub mod terminal;
pub mod visual;

#[derive(Debug, Clone)]
pub struct StatefulDataAccess {
  pub tree: TreeArc,
}

impl StatefulDataAccess {
  pub fn new(tree: TreeArc) -> Self {
    StatefulDataAccess { tree }
  }
}

pub trait Stateful {
  /// Handle user's keyboard/mouse event, this method can access the global state and update UI tree.
  ///
  /// Returns next state.
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful;

  /// Returns VIM mode.
  fn mode(&self) -> Mode;
}

#[derive(Debug, Copy, Clone)]
pub enum NextStateful {
  Normal(NormalStateful),
  Visual(VisualStateful),
  Select(SelectStateful),
  OperatorPending(OperatorPendingStateful),
  Insert(InsertStateful),
  CommandLine(CommandLineStateful),
  Terminal(TerminalStateful),
}

impl Default for NextStateful {
  fn default() -> Self {
    NextStateful::Normal(NormalStateful::default())
  }
}

impl Stateful for NextStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful {
    match self {
      NextStateful::Normal(s) => s.handle(data_access),
      NextStateful::Visual(s) => s.handle(data_access),
      NextStateful::Select(s) => s.handle(data_access),
      NextStateful::OperatorPending(s) => s.handle(data_access),
      NextStateful::Insert(s) => s.handle(data_access),
      NextStateful::CommandLine(s) => s.handle(data_access),
      NextStateful::Terminal(s) => s.handle(data_access),
    }
  }

  fn mode(&self) -> Mode {
    match self {
      NextStateful::Normal(s) => s.mode(),
      NextStateful::Visual(s) => s.mode(),
      NextStateful::Select(s) => s.mode(),
      NextStateful::OperatorPending(s) => s.mode(),
      NextStateful::Insert(s) => s.mode(),
      NextStateful::CommandLine(s) => s.mode(),
      NextStateful::Terminal(s) => s.mode(),
    }
  }
}
