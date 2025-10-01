use super::converter::*;
use crate::tests::js::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

#[test]
fn test_integer1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let obj1 = to_v8(scope, 10_i32);
  let val1 = from_v8::<i32>(scope, obj1);
  assert_eq!(val1, 10);

  let obj2 = to_v8(scope, 10_u32);
  let val2 = from_v8::<u32>(scope, obj2);
  assert_eq!(val2, 10_u32);
}

#[test]
fn test_number1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let obj1 = to_v8(scope, 1.23_f64);
  let val1 = from_v8::<f64>(scope, obj1);
  assert_eq!(val1, 1.23_f64);

  let obj2 = to_v8(scope, 8_f64);
  let val2 = from_v8::<f64>(scope, obj2);
  assert_eq!(val2, 8_f64);
}

#[test]
fn test_bool1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let obj1 = to_v8(scope, true);
  let val1 = from_v8::<bool>(scope, obj1);
  assert!(val1);

  let obj2 = to_v8(scope, false);
  let val2 = from_v8::<bool>(scope, obj2);
  assert!(!val2);
}

#[test]
fn test_string1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let obj1 = to_v8(scope, "Hello".to_string());
  let val1 = from_v8::<String>(scope, obj1);
  assert_eq!(val1, "Hello".to_string());

  let obj2 = to_v8(scope, "World".to_compact_string());
  let val2 = from_v8::<CompactString>(scope, obj2);
  assert_eq!(val2, "World".to_compact_string());
}

#[test]
fn test_string1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let obj1 = to_v8(scope, "Hello".to_string());
  let val1 = from_v8::<String>(scope, obj1);
  assert_eq!(val1, "Hello".to_string());

  let obj2 = to_v8(scope, "World".to_compact_string());
  let val2 = from_v8::<CompactString>(scope, obj2);
  assert_eq!(val2, "World".to_compact_string());
}
