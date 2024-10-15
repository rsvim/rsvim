//! Messages used inside [`EventLoop`](crate::evloop::EventLoop).

// Worker to Master message {

#[derive(Debug)]
/// Message.
pub enum WorkerToMasterMessage {
  ReadBytes(ReadBytes),
}

#[derive(Debug, Default)]
/// Read bytes.
pub struct ReadBytes {
  pub bytes: usize,
}

impl ReadBytes {
  pub fn new(bytes: usize) -> Self {
    ReadBytes { bytes }
  }
}

// Worker to Master message }
