//! The command-line search forward mode.

use crate::state::State;
use crate::state::StateContext;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search forward mode.
pub struct CmdlineSearchForward {}

impl Stateful for CmdlineSearchForward {
  fn handle(&self, _context: &StateContext, _event: Event) -> State {
    State::CmdlineSearchForward(CmdlineSearchForward::default())
  }
  fn handle_op(&self, _context: &StateContext, _op: Operation) -> State {
    State::CmdlineSearchForward(CmdlineSearchForward::default())
  }
}
