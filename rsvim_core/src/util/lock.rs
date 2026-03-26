//! Mutex utility.

/// Generate Arc<Mutex<_>> pointers.
#[macro_export]
macro_rules! arc_mutex_ptr {
  ($name:ident) => {
    paste::paste! {
      pub type [<$name Arc>] = std::sync::Arc<std::sync::Mutex<$name>>;
      pub type [<$name Wk>] = std::sync::Weak<std::sync::Mutex<$name>>;
      pub type [<$name MutexGuard>]<'a> = std::sync::MutexGuard<'a, $name>;

      impl $name {
        pub fn to_arc(value: $name) -> [<$name Arc>] {
          std::sync::Arc::new(std::sync::Mutex::new(value))
        }
      }
    }
  };
}

/// Generate Arc<_> pointers.
#[macro_export]
macro_rules! arc_ptr {
  ($name:ident) => {
    paste::paste! {
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
    ($id).lock().unwrap()
  };
}

/// Generate Rc<RefCell<_>> pointers.
#[macro_export]
macro_rules! rc_refcell_ptr {
  ($name:ident) => {
    paste::paste! {
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
    paste::paste! {
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
