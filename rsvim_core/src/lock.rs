//! Mutex utility.

/// Alias to `($id).try_read_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! mc_rlock {
  ($id:expr) => {
    ($id).try_read_for($crate::envar::MUTEX_TIMEOUT()).unwrap()
  };
}

/// Alias to `($id).try_write_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! mc_wlock {
  ($id:expr) => {
    ($id).try_write_for($crate::envar::MUTEX_TIMEOUT()).unwrap()
  };
}
