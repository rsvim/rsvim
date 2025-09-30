//! Converters between rust and v8 values.

use crate::prelude::*;

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

pub fn to_v8<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  input: T,
) -> Option<v8::Local<'s, v8::Value>>
where
  T: ToV8,
{
  input.to_v8(scope)
}

impl ToV8 for u8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Uint32::from(self);
    v.to_object(scope)
  }
}

impl ToV8 for i8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Int32::from(self);
    v.to_object(scope)
  }
}

impl ToV8 for u16 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Uint32::from(self);
    v.to_object(scope)
  }
}

impl ToV8 for i16 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Int32::from(self);
    v.to_object(scope)
  }
}

impl ToV8 for u32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Uint32::from(self);
    v.to_object(scope)
  }
}

impl ToV8 for i32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Int32::from(self);
    v.to_object(scope)
  }
}

impl ToV8 for f32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Number::new(scope, *self as f64);
    v.to_object(scope)
  }
}

impl ToV8 for f64 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let v = v8::Number::new(scope, self);
    v.to_object(scope)
  }
}
