//! Vim editing mode.

use crate::state::fsm::{StatefulValue, StatefulValueArc};
use crate::state::mode::Mode;

use parking_lot::RwLock;
use std::sync::{Arc, Weak};

pub mod command;
pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  // Last finite-state machine.
  last_state_machine: StatefulValue,

  // Editing mode.
  mode: Mode,
}

pub type StateArc = Arc<RwLock<State>>;
pub type StateWk = Weak<RwLock<State>>;

impl State {
  pub fn new() -> Self {
    State {
      last_state_machine: StatefulValue::default(),
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
  pub fn update_stateful_machine(
    &mut self,
    last_stateful: StatefulValueArc,
    next_stateful: StatefulValueArc,
  ) {
    // Save last stateful machine.
    self.last_state_machine = last_stateful;

    // Update mode.
    let next_mode = match *next_stateful {
      StatefulValue::NormalMode(_) => Some(Mode::Normal),
      StatefulValue::VisualMode(_) => Some(Mode::Visual),
      StatefulValue::SelectMode(_) => Some(Mode::Select),
      StatefulValue::OperatorPendingMode(_) => Some(Mode::OperatorPending),
      StatefulValue::InsertMode(_) => Some(Mode::Insert),
      StatefulValue::CommandLineMode(_) => Some(Mode::CommandLine),
      StatefulValue::TerminalMode(_) => Some(Mode::Terminal),
      _ => None,
    };
    if let Some(mode) = next_mode {
      self.mode = mode;
    }
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }
}
