use super::typescript::*;

use crate::test::log::init as test_log_init;

use tracing::info;

#[test]
fn test1() {
  test_log_init();

  let m1 = "./runtime/00__web.ts";
  let actual1 =
    TypeScript::compile(Some(m1), include_str!("./../runtime/00__web.ts"));
  assert!(actual1.is_ok());
  let actual1 = actual1.unwrap();
  info!("{m1}:\n{actual1}");

  let m2 = "./runtime/01__rsvim.ts";
  let actual2 =
    TypeScript::compile(Some(m2), include_str!("./../runtime/01__rsvim.ts"));
  assert!(actual2.is_ok());
  let actual2 = actual2.unwrap();
  info!("{m2}:\n{actual2}");
}
