use super::typescript::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;

#[test]
fn test1() {
  test_log_init();

  let input: &str = r#"
function isNull(arg: any): boolean {
  return arg === undefined || arg === null;
}
    "#;

  let expect: &str = r#"
function isNull(arg) {
  return arg === undefined || arg === null;
}
    "#;

  let actual = TypeScript::compile(None, input);
  assert!(actual.is_ok());
  let actual = actual.unwrap();
  info!("actual:\n{actual}");
  assert_eq!(actual.trim(), expect.trim());
}

#[test]
fn test2() {
  test_log_init();

  let actual1 = TypeScript::compile(None, "const let var function");
  assert!(actual1.is_err());
  info!("{:?}", actual1.err().unwrap().to_string());
}
