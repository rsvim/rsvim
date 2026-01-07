//! The finite-state machine for Vim editing mode.
//!
//! Vim's [editing mode](https://en.wikipedia.org/wiki/Vim_(text_editor)) is a global state,
//! i.e the editor starts with normal mode, then press `i` to insert mode, or press `SHIFT-V` to
//! visual mode. In insert mode, press `ESC` to back normal mode. And or so.
//!
//! Each editing mode handles user keyboard/mouse inputs in a different way, this a finite-state
//! machine (FSM) separates code logic in different modes. Each editing mode is a FSM state.
//!
//! Besides, there're some other internal states which are not editing modes or visible to
//! user, but help maintaining the internal state of the editor:

pub mod fsm;
pub mod mode;
pub mod ops;

#[cfg(test)]
mod mode_tests;

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::msg::JsMessage;
use crate::msg::MasterMessage;
use crate::state::ops::Operation;
use crate::ui::tree::TreeArc;
use crossterm::event::Event;
use fsm::CmdlineSearchBackward;
use fsm::CmdlineSearchForward;
use fsm::CommandLineEx;
use fsm::Insert;
use fsm::Normal;
use fsm::OperatorPending;
use fsm::Select;
use fsm::Terminal;
use fsm::Visual;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
/// The mutable data passed to each state handler, and allow them access the editor.
pub struct StateDataAccess {
  pub tree: TreeArc,
  pub buffers: BuffersManagerArc,
  pub contents: TextContentsArc,
  pub master_tx: UnboundedSender<MasterMessage>,
  pub jsrt_forwarder_tx: UnboundedSender<JsMessage>,
}

impl StateDataAccess {
  pub fn new(
    tree: TreeArc,
    buffers: BuffersManagerArc,
    contents: TextContentsArc,
    master_tx: UnboundedSender<MasterMessage>,
    jsrt_forwarder_tx: UnboundedSender<JsMessage>,
  ) -> Self {
    StateDataAccess {
      tree,
      buffers,
      contents,
      master_tx,
      jsrt_forwarder_tx,
    }
  }
}

/// The FSM trait.
pub trait Stateful {
  /// Handle user's keyboard/mouse event, this method can access the editor's data and update UI tree.
  ///
  /// Returns next state.
  fn handle(&self, data_access: StateDataAccess, event: Event) -> State;

  /// Handle user's operation, this method can access the editor's data and update UI tree.
  ///
  /// Returns next state.
  fn handle_op(&self, data_access: StateDataAccess, op: Operation) -> State;
}

/// Generate enum dispatcher for `Stateful`.
#[macro_export]
macro_rules! stateful_enum_impl {
  ($enum:ident, $($variant:tt),*) => {
    impl Stateful for $enum {
      fn handle(&self, data_access: StateDataAccess, event: Event) -> State {
        match self {
          $(
            $enum::$variant(e) => e.handle(data_access, event),
          )*
        }
      }

      fn handle_op(&self, data_access: StateDataAccess, op: Operation) -> State {
        match self {
          $(
            $enum::$variant(e) => e.handle_op(data_access, op),
          )*
        }
      }
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The value holder for each state machine.
pub enum State {
  Normal(Normal),
  Visual(Visual),
  Select(Select),
  OperatorPending(OperatorPending),
  Insert(Insert),
  CommandLineEx(CommandLineEx),
  CmdlineSearchForward(CmdlineSearchForward),
  CmdlineSearchBackward(CmdlineSearchBackward),
  Terminal(Terminal),
}

stateful_enum_impl!(
  State,
  Normal,
  Visual,
  Select,
  OperatorPending,
  Insert,
  CommandLineEx,
  CmdlineSearchForward,
  CmdlineSearchBackward,
  Terminal
);

impl Default for State {
  /// Returns the default FMS state, by default it's the
  /// [`Normal`](crate::state::fsm::normal::NormalStateful) editing mode.
  fn default() -> Self {
    State::Normal(Normal::default())
  }
}
