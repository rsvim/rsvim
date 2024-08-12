//! The command-line mode.

use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The command-line editing mode.
pub struct CommandLineStateful {}

impl Stateful for CommandLineStateful {
  fn handle(&self, data_access: StatefulDataAccessMut) -> StatefulValue {
    data_access.state.set_mode(Mode::CommandLine);
    StatefulValue::CommandLineMode(CommandLineStateful::default())
  }
}
