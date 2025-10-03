use super::attr::*;
use crate::js::converter::*;
use crate::tests::js::*;
use std::str::FromStr;

#[test]
fn test_nargs() {
  assert_eq!(format!("{}", Nargs::Zero), "0");
  assert_eq!(Nargs::from_str("0"), Ok(Nargs::Zero));

  assert_eq!(format!("{}", Nargs::One), "1");
  assert_eq!(Nargs::from_str("1"), Ok(Nargs::One));

  assert_eq!(format!("{}", Nargs::Optional), "?");
  assert_eq!(Nargs::from_str("?"), Ok(Nargs::Optional));

  assert_eq!(format!("{}", Nargs::More), "+");
  assert_eq!(Nargs::from_str("+"), Ok(Nargs::More));

  assert_eq!(format!("{}", Nargs::Any), "*");
  assert_eq!(Nargs::from_str("*"), Ok(Nargs::Any));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_converter1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1 = CommandAttributesBuilder::default().build().unwrap();
  let obj1 = to_v8(scope, a1.clone());
  let val1 = from_v8::<CommandAttributes>(scope, obj1);
  assert_eq!(val1, a1);

  let a2 = CommandAttributesBuilder::default()
    .nargs(Nargs::Any)
    .build()
    .unwrap();
  let obj2 = to_v8(scope, a2.clone());
  let val2 = from_v8::<CommandAttributes>(scope, obj2);
  assert_eq!(val2, a2);
}
