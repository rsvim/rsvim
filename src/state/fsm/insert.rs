//! The insert mode.

use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, data_access: StatefulDataAccessMut) -> StatefulValue {
    data_access.state.set_mode(Mode::Insert);
    StatefulValue::InsertMode(InsertStateful::default())
  }
}
