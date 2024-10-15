//! The terminal mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The terminal editing mode.
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    data_access.state.set_mode(Mode::Terminal);
    StatefulValue::TerminalMode(TerminalStateful::default())
  }
}
