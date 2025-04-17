//! Mutex utility.

/// Generate Arc pointers.
#[macro_export]
macro_rules! arc_impl {
  ($name:ident) => {
    paste! {
      pub type [<$name Arc>] = std::sync::Arc<parking_lot::RwLock<$name>>;
      pub type [<$name Wk>] = std::sync::Weak<parking_lot::RwLock<$name>>;
      pub type [<$name ReasGuard>]<'a> = parking_lot::RwLockReadGuard<'a, $name>;
      pub type [<$name WriteGuard>]<'a> = parking_lot::RwLockWriteGuard<'a, $name>;

      impl $name {
        pub fn to_arc(value: $name) -> [<$name Arc>] {
          std::sync::Arc::new(parking_lot::RwLock::new(value))
        }

        /// # Safety
        ///
        /// Converts mutable reference to `std::ptr::NonNull` raw pointers to allow both immutable
        /// and mutable calls on it.
        pub unsafe fn to_nonnull(value: &mut $name) -> std::ptr::NonNull<$name> {
          std::ptr::NonNull::new(&mut *value as *mut $name).unwrap()
        }
      }
    }
  };
}

/// Generate Rc pointers.
#[macro_export]
macro_rules! rc_impl {
  ($name:ident) => {
    paste! {
      pub type [<$name Rc>] = std::rc::Rc<std::cell::RefCell<$name>>;
      pub type [<$name Wk>] = std::rc::Weak<std::cell::RefCell<$name>>;

      impl $name {
        pub fn to_rc(value: $name) -> [<$name Rc>] {
          std::rc::Rc::new(std::cell::RefCell::new(value))
        }
      }
    }
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
