//! The command-line mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The command-line editing mode.
pub struct CommandLineStateful {}

impl Stateful for CommandLineStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    data_access.state.set_mode(Mode::CommandLine);
    StatefulValue::CommandLineMode(CommandLineStateful::default())
  }
}
