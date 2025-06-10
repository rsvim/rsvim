//! Mutex utility.

/// Generate Arc pointers.
#[macro_export]
macro_rules! arc_impl {
  ($name:ident) => {
    paste! {
      pub type [<$name Arc>] = std::sync::Arc<parking_lot::Mutex<$name>>;
      pub type [<$name Wk>] = std::sync::Weak<parking_lot::Mutex<$name>>;
      pub type [<$name ReasGuard>]<'a> = parking_lot::MutexGuard<'a, $name>;

      impl $name {
        pub fn to_arc(value: $name) -> [<$name Arc>] {
          std::sync::Arc::new(parking_lot::Mutex::new(value))
        }
      }
    }
  };
}

#[macro_export]
macro_rules! lock {
  ($id:expr) => {
    ($id).try_lock_for($crate::envar::MUTEX_TIMEOUT()).unwrap()
  };
}
