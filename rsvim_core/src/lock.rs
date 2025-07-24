//! Mutex utility.

/// Generate Arc<Mutex<_>> pointers.
#[macro_export]
macro_rules! arc_mutex_ptr {
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

/// Generate Arc<_> pointers.
#[macro_export]
macro_rules! arc_ptr {
  ($name:ident) => {
    paste! {
      pub type [<$name Arc>] = std::sync::Arc<$name>;
      pub type [<$name Wk>] = std::sync::Weak<$name>;

      impl $name {
        pub fn to_arc(value: $name) -> [<$name Arc>] {
          std::sync::Arc::new(value)
        }
      }
    }
  };
}

#[macro_export]
macro_rules! lock {
  ($id:expr) => {
    ($id)
      .try_lock_for(*$crate::constant::MUTEX_TIMEOUT)
      .unwrap()
  };
}

/// Generate Rc<RefCell<_>> pointers.
#[macro_export]
macro_rules! rc_refcell_ptr {
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

/// Generate Rc<_> pointers.
#[macro_export]
macro_rules! rc_ptr {
  ($name:ident) => {
    paste! {
      pub type [<$name Rc>] = std::rc::Rc<$name>;
      pub type [<$name Wk>] = std::rc::Weak<$name>;

      impl $name {
        pub fn to_rc(value: $name) -> [<$name Rc>] {
          std::rc::Rc::new(value)
        }
      }
    }
  };
}
