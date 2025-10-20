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
  ) -> v8::Local<'s, v8::Array>;
}

impl<T> VecToV8<T> for Vec<T> {
  fn to_v8<'s, F>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Array> {
    let elements = self.iter().map(|v| v.to_v8(scope)).collect::<Vec<_>>();
    v8::Array::new_with_elements(scope, &elements)
  }
}

pub trait VecFromV8<T> {
  fn from_v8<'s, F>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self;
}

impl<T> VecFromV8<T> for Vec<T> {
  fn from_v8<'s, F>(
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

/// Implement struct's to_v8 helpers
#[macro_export]
macro_rules! to_v8_impl {
  ($name:ident [$($property:tt)*] [$($optional_property:tt)*] [$($constant:tt)*] [$($optional_constant:tt)*]) => {
    paste::paste! {
      impl StructToV8 for $name {
        fn to_v8<'s>(
          &self,
          scope: &mut v8::PinScope<'s, '_>,
        ) -> v8::Local<'s, v8::Object> {
          let obj = v8::Object::new(scope);

          // properties
          $(
            let [< $property _value >] = self.$property.to_v8(scope);
            $crate::js::binding::set_property_to(scope, obj, [< $property:snake:upper >], [< $property _value >]);
          )*

          // optional properties
          $(
            if let Some($optional_property) = &self.$optional_property {
              let [< $optional_property _value >] = $optional_property.to_v8(scope);
              $crate::js::binding::set_property_to(scope, obj, [< $optional_property:snake:upper >], [< $optional_property _value >]);
            }
          )*

          // constants
          $(
            let [< $constant _value >] = self.$constant.to_v8(scope);
            $crate::js::binding::set_constant_to(scope, obj, [< $constant:snake:upper >], [< $constant _value >]);
          )*

          // optional constants
          $(
            if let Some($optional_constant) = &self.$optional_constant {
              let [< $optional_constant _value >] = $optional_constant.to_v8(scope);
              $crate::js::binding::set_constant_to(scope, obj, [< $optional_constant:snake:upper >], [< $optional_constant _value >]);
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
  ($name:ident [$($property:tt)*] [$($optional_property:tt)*]) => {
    paste::paste! {
      impl StructFromV8 for $name {
        fn from_v8<'s>(
          scope: &mut v8::PinScope<'s, '_>,
          obj: v8::Local<'s, v8::Object>,
        ) -> Self {
          let mut builder = [< $name Builder >]::default();

          // properties
          from_v8_impl! {@each_prop($name){} $($property)*}

          // optional properties
          from_v8_impl! {@each_optional_prop($name){} $($optional_property)*}

          builder.build().unwrap()
        }
      }
    }
  };

  (@each_prop($name:ident){$($collect:tt)*} $type:tt $property:tt $($rest:tt)*) => {
    from_v8_impl! {@each_prop($name){
      $($collect)*

      let [< $property _name >] = [< $property:snake:upper >].to_v8(scope);
      debug_assert!(obj.has_own_property(scope, [< $property _name >]).unwrap_or(false));
      let [< $property _value >] = obj.get(scope, [< $property _name >]).unwrap();
      builder.$property($type::from_v8(scope, [< $property _value >]));

    } $($rest)*}
  };

  (@each_prop($name:ident){$($collect:tt)*}) => {
    from_v8_impl! {@each_prop($name){
      $($collect)*
    }}
  };

  (@each_optional_prop($name:ident){$($collect:tt)*} $type:tt $property:tt $($rest:tt)*) => {
    from_v8_impl! {@each_optional_prop($name){
      $($collect)*

      let [< $property _name >] = [< $property:snake:upper >].to_v8(scope);
      if obj.has_own_property(scope, [< $property _name >]).unwrap_or(false) {
        let [< $property _value >] = obj.get(scope, [< $property _name >]).unwrap();
        builder.$property(Some($type::from_v8(scope, [< $property _value >])));
      } else {
        builder.$property(None);
      }

    } $($rest)*}
  };

  (@each_optional_prop($name:ident){$($collect:tt)*}) => {
    from_v8_impl! {@each_optional_prop($name){
      $($collect)*
    }}
  };
}
