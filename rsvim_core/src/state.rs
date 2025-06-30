//! Vim editing mode.

use crate::arc_mutex_ptr;
use crate::js::msg::EventLoopToJsRuntimeMessage;
use crate::state::fsm::StatefulValueDispatcher;
use crate::state::mode::Mode;

use paste::paste;
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
  pub fn new(jsrt_tick_dispatcher: Sender<EventLoopToJsRuntimeMessage>) -> Self {
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
}

impl State {
  pub fn update_state_machine(&mut self, next_stateful: &StatefulValueDispatcher) {
    // Save last stateful machine (only when it is different).
    if self.last_mode != self.mode {
      self.last_mode = self.mode;
    }

    // Update mode.
    let next_mode = match next_stateful {
      StatefulValueDispatcher::NormalMode(_) => Some(Mode::Normal),
      StatefulValueDispatcher::VisualMode(_) => Some(Mode::Visual),
      StatefulValueDispatcher::SelectMode(_) => Some(Mode::Select),
      StatefulValueDispatcher::OperatorPendingMode(_) => Some(Mode::OperatorPending),
      StatefulValueDispatcher::InsertMode(_) => Some(Mode::Insert),
      StatefulValueDispatcher::CommandLineExMode(_) => Some(Mode::CommandLineEx),
      StatefulValueDispatcher::CommandLineSearchForwardMode(_) => {
        Some(Mode::CommandLineSearchForward)
      }
      StatefulValueDispatcher::CommandLineSearchBackwardMode(_) => {
        Some(Mode::CommandLineSearchBackward)
      }
      StatefulValueDispatcher::TerminalMode(_) => Some(Mode::Terminal),
      // Internal states.
      StatefulValueDispatcher::QuitState(_) => None,
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
    state.update_state_machine(&StatefulValueDispatcher::InsertMode(
      fsm::InsertStateful::default(),
    ));
    assert_eq!(state.last_mode(), Mode::Normal);
    assert_eq!(state.mode(), Mode::Insert);
  }
}
