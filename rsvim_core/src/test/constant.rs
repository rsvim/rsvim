use parking_lot::{Mutex, MutexGuard};

static GLOBAL_SEQUENTIAL_LOCK: Mutex<()> = Mutex::new(());

pub fn acquire_sequential_guard() -> MutexGuard<'static, ()> {
  GLOBAL_SEQUENTIAL_LOCK.lock()
}

#[macro_export]
macro_rules! set_env_var {
  ($name:ident,$value:expr) => {
    unsafe {
      let saved = std::env::var($name);
      std::env::set_var($name, $value);
      saved
    }
  };
}

#[macro_export]
macro_rules! restore_env_var {
  ($name:ident,$saved_var:expr) => {
    match $saved_var {
      Ok(saved) => unsafe {
        std::env::set_var($name, saved);
      },
      Err(_) => { /* */ }
    }
  };
}
