//! The command-line search forward mode.

use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search forward mode.
pub struct CommandLineSearchForwardStateful {}

impl Stateful for CommandLineSearchForwardStateful {
  fn handle(&self, _data_access: StateDataAccess, _event: Event) -> State {
    State::CommandLineSearchForwardMode(
      CommandLineSearchForwardStateful::default(),
    )
  }
  fn handle_op(&self, _data_access: StateDataAccess, _op: Operation) -> State {
    State::CommandLineSearchForwardMode(
      CommandLineSearchForwardStateful::default(),
    )
  }
}
