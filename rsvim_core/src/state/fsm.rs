//! The finite-state machine for VIM's editing mode.
//!
//! The VIM's [editing mode](https://en.wikipedia.org/wiki/Vim_(text_editor)) is a global state,
//! i.e the editor starts with normal mode, then press `i` to insert mode, or press `SHIFT-V` to
//! visual mode. In insert mode, press `ESC` to back normal mode. And or so.
//!
//! Each editing mode handles user keyboard/mouse inputs in a different way, this a finite-state
//! machine (FSM) separates code logic in different modes. Each editing mode is a FSM state.
//!
//! Besides, there're some other internal states which are not editing modes or visible to
//! user, but help maintaining the internal state of the editor:
//!
//! * Quit state: The editor should quit on this state.

use crate::buf::BuffersManagerArc;
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

use crossterm::event::Event;
use std::sync::{Arc, Weak};

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

#[derive(Debug)]
/// The mutable data passed to each state handler, and allow them access the editor.
pub struct StatefulDataAccess {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersManagerArc,
  pub event: Event,
}

impl StatefulDataAccess {
  pub fn new(state: StateArc, tree: TreeArc, buffers: BuffersManagerArc, event: Event) -> Self {
    StatefulDataAccess {
      state,
      tree,
      buffers,
      event,
    }
  }
}

/// The FSM trait.
pub trait Stateful {
  /// Handle user's keyboard/mouse event, this method can access the editor's data and update UI tree.
  ///
  /// Returns next state.
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue;
}

#[derive(Debug, Copy, Clone)]
/// The value holder for each state machine.
pub enum StatefulValue {
  // Editing modes.
  NormalMode(NormalStateful),
  VisualMode(VisualStateful),
  SelectMode(SelectStateful),
  OperatorPendingMode(OperatorPendingStateful),
  InsertMode(InsertStateful),
  CommandLineMode(CommandLineStateful),
  TerminalMode(TerminalStateful),
  // Internal states.
  QuitState(QuitStateful),
}

impl Default for StatefulValue {
  /// Returns the default FMS state, by default it's the
  /// [`Normal`](crate::state::fsm::normal::NormalStateful) editing mode.
  fn default() -> Self {
    StatefulValue::NormalMode(NormalStateful::default())
  }
}

impl StatefulValue {
  /// Dispatch data with current FSM state.
  ///
  /// Returns the next FSM state.
  pub fn handle(self: Arc<Self>, data_access: StatefulDataAccess) -> StatefulValue {
    match *self {
      StatefulValue::NormalMode(s) => s.handle(data_access),
      StatefulValue::VisualMode(s) => s.handle(data_access),
      StatefulValue::SelectMode(s) => s.handle(data_access),
      StatefulValue::OperatorPendingMode(s) => s.handle(data_access),
      StatefulValue::InsertMode(s) => s.handle(data_access),
      StatefulValue::CommandLineMode(s) => s.handle(data_access),
      StatefulValue::TerminalMode(s) => s.handle(data_access),
      StatefulValue::QuitState(s) => s.handle(data_access),
    }
  }
}

impl StatefulValue {
  pub fn to_arc(s: StatefulValue) -> StatefulValueArc {
    Arc::new(s)
  }
}

pub type StatefulValueArc = Arc<StatefulValue>;
pub type StatefulValueWk = Weak<StatefulValue>;
