//! The finite-state machine for VIM's editing mode.
//! The editing mode of the editor is a global state, and moves from one to another.

use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod normal;

pub trait Fsm {
  fn event(state: StateArc, tree: TreeArc) -> dyn Fsm;
}
