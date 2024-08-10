//! The global editing state.

use crossterm::event::Event;
use parking_lot::Mutex;
use std::sync::{Arc, Weak};
use std::time::Duration;
use tracing::debug;

use crate::glovar;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  stateful: StatefulValue,
  last_stateful: StatefulValue,
}

pub type StateArc = Arc<Mutex<State>>;
pub type StateWk = Weak<Mutex<State>>;

impl State {
  pub fn new() -> Self {
    State {
      stateful: StatefulValue::default(),
      last_stateful: StatefulValue::default(),
    }
  }

  pub fn to_arc(s: State) -> StateArc {
    Arc::new(Mutex::new(s))
  }

  pub fn handle(self_: StateArc, tree: TreeArc, event: Event) {
    let stateful = {
      // Lock guard
      self_
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .stateful
    };

    let data_access = StatefulDataAccess::new(Arc::downgrade(&tree), Arc::downgrade(&self_), event);
    let next_stateful = stateful.handle(data_access);
    debug!("Stateful now:{:?}, next:{:?}", stateful, next_stateful);

    {
      // Lock guard
      let mut self2_ = self_
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap();
      self2_.last_stateful = self2_.stateful; // Save last stateful
      self2_.stateful = next_stateful; // Set next stateful
    }
  }

  pub fn mode(&self) -> Mode {
    self.stateful.mode()
  }
}

impl Default for State {
  fn default() -> Self {
    State::new()
  }
}
