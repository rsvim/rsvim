use quote::format_ident;
use syn::DeriveInput;
use syn::parse_macro_input;

pub fn get_named_fields(
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

pub fn is_type_match(ty: &syn::Type, ident_name: &str) -> bool {
  if let syn::Type::Path(p) = ty {
    if let Some(seg) = p.path.segments.last() {
      return seg.ident == ident_name;
    }
  }
  false
}

pub struct ToV8Tokens {
  pub field: Vec<syn::Ident>,
  pub uppercase: Vec<syn::Ident>,
  pub value: Vec<syn::Ident>,
}

impl ToV8Tokens {
  pub fn collect<'a, F>(
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

pub struct FromV8Tokens {
  pub field: Vec<syn::Ident>,
  pub name: Vec<syn::Ident>,
  pub ty: Vec<syn::Type>,
  pub uppercase: Vec<syn::Ident>,
  pub value: Vec<syn::Ident>,
}

impl FromV8Tokens {
  pub fn collect<'a, F>(
    fields: impl Iterator<Item = &'a syn::Field>,
    predicate: F,
  ) -> Self
  where
    F: Fn(&syn::Field) -> bool,
  {
    let mut res = Self {
      field: vec![],
      name: vec![],
      ty: vec![],
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

      let ty = match &f.ty {
        syn::Type::Path(p) => {
          let seg = p.path.segments.last().unwrap();
          if seg.ident == "Option" {
            match &seg.arguments {
              syn::PathArguments::AngleBracketed(angle) => {
                match angle.args.last().unwrap() {
                  syn::GenericArgument::Type(inner_ty) => inner_ty.clone(),
                  _ => unreachable!(
                    "Expected syn::GenericArgument::Type for {}",
                    ident
                  ),
                }
              }
              _ => unreachable!(
                "Expected syn::PathArguments::AngleBracketed for {}",
                ident
              ),
            }
          } else {
            f.ty.clone()
          }
        }
        _ => unreachable!("Expected syn::Type::Path for {}", ident),
      };
      res.ty.push(ty);
    }
    res
  }
}
