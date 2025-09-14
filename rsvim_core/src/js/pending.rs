//! Pending async tasks.

use crate::js::JsRuntimeState;
use crate::js::JsTaskId;
use crate::js::JsTimerId;
use crate::js::next_task_id;
use crate::js::next_timer_id;
use crate::msg;
use crate::msg::MasterMessage;
use crate::prelude::*;
use tokio::time::Instant;

pub type TimerCallback = Box<dyn FnMut() + 'static>;
pub type TaskCallback = Box<dyn FnMut(Option<AnyResult<Vec<u8>>>) + 'static>;

pub fn create_timer(
  state: &mut JsRuntimeState,
  expire_at: Instant,
  cb: TimerCallback,
) -> JsTimerId {
  let timer_id = next_timer_id();
  state.pending_timers.insert(timer_id, cb);
  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::TimeoutReq(msg::TimeoutReq {
      timer_id,
      expire_at,
    }),
  );
  timer_id
}

pub fn remove_timer(
  state: &mut JsRuntimeState,
  timer_id: JsTimerId,
) -> Option<JsTimerId> {
  state.pending_timers.remove(&timer_id).map(|_| timer_id)
}

pub fn create_loader(
  state: &mut JsRuntimeState,
  specifier: &str,
  cb: TaskCallback,
) -> JsTaskId {
  let task_id = next_task_id();
  state.pending_imports.insert(task_id, cb);
  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::LoadImportReq(msg::LoadImportReq {
      task_id,
      specifier: specifier.to_string(),
    }),
  );
  task_id
}
