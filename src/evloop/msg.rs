//! Messages used inside [`EventLoop`](crate::evloop::EventLoop).

// Worker to Master message {

#[derive(Debug)]
/// Message.
pub enum WorkerToMasterMessage {
  Dummy(Dummy),
}

#[derive(Debug, Default)]
/// Dummy message.
pub struct Dummy {}

impl Dummy {
  pub fn new() -> Self {
    Dummy {}
  }
}

// Worker to Master message }
