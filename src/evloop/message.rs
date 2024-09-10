//! Messages sync between sender and receiver, worker and master.

// Notification {

pub struct NotifyDone {
  result: Result<(), String>,
}

// Notification }
