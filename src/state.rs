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
  handler: FsmEventHandler,
}

pub type StateArc = Arc<Mutex<State>>;
pub type StateWk = Weak<Mutex<State>>;

impl State {
  pub fn new() -> Self {
    State {
      handler: FsmEventHandler::default(),
    }
  }

  pub fn to_arc(s: State) -> StateArc {
    Arc::new(Mutex::new(s))
  }

  pub fn mode(&self) -> Mode {
    self.handler.mode()
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}
