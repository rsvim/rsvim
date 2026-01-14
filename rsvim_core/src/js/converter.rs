//! Converters between rust and v8 values.

use crate::buf::BufferId;
use crate::js::JsTimerId;
use crate::ui::tree::TreeNodeId;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::rc::Rc;

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

pub trait TreeNodeIdToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer>;
}

impl TreeNodeIdToV8 for TreeNodeId {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer> {
    v8::Integer::new(scope, std::convert::Into::<i32>::into(*self))
  }
}

pub trait TreeNodeIdFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self;
}

impl TreeNodeIdFromV8 for TreeNodeId {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self {
    TreeNodeId::from(value.int32_value(scope).unwrap())
  }
}

pub trait BufferIdToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer>;
}

impl BufferIdToV8 for BufferId {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer> {
    v8::Integer::new(scope, std::convert::Into::<i32>::into(*self))
  }
}

pub trait BufferIdFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self;
}

impl BufferIdFromV8 for BufferId {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self {
    BufferId::from(value.int32_value(scope).unwrap())
  }
}

pub trait JsTimerIdToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer>;
}

impl JsTimerIdToV8 for JsTimerId {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Integer> {
    v8::Integer::new(scope, std::convert::Into::<i32>::into(*self))
  }
}

pub trait JsTimerIdFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self;
}

impl JsTimerIdFromV8 for JsTimerId {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Integer>,
  ) -> Self {
    BufferId::from(value.int32_value(scope).unwrap())
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

impl StringFromV8 for CompactString {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::String>,
  ) -> Self {
    value.to_rust_string_lossy(scope).to_compact_string()
  }
}

pub trait CallbackToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Function>;
}

impl CallbackToV8 for Rc<v8::Global<v8::Function>> {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Function> {
    v8::Local::new(scope, Rc::unwrap_or_clone(self.clone()))
  }
}

pub trait CallbackFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Function>,
  ) -> Self;
}

impl CallbackFromV8 for Rc<v8::Global<v8::Function>> {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Function>,
  ) -> Self {
    Rc::new(v8::Global::new(scope, value))
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

/// Property to_v8 helper
#[macro_export]
macro_rules! to_v8_prop {
  ($self:ident, $obj:ident, $scope:ident, $prop:tt) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop.to_v8($scope);
      $crate::js::binding::set_property_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, ()) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop().to_v8($scope);
      $crate::js::binding::set_property_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, optional) => {
    paste::paste! {
      if let Some($prop) = &$self.$prop {
        let [< $prop _value >] = $prop.to_v8($scope);
        $crate::js::binding::set_property_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
      }
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, Vec) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop.to_v8($scope, |scope, i| i.to_v8(scope).into());
      $crate::js::binding::set_property_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, (), Vec) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop().to_v8($scope, |scope, i| i.to_v8(scope).into());
      $crate::js::binding::set_property_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };
}

/// Constant to_v8 helper
#[macro_export]
macro_rules! to_v8_const {
  ($self:ident, $obj:ident, $scope:ident, $prop:tt) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop.to_v8($scope);
      $crate::js::binding::set_constant_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, ()) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop().to_v8($scope);
      $crate::js::binding::set_constant_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, optional) => {
    paste::paste! {
      if let Some($prop) = &$self.$prop {
        let [< $prop _value >] = $prop.to_v8($scope);
        $crate::js::binding::set_constant_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
      }
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, Vec) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop.to_v8($scope, |scope, i| i.to_v8(scope).into());
      $crate::js::binding::set_constant_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };

  ($self:ident, $obj:ident, $scope:ident, $prop:tt, (), Vec) => {
    paste::paste! {
      let [< $prop _value >] = $self.$prop().to_v8($scope, |scope, i| i.to_v8(scope).into());
      $crate::js::binding::set_constant_to($scope, $obj, [< $prop:snake:upper >], [< $prop _value >].into());
    }
  };
}

/// Property from_v8 helpers
#[macro_export]
macro_rules! from_v8_prop {
  (@assert_each(bool, $prop:tt)) => {
    debug_assert!($crate::is_v8_bool!($prop));
  };

  (@each($scope:ident, bool, $prop:tt)) => {
    $prop.to_boolean($scope)
  };

  (@assert_each(String, $prop:tt)) => {
    debug_assert!($crate::is_v8_str!($prop));
  };

  (@each($scope:ident, String, $prop:tt)) => {
    $prop.to_string($scope).unwrap()
  };

  (@assert_each(CompactString, $prop:tt)) => {
    debug_assert!($crate::is_v8_str!($prop));
  };

  (@each($scope:ident, CompactString, $prop:tt)) => {
    $prop.to_string($scope).unwrap()
  };

  (@assert_each(js_command_attr_Nargs, $prop:tt)) => {
    debug_assert!($crate::is_v8_str!($prop));
  };

  (@each($scope:ident, js_command_attr_Nargs, $prop:tt)) => {
    $prop.to_string($scope).unwrap()
  };

  ($builder:ident, $obj:ident, $scope:ident, $ty:tt, $prop:tt) => {
    paste::paste! {
      let [< $prop _name >] = [< $prop:snake:upper >].to_v8($scope);
      debug_assert!($obj.has_own_property($scope, [< $prop _name >].into()).unwrap_or(false));
      let [< $prop _value >] = $obj.get($scope, [< $prop _name >].into()).unwrap();
      from_v8_prop!{@assert_each($ty, [< $prop _value>])};
      $builder.$prop($ty::from_v8($scope, from_v8_prop!{@each($scope, $ty, [< $prop _value>])} ));
    }
  };

  ($builder:ident, $obj:ident, $scope:ident, $ty:tt, $prop:tt, optional) => {
    paste::paste! {
      let [< $prop _name >] = [< $prop:snake:upper >].to_v8($scope);
      if $obj.has_own_property($scope, [< $prop _name >].into()).unwrap_or(false) {
        let [< $prop _value >] = $obj.get($scope, [< $prop _name >].into()).unwrap();
        from_v8_prop!{@assert_each($ty, [< $prop _value>])};
        $builder.$prop(Some($ty::from_v8($scope, from_v8_prop!{@each($scope, $ty, [< $prop _value>])} )));
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
macro_rules! is_v8_int {
  ($value:expr) => {
    $value.is_int32() || $value.is_uint32()
  };
}

/// Wrap rust handle with v8's cppgc.
///
/// Some resources will be manually managed by users, such as file open/close,
/// network sockets, etc. Thus we will have to store the resource handler with
/// v8's cppgc wrapper, so v8 GC will close the resource and free the memory
/// allocated for us, instead of rust's "safe" resource management.
///
/// The basic idea is:
/// 1. Leak the rust's handler and get raw handler pointer.
/// 2. Set the raw handler to v8 object's internal reference.
/// 3. Create a v8::Weak finalizer callback, thus v8 GC will close it for us.
///
/// We will create a v8 object and:
/// 1. Set the handle's raw pointer to its internal field with index-0.
/// 2. Set a v8::Weak finalizer to its internal field with index-1.
#[macro_export]
macro_rules! wrap_cppgc_handle {
  ($scope:ident, $handle:expr, $ty:ty) => {{
    let wrapper_template = v8::ObjectTemplate::new($scope);

    // Allocate internal field for the wrapped `std::fs::File`:
    // 0-index: The `file_handle`, i.e. the `std::fs::File`
    // 1-index: The `file_weak` finalizer, it helps release the `file_handle`
    wrapper_template.set_internal_field_count(2);
    let wrapper = wrapper_template.new_instance($scope).unwrap();

    let handle_ptr =
      $crate::js::binding::set_internal_ref::<$ty>($scope, wrapper, 0, $handle);
    let weak_rc = std::rc::Rc::new(std::cell::Cell::new(None));

    // To automatically drop the handle instance when
    // V8 garbage collects the object that internally holds the Rust instance,
    // we use a Weak reference with a finalizer callback.
    let handle_weak = v8::Weak::with_finalizer(
      $scope,
      wrapper,
      Box::new({
        let weak_rc = weak_rc.clone();
        move |isolate| unsafe {
          drop(Box::from_raw(handle_ptr));
          drop(v8::Weak::from_raw(isolate, weak_rc.get()));
          trace!("|wrap_handle| dropped handle");
        }
      }),
    );

    // Store the weak ref pointer into the "shared" cell.
    weak_rc.set(handle_weak.into_raw());
    $crate::js::binding::set_internal_ref($scope, wrapper, 1, weak_rc);

    wrapper
  }};
}

/// Get the wrapped handle from v8 object.
#[macro_export]
macro_rules! get_cppgc_handle {
  ($scope:ident, $wrapper:ident, $ty:ty) => {
    $crate::js::binding::get_internal_ref::<$ty>($scope, $wrapper, 0)
  };
}
