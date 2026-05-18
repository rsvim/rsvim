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

  let input_ident = input.ident;
  println!("ToV8 input_ident: {}({:?})", input_ident, input_ident);

  let input_named_fields = match input.data {
    syn::Data::Struct(struct_data) => match struct_data.fields {
      syn::Fields::Named(named_field) => named_field.named,
      _ => unreachable!("Failed to derive ToV8 macro on non-named field!"),
    },
    _ => unreachable!("Failed to derive ToV8 macro on non-struct data!"),
  };

  let input_fields = input_named_fields
    .iter()
    .filter_map(|n| n.ident.clone())
    .collect::<Vec<_>>();
  let input_field_uppercases = input_named_fields
    .iter()
    .filter_map(|n| n.ident.clone())
    .map(|i| format_ident!("{}", i.to_string().to_uppercase()))
    .collect::<Vec<_>>();
  let input_field_values = input_named_fields
    .iter()
    .filter_map(|n| n.ident.clone())
    .map(|i| format_ident!("{}_value", i))
    .collect::<Vec<_>>();

  let expanded = quote! {

  impl crate::js::converter::StructToV8 for #input_ident {
    fn to_v8<'s>(
      &self,
      scope: &mut v8::PinScope<'s, '_>,
    ) -> v8::Local<'s, v8::Object> {
      let obj = v8::Object::new(scope);

      #(
      {
        let #input_field_values = self.#input_fields.to_v8(scope);
        crate::js::binding::set_property_to(scope, obj, #input_field_uppercases, #input_field_values.into());
      }
      )*

      obj
    }
  }

  };

  TokenStream::default()
}
