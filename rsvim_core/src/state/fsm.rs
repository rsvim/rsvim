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
use crate::content::TextContentsArc;
use crate::state::StateArc;
use crate::state::ops::Operation;
use crate::ui::tree::TreeArc;

use crossterm::event::Event;
use enum_dispatch::enum_dispatch;
use paste::paste;

// Re-export
pub use command_line_ex::CommandLineExStateful;
pub use command_line_search_backward::CommandLineSearchBackwardStateful;
pub use command_line_search_forward::CommandLineSearchForwardStateful;
pub use insert::InsertStateful;
pub use normal::NormalStateful;
pub use operator_pending::OperatorPendingStateful;
pub use quit::QuitStateful;
pub use select::SelectStateful;
pub use terminal::TerminalStateful;
pub use visual::VisualStateful;

pub mod command_line_ex;
pub mod command_line_search_backward;
pub mod command_line_search_forward;
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
  pub contents: TextContentsArc,
  pub event: Event,
}

impl StatefulDataAccess {
  pub fn new(
    state: StateArc,
    tree: TreeArc,
    buffers: BuffersManagerArc,
    contents: TextContentsArc,
    event: Event,
  ) -> Self {
    StatefulDataAccess {
      state,
      tree,
      buffers,
      contents,
      event,
    }
  }
}

#[enum_dispatch]
/// The FSM trait.
pub trait Stateful {
  /// Handle user's keyboard/mouse event, this method can access the editor's data and update UI tree.
  ///
  /// Returns next state.
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValueDispatcher;

  /// Handle user's operation, this method can access the editor's data and update UI tree.
  ///
  /// Returns next state.
  fn handle_op(&self, data_access: StatefulDataAccess, op: Operation) -> StatefulValueDispatcher;
}

#[enum_dispatch(Stateful)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The value holder for each state machine.
pub enum StatefulValueDispatcher {
  // Editing modes.
  NormalMode(NormalStateful),
  VisualMode(VisualStateful),
  SelectMode(SelectStateful),
  OperatorPendingMode(OperatorPendingStateful),
  InsertMode(InsertStateful),
  CommandLineExMode(CommandLineExStateful),
  CommandLineSearchForwardMode(CommandLineSearchForwardStateful),
  CommandLineSearchBackwardMode(CommandLineSearchBackwardStateful),
  TerminalMode(TerminalStateful),
  // Internal states.
  QuitState(QuitStateful),
}

impl Default for StatefulValueDispatcher {
  /// Returns the default FMS state, by default it's the
  /// [`Normal`](crate::state::fsm::normal::NormalStateful) editing mode.
  fn default() -> Self {
    StatefulValueDispatcher::NormalMode(NormalStateful::default())
  }
}
