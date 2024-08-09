//! The normal mode editing state.

use crate::state::fsm::{Fsm, FsmHandler};
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

#[derive(Debug, Copy, Clone, Default)]
pub struct NormalHandler {}

impl Fsm for NormalHandler {
  fn handle(_state: StateArc, _tree: TreeArc) -> FsmHandler {
    FsmHandler::Normal(NormalHandler::default())
  }

  fn mode(&self) -> Mode {
    Mode::Normal
  }
}
