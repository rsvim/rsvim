//! The macros for RSVIM text editor core.

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(ToV8)]
/// Convert rust struct to js object. A js object is like a key-value map
/// that contains multiple data fields. In most use cases, we only need 1 layer
/// map, i.e. all the values are not nested js objects, they are simply plain
/// data values and array with plain values.
pub fn to_v8(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let struct_fields = match input.data {
    syn::Data::Struct(struct_data) => match struct_data.fields {
      syn::Fields::Named(named_field) => named_field.named,
      _ => unreachable!("Failed to derive ToV8 macro on non-named field!"),
    },
    _ => unreachable!("Failed to derive ToV8 macro on non-struct data!"),
  };

  let is_option = |field_type: &syn::Type| match field_type {
    syn::Type::Path(p) => match p.path.segments.last() {
      Some(seg) => seg.ident == "Option",
      None => false,
    },
    _ => false,
  };

  let is_vec = |field_type: &syn::Type| match field_type {
    syn::Type::Path(p) => match p.path.segments.last() {
      Some(seg) => seg.ident == "Vec",
      None => false,
    },
    _ => false,
  };

  let fields = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .collect::<Vec<_>>();
  let uppercases = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let values = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  let optional_fields = struct_fields
    .iter()
    .filter(|n| is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .collect::<Vec<_>>();
  let optional_uppercases = struct_fields
    .iter()
    .filter(|n| is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let optional_values = struct_fields
    .iter()
    .filter(|n| is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  let vec_fields = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .collect::<Vec<_>>();
  let vec_uppercases = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let vec_values = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  quote! {

  impl crate::js::converter::ToV8 for #struct_ident {
    fn to_v8<'s>(
      &self,
      scope: &mut v8::PinScope<'s, '_>,
    ) -> v8::Local<'s, v8::Value> {
      use crate::js::binding;

      let obj = v8::Object::new(scope);

      #(
      {
        let #values = self.#fields.to_v8(scope);
        binding::set_property_to(scope, obj, #uppercases, #values);
      }
      )*

      #(
      {
        if let Some(#optional_fields) = &self.#optional_fields {
          let #optional_values = #optional_fields.to_v8(scope);
          binding::set_property_to(scope, obj, #optional_uppercases, #optional_values);
        }
      }
      )*

      #(
      {
        let #vec_values = self.#vec_fields.to_v8(scope, |scope, i| i.to_v8(scope));
        binding::set_property_to(scope, obj, #vec_uppercases, #vec_values);
      }
      )*


      obj.into()
    }
  }

  }.into()
}

#[proc_macro_derive(FromV8)]
/// Convert js object to rust struct.
pub fn from_v8(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  println!("from_v8:{:?}", input);

  let struct_ident = input.ident;
  let struct_ident_builder = format_ident!("{}Builder", struct_ident);
  let struct_fields = match input.data {
    syn::Data::Struct(struct_data) => match struct_data.fields {
      syn::Fields::Named(named_field) => named_field.named,
      _ => unreachable!("Failed to derive ToV8 macro on non-named field!"),
    },
    _ => unreachable!("Failed to derive ToV8 macro on non-struct data!"),
  };

  let _is_option = |field_type: &syn::Type| match field_type {
    syn::Type::Path(p) => match p.path.segments.last() {
      Some(seg) => seg.ident == "Option",
      None => false,
    },
    _ => false,
  };

  let _is_vec = |field_type: &syn::Type| match field_type {
    syn::Type::Path(p) => match p.path.segments.last() {
      Some(seg) => seg.ident == "Vec",
      None => false,
    },
    _ => false,
  };

  let is_bool = |field_type: &syn::Type| match field_type {
    syn::Type::Path(p) => match p.path.segments.last() {
      Some(seg) => seg.ident == "bool",
      None => false,
    },
    _ => false,
  };

  let bool_fields = struct_fields
    .iter()
    .filter(|n| is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .collect::<Vec<_>>();
  let bool_names = struct_fields
    .iter()
    .filter(|n| is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_name", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let bool_types = struct_fields
    .iter()
    .filter(|n| is_bool(&n.ty))
    .map(|n| match &n.ty {
      syn::Type::Path(p) => match p.path.segments.last() {
        Some(seg) => seg.ident.clone(),
        _ => unreachable!(),
      },
      _ => unreachable!(),
    })
    .collect::<Vec<_>>();
  let bool_uppercases = struct_fields
    .iter()
    .filter(|n| is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let bool_values = struct_fields
    .iter()
    .filter(|n| is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  let stringify_fields = struct_fields
    .iter()
    .filter(|n| !is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .collect::<Vec<_>>();
  let stringify_names = struct_fields
    .iter()
    .filter(|n| !is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_name", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let stringify_types = struct_fields
    .iter()
    .filter(|n| !is_bool(&n.ty))
    .map(|n| match &n.ty {
      syn::Type::Path(p) => match p.path.segments.last() {
        Some(seg) => seg.ident.clone(),
        _ => unreachable!(),
      },
      _ => unreachable!(),
    })
    .collect::<Vec<_>>();
  let stringify_uppercases = struct_fields
    .iter()
    .filter(|n| !is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let stringify_values = struct_fields
    .iter()
    .filter(|n| !is_bool(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  quote! {

  impl crate::js::converter::FromV8 for #struct_ident {
    fn from_v8<'s>(
      scope: &mut v8::PinScope<'s, '_>,
      obj: v8::Local<'s, v8::Value>,
    ) -> Self {
      debug_assert!(obj.is_object() || obj.is_object_template());
      let obj = obj.to_object(scope).unwrap();

      let mut builder = #struct_ident_builder::default();


      // bool
      #(
      {
        let #bool_values = self.#bool_fields.to_v8(scope);
        binding::set_property_to(scope, obj, #bool_uppercases, #bool_values);

        let #bool_names = v8::String::new(scope, #bool_uppercases).unwrap();
        debug_assert!(obj.has_own_property(scope, #bool_names.into()).unwrap_or(false));
        let #bool_values = obj.get(scope, #bool_names.into()).unwrap();
        debug_assert!(#bool_values.is_boolean() || #bool_values.is_boolean_object());
        let #bool_values = #bool_values.to_boolean(scope);
        builder.#bool_fields(#bool_types::from_v8(scope, #bool_values.into()));
      }
      )*

      // stringify
      #(
      {
        let #stringify_values = self.#stringify_fields.to_v8(scope);
        binding::set_property_to(scope, obj, #stringify_uppercases, #stringify_values);

        let #stringify_names = v8::String::new(scope, #stringify_uppercases).unwrap();
        debug_assert!(obj.has_own_property(scope, #stringify_names.into()).unwrap_or(false));
        let #stringify_values = obj.get(scope, #stringify_names.into()).unwrap();
        debug_assert!(#stringify_values.is_string() || #stringify_values.is_string_object());
        let #stringify_values = #stringify_values.to_string(scope).unwrap();
        builder.#stringify_fields(#stringify_types::from_v8(scope, #stringify_values.into()));
      }
      )*

      obj.into()
    }
  }

  }.into()
}

#[proc_macro_derive(FromV8String)]
/// Helper tag for "FromV8" macro.
pub fn from_v8_string(_input: TokenStream) -> TokenStream {
  TokenStream::default()
}
