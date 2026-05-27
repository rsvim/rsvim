//! The macros for RSVIM text editor core.

mod js;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::parse_macro_input;

// js {{{

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
  use js::*;

  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let struct_fields = get_named_fields(&input.data);

  let is_option = |f: &syn::Field| is_type_match(&f.ty, "Option");

  let plain = ToV8Tokens::collect(struct_fields.iter(), |f| !is_option(f));
  let optional = ToV8Tokens::collect(struct_fields.iter(), is_option);

  // Destructure for `quote!` use
  let (field, uppercase, value) =
    (&plain.field, &plain.uppercase, &plain.value);
  let (optional_fields, optional_uppercase, optional_value) =
    (&optional.field, &optional.uppercase, &optional.value);

  quote! {

  impl crate::js::converter::ToV8 for #struct_ident {
    fn to_v8<'s>(
      &self,
      scope: &mut v8::PinScope<'s, '_>,
    ) -> v8::Local<'s, v8::Value> {
      let obj = v8::Object::new(scope);

      // plain
      #(
      {
        let #value = self.#field.to_v8(scope);
        crate::js::binding::set_property_to(scope, obj, #uppercase, #value);
      }
      )*

      // optional
      #(
      {
        if let Some(#optional_fields) = &self.#optional_fields {
          let #optional_value = #optional_fields.to_v8(scope);
          crate::js::binding::set_property_to(scope, obj, #optional_uppercase, #optional_value);
        }
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
  use js::*;

  let input = parse_macro_input!(input as DeriveInput);

  let struct_ident = input.ident;
  let struct_ident_builder = format_ident!("{}Builder", struct_ident);
  let struct_fields = get_named_fields(&input.data);

  let is_option = |f: &syn::Field| is_type_match(&f.ty, "Option");

  let tokens = FromV8Tokens::collect(struct_fields.iter(), |f| !is_option(f));
  let optional_tokens = FromV8Tokens::collect(struct_fields.iter(), is_option);

  // Destructure for `quote!` use
  let (field, name, ty, uppercase, value) = (
    &tokens.field,
    &tokens.name,
    &tokens.ty,
    &tokens.uppercase,
    &tokens.value,
  );
  let (
    optional_field,
    optional_name,
    optional_ty,
    optional_uppercase,
    optional_value,
  ) = (
    &optional_tokens.field,
    &optional_tokens.name,
    &optional_tokens.ty,
    &optional_tokens.uppercase,
    &optional_tokens.value,
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

      // plain
      #(
      {
        let #name = v8::String::new(scope, #uppercase).unwrap();
        debug_assert!(obj.has_own_property(scope, #name.into()).unwrap_or(false));
        let #value = obj.get(scope, #name.into()).unwrap();
        builder.#field(<#ty as crate::js::converter::FromV8>::from_v8(scope, #value));
      }
      )*

      // optional
      #(
      {
        let #optional_name = v8::String::new(scope, #optional_uppercase).unwrap();
        if obj.has_own_property(scope, #optional_name.into()).unwrap_or(false) {
          let #optional_value = obj.get(scope, #optional_name.into()).unwrap();
          builder.#optional_field(Some(<#optional_ty as crate::js::converter::FromV8>::from_v8(scope, #optional_value)));
        }
      }
      )*

      builder.build().unwrap()
    }
  }

  }.into()
}

// js }}}

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

// ui::widget::Widgetable {{{

#[proc_macro_derive(WidgetableEnum)]
/// Generate enum disaptcher for `rsvim_core::ui::widget::Widgetable` trait.
pub fn widgetable_enum(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let enum_ident = input.ident;
  let enum_variant = match &input.data {
    syn::Data::Enum(enum_data) => enum_data
      .variants
      .iter()
      .map(|v| v.ident.clone())
      .collect::<Vec<_>>(),
    _ => unreachable!("Failed to derive macro on non-enum data!"),
  };

  quote! {

  impl Widgetable for #enum_ident {
    fn draw(&self, canvas: &mut Canvas, context: &WidgetContext) {
      match self {
        #(
          #enum_ident::#enum_variant(w) => w.draw(canvas, context),
        )*
      }
    }
  }

  }
  .into()
}

// ui::widget::Widgetable }}}

// state::Stateful {{{

#[proc_macro_derive(StatefulEnum)]
/// Generate enum disaptcher for `rsvim_core::state::Stateful` trait.
pub fn stateful_enum(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let enum_ident = input.ident;
  let enum_variant = match &input.data {
    syn::Data::Enum(enum_data) => enum_data
      .variants
      .iter()
      .map(|v| v.ident.clone())
      .collect::<Vec<_>>(),
    _ => unreachable!("Failed to derive macro on non-enum data!"),
  };

  quote! {

  impl crate::state::Stateful for #enum_ident {
    fn handle(&self, context: &crate::state::StateContext, event: crossterm::event::Event) -> crate::state::State {
      match self {
        #(
          #enum_ident::#enum_variant(s) => s.handle(context, event),
        )*
      }
    }
    fn handle_op(&self, context: &crate::state::StateContext, op: crate::state::ops::Operation) -> crate::state::State {
      match self {
        #(
          #enum_ident::#enum_variant(s) => s.handle_op(context, op),
        )*
      }
    }
  }

  }
  .into()
}

// state::Stateful }}}

// ui::tree::internal::Inodify {{{

#[proc_macro_derive(Inodify, attributes(inode_base))]
/// Generate inode body for `rsvim_core::ui::tree::internal::Inodify` trait.
pub fn inodeable(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let struct_ident = &input.ident;

  let fields = match &input.data {
    syn::Data::Struct(struct_data) => match &struct_data.fields {
      syn::Fields::Named(fields) => &fields.named,
      _ => unreachable!("Failed to derive macro on non-named field!"),
    },
    _ => unreachable!("Failed to derive macro on non-struct data!"),
  };

  let mut target_field_ident = None;
  for field in fields {
    for attr in &field.attrs {
      if attr.path().is_ident("inode_base") {
        assert!(
          target_field_ident.is_none(),
          "Attribute is only allowed for once!"
        );
        target_field_ident = Some(field.ident.as_ref().unwrap());
      }
    }
  }
  let field_ident = match target_field_ident {
    Some(ident) => ident,
    None => {
      unreachable!("Missing attribute!")
    }
  };

  let (impl_generics, ty_generics, where_clause) =
    input.generics.split_for_impl();

  quote! {
  impl #impl_generics crate::ui::tree::internal::Inodify for #struct_ident #ty_generics #where_clause {
    fn id(&self) -> crate::ui::tree::internal::NodeId {
      self.#field_ident.id()
    }
    fn shape(&self) -> crate::coord::IRect {
      self.#field_ident.shape()
    }

    fn actual_shape(&self) -> crate::coord::U16Rect {
      self.#field_ident.actual_shape()
    }
    fn zindex(&self) -> usize {
      self.#field_ident.zindex()
    }
    fn enabled(&self) -> bool {
      self.#field_ident.enabled()
    }
    fn truncate_policy(&self) -> crate::ui::tree::internal::TruncatePolicy {
      self.#field_ident.truncate_policy()
    }
  }
  }
  .into()
}

#[proc_macro_derive(InodifyEnum)]
/// Generate enum disaptcher for `rsvim_core::ui::tree::internal::Inodify` trait.
pub fn inodify_enum(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let enum_ident = input.ident;
  let enum_variant = match &input.data {
    syn::Data::Enum(enum_data) => enum_data
      .variants
      .iter()
      .map(|v| v.ident.clone())
      .collect::<Vec<_>>(),
    _ => unreachable!("Failed to derive macro on non-enum data!"),
  };

  let (impl_generics, ty_generics, where_clause) =
    input.generics.split_for_impl();

  quote! {

  impl #impl_generics crate::ui::tree::internal::Inodify for #enum_ident #ty_generics #where_clause {
    fn id(&self) -> crate::ui::tree::internal::NodeId {
      match self {
        #(
          #enum_ident::#enum_variant(e) => e.id(),
        )*
      }
    }
    fn shape(&self) -> crate::coord::IRect {
      match self {
        #(
          #enum_ident::#enum_variant(e) => e.shape(),
        )*
      }
    }
    fn actual_shape(&self) -> crate::coord::U16Rect {
      match self {
        #(
          #enum_ident::#enum_variant(e) => e.actual_shape(),
        )*
      }
    }
    fn zindex(&self) -> usize {
      match self {
        #(
          #enum_ident::#enum_variant(e) => e.zindex(),
        )*
      }
    }
    fn enabled(&self) -> bool {
      match self {
        #(
          #enum_ident::#enum_variant(e) => e.enabled(),
        )*
      }
    }
    fn truncate_policy(&self) -> TruncatePolicy {
      match self {
        #(
          #enum_ident::#enum_variant(e) => e.truncate_policy(),
        )*
      }
    }
  }

  }
  .into()
}

// ui::tree::internal::Inodify }}}
