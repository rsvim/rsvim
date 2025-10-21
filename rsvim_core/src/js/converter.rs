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
    F: Fn(&mut v8::PinScope<'s, '_>, &T) -> v8::Local<'s, v8::Value>;
}

impl<T> VecToV8<T> for Vec<T> {
  fn to_v8<'s, F>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
    f: F,
  ) -> v8::Local<'s, v8::Array>
  where
    F: Fn(&mut v8::PinScope<'s, '_>, &T) -> v8::Local<'s, v8::Value>,
  {
    let elements = self
      .iter()
      .map(|v| f(scope, v))
      .collect::<Vec<v8::Local<'s, v8::Value>>>();
    v8::Array::new_with_elements(scope, &elements)
  }
}

pub trait VecFromV8<T> {
  fn from_v8<'s, F>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Array>,
    f: F,
  ) -> Vec<T>
  where
    F: Fn(&mut v8::PinScope<'s, '_>, v8::Local<'s, v8::Value>) -> T;
}

impl<T> VecFromV8<T> for Vec<T> {
  fn from_v8<'s, F>(
    scope: &mut v8::PinScope<'s, '_>,
    elements: v8::Local<'s, v8::Array>,
    f: F,
  ) -> Vec<T>
  where
    F: Fn(&mut v8::PinScope<'s, '_>, v8::Local<'s, v8::Value>) -> T,
  {
    let n = elements.length();
    let mut v: Vec<T> = Vec::with_capacity(n as usize);
    for i in 0..n {
      let e = elements.get_index(scope, i).unwrap();
      let t = f(scope, e);
      v.push(t);
    }
    v
  }
}

pub trait StructToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object>;
}

pub trait StructFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Object>,
  ) -> Self;
}

pub trait StructFromV8CallbackArguments {
  fn from_v8_callback_arguments<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
  ) -> Self;
}

/// Implement struct's to_v8 helpers
#[macro_export]
macro_rules! to_v8_impl {
  ($name:ident, [$($prop:tt),*], [$($optional_prop:tt),*], [$($constant:tt),*], [$($opt_constant:tt),*]) => {
    paste::paste! {
      impl StructToV8 for $name {
        fn to_v8<'s>(
          &self,
          scope: &mut v8::PinScope<'s, '_>,
        ) -> v8::Local<'s, v8::Object> {
          let obj = v8::Object::new(scope);

          // properties
          $(
            let [< $prop _value >] = self.$prop.to_v8(scope);
            $crate::js::binding::set_property_to(scope, obj, [< $prop:snake:upper >], [< $prop _value >].into());
          )*

          // optional properties
          $(
            if let Some($optional_prop) = &self.$optional_prop {
              let [< $optional_prop _value >] = $optional_prop.to_v8(scope);
              $crate::js::binding::set_property_to(scope, obj, [< $optional_prop:snake:upper >], [< $optional_prop _value >].into());
            }
          )*

          // constants
          $(
            let [< $constant _value >] = self.$constant.to_v8(scope);
            $crate::js::binding::set_constant_to(scope, obj, [< $constant:snake:upper >], [< $constant _value >]);
          )*

          // optional constants
          $(
            if let Some($opt_constant) = &self.$opt_constant {
              let [< $opt_constant _value >] = $opt_constant.to_v8(scope);
              $crate::js::binding::set_constant_to(scope, obj, [< $opt_constant:snake:upper >], [< $opt_constant _value >]);
            }
          )*

          obj
        }
      }
    }
  };
}

/// Implement struct's from_v8 helpers
#[macro_export]
macro_rules! from_v8_impl {
  ($name:ident, [$(($ty:tt,$prop:tt)),*], [$(($optional_ty:tt,$optional_prop:tt)),*]) => {
    paste::paste!{
      impl StructFromV8 for $name {
        fn from_v8<'s>(
          scope: &mut v8::PinScope<'s, '_>,
          obj: v8::Local<'s, v8::Object>,
        ) -> Self {
          let mut builder = [< $name Builder >]::default();

          // properties
          $(
            let [< $prop _name >] = [< $prop:snake:upper >].to_v8(scope);
            debug_assert!(obj.has_own_property(scope, [< $prop _name >].into()).unwrap_or(false));
            let [< $prop _value >] = obj.get(scope, [< $prop _name >].into()).unwrap();
            builder.$prop($ty::from_v8(scope, [< $prop _value >]));
          )*

          // optional properties
          $(
            let [< $optional_prop _name >] = [< $optional_prop:snake:upper >].to_v8(scope);
            if obj.has_own_property(scope, [< $optional_prop _name >].into()).unwrap_or(false) {
              let [< $optional_prop _value >] = obj.get(scope, [< $optional_prop _name >].into()).unwrap();
              builder.$optional_prop(Some($optional_ty::from_v8(scope, [< $optional_prop _value >])));
            } else {
              builder.$optional_prop(None);
            }
          )*

          builder.build().unwrap()
        }
      }
    }
  };
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
macro_rules! is_v8_obj {
  ($value:expr) => {
    $value.is_object()
  };
}

#[macro_export]
macro_rules! is_v8_nil {
  ($value:expr) => {
    $value.is_null_or_undefined()
  };
}

#[macro_export]
macro_rules! is_v8_int {
  ($value:expr) => {
    $value.is_int32() || $value.is_uint32()
  };
}

#[macro_export]
macro_rules! is_v8_func {
  ($value:expr) => {
    $value.is_function()
  };
}
