//! The terminal mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, data_access: StatefulDataAccessMut) -> StatefulValue {
    data_access.state.set_mode(Mode::Terminal);
    StatefulValue::TerminalMode(TerminalStateful::default())
  }
}
