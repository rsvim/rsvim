//! The normal mode editing state.

use crate::state::fsm::{NextStateful, Stateful};
use crate::state::mode::Mode;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

#[derive(Debug, Copy, Clone, Default)]
pub struct NormalStateful {}

impl Stateful for NormalStateful {
  fn handle(&self, tree: TreeArc) -> NextStateful {
    NextStateful::Normal(NormalStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Normal
  }
}
