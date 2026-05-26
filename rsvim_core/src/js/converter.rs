//! Converters between rust and v8 values.

use crate::buf::BufferId;
use crate::is_v8_bool;
use crate::is_v8_func;
use crate::is_v8_int;
use crate::is_v8_number;
use crate::is_v8_str;
use crate::js::TimerId;
use crate::ui::tree::NodeId;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::rc::Rc;

pub trait ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value>;
}

pub trait FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self;
}

pub trait VecFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self;
}

pub trait FromV8CallbackArgs {
  fn from_v8_callback_args<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
  ) -> Self;
}

impl<T> ToV8 for Vec<T>
where
  T: ToV8,
{
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let elements = self
      .iter()
      .map(|v| v.to_v8(scope))
      .collect::<Vec<v8::Local<'s, v8::Value>>>();
    v8::Array::new_with_elements(scope, &elements).into()
  }
}

impl<T: ToV8 + ?Sized> ToV8 for &T {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    (*self).to_v8(scope)
  }
}

impl ToV8 for u32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Integer::new_from_unsigned(scope, *self).into()
  }
}

impl FromV8 for u32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_int!(value));
    value
      .to_integer(scope)
      .unwrap()
      .uint32_value(scope)
      .unwrap()
  }
}

impl ToV8 for i32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Integer::new(scope, *self).into()
  }
}

impl FromV8 for i32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_int!(value));
    value.to_integer(scope).unwrap().int32_value(scope).unwrap()
  }
}

impl ToV8 for NodeId {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Integer::new(scope, Into::<i32>::into(*self)).into()
  }
}

impl FromV8 for NodeId {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_int!(value));
    NodeId::from(value.to_integer(scope).unwrap().int32_value(scope).unwrap())
  }
}

impl ToV8 for BufferId {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Integer::new(scope, Into::<i32>::into(*self)).into()
  }
}

impl FromV8 for BufferId {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_int!(value));
    BufferId::from(value.to_integer(scope).unwrap().int32_value(scope).unwrap())
  }
}

impl ToV8 for TimerId {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Integer::new(scope, Into::<i32>::into(*self)).into()
  }
}

impl FromV8 for TimerId {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_int!(value));
    TimerId::from(value.to_integer(scope).unwrap().int32_value(scope).unwrap())
  }
}

impl ToV8 for f64 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Number::new(scope, *self).into()
  }
}

impl FromV8 for f64 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_number!(value));
    value.to_number(scope).unwrap().number_value(scope).unwrap()
  }
}

impl ToV8 for bool {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Boolean::new(scope, *self).into()
  }
}

impl FromV8 for bool {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_bool!(value));
    value.to_boolean(scope).boolean_value(scope)
  }
}

impl ToV8 for str {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::String::new(scope, self).unwrap().into()
  }
}

impl ToV8 for String {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::String::new(scope, self).unwrap().into()
  }
}

impl ToV8 for CompactString {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::String::new(scope, self).unwrap().into()
  }
}

impl FromV8 for String {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_str!(value));
    value.to_string(scope).unwrap().to_rust_string_lossy(scope)
  }
}

impl FromV8 for CompactString {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_str!(value));
    value
      .to_string(scope)
      .unwrap()
      .to_rust_string_lossy(scope)
      .to_compact_string()
  }
}

impl ToV8 for Rc<v8::Global<v8::Function>> {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Local::new(scope, Rc::unwrap_or_clone(self.clone())).into()
  }
}

impl FromV8 for Rc<v8::Global<v8::Function>> {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(is_v8_func!(value));
    let value = v8::Local::<'s, v8::Function>::try_from(value).unwrap();
    Rc::new(v8::Global::new(scope, value))
  }
}

impl<T> VecFromV8 for Vec<T>
where
  T: FromV8,
{
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    elements: v8::Local<'s, v8::Value>,
  ) -> Vec<T> {
    debug_assert!(elements.is_array());
    let elements = v8::Local::<v8::Array>::try_from(elements).unwrap();
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

#[macro_export]
macro_rules! is_v8_str {
  ($value:expr) => {
    $value.is_string() || $value.is_string_object()
  };
}

#[macro_export]
macro_rules! is_v8_bool {
  ($value:expr) => {
    $value.is_boolean() || $value.is_boolean_object()
  };
}

#[macro_export]
macro_rules! is_v8_int {
  ($value:expr) => {
    $value.is_int32() || $value.is_uint32()
  };
}

#[macro_export]
macro_rules! is_v8_number {
  ($value:expr) => {
    $value.is_number()
      || $value.is_number_object()
      || $value.is_int32()
      || $value.is_uint32()
  };
}

#[macro_export]
macro_rules! is_v8_func {
  ($value:expr) => {
    $value.is_function() || $value.is_function_template()
  };
}
