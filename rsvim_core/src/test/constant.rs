use std::env::VarError;
use std::ffi::OsStr;

use parking_lot::{Mutex, MutexGuard};

static GLOBAL_SEQUENTIAL_LOCK: Mutex<()> = Mutex::new(());

pub fn acquire_sequential_guard() -> MutexGuard<'static, ()> {
  GLOBAL_SEQUENTIAL_LOCK.lock()
}

pub fn set_env_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(
  name: K,
  value: V,
) -> Result<String, VarError> {
  let saved = std::env::var(&name);
  unsafe {
    std::env::set_var(name, value);
  }
  saved
}

pub fn restore_env_var(name: &str, saved_value: Result<String, VarError>) {
  match saved_value {
    Ok(saved) => unsafe {
      std::env::set_var(name, saved);
    },
    Err(_) => { /* */ }
  }
}
