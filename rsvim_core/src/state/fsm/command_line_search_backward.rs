//! The command-line search backward mode.

use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search backward mode.
pub struct CommandLineSearchBackwardStateful {}

impl Stateful for CommandLineSearchBackwardStateful {
  fn handle(&self, _data_access: StateDataAccess, _event: Event) -> State {
    State::CommandLineSearchBackwardMode(
      CommandLineSearchBackwardStateful::default(),
    )
  }
  fn handle_op(&self, _data_access: StateDataAccess, _op: Operation) -> State {
    State::CommandLineSearchBackwardMode(
      CommandLineSearchBackwardStateful::default(),
    )
  }
}
