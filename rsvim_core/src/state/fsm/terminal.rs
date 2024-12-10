//! The terminal mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default)]
/// The terminal editing mode.
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::TerminalMode(TerminalStateful::default())
  }
}
