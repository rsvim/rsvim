#[cfg(test)]
use parking_lot::{Mutex, MutexGuard};

#[cfg(test)]
static GLOBAL_SEQUENTIAL_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub fn acquire_sequential_guard() -> MutexGuard<'static, ()> {
  GLOBAL_SEQUENTIAL_LOCK.lock()
}
