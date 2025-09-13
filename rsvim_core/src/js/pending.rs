//! Pending futures, i.e. async tasks.

use crate::js::JsFutureId;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

fn next_timer_id() -> JsFutureId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub struct PendingFutures {}
