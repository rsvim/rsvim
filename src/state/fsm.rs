//! The finite-state machine for VIM's editing mode.
//!
//! The VIM's [editing mode](https://en.wikipedia.org/wiki/Vim_(text_editor)) is a global state,
//! i.e the editor starts with normal mode, then press `i` to insert mode, or press `SHIFT-V` to
//! visual mode. In insert mode, press `ESC` to back normal mode. And more similar cases.
//!
//! Each editing mode handles user keyboard/mouse inputs in a different way, so a finite-state
//! machine (FSM) separates code logic in these different modes. Each editing mode is a state
//! inside this FSM. Besides, there're some other internal states which are not editing modes or
//! visible to user, but help maintaining the internal state of the editor:
//!
//! * Quit state: The editor instance should exit in this state.

use crossterm::event::Event;

use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

// Re-export
pub use crate::state::fsm::command_line::CommandLineStateful;
pub use crate::state::fsm::insert::InsertStateful;
pub use crate::state::fsm::normal::NormalStateful;
pub use crate::state::fsm::operator_pending::OperatorPendingStateful;
pub use crate::state::fsm::quit::QuitStateful;
pub use crate::state::fsm::select::SelectStateful;
pub use crate::state::fsm::terminal::TerminalStateful;
pub use crate::state::fsm::visual::VisualStateful;

pub mod command_line;
pub mod insert;
pub mod normal;
pub mod operator_pending;
pub mod quit;
pub mod select;
pub mod terminal;
pub mod visual;

#[derive(Debug, Clone)]
pub struct StatefulDataAccess {
  pub tree: TreeArc,
  pub event: Event,
}

impl StatefulDataAccess {
  pub fn new(tree: TreeArc, event: Event) -> Self {
    StatefulDataAccess { tree, event }
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
