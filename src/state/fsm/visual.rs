//! The visual mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The visual editing mode.
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    data_access.state.set_mode(Mode::Visual);
    StatefulValue::VisualMode(VisualStateful::default())
  }
}
