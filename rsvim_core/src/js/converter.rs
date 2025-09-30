//! Converters between rust and v8 values.

use crate::prelude::*;
use compact_str::CompactString;
use std::collections::LinkedList;

pub trait ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>>;
}

pub trait FromV8CallbackArguments {
  fn from_v8_callback_arguments(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
  ) -> Option<Self>;
}

pub trait FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self>;
}

impl ToV8 for u8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Integer::new_from_unsigned(scope, self).into()
  }
}

impl ToV8 for i8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Integer::new(scope, self).into()
  }
}

impl ToV8 for u16 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Integer::new_from_unsigned(scope, self).into()
  }
}

impl ToV8 for i16 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Integer::new(scope, self).into()
  }
}

impl ToV8 for u32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Integer::new_from_unsigned(scope, self).into()
  }
}

impl ToV8 for i32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Integer::new(scope, self).into()
  }
}

impl ToV8 for f32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Number::new(scope, self).into()
  }
}

impl ToV8 for f64 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Number::new(scope, self).into()
  }
}

impl ToV8 for bool {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::Boolean::new(scope, self).into()
  }
}

impl ToV8 for String {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::String::new(scope, self).into()
  }
}

impl ToV8 for CompactString {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::String::new(scope, self).into()
  }
}

impl ToV8 for char {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    v8::String::new(scope, self).into()
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
    v8::Array::new_with_elements(
      scope,
      &self.iter().map(|v| v.to_v8(scope)).collect(),
    )
    .into()
  }
}

impl<T> ToV8 for LinkedList<T>
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

impl FromV8 for u8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    match value.integer_value(scope) {
      Some(value) => value as u8,
      None => None,
    }
  }
}

impl FromV8 for i8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    match value.integer_value(scope) {
      Some(value) => value as i8,
      None => None,
    }
  }
}
