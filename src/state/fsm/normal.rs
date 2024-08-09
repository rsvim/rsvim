//! The normal mode editing state.

use crate::state::fsm::Fsm;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub struct NormalFsm {}

impl Fsm for NormalFsm {
  fn event(state: StateArc, tree: TreeArc) -> dyn Fsm {
    Self
  }
}
