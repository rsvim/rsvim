//! Vim editing mode.

use crate::js::msg::EventLoopToJsRuntimeMessage;
use crate::prelude::*;
use crate::state::fsm::StatefulValue;
use crate::state::mode::Mode;

use tokio::sync::mpsc::Sender;

pub mod fsm;
pub mod mode;
pub mod ops;

#[derive(Debug, Clone)]
pub struct State {
  // Current editing mode.
  mode: Mode,
  // Last editing mode.
  last_mode: Mode,

  // Js runtime tick dispatcher
  jsrt_tick_dispatcher: Sender<EventLoopToJsRuntimeMessage>,
}

arc_mutex_ptr!(State);

impl State {
  pub fn new(
    jsrt_tick_dispatcher: Sender<EventLoopToJsRuntimeMessage>,
  ) -> Self {
    State {
      mode: Mode::Normal,
      last_mode: Mode::Normal,
      jsrt_tick_dispatcher,
    }
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }

  pub fn last_mode(&self) -> Mode {
    self.last_mode
  }

  pub fn jsrt_tick_dispatcher(&self) -> &Sender<EventLoopToJsRuntimeMessage> {
    &self.jsrt_tick_dispatcher
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
      StatefulValue::CommandLineMessageMode(_) => {
        Some(Mode::CommandLineMessage)
      }
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

#[cfg(test)]
mod tests {
  use super::*;

  use tokio::sync::mpsc::channel;

  #[test]
  fn update_state_machine1() {
    let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
    let mut state = State::new(jsrt_tick_dispatcher);
    assert_eq!(state.last_mode(), Mode::Normal);
    assert_eq!(state.mode(), Mode::Normal);
    state.update_state_machine(&StatefulValue::InsertMode(
      fsm::InsertStateful::default(),
    ));
    assert_eq!(state.last_mode(), Mode::Normal);
    assert_eq!(state.mode(), Mode::Insert);
  }
}
