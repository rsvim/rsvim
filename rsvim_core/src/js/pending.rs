//! Pending futures, i.e. async tasks.

use crate::js::JsFutureId;
use crate::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

pub struct PendingFutures {
  timer_queue: BTreeMap<JsFutureId, Box<dyn FnMut() + 'static>>,
}

impl Debug for PendingFutures {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PendingFutures").finish()
  }
}
