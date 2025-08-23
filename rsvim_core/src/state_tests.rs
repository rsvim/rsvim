use super::state::*;

use crate::state::fsm::StatefulValue;
use crate::state::mode::Mode;

#[test]
fn update_state_machine1() {
  let mut state = EditingState::new();
  assert_eq!(state.last_mode(), Mode::Normal);
  assert_eq!(state.mode(), Mode::Normal);
  state.update_state_machine(&StatefulValue::InsertMode(
    fsm::InsertStateful::default(),
  ));
  assert_eq!(state.last_mode(), Mode::Normal);
  assert_eq!(state.mode(), Mode::Insert);
}
