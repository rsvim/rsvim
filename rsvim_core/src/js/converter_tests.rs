use super::converter::*;
use crate::tests::js::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

#[test]
fn test_integer1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1 = 10_i32;
  let obj1 = to_v8(scope, a1);
  let val1 = from_v8::<i32>(scope, obj1);
  assert_eq!(val1, a1);

  let a2 = 10_u32;
  let obj2 = to_v8(scope, a2);
  let val2 = from_v8::<u32>(scope, obj2);
  assert_eq!(val2, a2);
}

#[test]
fn test_number1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1 = 1.23_f64;
  let obj1 = to_v8(scope, a1);
  assert!(obj1.is_some());
  let obj1 = obj1.unwrap();
  let val1 = from_v8::<f64>(scope, obj1);
  assert!(val1.is_some());
  assert_eq!(val1.unwrap(), a1);

  let a2 = 8_f64;
  let obj2 = to_v8(scope, a2);
  assert!(obj2.is_some());
  let obj2 = obj2.unwrap();
  let val2 = from_v8::<f64>(scope, obj2);
  assert!(val2.is_some());
  assert_eq!(val2.unwrap(), a2);
}

#[test]
fn test_bool1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1 = true;
  let obj1 = to_v8(scope, a1);
  assert!(obj1.is_some());
  let obj1 = obj1.unwrap();
  let val1 = from_v8::<bool>(scope, obj1);
  assert!(val1.is_some());
  assert!(val1.unwrap());

  let a2 = false;
  let obj2 = to_v8(scope, a2);
  assert!(obj2.is_some());
  let obj2 = obj2.unwrap();
  let val2 = from_v8::<bool>(scope, obj2);
  assert!(val2.is_some());
  assert!(!val2.unwrap());
}

#[test]
fn test_string1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1 = "Hello".to_string();
  let obj1 = to_v8(scope, a1.clone());
  assert!(obj1.is_some());
  let obj1 = obj1.unwrap();
  let val1 = from_v8::<String>(scope, obj1);
  assert!(val1.is_some());
  assert_eq!(val1.unwrap(), a1);

  let a2 = "World".to_compact_string();
  let obj2 = to_v8(scope, a2.clone());
  assert!(obj2.is_some());
  let obj2 = obj2.unwrap();
  let val2 = from_v8::<CompactString>(scope, obj2);
  assert!(val2.is_some());
  assert_eq!(val2.unwrap(), a2);
}

#[test]
fn test_array1() {
  let mut jsrt = make_js_runtime();
  let context = jsrt.context();
  v8::scope_with_context!(scope, &mut jsrt.isolate, context);

  let a1: Vec<i32> = vec![1, 2, 3];
  let obj1 = to_v8(scope, a1.clone());
  assert!(obj1.is_some());
  let obj1 = obj1.unwrap();
  let val1 = from_v8::<Vec<i32>>(scope, obj1);
  assert!(val1.is_some());
  assert_eq!(val1.unwrap(), a1);

  let a2: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
  let obj2 = to_v8(scope, a2.clone());
  assert!(obj2.is_some());
  let obj2 = obj2.unwrap();
  let val2 = from_v8::<Vec<String>>(scope, obj2);
  assert!(val2.is_some());
  assert_eq!(val2.unwrap(), a2);
}
