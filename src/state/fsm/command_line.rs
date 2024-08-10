//! The command-line mode.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct CommandLineStateful {}

impl Stateful for CommandLineStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::CommandLine(CommandLineStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::CommandLine
  }
}
