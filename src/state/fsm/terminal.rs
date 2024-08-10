//! The terminal mode.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::Terminal(TerminalStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Terminal
  }
}
