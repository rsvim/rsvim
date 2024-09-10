//! Messages sync between sender and receiver, worker and master.

// Notification {

#[derive(Debug, Default)]
pub struct Dummy {}

impl Dummy {
  pub fn new() -> Self {
    Dummy {}
  }
}

#[derive(Debug)]
/// Notification.
pub enum Notify {
  Dummy(Dummy),
}

// Notification }
