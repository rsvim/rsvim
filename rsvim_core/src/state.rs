//! Vim editing mode.

use crate::buf::BuffersManagerArc;
use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

use crossterm::event::Event;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use tracing::trace;

pub mod command;
pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  stateful: StateMachine,
  last_stateful: StateMachine,

  // Editing mode.
  mode: Mode,
}

#[derive(Debug, Copy, Clone)]
pub struct StateHandleResponse {
  pub stateful: StateMachine,
  pub next_stateful: StateMachine,
}

impl StateHandleResponse {
  pub fn new(stateful: StateMachine, next_stateful: StateMachine) -> Self {
    StateHandleResponse {
      stateful,
      next_stateful,
    }
  }
}

pub type StateArc = Arc<RwLock<State>>;
pub type StateWk = Weak<RwLock<State>>;

impl State {
  pub fn new() -> Self {
    State {
      stateful: StateMachine::default(),
      last_stateful: StateMachine::default(),
      mode: Mode::Normal,
    }
  }

  /// Convert struct to Arc pointer.
  pub fn to_arc(s: State) -> StateArc {
    Arc::new(RwLock::new(s))
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}

impl State {
  pub fn handle(
    &mut self,
    tree: TreeArc,
    buffers: BuffersManagerArc,
    event: Event,
  ) -> StateHandleResponse {
    // Update current mode.
    let state_mode = match self.stateful {
      StateMachine::NormalMode(_) => Some(Mode::Normal),
      StateMachine::VisualMode(_) => Some(Mode::Visual),
      StateMachine::SelectMode(_) => Some(Mode::Select),
      StateMachine::OperatorPendingMode(_) => Some(Mode::OperatorPending),
      StateMachine::InsertMode(_) => Some(Mode::Insert),
      StateMachine::CommandLineMode(_) => Some(Mode::CommandLine),
      StateMachine::TerminalMode(_) => Some(Mode::Terminal),
      _ => None,
    };
    if let Some(mode) = state_mode {
      self.mode = mode;
    }

    // Current stateful
    let stateful = self.stateful;

    let data_access = StatefulDataAccess::new(self, tree, buffers, event);
    let next_stateful = stateful.handle(data_access);
    trace!("Stateful now:{:?}, next:{:?}", stateful, next_stateful);

    // Save current stateful
    self.last_stateful = stateful;
    // Set next stateful
    self.stateful = next_stateful;

    StateHandleResponse::new(stateful, next_stateful)
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }
}
