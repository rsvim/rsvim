//! Pending futures.

use crate::js::JsFutureId;
use crate::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Default)]
pub struct PendingFutures {
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
  pub fn timer_req(&mut self, cb: Box<dyn FnMut() + 'static>) {
    let timer_id = next_timer_id();
  }
}
