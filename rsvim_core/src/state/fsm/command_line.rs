//! The command-line mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line editing mode.
pub struct CommandLineStateful {}

impl Stateful for CommandLineStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineMode(CommandLineStateful::default())
  }
}
