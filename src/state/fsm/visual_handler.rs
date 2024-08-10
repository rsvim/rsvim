//! The visual mode editing state.

use crate::state::fsm::{NextStateful, Stateful};
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

#[derive(Debug, Copy, Clone, Default)]
pub struct VisualHandler {}

impl Stateful for VisualHandler {
  fn handle(&self, tree: TreeArc) -> NextStateful {
    NextStateful::Visual(VisualHandler::default())
  }

  fn mode(&self) -> Mode {
    Mode::Visual
  }
}
