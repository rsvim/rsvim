//! The macros for RSVIM text editor core.

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

// js::converter {{{

fn get_named_fields(
  data: &syn::Data,
) -> &syn::punctuated::Punctuated<syn::Field, syn::token::Comma> {
  match data {
    syn::Data::Struct(struct_data) => match &struct_data.fields {
      syn::Fields::Named(named_field) => &named_field.named,
      _ => unreachable!("Failed to derive macro on non-named field!"),
    },
    _ => unreachable!("Failed to derive macro on non-struct data!"),
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
  field: Vec<syn::Ident>,
  uppercase: Vec<syn::Ident>,
  value: Vec<syn::Ident>,
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
      field: vec![],
      uppercase: vec![],
      value: vec![],
    };
    for f in fields.filter(|&f| predicate(f)) {
      let ident = f.ident.clone().unwrap();
      res
        .uppercase
        .push(format_ident!("{}", ident.to_string().to_uppercase()));
      res.value.push(format_ident!("{}_value", ident));
      res.field.push(ident);
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
  let struct_fields = get_named_fields(&input.data);

  let is_option = |f: &syn::Field| is_type_match(&f.ty, "Option");
  let is_vec = |f: &syn::Field| is_type_match(&f.ty, "Vec");

  let plain =
    ToV8Tokens::collect(struct_fields.iter(), |f| !is_option(f) && !is_vec(f));
  let optional = ToV8Tokens::collect(struct_fields.iter(), is_option);
  let vec = ToV8Tokens::collect(struct_fields.iter(), is_vec);

  // Destructure for `quote!` use
  let (field, uppercase, value) =
    (&plain.field, &plain.uppercase, &plain.value);
  let (optional_fields, optional_uppercase, optional_value) =
    (&optional.field, &optional.uppercase, &optional.value);
  let (vec_fields, vec_uppercase, vec_value) =
    (&vec.field, &vec.uppercase, &vec.value);

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
        let #value = self.#field.to_v8(scope);
        binding::set_property_to(scope, obj, #uppercase, #value);
      }
      )*

      // optional
      #(
      {
        if let Some(#optional_fields) = &self.#optional_fields {
          let #optional_value = #optional_fields.to_v8(scope);
          binding::set_property_to(scope, obj, #optional_uppercase, #optional_value);
        }
      }
      )*

      // vec
      #(
      {
        let #vec_value = self.#vec_fields.to_v8(scope, |scope, i| i.to_v8(scope));
        binding::set_property_to(scope, obj, #vec_uppercase, #vec_value);
      }
      )*


      obj.into()
    }
  }

  }.into()
}

struct FromV8Tokens {
  field: Vec<syn::Ident>,
  name: Vec<syn::Ident>,
  r#type: Vec<syn::Ident>,
  uppercase: Vec<syn::Ident>,
  value: Vec<syn::Ident>,
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
      field: vec![],
      name: vec![],
      r#type: vec![],
      uppercase: vec![],
      value: vec![],
    };

    for f in fields.filter(|&f| predicate(f)) {
      let ident = f.ident.clone().unwrap();
      res.name.push(format_ident!("{}_name", ident));
      res
        .uppercase
        .push(format_ident!("{}", ident.to_string().to_uppercase()));
      res.value.push(format_ident!("{}_value", ident));
      res.field.push(ident.clone());

      let ty_ident = match &f.ty {
        syn::Type::Path(p) => {
          let seg = p.path.segments.last().unwrap();
          if seg.ident == "Option" || seg.ident == "Vec" {
            match &seg.arguments {
              syn::PathArguments::AngleBracketed(angle) => {
                match angle.args.last().unwrap() {
                  // Match inner type here
                  syn::GenericArgument::Type(syn::Type::Path(inner_p)) => {
                    inner_p.path.segments.last().unwrap().ident.clone()
                  }
                  _ => unreachable!(
                    "Expected syn::GenericArgument::Type(syn::Type::Path(...)) for {}",
                    ident
                  ),
                }
              }
              _ => unreachable!(
                "Expected syn::PathArguments::AngleBracketed(...) for {}",
                ident
              ),
            }
          } else {
            seg.ident.clone()
          }
        }
        _ => unreachable!("Expected syn::Type::Path(...) for {}", ident),
      };
      res.r#type.push(ty_ident);
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
  let struct_fields = get_named_fields(&input.data);

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
  let (bool_field, bool_name, bool_type, bool_uppercase, bool_value) = (
    &bool_tokens.field,
    &bool_tokens.name,
    &bool_tokens.r#type,
    &bool_tokens.uppercase,
    &bool_tokens.value,
  );
  let (
    optional_bool_field,
    optional_bool_name,
    optional_bool_type,
    optional_bool_uppercase,
    optional_bool_value,
  ) = (
    &optional_bool_tokens.field,
    &optional_bool_tokens.name,
    &optional_bool_tokens.r#type,
    &optional_bool_tokens.uppercase,
    &optional_bool_tokens.value,
  );
  let (string_field, string_name, string_type, string_uppercase, string_value) = (
    &string_tokens.field,
    &string_tokens.name,
    &string_tokens.r#type,
    &string_tokens.uppercase,
    &string_tokens.value,
  );
  let (
    optional_string_field,
    optional_string_name,
    optional_string_type,
    optional_string_uppercase,
    optional_string_value,
  ) = (
    &optional_string_tokens.field,
    &optional_string_tokens.name,
    &optional_string_tokens.r#type,
    &optional_string_tokens.uppercase,
    &optional_string_tokens.value,
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
        let #bool_name = v8::String::new(scope, #bool_uppercase).unwrap();
        debug_assert!(obj.has_own_property(scope, #bool_name.into()).unwrap_or(false));
        let #bool_value = obj.get(scope, #bool_name.into()).unwrap();
        debug_assert!(#bool_value.is_boolean() || #bool_value.is_boolean_object());
        let #bool_value = #bool_value.to_boolean(scope);
        builder.#bool_field(#bool_type::from_v8(scope, #bool_value.into()));
      }
      )*

      // optional bool
      #(
      {
        let #optional_bool_name = v8::String::new(scope, #optional_bool_uppercase).unwrap();
        if obj.has_own_property(scope, #optional_bool_name.into()).unwrap_or(false) {
          let #optional_bool_value = obj.get(scope, #optional_bool_name.into()).unwrap();
          debug_assert!(#optional_bool_value.is_boolean() || #optional_bool_value.is_boolean_object());
          let #optional_bool_value = #optional_bool_value.to_boolean(scope);
          builder.#optional_bool_field(Some(#optional_bool_type::from_v8(scope, #optional_bool_value.into())));
        }
      }
      )*

      // string
      #(
      {
        let #string_name = v8::String::new(scope, #string_uppercase).unwrap();
        debug_assert!(obj.has_own_property(scope, #string_name.into()).unwrap_or(false));
        let #string_value = obj.get(scope, #string_name.into()).unwrap();
        debug_assert!(#string_value.is_string() || #string_value.is_string_object());
        let #string_value = #string_value.to_string(scope).unwrap();
        builder.#string_field(#string_type::from_v8(scope, #string_value.into()));
      }
      )*

      // optional string
      #(
      {
        let #optional_string_name = v8::String::new(scope, #optional_string_uppercase).unwrap();
        if obj.has_own_property(scope, #optional_string_name.into()).unwrap_or(false) {
          let #optional_string_value = obj.get(scope, #optional_string_name.into()).unwrap();
          debug_assert!(#optional_string_value.is_string() || #optional_string_value.is_string_object());
          let #optional_string_value = #optional_string_value.to_string(scope).unwrap();
          builder.#optional_string_field(Some(#optional_string_type::from_v8(scope, #optional_string_value.into())));
        }
      }
      )*

      builder.build().unwrap()
    }
  }

  }.into()
}

// js::converter }}}

// incremental_id {{{

#[proc_macro_derive(IncrementalId, attributes(start_from))]
/// Generate incremental ID.
///
/// We don't simply use integer types such as `usize`, `i32` as ID, because we
/// can have multiple ID scopes such as buffer ID, window ID and other task
/// IDs.
/// When these different scopes working together, it can be possible that an ID
/// is passed to a different scope which it doesn't belong. So we usually want
/// to define the ID with a struct type such as:
///
/// ```
/// pub struct BufferId(i32);
/// pub struct WindowId(i32);
/// pub struct SyntaxId(usize);
/// ```
///
/// Even the IDs still use the same `usize` internal data type, they are type
/// safe in the code base.
///
/// This macro helps defining a new ID struct and generate all the methods it
/// needs. And ID by default starts from 0.
pub fn incremental_id(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let struct_ident = input.ident;
  let struct_field = match input.data {
    syn::Data::Struct(data) => match data.fields {
      syn::Fields::Unnamed(fields) => fields.unnamed.first().unwrap().clone(),
      _ => {
        unreachable!("Expect syn::Fields::Unnamed(...) for {}", struct_ident)
      }
    },
    _ => unreachable!("Expect syn::Data::Struct(...) for {}", struct_ident),
  };

  let field_ty = match struct_field.ty {
    syn::Type::Path(p) => p.path.segments.last().unwrap().ident.clone(),
    _ => unreachable!("Expect syn::Type::Path(...) for {}", struct_ident),
  };
  let field_ty_str = field_ty.to_string();
  let is_signed = matches!(
    field_ty_str.as_str(),
    "i8" | "i16" | "i32" | "i64" | "isize"
  );
  let is_unsigned = matches!(
    field_ty_str.as_str(),
    "u8" | "u16" | "u32" | "u64" | "usize"
  );
  if !is_signed && !is_unsigned {
    unreachable!("Expect integer type for {}", struct_ident);
  }

  let atomic_ty = match field_ty.to_string().as_str() {
    "i8" => quote!(std::sync::atomic::AtomicI8),
    "u8" => quote!(std::sync::atomic::AtomicU8),
    "i16" => quote!(std::sync::atomic::AtomicI16),
    "u16" => quote!(std::sync::atomic::AtomicU16),
    "i32" => quote!(std::sync::atomic::AtomicI32),
    "u32" => quote!(std::sync::atomic::AtomicU32),
    "i64" => quote!(std::sync::atomic::AtomicI64),
    "u64" => quote!(std::sync::atomic::AtomicU64),
    "isize" => quote!(std::sync::atomic::AtomicIsize),
    "usize" => quote!(std::sync::atomic::AtomicUsize),
    _ => unreachable!("Expect integer type for {}", struct_ident),
  };

  let start_from_value = struct_field
    .attrs
    .iter()
    .filter(|a| a.path().is_ident("start_from"))
    .find_map(|a| a.parse_args::<syn::LitInt>().ok())
    .map(|lit| quote!(#lit))
    .unwrap_or_else(|| quote!(0)); // Default to 0 token

  let signed_methods = if is_signed {
    quote! {
        pub const fn negative_one() -> Self {
            Self(-1)
        }
    }
  } else {
    quote!()
  };

  quote! {
      impl std::cmp::PartialEq<#field_ty> for #struct_ident {
          fn eq(&self, other: &#field_ty) -> bool {
              self.0.eq(other)
          }
      }
      impl std::cmp::PartialEq<#struct_ident> for #struct_ident {
          fn eq(&self, other: &#struct_ident) -> bool {
              self.0 == other.0
          }
      }
      impl std::cmp::Eq for #struct_ident {}
      impl std::cmp::PartialOrd<#field_ty> for #struct_ident {
          fn partial_cmp(&self, other: &#field_ty) -> Option<std::cmp::Ordering> {
              self.0.partial_cmp(other)
          }
      }
      impl std::cmp::PartialOrd<#struct_ident> for #struct_ident {
          fn partial_cmp(&self, other: &#struct_ident) -> Option<std::cmp::Ordering> {
              self.0.partial_cmp(&other.0)
          }
      }
      impl std::cmp::Ord for #struct_ident {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
          self.0.cmp(&other.0)
        }
      }
      impl std::hash::Hash for #struct_ident {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
      }
      impl std::fmt::Debug for #struct_ident {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
              f.write_fmt(format_args!("{:?}", self.0))
          }
      }
      impl std::fmt::Display for #struct_ident {
          fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
              f.write_fmt(format_args!("{}", self.0))
          }
      }
      impl From<#field_ty> for #struct_ident {
          fn from(value: #field_ty) -> Self {
              Self(value)
          }
      }
      impl From<#struct_ident> for #field_ty {
          fn from(value: #struct_ident) -> Self {
              value.0
          }
      }
      impl #struct_ident {
          pub fn next() -> Self {
              static VALUE: #atomic_ty = #atomic_ty::new(#start_from_value);
              let v = VALUE
                  .fetch_update(
                      std::sync::atomic::Ordering::Relaxed,
                      std::sync::atomic::Ordering::Relaxed,
                      |x| {
                          Some(if x == #field_ty::MAX {
                              #start_from_value
                          } else {
                              x + 1
                          })
                      },
                  )
                  .unwrap();
              Self::from(v)
          }
          pub const fn zero() -> Self {
              Self(0)
          }
          // This will emit methods only for signed integers
          #signed_methods
      }
  }.into()
}

// incremental_id }}}

// arc/rc pointers {{{

#[proc_macro_derive(ArcMutexPtr)]
/// Generate `Arc<Mutex<...>>` pointer for a struct type.
pub fn arc_mutex_ptr(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let arc_ident = format_ident!("{}Arc", struct_ident);
  let weak_ident = format_ident!("{}Wk", struct_ident);
  let mutex_guard_ident = format_ident!("{}MutexGuard", struct_ident);

  quote! {

  pub type #arc_ident = std::sync::Arc<std::sync::Mutex<#struct_ident>>;
  pub type #weak_ident = std::sync::Weak<std::sync::Mutex<#struct_ident>>;
  pub type #mutex_guard_ident<'a> = std::sync::MutexGuard<'a, #struct_ident>;

  impl #struct_ident {
    pub fn to_arc(value: #struct_ident) -> #arc_ident {
      std::sync::Arc::new(std::sync::Mutex::new(value))
    }
  }

  }
  .into()
}

#[proc_macro_derive(ArcPtr)]
/// Generate `Arc<...>` pointer for a struct type.
pub fn arc_ptr(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let arc_ident = format_ident!("{}Arc", struct_ident);
  let weak_ident = format_ident!("{}Wk", struct_ident);

  quote! {

  pub type #arc_ident = std::sync::Arc<#struct_ident>;
  pub type #weak_ident = std::sync::Weak<#struct_ident>;

  impl #struct_ident {
    pub fn to_arc(value: #struct_ident) -> #arc_ident {
      std::sync::Arc::new(value)
    }
  }

  }
  .into()
}

#[proc_macro_derive(RcRefCellPtr)]
/// Generate `Rc<RefCell<...>>` pointer for a struct type.
pub fn rc_refcell_ptr(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let rc_ident = format_ident!("{}Rc", struct_ident);
  let weak_ident = format_ident!("{}Wk", struct_ident);

  quote! {

  pub type #rc_ident = std::rc::Rc<std::cell::RefCell<#struct_ident>>;
  pub type #weak_ident = std::rc::Weak<std::cell::RefCell<#struct_ident>>;

  impl #struct_ident {
    pub fn to_rc(value: #struct_ident) -> #rc_ident {
      std::rc::Rc::new(std::cell::RefCell::new(value))
    }
  }

  }
  .into()
}

#[proc_macro_derive(RcPtr)]
/// Generate `Rc<...>` pointer for a struct type.
pub fn rc_ptr(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let rc_ident = format_ident!("{}Rc", struct_ident);
  let weak_ident = format_ident!("{}Wk", struct_ident);

  quote! {

  pub type #rc_ident = std::rc::Rc<#struct_ident>;
  pub type #weak_ident = std::rc::Weak<#struct_ident>;

  impl #struct_ident {
    pub fn to_rc(value: #struct_ident) -> #rc_ident {
      std::rc::Rc::new(value)
    }
  }

  }
  .into()
}

// arc/rc pointers }}}

// ui::widgetable {{{

#[proc_macro_derive(WidgetableEnum)]
pub fn widgetable_enum(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let enum_ident = input.ident;

  let enum_variants = match &input.data {
    syn::Data::Enum(enum_data) => {
      let mut vars = vec![];
      for var in &enum_data.variants {
        vars.push(var.ident.clone());
      }
      vars
    }
    _ => unreachable!("Failed to derive macro on non-enum field!"),
  };
  println!(
    "widgetable_enum:{}, vars:{:?}",
    enum_ident,
    enum_variants
      .iter()
      .map(|v| v.to_string())
      .collect::<Vec<_>>()
  );

  TokenStream::default()
}

// ui::widgetable }}}
