//! The terminal mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::TerminalMode(TerminalStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Terminal
  }
}
