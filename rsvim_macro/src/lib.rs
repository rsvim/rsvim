//! The macros for RSVIM text editor core.

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(ToV8)]
/// Serialize rust struct to v8 js object. A js object is like a key-value map
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

  let non_optional_names = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .collect::<Vec<_>>();
  let non_optional_uppercases = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let non_optional_values = struct_fields
    .iter()
    .filter(|n| !is_option(&n.ty) && !is_vec(&n.ty))
    .map(|n| n.ident.clone().unwrap())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  let optional_names = struct_fields
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

  let vec_names = struct_fields
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

  impl crate::js::converter::StructToV8 for #struct_ident {
    fn to_v8<'s>(
      &self,
      scope: &mut v8::PinScope<'s, '_>,
    ) -> v8::Local<'s, v8::Object> {
      let obj = v8::Object::new(scope);

      #(
      {
        let #non_optional_values = self.#non_optional_names.to_v8(scope);
        crate::js::binding::set_property_to(scope, obj, #non_optional_uppercases, #non_optional_values.into());
      }
      )*

      #(
      {
        if let Some(#optional_names) = &self.#optional_names {
          let #optional_values = #optional_names.to_v8(scope);
          crate::js::binding::set_property_to(scope, obj, #optional_uppercases, #optional_values.into());
        }
      }
      )*

      #(
      {
        let #vec_values = self.#vec_names.to_v8(scope, |scope, i| i.to_v8(scope).into());
        crate::js::binding::set_property_to(scope, obj, #vec_uppercases, #vec_values.into());
      }
      )*


      obj
    }
  }

  }.into()

  // TokenStream::default()
}
