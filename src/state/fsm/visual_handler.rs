//! The visual mode editing state.

use crate::state::fsm::{Fsm, FsmHandler};
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

#[derive(Debug, Copy, Clone, Default)]
pub struct VisualHandler {}

impl Fsm for VisualHandler {
  fn handle(&self, state: &mut State, tree: TreeArc) -> FsmHandler {
    FsmHandler::Visual(VisualHandler::default())
  }

  fn mode(&self) -> Mode {
    Mode::Visual
  }
}
