//! Mutex utility.

#[macro_export]
macro_rules! lock {
  ($id:expr) => {
    ($id).lock().unwrap()
  };
}
