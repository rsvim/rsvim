//! The macros for RSVIM text editor core.

use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(ToV8)]
/// Serialize rust struct to v8 js object. A js object is like a key-value map
/// that contains multiple data fields. In most use cases, we only need 1 layer
/// map, i.e. all the values are not nested js objects, they are simply plain
/// data values and array with plain values.
pub fn to_v8(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  println!("ToV8 input_ident: {}({:?})", input.ident, input.ident);

  match input.data {
    syn::Data::Struct(struct_data) => match struct_data.fields {
      syn::Fields::Named(named_field) => {
        for (i, named) in named_field.named.iter().enumerate() {
          if let Some(named_ident) = &named.ident {
            println!(
              "ToV8 named_field [{}]:{} ({:?})",
              i, named_ident, named_ident
            );
          }
        }
      }
      _ => unreachable!("Failed to derive ToV8 macro on non-named field!"),
    },
    _ => unreachable!("Failed to derive ToV8 macro on non-struct data!"),
  }

  // let expanded = quote! {};

  TokenStream::default()
}
