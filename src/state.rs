//! The global editing state.

use parking_lot::Mutex;
use std::sync::{Arc, Weak};
use tracing::debug;

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

pub mod fsm;
pub mod mode;

#[derive(Debug, Clone)]
pub struct State {
  stateful: NextStateful,
}

pub type StateArc = Arc<Mutex<State>>;
pub type StateWk = Weak<Mutex<State>>;

impl State {
  pub fn new() -> Self {
    State {
      stateful: NextStateful::default(),
    }
  }

  pub fn to_arc(s: State) -> StateArc {
    Arc::new(Mutex::new(s))
  }

  pub fn handle(&mut self, tree: TreeArc) {
    let data_access = StatefulDataAccess::new(tree);
    let next_stateful = self.stateful.handle(data_access);
    debug!("Stateful now:{:?}, next:{:?}", self.stateful, next_stateful);
    self.stateful = next_stateful;
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
