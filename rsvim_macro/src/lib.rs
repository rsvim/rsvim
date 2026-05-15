//! The macros for RSVIM text editor core.

use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::parse_macro_input;

#[proc_macro_derive(Builder)]
/// For `js::converter`
pub fn to_v8_obj(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  println!("to_v8_obj:{:?}", input);
  TokenStream::default()
}
