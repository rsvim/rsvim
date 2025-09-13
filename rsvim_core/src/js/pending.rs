//! Pending async tasks.

use crate::js::JsFutureId;
use crate::js::JsRuntimeStateRc;
use crate::js::module::EsModuleFuture;
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

type JsTaskId = usize;

pub type TimerCallback = Box<dyn FnMut() + 'static>;
pub type TaskCallback =
  Box<dyn FnMut() -> Option<AnyResult<Vec<u8>>> + 'static>;

/// Next timer task ID.
///
/// NOTE: Start form 1.
fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

fn next_task_id() -> JsTaskId {
  static VALUE: AtomicUsize = AtomicUsize::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
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

  pub fn create_timer(
    &mut self,
    expire_at: Instant,
    cb: TimerCallback,
  ) -> JsFutureId {
    let timer_id = next_timer_id();
    self.timers.insert(timer_id, cb);
    msg::sync_send_to_master(
      self.master_tx.clone(),
      MasterMessage::TimeoutReq(msg::TimeoutReq {
        timer_id,
        expire_at,
      }),
    );
    timer_id
  }

  pub fn remove_timer(&mut self, timer_id: JsFutureId) -> Option<JsFutureId> {
    self.timers.remove(&timer_id).map(|_| timer_id)
  }

  pub fn load_import(&mut self, specifier: &str, cb: TaskCallback) -> JsTaskId {
    let task_id = next_task_id();
    self.load_imports.insert(task_id, cb);
    msg::sync_send_to_master(
      self.master_tx.clone(),
      MasterMessage::LoadImportReq(msg::LoadImportReq {
        specifier: specifier.to_string(),
      }),
    );
    task_id
  }
}

impl PendingQueue {
  pub fn prepare(&mut self, state_rc: JsRuntimeStateRc) {
    while let Ok(msg) = state_rc.borrow().jsrt_rx.try_recv() {
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
          trace!("Recv LoadImportResp:{:?}", resp.future_id);
          debug_assert!(state.pending_futures.contains_key(&resp.future_id));
          let mut load_cb =
            state.pending_futures.remove(&resp.future_id).unwrap();
          let load_cb_impl = load_cb.downcast_mut::<EsModuleFuture>().unwrap();
          load_cb_impl.source = Some(resp.source);
          self.futures.push(load_cb);
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
