use super::module::*;

use crate::test::constant::acquire_sequential_guard;
use crate::test::js::make_js_runtime;
use crate::test::log::init as test_log_init;

use assert_fs::TempDir;
use std::io::Write;

#[test]
fn test_fetch1() {
  let _guard = acquire_sequential_guard();

  let tmpdir = TempDir::new().unwrap();

  const SRC1: &str = r#"
  export const PI = 3.14159;
  export function Hello(a, b) {
    return a+b;
  }
  function World(a, b) {
    return a-b;
  }
  export { World };
  "#;

  let fetch1 = tmpdir.join("fetch1.rs");

  {
    let mut src1 = std::fs::File::create(&fetch1).unwrap();
    src1.write_all(SRC1.as_bytes()).unwrap();
    src1.flush().unwrap();
  }

  test_log_init();
  let mut jsrt = make_js_runtime();
  let mut scope = jsrt.handle_scope();
  let actual1 =
    fetch_module(&mut scope, fetch1.as_os_str().to_str().unwrap(), None);
  assert!(actual1.is_some());
  let actual1 = actual1.unwrap();
  assert!(actual1.script_id().is_some());
  assert!(actual1.script_id().unwrap() > 0);
}
