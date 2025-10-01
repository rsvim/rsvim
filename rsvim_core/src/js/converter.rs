//! Converters between rust and v8 values.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

pub trait ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>>;
}

pub trait FromV8CallbackArguments {
  fn from_v8_callback_arguments<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
  ) -> Option<Self>
  where
    Self: Sized;
}

pub trait FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self>
  where
    Self: Sized;
}

pub fn to_v8<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  input: &T,
) -> Option<v8::Local<'s, v8::Value>>
where
  T: ToV8 + Sized,
{
  input.to_v8(scope)
}

pub fn from_v8_callback_arguments<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  value: v8::FunctionCallbackArguments<'s>,
) -> Option<T>
where
  T: FromV8CallbackArguments + Sized,
{
  T::from_v8_callback_arguments(scope, value)
}

pub fn from_v8<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  value: v8::Local<'s, v8::Value>,
) -> Option<T>
where
  T: FromV8 + Sized,
{
  T::from_v8(scope, value)
}

impl ToV8 for u32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    Some(v8::Integer::new_from_unsigned(scope, *self).into())
  }
}

impl ToV8 for i32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    Some(v8::Integer::new(scope, *self).into())
  }
}

impl ToV8 for f64 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    Some(v8::Number::new(scope, *self).into())
  }
}

impl ToV8 for bool {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    Some(v8::Boolean::new(scope, *self).into())
  }
}

impl ToV8 for String {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::String::new(scope, self).map(|s| s.into())
  }
}

impl ToV8 for CompactString {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::String::new(scope, self).map(|s| s.into())
  }
}

impl<T> ToV8 for Vec<T>
where
  T: ToV8,
{
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let elements = self
      .iter()
      .map(|v| v.to_v8(scope).unwrap())
      .collect::<Vec<_>>();
    Some(v8::Array::new_with_elements(scope, &elements).into())
  }
}

impl<T> ToV8 for [T]
where
  T: ToV8,
{
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Array::new_with_elements(
      scope,
      &self.iter().map(|v| v.to_v8(scope)).collect(),
    )
    .into()
  }
}

impl FromV8 for u32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_uint32() {
      match value.uint32_value(scope) {
        Some(value) => Some(value),
        None => None,
      }
    } else {
      None
    }
  }
}

impl FromV8 for i32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_int32() {
      value.int32_value(scope)
    } else {
      None
    }
  }
}

impl FromV8 for f64 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_number() || value.is_number_object() {
      value.number_value(scope)
    } else {
      None
    }
  }
}

impl FromV8 for bool {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_boolean() || value.is_boolean_object() {
      value.boolean_value(scope)
    } else {
      None
    }
  }
}

impl FromV8 for String {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_string() || value.is_string_object() {
      Some(value.to_rust_string_lossy(scope))
    } else {
      None
    }
  }
}

impl FromV8 for CompactString {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_string() || value.is_string_object() {
      Some(value.to_rust_string_lossy(scope).to_compact_string())
    } else {
      None
    }
  }
}

impl<T> FromV8 for Vec<T>
where
  T: FromV8,
{
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    if value.is_array() {
      let a: v8::Local<'s, v8::Array> = value.into();
      let mut v: Vec<T> = Vec::with_capacity(a.length());
      let mut i = 0;
      while i < a.length() {
        let e = a.get_index(scope, i).unwrap();
        let t = T::from_v8(scope, e);
        v.push(t);
        i += 1;
      }
    } else {
      None
    }
  }
}
