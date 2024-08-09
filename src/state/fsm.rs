//! The finite-state machine for VIM's editing mode.
//! The editing mode of the editor is a global state, and moves from one to another.

use crate::state::fsm::normal_event_handler::NormalEventHandler;
use crate::state::fsm::visual_event_handler::VisualEventHandler;
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod normal_event_handler;
pub mod visual_event_handler;

pub trait Fsm {
  /// Handle user's keyboard/mouse event, this method can access the state and update UI tree.
  ///
  /// Returns next Fsm state.
  fn handle_event(state: StateArc, tree: TreeArc) -> FsmEventHandler;

  /// Returns VIM mode.
  fn mode(&self) -> Mode;
}

#[derive(Debug, Copy, Clone)]
pub enum FsmEventHandler {
  Normal(NormalEventHandler),
  Visual(VisualEventHandler),
}

impl Default for FsmEventHandler {
  fn default() -> Self {
    FsmEventHandler::Normal(NormalEventHandler::default())
  }
}

impl FsmEventHandler {
  pub fn mode(&self) -> Mode {
    match self {
      FsmEventHandler::Normal(h) => h.mode(),
      FsmEventHandler::Visual(h) => h.mode(),
    }
  }
}
