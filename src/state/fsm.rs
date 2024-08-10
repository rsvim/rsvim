//! The finite-state machine for VIM's editing mode.
//! The editing mode of the editor is a global state, and moves from one to another.

use crate::state::fsm::normal_handler::NormalHandler;
use crate::state::fsm::visual_handler::VisualHandler;
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod normal_handler;
pub mod visual_handler;

pub trait Fsm {
  /// Handle user's keyboard/mouse event, this method can access the state and update UI tree.
  ///
  /// Returns next Fsm state.
  fn handle(&self, state: &mut State, tree: TreeArc) -> FsmHandler;

  /// Returns VIM mode.
  fn mode(&self) -> Mode;
}

#[derive(Debug, Copy, Clone)]
pub enum FsmHandler {
  Normal(NormalHandler),
  Visual(VisualHandler),
}

impl Default for FsmHandler {
  fn default() -> Self {
    FsmHandler::Normal(NormalHandler::default())
  }
}

impl FsmHandler {
  pub fn handle(&self, state: &mut State, tree: TreeArc) -> FsmHandler {
    match self {
      FsmHandler::Normal(h) => h.handle(state, tree),
      FsmHandler::Visual(h) => h.handle(state, tree),
    }
  }

  pub fn mode(&self) -> Mode {
    match self {
      FsmHandler::Normal(h) => h.mode(),
      FsmHandler::Visual(h) => h.mode(),
    }
  }
}
