//! Pending futures.

use crate::js::JsFuture;
use crate::js::JsFutureId;
use crate::js::JsRuntimeState;
use crate::js::command::ExCommandsManagerArc;
use crate::msg;
use crate::msg::JsMessage;
use crate::msg::MasterMessage;
use crate::prelude::*;
use crate::report_js_error;
use crate::state::ops::cmdline_ops;
use compact_str::ToCompactString;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

pub type TimeoutCallback = FnMut() -> Box<dyn JsFuture> + 'static;

pub struct PendingFutures {
  master_tx: Sender<MasterMessage>,
  commands: ExCommandsManagerArc,
  timer_queue:
    HashMap<JsFutureId, Box<dyn TimeoutCallback>,
  import_queue: HashMap<JsFutureId, Box<dyn FnMut()>>,
}

impl Debug for PendingFutures {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PendingFutures")
      .field(
        "timer_queue",
        &self
          .timer_queue
          .keys()
          .map(|k| (*k, "FnMut()".to_string()))
          .collect::<HashMap<JsFutureId, String>>(),
      )
      .finish()
  }
}

impl PendingFutures {
  pub fn new(
    master_tx: Sender<MasterMessage>,
    commands: ExCommandsManagerArc,
  ) -> Self {
    Self {
      master_tx,
      commands,
      timer_queue: HashMap::new(),
    }
  }

  pub fn set_timeout<F>(&mut self, delay: Duration, cb: F) -> JsFutureId
  where
    F: FnMut() -> Box<dyn JsFuture> + 'static,
  {
    let timer_id = next_timer_id();
    self.timer_queue.insert(timer_id, Box::new(cb));
    msg::sync_send_to_master(
      self.master_tx.clone(),
      MasterMessage::TimeoutReq(msg::TimeoutReq::new(timer_id, delay)),
    );
    timer_id
  }

  pub fn clear_timeout<F>(
    &mut self,
    timer_id: JsFutureId,
  ) -> Option<JsFutureId> {
    self.timer_queue.remove(&timer_id).map(|_| timer_id)
  }

  pub fn prepare(
    &mut self,
    state: &mut JsRuntimeState,
    jsrt_rx: Receiver<JsMessage>,
  ) -> Vec<Box<dyn JsFuture>> {
    let mut futures: Vec<Box<dyn JsFuture>> = vec![];

    while let Ok(msg) = jsrt_rx.try_recv() {
      match msg {
        JsMessage::TimeoutResp(resp) => {
          trace!("Prepare TimeoutResp:{:?}", resp.future_id);
          if self.timer_queue.contains_key(&resp.future_id) {
            trace!("Timer exists:{:?}", resp.future_id);
            let timer_cb = self.timer_queue.remove(&resp.future_id).unwrap();
            let fut = timer_cb();
            futures.push(fut);
          } else {
            trace!("Timer not exist:{:?}", resp.future_id);
          }
        }
        JsMessage::ExCommandReq(req) => {
          trace!("Prepare ExCommandReq:{:?}", req.future_id);
          let commands = self.commands.clone();
          let commands = lock!(commands);
          if let Some(command_cb) = commands.parse(&req.payload) {
            futures.push(Box::new(command_cb));
          } else {
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
          futures.push(load_cb);
        }
        JsMessage::TickAgainResp => trace!("Recv TickAgainResp"),
      }
    }
    futures
  }
}
