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
//!
//! * Quit state: The editor should quit on this state.

use crate::buf::BuffersManagerArc;
use crate::content::TextContentsArc;
use crate::msg::{JsMessage, MasterMessage};
use crate::state::ops::Operation;
use crate::ui::tree::TreeArc;
use fsm::{
  CommandLineExStateful, CommandLineSearchBackwardStateful,
  CommandLineSearchForwardStateful, InsertStateful, NormalStateful,
  OperatorPendingStateful, SelectStateful, TerminalStateful, VisualStateful,
};

use crossterm::event::Event;
use tokio::sync::mpsc::Sender;

pub mod fsm;
pub mod mode;
pub mod ops;

#[derive(Debug)]
/// The mutable data passed to each state handler, and allow them access the editor.
pub struct StateDataAccess {
  pub tree: TreeArc,
  pub buffers: BuffersManagerArc,
  pub contents: TextContentsArc,
  pub master_tx: Sender<MasterMessage>,
  pub jsrt_forwarder_tx: Sender<JsMessage>,
}

impl StateDataAccess {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    tree: TreeArc,
    buffers: BuffersManagerArc,
    contents: TextContentsArc,
    master_tx: Sender<MasterMessage>,
    jsrt_forwarder_tx: Sender<JsMessage>,
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
  fn handle(&self, data_access: StateDataAccess, event: Event) -> StateMachine;

  /// Handle user's operation, this method can access the editor's data and update UI tree.
  ///
  /// Returns next state.
  fn handle_op(
    &self,
    data_access: StateDataAccess,
    op: Operation,
  ) -> StateMachine;
}

/// Generate enum dispatcher for `Stateful`.
#[macro_export]
macro_rules! state_machine_dispatcher {
  ($enum:ident, $($variant:tt),*) => {
    impl Stateful for $enum {
      fn handle(&self, data_access: StateDataAccess, event: Event) -> StateMachine {
        match self {
          $(
            $enum::$variant(e) => e.handle(data_access, event),
          )*
        }
      }

      fn handle_op(&self, data_access: StateDataAccess, op: Operation) -> StateMachine {
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
pub enum StateMachine {
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
}

state_machine_dispatcher!(
  StateMachine,
  NormalMode,
  VisualMode,
  SelectMode,
  OperatorPendingMode,
  InsertMode,
  CommandLineExMode,
  CommandLineSearchForwardMode,
  CommandLineSearchBackwardMode,
  TerminalMode,
);

impl Default for StateMachine {
  /// Returns the default FMS state, by default it's the
  /// [`Normal`](crate::state::fsm::normal::NormalStateful) editing mode.
  fn default() -> Self {
    StateMachine::NormalMode(NormalStateful::default())
  }
}
