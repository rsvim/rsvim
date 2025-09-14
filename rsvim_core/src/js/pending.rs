//! Pending async tasks.

use crate::js::JsFutureId;
use crate::js::JsRuntimeState;
use crate::js::JsTaskId;
use crate::msg;
use crate::msg::MasterMessage;
use crate::prelude::*;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::time::Instant;

pub type TimerCallback = Box<dyn FnMut() + 'static>;
pub type TaskCallback = Box<dyn FnMut(Option<AnyResult<Vec<u8>>>) + 'static>;

pub fn next_task_id() -> JsTaskId {
  static VALUE: AtomicUsize = AtomicUsize::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

/// Next timer task ID.
///
/// NOTE: Start form 1.
pub fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

pub fn create_timer(
  state: &mut JsRuntimeState,
  expire_at: Instant,
  cb: TimerCallback,
) -> JsFutureId {
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
  timer_id: JsFutureId,
) -> Option<JsFutureId> {
  state.pending_timers.remove(&timer_id).map(|_| timer_id)
}

pub fn load_import(
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
