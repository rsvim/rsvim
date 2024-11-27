//! Messages used inside [`EventLoop`](crate::evloop::EventLoop).

// Worker to Master message {

#[derive(Debug)]
/// Message.
pub enum WorkerToMasterMessage {
  BufferLoadedBytes(BufferLoadedBytes),
}

#[derive(Debug, Default)]
/// Read bytes.
pub struct BufferLoadedBytes {
  pub bytes: usize,
}

impl BufferLoadedBytes {
  pub fn new(bytes: usize) -> Self {
    BufferLoadedBytes { bytes }
  }
}

// Worker to Master message }
