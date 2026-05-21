//! The macros for RSVIM text editor core.

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

fn get_struct_fields(
  data: &syn::Data,
) -> &syn::punctuated::Punctuated<syn::Field, syn::token::Comma> {
  match data {
    syn::Data::Struct(struct_data) => match &struct_data.fields {
      syn::Fields::Named(named_field) => &named_field.named,
      _ => panic!("Failed to derive macro on non-named field!"),
    },
    _ => panic!("Failed to derive macro on non-struct data!"),
  }
}

fn is_type_match(ty: &syn::Type, ident_name: &str) -> bool {
  if let syn::Type::Path(p) = ty {
    if let Some(seg) = p.path.segments.last() {
      return seg.ident == ident_name;
    }
  }
  false
}

fn has_attr(field: &syn::Field, attr_name: &str) -> bool {
  field.attrs.iter().any(|a| a.path().is_ident(attr_name))
}

struct ToV8Tokens {
  fields: Vec<syn::Ident>,
  uppercases: Vec<syn::Ident>,
  values: Vec<syn::Ident>,
}

impl ToV8Tokens {
  fn collect<'a, F>(
    fields: impl Iterator<Item = &'a syn::Field>,
    predicate: F,
  ) -> Self
  where
    F: Fn(&syn::Field) -> bool,
  {
    let mut res = Self {
      fields: vec![],
      uppercases: vec![],
      values: vec![],
    };
    for f in fields.filter(|&f| predicate(f)) {
      let ident = f.ident.clone().unwrap();
      res
        .uppercases
        .push(format_ident!("{}", ident.to_string().to_uppercase()));
      res.values.push(format_ident!("{}_value", ident));
      res.fields.push(ident);
    }
    res
  }
}

#[proc_macro_derive(ToV8)]
/// Convert rust struct to js object.
///
/// A js object is like a key-value map that contains multiple data fields.
/// When passing key-value map data between js and rust, we try to keep these
/// data objects to be simple. Here are some rules:
///
/// - Js object only contains 1-layer, all the field values are no more js
///   object, i.e. the js object is not nested.
/// - All the field values can be either plain values such as
///   boolean/string/etc, or js array that only contains plain values (again,
///   such as boolean/string/etc).
pub fn to_v8(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let struct_fields = get_struct_fields(&input.data);

  let is_option = |f: &syn::Field| is_type_match(&f.ty, "Option");
  let is_vec = |f: &syn::Field| is_type_match(&f.ty, "Vec");

  let plain =
    ToV8Tokens::collect(struct_fields.iter(), |f| !is_option(f) && !is_vec(f));
  let optional = ToV8Tokens::collect(struct_fields.iter(), is_option);
  let vec = ToV8Tokens::collect(struct_fields.iter(), is_vec);

  // Destructure for `quote!` use
  let (fields, uppercases, values) =
    (&plain.fields, &plain.uppercases, &plain.values);
  let (optional_fields, optional_uppercases, optional_values) =
    (&optional.fields, &optional.uppercases, &optional.values);
  let (vec_fields, vec_uppercases, vec_values) =
    (&vec.fields, &vec.uppercases, &vec.values);

  quote! {

  impl crate::js::converter::ToV8 for #struct_ident {
    fn to_v8<'s>(
      &self,
      scope: &mut v8::PinScope<'s, '_>,
    ) -> v8::Local<'s, v8::Value> {
      use crate::js::binding;

      let obj = v8::Object::new(scope);

      // plain
      #(
      {
        let #values = self.#fields.to_v8(scope);
        binding::set_property_to(scope, obj, #uppercases, #values);
      }
      )*

      // optional
      #(
      {
        if let Some(#optional_fields) = &self.#optional_fields {
          let #optional_values = #optional_fields.to_v8(scope);
          binding::set_property_to(scope, obj, #optional_uppercases, #optional_values);
        }
      }
      )*

      // vec
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

struct FromV8Tokens {
  fields: Vec<syn::Ident>,
  names: Vec<syn::Ident>,
  types: Vec<syn::Ident>,
  uppercases: Vec<syn::Ident>,
  values: Vec<syn::Ident>,
}

impl FromV8Tokens {
  /// Collects all related identifiers in a single pass over the filtered fields.
  fn collect<'a, F>(
    fields: impl Iterator<Item = &'a syn::Field>,
    predicate: F,
  ) -> Self
  where
    F: Fn(&syn::Field) -> bool,
  {
    let mut res = Self {
      fields: vec![],
      names: vec![],
      types: vec![],
      uppercases: vec![],
      values: vec![],
    };

    for f in fields.filter(|&f| predicate(f)) {
      let ident = f.ident.clone().unwrap();
      res.names.push(format_ident!("{}_name", ident));
      res
        .uppercases
        .push(format_ident!("{}", ident.to_string().to_uppercase()));
      res.values.push(format_ident!("{}_value", ident));
      res.fields.push(ident.clone());

      let ty_ident = match &f.ty {
        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.clone(),
        _ => unreachable!("Expected TypePath for field {}", ident),
      };
      res.types.push(ty_ident);
    }
    res
  }
}

#[proc_macro_derive(FromV8, attributes(from_v8_bool, from_v8_string))]
/// Convert js object to rust struct.
pub fn from_v8(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let struct_ident_builder = format_ident!("{}Builder", struct_ident);
  let struct_fields = get_struct_fields(&input.data);

  let is_option = |f: &syn::Field| is_type_match(&f.ty, "Option");
  let is_vec = |f: &syn::Field| is_type_match(&f.ty, "Vec");

  let bool_tokens = FromV8Tokens::collect(struct_fields.iter(), |f| {
    has_attr(f, "from_v8_bool") && !is_option(f) && !is_vec(f)
  });
  let optional_bool_tokens = FromV8Tokens::collect(struct_fields.iter(), |f| {
    has_attr(f, "from_v8_bool") && is_option(f)
  });
  let string_tokens = FromV8Tokens::collect(struct_fields.iter(), |f| {
    has_attr(f, "from_v8_string") && !is_option(f) && !is_vec(f)
  });
  let optional_string_tokens =
    FromV8Tokens::collect(struct_fields.iter(), |f| {
      has_attr(f, "from_v8_string") && is_option(f)
    });

  // Destructure for `quote!` use
  let (bool_fields, bool_names, bool_types, bool_uppercases, bool_values) = (
    &bool_tokens.fields,
    &bool_tokens.names,
    &bool_tokens.types,
    &bool_tokens.uppercases,
    &bool_tokens.values,
  );
  let (
    optional_bool_fields,
    optional_bool_names,
    optional_bool_types,
    optional_bool_uppercases,
    optional_bool_values,
  ) = (
    &optional_bool_tokens.fields,
    &optional_bool_tokens.names,
    &optional_bool_tokens.types,
    &optional_bool_tokens.uppercases,
    &optional_bool_tokens.values,
  );
  let (
    string_fields,
    string_names,
    string_types,
    string_uppercases,
    string_values,
  ) = (
    &string_tokens.fields,
    &string_tokens.names,
    &string_tokens.types,
    &string_tokens.uppercases,
    &string_tokens.values,
  );
  let (
    optional_string_fields,
    optional_string_names,
    optional_string_types,
    optional_string_uppercases,
    optional_string_values,
  ) = (
    &optional_string_tokens.fields,
    &optional_string_tokens.names,
    &optional_string_tokens.types,
    &optional_string_tokens.uppercases,
    &optional_string_tokens.values,
  );

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
        let #bool_names = v8::String::new(scope, #bool_uppercases).unwrap();
        debug_assert!(obj.has_own_property(scope, #bool_names.into()).unwrap_or(false));
        let #bool_values = obj.get(scope, #bool_names.into()).unwrap();
        debug_assert!(#bool_values.is_boolean() || #bool_values.is_boolean_object());
        let #bool_values = #bool_values.to_boolean(scope);
        builder.#bool_fields(#bool_types::from_v8(scope, #bool_values.into()));
      }
      )*

      // optional bool
      #(
      {
        let #optional_bool_names = v8::String::new(scope, #optional_bool_uppercases).unwrap();
        if obj.has_own_property(scope, #optional_bool_names.into()).unwrap_or(false) {
          let #optional_bool_values = obj.get(scope, #optional_bool_names.into()).unwrap();
          debug_assert!(#optional_bool_values.is_boolean() || #optional_bool_values.is_boolean_object());
          let #optional_bool_values = #optional_bool_values.to_boolean(scope);
          builder.#optional_bool_fields(#optional_bool_types::from_v8(scope, #optional_bool_values.into()));
        }
      }
      )*

      // string
      #(
      {
        let #string_names = v8::String::new(scope, #string_uppercases).unwrap();
        debug_assert!(obj.has_own_property(scope, #string_names.into()).unwrap_or(false));
        let #string_values = obj.get(scope, #string_names.into()).unwrap();
        debug_assert!(#string_values.is_string() || #string_values.is_string_object());
        let #string_values = #string_values.to_string(scope).unwrap();
        builder.#string_fields(#string_types::from_v8(scope, #string_values.into()));
      }
      )*

      // optional string
      #(
      {
        let #optional_string_names = v8::String::new(scope, #optional_string_uppercases).unwrap();
        if obj.has_own_property(scope, #optional_string_names.into()).unwrap_or(false) {
          let #optional_string_values = obj.get(scope, #optional_string_names.into()).unwrap();
          debug_assert!(#optional_string_values.is_string() || #optional_string_values.is_string_object());
          let #optional_string_values = #optional_string_values.to_string(scope).unwrap();
          builder.#optional_string_fields(#optional_string_types::from_v8(scope, #optional_string_values.into()));
        }
      }
      )*

      builder.build().unwrap()
    }
  }

  }.into()
}
