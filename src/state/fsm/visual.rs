//! The visual mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::VisualMode(VisualStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Visual
  }
}
