//! Pending async tasks.

use crate::js::JsRuntimeState;
use crate::js::JsTaskId;
use crate::js::JsTimerId;
use crate::msg;
use crate::msg::MasterMessage;
use crate::prelude::*;
use tokio::time::Instant;

pub type TimerCallback = Box<dyn FnMut() + 'static>;
pub type TaskCallback = Box<dyn FnMut(Option<AnyResult<Vec<u8>>>) + 'static>;

pub fn create_timer(
  state: &mut JsRuntimeState,
  timer_id: JsTimerId,
  delay: u64,
  repeated: bool,
  cb: TimerCallback,
) {
  state.pending_timers.insert(timer_id, cb);
  let start_at = Instant::now();
  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::TimeoutReq(msg::TimeoutReq {
      timer_id,
      start_at,
      delay,
      repeated,
    }),
  );
}

pub fn remove_timer(
  state: &mut JsRuntimeState,
  timer_id: JsTimerId,
) -> Option<JsTimerId> {
  state.pending_timers.remove(&timer_id).map(|_| timer_id)
}

pub fn create_import_loader(
  state: &mut JsRuntimeState,
  task_id: JsTaskId,
  specifier: &str,
  cb: TaskCallback,
) {
  state.pending_import_loaders.insert(task_id, cb);
  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::LoadImportReq(msg::LoadImportReq {
      task_id,
      specifier: specifier.to_string(),
    }),
  );
}
