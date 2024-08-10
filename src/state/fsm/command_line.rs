//! The command-line mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct CommandLineStateful {}

impl Stateful for CommandLineStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineMode(CommandLineStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::CommandLine
  }
}
