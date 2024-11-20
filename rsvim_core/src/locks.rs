//! Lock utils

/// Alias to `($id).try_read_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! rlock {
  ($id:expr) => {
    ($id).try_read_for(envar::MUTEX_TIMEOUT()).unwrap()
  };
}

/// Alias to `($id).try_write_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! wlock {
  ($id:expr) => {
    ($id).try_write_for(envar::MUTEX_TIMEOUT()).unwrap()
  };
}
