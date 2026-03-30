//! The command-line search backward mode.

use crate::state::State;
use crate::state::StateContext;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search backward mode.
pub struct CmdlineSearchBackward {}

impl Stateful for CmdlineSearchBackward {
  fn handle(&self, _context: StateContext, _event: Event) -> State {
    State::CmdlineSearchBackward(CmdlineSearchBackward::default())
  }
  fn handle_op(&self, _context: StateContext, _op: Operation) -> State {
    State::CmdlineSearchBackward(CmdlineSearchBackward::default())
  }
}
