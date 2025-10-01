use super::converter::*;
use crate::tests::js::*;

#[test]
fn test_integer1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let obj1 = to_v8(scope, 10_i32);
  let val1 = from_v8(scope, obj1);
  assert_eq!(val1, 10);
}
