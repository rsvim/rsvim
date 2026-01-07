//! The command-line search backward mode.

use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search backward mode.
pub struct CmdlineSearchBackward {}

impl Stateful for CmdlineSearchBackward {
  fn handle(&self, _data_access: StateDataAccess, _event: Event) -> State {
    State::CmdlineSearchBackward(CmdlineSearchBackward::default())
  }
  fn handle_op(&self, _data_access: StateDataAccess, _op: Operation) -> State {
    State::CmdlineSearchBackward(CmdlineSearchBackward::default())
  }
}
