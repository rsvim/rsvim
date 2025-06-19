//! The command-line mode, search forward variant.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line mode, search forward variant.
pub struct CommandLineSearchForwardVariantStateful {}

impl Stateful for CommandLineSearchForwardVariantStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineSearchForwardMode(CommandLineSearchForwardVariantStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValue {
    StatefulValue::CommandLineSearchForwardMode(CommandLineSearchForwardVariantStateful::default())
  }
}
