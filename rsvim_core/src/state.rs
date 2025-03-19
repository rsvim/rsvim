//! Vim editing mode.

use crate::state::fsm::StatefulValue;
use crate::state::mode::Mode;

use parking_lot::RwLock;
use std::sync::{Arc, Weak};

pub mod command;
pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  // Current editing mode.
  mode: Mode,
  // Last editing mode.
  last_mode: Mode,
}

pub type StateArc = Arc<RwLock<State>>;
pub type StateWk = Weak<RwLock<State>>;

impl State {
  pub fn new() -> Self {
    State {
      mode: Mode::Normal,
      last_mode: Mode::Normal,
    }
  }

  /// Convert struct to Arc pointer.
  pub fn to_arc(s: State) -> StateArc {
    Arc::new(RwLock::new(s))
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }

  pub fn last_mode(&self) -> Mode {
    self.last_mode
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}

impl State {
  pub fn update_state_machine(&mut self, next_stateful: &StatefulValue) {
    // Save last stateful machine.
    self.last_mode = self.mode;

    // Update mode.
    let next_mode = match next_stateful {
      StatefulValue::NormalMode(_) => Some(Mode::Normal),
      StatefulValue::VisualMode(_) => Some(Mode::Visual),
      StatefulValue::SelectMode(_) => Some(Mode::Select),
      StatefulValue::OperatorPendingMode(_) => Some(Mode::OperatorPending),
      StatefulValue::InsertMode(_) => Some(Mode::Insert),
      StatefulValue::CommandLineMode(_) => Some(Mode::CommandLine),
      StatefulValue::TerminalMode(_) => Some(Mode::Terminal),
      // Internal states.
      StatefulValue::QuitState(_) => None,
    };
    if let Some(mode) = next_mode {
      self.mode = mode;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn update_state_machine1() {
    let mut state = State::new();
    assert_eq!(state.last_mode(), Mode::Normal);
    assert_eq!(state.mode(), Mode::Normal);
    state.update_state_machine(&StatefulValue::InsertMode(fsm::InsertStateful::default()));
    assert_eq!(state.last_mode(), Mode::Normal);
    assert_eq!(state.mode(), Mode::Insert);
  }
}
