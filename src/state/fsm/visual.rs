//! The visual mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, data_access: StatefulDataAccessMut) -> StatefulValue {
    data_access.state.set_mode(Mode::Visual);
    StatefulValue::VisualMode(VisualStateful::default())
  }
}
