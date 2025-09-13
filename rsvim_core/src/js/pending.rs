//! Pending futures.

use crate::js::JsFutureId;
use crate::msg;
use crate::msg::MasterMessage;
use crate::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

pub struct PendingFutures {
  master_tx: Sender<MasterMessage>,
  timer_queue: HashMap<JsFutureId, Box<dyn FnMut() + 'static>>,
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
  pub fn new(master_tx: Sender<MasterMessage>) -> Self {
    Self {
      master_tx,
      timer_queue: HashMap::new(),
    }
  }

  pub fn set_timeout<F>(&mut self, delay: Duration, cb: F) -> JsFutureId
  where
    F: FnMut() + 'static,
  {
    let timer_id = next_timer_id();
    self.timer_queue.insert(timer_id, Box::new(cb));
    msg::sync_send_to_master(
      self.master_tx.clone(),
      MasterMessage::TimeoutReq(msg::TimeoutReq::new(timer_id, delay)),
    );
    timer_id
  }
}
