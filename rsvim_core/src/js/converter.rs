//! Converters between rust and v8 values.

use compact_str::CompactString;
use compact_str::ToCompactString;

pub trait ToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value>;
}

pub trait FromV8CallbackArguments {
  fn from_v8_callback_arguments<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
  ) -> Self;
}

pub trait FromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self;
}

pub fn to_v8<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  input: T,
) -> v8::Local<'s, v8::Value>
where
  T: ToV8,
{
  input.to_v8(scope)
}

pub fn from_v8_callback_arguments<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  value: v8::FunctionCallbackArguments<'s>,
) -> T
where
  T: FromV8CallbackArguments,
{
  T::from_v8_callback_arguments(scope, value)
}

pub fn from_v8<'s, 'b, T>(
  scope: &mut v8::PinScope<'s, 'b>,
  value: v8::Local<'s, v8::Value>,
) -> T
where
  T: FromV8,
{
  T::from_v8(scope, value)
}

impl ToV8 for u32 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Integer::new_from_unsigned(scope, *self).into()
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

impl ToV8 for f64 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    v8::Number::new(scope, *self).into()
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

impl ToV8 for &'static str {
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

impl<T> ToV8 for Vec<T>
where
  T: ToV8,
{
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let elements = self.iter().map(|v| v.to_v8(scope)).collect::<Vec<_>>();
    v8::Array::new_with_elements(scope, &elements).into()
  }
}

impl FromV8 for u32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    value.uint32_value(scope).unwrap()
  }
}

impl FromV8 for i32 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    value.int32_value(scope).unwrap()
  }
}

impl FromV8 for f64 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    value.number_value(scope).unwrap()
  }
}

impl FromV8 for bool {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    value.boolean_value(scope)
  }
}

impl FromV8 for String {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    value.to_rust_string_lossy(scope)
  }
}

impl FromV8 for CompactString {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    value.to_rust_string_lossy(scope).to_compact_string()
  }
}

impl<T> FromV8 for Vec<T>
where
  T: FromV8,
{
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
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
