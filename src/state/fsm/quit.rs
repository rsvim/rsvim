//! The quit state.

use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};

#[derive(Debug, Copy, Clone, Default)]
/// The quit state
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, _data_access: StatefulDataAccessMut) -> StatefulValue {
    unreachable!("Never enter QuitStateful");
  }
}
