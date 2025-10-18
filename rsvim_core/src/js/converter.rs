//! Converters between rust and v8 values.

use compact_str::CompactString;
use compact_str::ToCompactString;

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

pub fn bool_to_v8<'s>(
  value: bool,
  scope: &mut v8::PinScope<'s, '_>,
) -> v8::Local<'s, v8::Boolean> {
  v8::Boolean::new(scope, value)
}

fn str_to_v8<'s>(
  value: &str,
  scope: &mut v8::PinScope<'s, '_>,
) -> v8::Local<'s, v8::String> {
  v8::String::new(scope, value).unwrap()
}

fn vec_to_v8<'s, T, F>(
  value: &Vec<T>,
  scope: &mut v8::PinScope<'s, '_>,
  f: F,
) -> v8::Local<'s, v8::Array>
where
  F: FnOnce(&mut v8::PinScope, &T) -> v8::Local<'s, v8::Value>,
{
  let elements = value.iter().map(|v| f(scope, v)).collect::<Vec<_>>();
  v8::Array::new_with_elements(scope, &elements).into()
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

pub fn to_v8_uint8_array<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  input: Vec<u8>,
) -> v8::Local<'s, v8::Uint8Array> {
  let store = v8::ArrayBuffer::new_backing_store_from_vec(input);
  let buf = v8::ArrayBuffer::with_backing_store(scope, &store.make_shared());
  v8::Uint8Array::new(scope, buf, 0, buf.byte_length()).unwrap()
}
