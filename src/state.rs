//! The global editing state.

use crossterm::event::Event;
use parking_lot::Mutex;
use std::sync::{Arc, Weak};
use tracing::debug;

use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  stateful: StatefulValue,
  last_stateful: StatefulValue,

  // Editing mode
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

pub type StateArc = Arc<Mutex<State>>;
pub type StateWk = Weak<Mutex<State>>;

impl State {
  pub fn new() -> Self {
    State {
      stateful: StatefulValue::default(),
      last_stateful: StatefulValue::default(),
      mode: Mode::Normal,
    }
  }

  pub fn to_arc(s: State) -> StateArc {
    Arc::new(Mutex::new(s))
  }

  pub fn handle(&mut self, tree: TreeArc, event: Event) -> StateHandleResponse {
    // Current stateful
    let stateful = self.stateful;

    let data_access = StatefulDataAccessMut::new(self, tree, event);
    let next_stateful = stateful.handle(data_access);
    debug!("Stateful now:{:?}, next:{:?}", stateful, next_stateful);

    // Save current stateful
    self.last_stateful = stateful;
    // Set next stateful
    self.stateful = next_stateful;

    StateHandleResponse::new(stateful, next_stateful)
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }

  pub fn set_mode(&mut self, mode: Mode) -> Mode {
    let last_mod = self.mode;
    self.mode = mode;
    last_mod
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}
