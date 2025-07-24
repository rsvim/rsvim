use std::env::VarError;

use parking_lot::{Mutex, MutexGuard};

static GLOBAL_SEQUENTIAL_LOCK: Mutex<()> = Mutex::new(());

pub fn acquire_sequential_guard() -> MutexGuard<'static, ()> {
  GLOBAL_SEQUENTIAL_LOCK.lock()
}

pub unsafe fn set_env_var(name: &str, value: &str) -> Result<String, VarError> {
  let saved = std::env::var(name);
  unsafe {
    std::env::set_var(name, value);
  }
  saved
}

pub unsafe fn restore_env_var(
  name: &str,
  saved_value: Result<String, VarError>,
) {
  match saved_value {
    Ok(saved) => unsafe {
      std::env::set_var(name, saved);
    },
    Err(_) => { /* */ }
  }
}
