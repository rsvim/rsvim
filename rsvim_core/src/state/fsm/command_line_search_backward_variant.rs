//! The command-line mode, search pattern variant.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line mode, search pattern variant.
pub struct CommandLineSearchPatternVariantStateful {}

impl Stateful for CommandLineSearchPatternVariantStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineModeSearchPatternVariant(
      CommandLineSearchPatternVariantStateful::default(),
    )
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValue {
    StatefulValue::CommandLineModeSearchPatternVariant(
      CommandLineSearchPatternVariantStateful::default(),
    )
  }
}
