//! The normal mode editing state.

use crate::state::fsm::{Fsm, FsmEventHandler};
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

#[derive(Debug, Copy, Clone, Default)]
pub struct NormalEventHandler {}

impl Fsm for NormalEventHandler {
  fn handle_event(_state: StateArc, _tree: TreeArc) -> FsmEventHandler {
    FsmEventHandler::Normal(NormalEventHandler {})
  }

  fn mode(&self) -> Mode {
    Mode::Normal
  }
}
