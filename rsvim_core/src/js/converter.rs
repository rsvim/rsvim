//! Converters between rust and v8 values.

use compact_str::CompactString;

pub trait U32ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer>;
}

impl U32ToV8 for u32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer> {
    v8::Integer::new_from_unsigned(scope, *self)
  }
}

pub trait U32FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self;
}

impl U32FromV8 for u32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self {
    value.uint32_value(scope).unwrap()
  }
}

pub trait I32ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer>;
}

impl I32ToV8 for i32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer> {
    v8::Integer::new(scope, *self)
  }
}

pub trait I32FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self;
}

impl I32FromV8 for i32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self {
    value.int32_value(scope).unwrap()
  }
}

pub trait F64ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Number>;
}

impl F64ToV8 for f64 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Number> {
    v8::Number::new(scope, *self)
  }
}

pub trait F64FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Number>,
  ) -> Self;
}

impl F64FromV8 for f64 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Number>,
  ) -> Self {
    value.number_value(scope).unwrap()
  }
}

pub trait BoolToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Boolean>;
}

impl BoolToV8 for bool {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Boolean> {
    v8::Boolean::new(scope, *self)
  }
}

pub trait BoolFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Boolean>,
  ) -> Self;
}

impl BoolFromV8 for bool {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Boolean>,
  ) -> Self {
    value.boolean_value(scope)
  }
}

pub trait StringToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::String>;
}

impl StringToV8 for str {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::String> {
    v8::String::new(scope, self).unwrap()
  }
}

impl StringToV8 for String {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::String> {
    v8::String::new(scope, self).unwrap()
  }
}

impl StringToV8 for CompactString {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::String> {
    v8::String::new(scope, self).unwrap()
  }
}

pub trait StringFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::String>,
  ) -> Self;
}

impl StringFromV8 for String {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::String>,
  ) -> Self {
    value.to_rust_string_lossy(scope)
  }
}

pub trait VecToV8<T> {
  fn to_v8<'s, F>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
    f: F,
  ) -> v8::Local<'s, v8::Array>
  where
    F: FnOnce(&mut v8::PinScope, &T) -> v8::Local<'s, v8::Value>;
}

impl<T> VecToV8<T> for Vec<T> {
  fn to_v8<'s, F>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
    f: F,
  ) -> v8::Local<'s, v8::Array>
  where
    F: FnOnce(&mut v8::PinScope, &T) -> v8::Local<'s, v8::Value>,
  {
    let elements = self.iter().map(|v| f(scope, v)).collect::<Vec<_>>();
    v8::Array::new_with_elements(scope, &elements).into()
  }
}

pub trait VecFromV8<T> {
  fn from_v8<'s, F>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    f: F,
  ) -> Self
  where
    F: FnOnce(&T, &mut v8::PinScope) -> T;
}

impl<T> VecFromV8<T> for Vec<T> {
  fn from_v8<'s, F>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    f: F,
  ) -> Self
  where
    F: FnOnce(&T, &mut v8::PinScope) -> T,
  {
    let elements = v8::Local::<v8::Array>::try_from(value).unwrap();
    let n = elements.length();
    let mut v: Vec<T> = Vec::with_capacity(n as usize);
    for i in 0..n {
      let e = elements.get_index(scope, i).unwrap();
      let t = T::from_v8(scope, e);
      v.push(t);
    }
    v
  }
}

/// Convert rust to v8.
#[macro_export]
macro_rules! to_v8 {
  (bool $value:expr) => {
    v8::Boolean::new(scope, $value)
  };

  (u32 $value:expr) => {
    v8::Integer::new_from_unsigned(scope, $value)
  };

  (i32 $value:expr) => {
    v8::Integer::new(scope, $value)
  };

  (f64 $value:expr) => {
    v8::Number::new(scope, $value)
  };

  (str $value:expr) => {
    v8::String::new(scope, $value).unwrap()
  };

  ($enum:ident, $($variant:tt),*) => {
    impl Stateful for $enum {
      fn handle(&self, data_access: StateDataAccess, event: Event) -> StateMachine {
        match self {
          $(
            $enum::$variant(e) => e.handle(data_access, event),
          )*
        }
      }

      fn handle_op(&self, data_access: StateDataAccess, op: Operation) -> StateMachine {
        match self {
          $(
            $enum::$variant(e) => e.handle_op(data_access, op),
          )*
        }
      }
    }
  }
}
