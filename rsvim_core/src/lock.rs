//! Mutex utility.

/// Generate Arc pointers.
#[macro_export]
macro_rules! arc_pointer_impl {
  ($name:ident) => {
    paste! {
      pub type [<$name Arc>] = std::sync::Arc<parking_lot::Mutex<$name>>;
      pub type [<$name Wk>] = std::sync::Weak<parking_lot::Mutex<$name>>;
      pub type [<$name MutexGuard>]<'a> = parking_lot::MutexGuard<'a, $name>;

      impl $name {
        pub fn to_arc(value: $name) -> [<$name Arc>] {
          std::sync::Arc::new(parking_lot::Mutex::new(value))
        }
      }
    }
  };
}

/// Generate Rc pointers.
#[macro_export]
macro_rules! rc_pointer_impl {
  ($name:ident) => {
    paste! {
      pub type [<$name Rc>] = std::rc::Rc<std::cell::RefCell<$name>>;

      impl $name {
        pub fn to_rc(value: $name) -> [<$name Rc>] {
          std::rc::Rc::new(std::cell::RefCell::new(value))
        }
      }
    }
  };
}

/// Alias to `($id).try_lock_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! lock {
  ($id:expr) => {
    ($id).try_lock_for($crate::envar::MUTEX_TIMEOUT()).unwrap()
  };
}

/// Alias to `($id).try_read_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! rlock {
  ($id:expr) => {
    ($id).try_read_for($crate::envar::MUTEX_TIMEOUT()).unwrap()
  };
}

/// Alias to `($id).try_write_for(envar::MUTEX_TIMEOUT()).unwrap()`.
#[macro_export]
macro_rules! wlock {
  ($id:expr) => {
    ($id).try_write_for($crate::envar::MUTEX_TIMEOUT()).unwrap()
  };
}
