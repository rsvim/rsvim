//! Pending async tasks.

use crate::js::JsFutureId;
use crate::js::JsRuntimeState;
use crate::js::JsRuntimeStateRc;
use crate::js::JsTaskId;
use crate::msg;
use crate::msg::JsMessage;
use crate::msg::MasterMessage;
use crate::prelude::*;
use crate::report_js_error;
use crate::state::ops::cmdline_ops;
use compact_str::ToCompactString;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc::Sender;
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

pub struct PendingQueue {
  master_tx: Sender<MasterMessage>,
  timers: HashMap<JsFutureId, TimerCallback>,
  load_imports: HashMap<JsTaskId, TaskCallback>,
}

impl PendingQueue {
  pub fn new(master_tx: Sender<MasterMessage>) -> Self {
    Self {
      master_tx,
      timers: HashMap::new(),
      load_imports: HashMap::new(),
    }
  }
}

impl PendingQueue {
  pub fn prepare(&mut self, state_rc: JsRuntimeStateRc) {
    let mut messages: Vec<JsMessage> = vec![];

    // Drain all pending messages
    {
      let state = state_rc.borrow();
      while let Ok(msg) = state.jsrt_rx.try_recv() {
        messages.push(msg);
      }
      // Drop state
    }

    for msg in messages {
      match msg {
        JsMessage::TimeoutResp(resp) => {
          trace!("Recv TimeResp:{:?}", resp.timer_id);
          match self.timers.remove(&resp.timer_id) {
            Some(mut timer_cb) => {
              timer_cb();
            }
            None => {
              // Only execute 'timeout_cb' if timer_id still exists,
              // otherwise it means the 'timer_cb' is been cleared by
              // `clear_timeout` API.
            }
          }
        }
        JsMessage::ExCommandReq(req) => {
          trace!("Recv ExCommandReq:{:?}", req.future_id);
          let mut state = state_rc.borrow();
          let commands = state.commands.clone();
          let commands = lock!(commands);
          if let Some(command_cb) = commands.parse(&req.payload) {
            state.pending_futures.push(Box::new(command_cb));
          } else {
            // Print error message
            let e = format!("Error: invalid command {:?}", req.payload);
            report_js_error!(state, e);
          }
        }
        JsMessage::LoadImportResp(resp) => {
          trace!("Recv LoadImportResp:{:?}", resp.task_id);
          match self.load_imports.remove(&resp.task_id) {
            Some(mut load_cb) => {
              load_cb(resp.maybe_source);
            }
            None => unreachable!(),
          }
        }
        JsMessage::TickAgainResp => trace!("Recv TickAgainResp"),
      }
    }
  }
}

impl Debug for PendingQueue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PendingQueue")
      .field(
        "timer_queue",
        &self.timers.keys().map(|k| *k).collect::<HashSet<_>>(),
      )
      .finish()
  }
}
