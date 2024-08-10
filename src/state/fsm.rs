//! The finite-state machine for VIM's editing mode.
//! The editing mode of the editor is a global state, and moves from one to another.

use crate::state::fsm::normal_stateful::NormalStateful;
use crate::state::fsm::visual_handler::VisualHandler;
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod normal_stateful;
pub mod visual_handler;

pub trait Stateful {
  /// Handle user's keyboard/mouse event, this method can access the state and update UI tree.
  ///
  /// Returns next state.
  fn handle(&self, tree: TreeArc) -> NextStateful;

  /// Returns VIM mode.
  fn mode(&self) -> Mode;
}

#[derive(Debug, Copy, Clone)]
pub enum NextStateful {
  Normal(NormalStateful),
  Visual(VisualHandler),
}

impl Default for NextStateful {
  fn default() -> Self {
    NextStateful::Normal(NormalStateful::default())
  }
}

impl NextStateful {
  pub fn handle(&self, tree: TreeArc) -> NextStateful {
    match self {
      NextStateful::Normal(h) => h.handle(tree),
      NextStateful::Visual(h) => h.handle(tree),
    }
  }

  pub fn mode(&self) -> Mode {
    match self {
      NextStateful::Normal(h) => h.mode(),
      NextStateful::Visual(h) => h.mode(),
    }
  }
}
