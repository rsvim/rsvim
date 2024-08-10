//! The global editing state.

use crossterm::event::Event;
use parking_lot::Mutex;
use std::io::Result as IoResult;
use std::sync::{Arc, Weak};
use tracing::debug;

use crate::state::fsm::{QuitStateful, Stateful, StatefulDataAccessMut, StatefulValue};
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

  pub fn handle(&mut self, tree: TreeArc, event: Event) -> IoResult<bool> {
    // Current stateful
    let stateful = self.stateful;

    match stateful {
      // Quit state, exit.
      StatefulValue::QuitState(_s) => {
        return Ok(false);
      }
      // Other states, continue.
      _ => { /* Skip */ }
    }

    let data_access = StatefulDataAccessMut::new(self, tree, event);
    let next_stateful = stateful.handle(data_access);
    debug!("Stateful now:{:?}, next:{:?}", stateful, next_stateful);

    // Save current stateful
    self.last_stateful = stateful;
    // Set next stateful
    self.stateful = next_stateful;

    Ok(true)
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
