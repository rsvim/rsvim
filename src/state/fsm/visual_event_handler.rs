//! The visual mode editing state.

use crate::state::fsm::{Fsm, FsmEventHandler};
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

#[derive(Debug, Copy, Clone, Default)]
pub struct VisualEventHandler {}

impl Fsm for VisualEventHandler {
  fn handle_event(_state: StateArc, _tree: TreeArc) -> FsmEventHandler {
    FsmEventHandler::Visual(VisualEventHandler {})
  }

  fn mode(&self) -> Mode {
    Mode::Visual
  }
}
