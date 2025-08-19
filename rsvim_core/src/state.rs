//! Vim editing mode.

use crate::prelude::*;
use crate::state::fsm::StatefulValue;
use crate::state::mode::Mode;

pub mod fsm;
pub mod mode;
pub mod ops;

#[derive(Debug, Clone)]
pub struct State {
  // Current editing mode.
  mode: Mode,
  // Last editing mode.
  last_mode: Mode,
}

arc_mutex_ptr!(State);

impl State {
  pub fn new() -> Self {
    State {
      mode: Mode::Normal,
      last_mode: Mode::Normal,
    }
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }

  pub fn last_mode(&self) -> Mode {
    self.last_mode
  }
}

impl State {
  pub fn update_state_machine(&mut self, next_stateful: &StatefulValue) {
    // Save last stateful machine (only when it is different).
    if self.last_mode != self.mode {
      self.last_mode = self.mode;
    }

    // Update mode.
    let next_mode = match next_stateful {
      StatefulValue::NormalMode(_) => Some(Mode::Normal),
      StatefulValue::VisualMode(_) => Some(Mode::Visual),
      StatefulValue::SelectMode(_) => Some(Mode::Select),
      StatefulValue::OperatorPendingMode(_) => Some(Mode::OperatorPending),
      StatefulValue::InsertMode(_) => Some(Mode::Insert),
      StatefulValue::CommandLineExMode(_) => Some(Mode::CommandLineEx),
      StatefulValue::CommandLineSearchForwardMode(_) => {
        Some(Mode::CommandLineSearchForward)
      }
      StatefulValue::CommandLineSearchBackwardMode(_) => {
        Some(Mode::CommandLineSearchBackward)
      }
      StatefulValue::TerminalMode(_) => Some(Mode::Terminal),
      // Internal states.
      StatefulValue::QuitState(_) => None,
    };

    if let Some(mode) = next_mode {
      self.mode = mode;
    }
  }
}

impl Default for State {
  fn default() -> Self {
    Self::new()
  }
}
