use super::opt::*;
use crate::js::converter::*;
use crate::tests::js::*;
use compact_str::ToCompactString;

#[test]
fn test1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1 = CommandOptionsBuilder::default().build().unwrap();
  let obj1 = to_v8(scope, a1.clone());
  let val1 = from_v8::<CommandOptions>(scope, obj1);
  assert_eq!(val1, a1);

  let a2 = CommandOptionsBuilder::default()
    .alias(Some("w".to_compact_string()))
    .build()
    .unwrap();
  let obj2 = to_v8(scope, a2.clone());
  let val2 = from_v8::<CommandOptions>(scope, obj2);
  assert_eq!(val2, a2);
}
