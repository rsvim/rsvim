//! The global editing state.

use parking_lot::Mutex;
use std::sync::{Arc, Weak};
use tracing::debug;

use crate::state::fsm::{Fsm, FsmEventHandler};
use crate::state::mode::{Mode, Modes};

pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  current_mode: Mode,
  current_event_handler: FsmEventHandler,
}

pub type StateArc = Arc<Mutex<State>>;
pub type StateWk = Weak<Mutex<State>>;

impl State {
  pub fn new() -> Self {
    State {
      current_mode: Mode::Normal,
      current_event_handler: FsmEventHandler::default(),
    }
  }

  pub fn to_arc(s: State) -> StateArc {
    Arc::new(Mutex::new(s))
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}
