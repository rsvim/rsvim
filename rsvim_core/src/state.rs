//! Vim editing mode.

use crate::buf::BuffersManagerArc;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;
use crate::wlock;

use crossterm::event::Event;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};
use tracing::trace;

pub mod command;
pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  stateful: StatefulValue,
  last_stateful: StatefulValue,

  // Editing mode.
  mode: Mode,
}

#[derive(Debug, Copy, Clone)]
pub struct StateHandleResponse {
  pub stateful: StatefulValue,
  pub next_stateful: StatefulValue,
}

impl StateHandleResponse {
  pub fn new(stateful: StatefulValue, next_stateful: StatefulValue) -> Self {
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
      stateful: StatefulValue::default(),
      last_stateful: StatefulValue::default(),
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
    self_: StateArc,
    tree: TreeArc,
    buffers: BuffersManagerArc,
    event: Event,
  ) -> StateHandleResponse {
    let mut self_ = wlock!(self_);

    // Update current mode.
    let state_mode = match self_.stateful {
      StatefulValue::NormalMode(_) => Some(Mode::Normal),
      StatefulValue::VisualMode(_) => Some(Mode::Visual),
      StatefulValue::SelectMode(_) => Some(Mode::Select),
      StatefulValue::OperatorPendingMode(_) => Some(Mode::OperatorPending),
      StatefulValue::InsertMode(_) => Some(Mode::Insert),
      StatefulValue::CommandLineMode(_) => Some(Mode::CommandLine),
      StatefulValue::TerminalMode(_) => Some(Mode::Terminal),
      _ => None,
    };
    if let Some(mode) = state_mode {
      self_.mode = mode;
    }

    // Current stateful
    let stateful = self_.stateful;

    let data_access = StatefulDataAccess::new(self.clone(), tree, buffers, event);
    let next_stateful = stateful.handle(data_access);
    trace!("Stateful now:{:?}, next:{:?}", stateful, next_stateful);

    // Save current stateful
    self_.last_stateful = stateful;
    // Set next stateful
    self_.stateful = next_stateful;

    StateHandleResponse::new(stateful, next_stateful)
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }
}
