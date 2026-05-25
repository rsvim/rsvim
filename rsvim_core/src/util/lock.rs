//! Mutex utility.

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
